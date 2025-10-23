use crate::{
    gdt::Gdt,
    utils::{error_screen, get_flags, hlt},
};
use core::{arch::asm, cell::SyncUnsafeCell, fmt::Write, mem::offset_of};

#[derive(Debug)]
#[repr(C, packed)]
struct IdtDescriptor {
    pub size: u16,
    pub offset: *const Idt,
}

const _: () = assert!(size_of::<IdtDescriptor>() == 10);

#[derive(Debug)]
#[repr(C)]
pub struct Entry {
    pub offset0: u16,
    pub selector: u16,
    pub ist: u8,
    pub types_attributes: u8,
    pub offset1: u16,
    pub offset2: u32,
    pub reserved: u32,
}

const _: () = assert!(size_of::<Entry>() == 16);

pub enum InterruptType {
    Interrupt,
    Trap,
}

impl Entry {
    fn set_offset(&mut self, offset: usize) {
        self.offset0 = (offset & 0x000000000000FFFF) as u16;
        self.offset1 = ((offset & 0x00000000FFFF0000) >> 16) as u16;
        self.offset2 = ((offset & 0xFFFFFFFF00000000) >> 32) as u32;
    }

    fn set_handler_(&mut self, handler: usize, interrupt_type: InterruptType) {
        self.set_offset(handler);
        self.selector = offset_of!(Gdt, kernel_code) as u16;
        self.ist = 0;
        self.types_attributes = match interrupt_type {
            InterruptType::Interrupt => 0b1000_1110,
            InterruptType::Trap => 0b1000_1111,
        };
    }

    pub fn set_handler(
        &mut self,
        handler: unsafe extern "x86-interrupt" fn(InterruptStackFrame),
        interrupt_type: InterruptType,
    ) {
        self.set_handler_(handler as usize, interrupt_type);
    }

    pub fn set_handler_with_error(
        &mut self,
        handler: unsafe extern "x86-interrupt" fn(InterruptStackFrame, u64),
    ) {
        self.set_handler_(handler as usize, InterruptType::Interrupt);
    }

    pub fn set_abort_handler_with_error(
        &mut self,
        handler: unsafe extern "x86-interrupt" fn(InterruptStackFrame, u64) -> !,
    ) {
        self.set_handler_(handler as usize, InterruptType::Interrupt);
    }
}

#[repr(C)]
pub struct Idt {
    entries: [Entry; 256],
}

const _: () = assert!(size_of::<Idt>() == 0x1000);

static IDT: SyncUnsafeCell<Idt> = SyncUnsafeCell::new(Idt {
    entries: [const {
        Entry {
            offset0: !0,
            selector: 0,
            ist: 0,
            types_attributes: 0,
            offset1: !0,
            offset2: !0,
            reserved: 0,
        }
    }; _],
});

pub unsafe fn setup_idt() {
    {
        let idt = unsafe { &mut *IDT.get() };

        idt.entries[0x03].set_handler(debug_break_handler, InterruptType::Trap);

        idt.entries[0x08].set_abort_handler_with_error(double_fault_handler);
        idt.entries[0x0D].set_handler_with_error(general_protection_handler);
        idt.entries[0x0E].set_handler_with_error(page_fault_handler);
    }

    let descriptor = IdtDescriptor {
        size: (size_of::<Idt>() - 1) as _,
        offset: IDT.get(),
    };

    // load the idt into the idtr resgister
    unsafe { asm!("lidt [{}]", in(reg) &raw const descriptor, options(nostack)) };
}

pub fn is_interrupts_enabled() -> bool {
    let flags = get_flags();
    flags & (1 << 9) != 0
}

pub unsafe fn disable_interrupts() {
    unsafe { asm!("cli", options(nomem, nostack)) };
}

pub unsafe fn enable_interrupts() {
    unsafe { asm!("sti", options(nomem, nostack)) };
}

pub fn with_disabled_interrupts<R>(f: impl FnOnce() -> R) -> R {
    let was_interrupts_enabled = is_interrupts_enabled();
    if was_interrupts_enabled {
        unsafe { disable_interrupts() };
    }
    let value = f();
    if was_interrupts_enabled {
        unsafe { enable_interrupts() };
    }
    value
}

pub unsafe fn with_idt_entry<R>(interrupt: u8, f: impl FnOnce(&mut Entry) -> R) -> R {
    with_disabled_interrupts(|| f(unsafe { &mut (*IDT.get()).entries[interrupt as usize] }))
}

#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub ip: usize,
    pub cs: usize,
    pub flags: usize,
    pub sp: usize,
    pub ss: usize,
}

unsafe extern "x86-interrupt" fn debug_break_handler(stack_frame: InterruptStackFrame) {
    error_screen(|text_writer| {
        writeln!(text_writer, "Debug Break:").unwrap();
        writeln!(text_writer, "{stack_frame:#x?}").unwrap();
    });

    hlt()
}

unsafe extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    error_screen(|text_writer| {
        writeln!(text_writer, "Double Fault:").unwrap();
        writeln!(text_writer, "{stack_frame:#x?}").unwrap();
        writeln!(text_writer, "{error_code:#x}").unwrap();
    });

    hlt()
}

unsafe extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    error_screen(|text_writer| {
        writeln!(text_writer, "General Protection Fault:").unwrap();
        writeln!(text_writer, "{stack_frame:#x?}").unwrap();
        writeln!(text_writer, "{error_code:#x}").unwrap();
    });

    hlt()
}

unsafe extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    error_screen(|text_writer| {
        writeln!(text_writer, "Page Fault:").unwrap();
        writeln!(text_writer, "{stack_frame:#x?}").unwrap();
        writeln!(text_writer, "{error_code:#x}").unwrap();
    });

    hlt()
}
