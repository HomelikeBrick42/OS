use crate::{
    print::println,
    utils::{error_screen, hlt},
};
use core::{arch::asm, cell::SyncUnsafeCell, fmt::Write};

#[repr(C, packed)]
struct IdtDescriptor {
    pub size: u16,
    pub offset: *const Idt,
}

const _: () = assert!(size_of::<IdtDescriptor>() == 10);

#[repr(C, packed)]
struct Entry {
    pub offset0: u16,
    pub selector: u16,
    pub ist: u8,
    pub types_attributes: u8,
    pub offset1: u16,
    pub offset2: u32,
    pub reserved: u32,
}

impl Entry {
    pub const fn set_offset(&mut self, offset: usize) {
        self.offset0 = (offset & 0x000000000000FFFF) as u16;
        self.offset1 = ((offset & 0x00000000FFFF0000) >> 16) as u16;
        self.offset2 = ((offset & 0xFFFFFFFF00000000) >> 32) as u32;
    }
}

const _: () = assert!(size_of::<Entry>() == 16);

#[repr(C, align(0x1000))]
pub struct Idt {
    entries: [Entry; 256],
}

static IDT: SyncUnsafeCell<Idt> = SyncUnsafeCell::new(Idt {
    entries: unsafe { core::mem::zeroed() },
});

pub unsafe fn setup_idt() {
    {
        let idt = unsafe { &mut *IDT.get() };

        {
            let general_protection = &mut idt.entries[0x0D];
            general_protection.set_offset(general_protection_handler as usize);
            general_protection.selector = 0x08;
            general_protection.ist = 0;
            general_protection.types_attributes = 0b1000_1110;
        }
    }

    println!("IDT = {:p}", IDT.get());

    let descriptor = IdtDescriptor {
        size: (size_of::<Idt>() - 1) as _,
        offset: IDT.get(),
    };

    // load the idt into the idtr resgister
    unsafe { asm!("lidt [{}]", in(reg) &raw const descriptor) };
}

pub unsafe fn disable_interrupts() {
    unsafe { asm!("cli") };
}

pub unsafe fn enable_interrupts() {
    unsafe { asm!("sti") };
}

#[derive(Debug)]
#[repr(C)]
struct InterruptStackFrame {
    pub ip: usize,
    pub cs: usize,
    pub flags: usize,
    pub sp: usize,
    pub ss: usize,
}

extern "x86-interrupt" fn general_protection_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    error_screen(|text_writer| {
        writeln!(text_writer, "General Protection Fault:").unwrap();
        writeln!(text_writer, "{stack_frame:#x?}").unwrap();
        writeln!(text_writer, "{error_code:x?}").unwrap();
    });

    hlt()
}
