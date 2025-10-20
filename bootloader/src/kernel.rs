use crate::{
    framebuffer::{Color, framebuffer},
    gdt::setup_gdt,
    page_allocator::with_page_allocator,
    text_writer::TextWriter,
    utils::hlt,
};
use core::fmt::Write;
use font::SPACE_MONO;

pub extern "win64" fn kernel_main() -> ! {
    let framebuffer = framebuffer();

    let background = Color {
        r: 20,
        g: 20,
        b: 20,
    };
    framebuffer.fill(
        0,
        0,
        framebuffer.width(),
        framebuffer.height(),
        framebuffer.color(background),
    );

    let mut text_writer = TextWriter {
        x: 0,
        y: 0,
        left_margin: 0,
        color: Color {
            r: 255,
            g: 255,
            b: 255,
        },
        background,
        font: &SPACE_MONO,
        framebuffer,
    };

    unsafe { setup_gdt() };

    with_page_allocator(|alloc| {
        for block in alloc.blocks() {
            writeln!(text_writer, "{block:x?}").unwrap();
        }

        writeln!(
            text_writer,
            "Total Memory: {} KiB",
            alloc.total_pages() * 4096 / 1024
        )
        .unwrap();
        writeln!(
            text_writer,
            "Allocated Memory: {} KiB",
            alloc.allocated_pages() * 4096 / 1024
        )
        .unwrap();
        writeln!(
            text_writer,
            "Free Memory: {} KiB",
            alloc.free_pages() * 4096 / 1024
        )
        .unwrap();
    });

    hlt()
}
