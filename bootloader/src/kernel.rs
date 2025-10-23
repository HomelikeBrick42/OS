use crate::{
    framebuffer::framebuffer,
    gdt::setup_gdt,
    idt::{
        InterruptStackFrame, InterruptType, disable_interrupts, enable_interrupts, setup_idt,
        with_idt_entry,
    },
    page_allocator::with_page_allocator,
    print::{println, with_global_printer},
    utils::{hlt, inb, io_wait, outb},
};

pub unsafe extern "win64" fn kernel_main() -> ! {
    let framebuffer = framebuffer();

    let background_color = with_global_printer(|printer| {
        printer.x = 0;
        printer.y = 0;
        printer.left_margin = 0;
        printer.background_color
    });
    framebuffer.fill(
        0,
        0,
        framebuffer.width(),
        framebuffer.height(),
        framebuffer.color(background_color),
    );

    unsafe { disable_interrupts() };
    unsafe { setup_gdt() };
    unsafe { setup_idt() };

    unsafe {
        with_idt_entry(0x21, |entry| {
            entry.set_handler(keyboard_handler, InterruptType::Interrupt);
        });
    }
    unsafe { setup_keyboard() };

    unsafe { enable_interrupts() };

    with_page_allocator(|alloc| {
        println!("Total Memory: {} KiB", alloc.total_pages() * 4096 / 1024);
        println!(
            "Allocated Memory: {} KiB",
            alloc.allocated_pages() * 4096 / 1024
        );
        println!("Free Memory: {} KiB", alloc.free_pages() * 4096 / 1024);
    });

    hlt()
}

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const PIC_EOI: u8 = 0x20;
const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

unsafe fn setup_keyboard() {
    let a1 = unsafe { inb(PIC1_DATA) };
    io_wait();
    let a2 = unsafe { inb(PIC2_DATA) };
    io_wait();

    unsafe { outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4) };
    io_wait();
    unsafe { outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4) };
    io_wait();

    unsafe { outb(PIC1_DATA, 0x20) };
    io_wait();
    unsafe { outb(PIC2_DATA, 0x28) };
    io_wait();

    unsafe { outb(PIC1_DATA, 4) };
    io_wait();
    unsafe { outb(PIC2_DATA, 2) };
    io_wait();

    unsafe { outb(PIC1_DATA, ICW4_8086) };
    io_wait();
    unsafe { outb(PIC2_DATA, ICW4_8086) };
    io_wait();

    unsafe { outb(PIC1_DATA, a1) };
    io_wait();
    unsafe { outb(PIC2_DATA, a2) };

    unsafe { outb(PIC1_DATA, 0b11111101) };
    unsafe { outb(PIC2_DATA, 0b11111111) };
}

unsafe extern "x86-interrupt" fn keyboard_handler(_: InterruptStackFrame) {
    let scancode = unsafe { inb(0x60) };

    // this is bad because it involves locks, but who cares for now
    println!("Scancode: {}", scancode);

    unsafe { outb(PIC1_COMMAND, PIC_EOI) };
}
