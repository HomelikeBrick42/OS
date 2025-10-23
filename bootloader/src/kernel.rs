use crate::{
    drivers::pic::{PIC1_DATA, PIC2_DATA, pic1_end, remap_pic},
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

    unsafe { remap_pic(0x20, 0x28) };

    unsafe {
        with_idt_entry(0x21, |entry| {
            entry.set_handler(keyboard_handler, InterruptType::Interrupt);
        });
    }
    unsafe { outb(PIC1_DATA, 0b11111101) };
    io_wait();
    unsafe { outb(PIC2_DATA, 0b11111111) };
    io_wait();

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

unsafe extern "x86-interrupt" fn keyboard_handler(_: InterruptStackFrame) {
    let scancode = unsafe { inb(0x60) };
    io_wait();

    // this is bad because it involves locks, but who cares for now
    println!("Scancode: {}", scancode);

    unsafe { pic1_end() };
}
