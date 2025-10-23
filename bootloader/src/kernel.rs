use crate::{
    framebuffer::framebuffer,
    gdt::setup_gdt,
    idt::{disable_interrupts, enable_interrupts, setup_idt},
    page_allocator::with_page_allocator,
    print::{println, with_global_printer},
    utils::hlt,
};

pub extern "win64" fn kernel_main() -> ! {
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
