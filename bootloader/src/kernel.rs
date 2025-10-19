use crate::{
    framebuffer::{Color, framebuffer},
    gdt::setup_gdt,
    page_allocator::with_page_allocator,
    paging::{enable_paging, init_paging_and_identity_map_all_pages_from_page_allocator, map_page},
    text_writer::TextWriter,
    utils::hlt,
};
use core::{fmt::Write, num::NonZeroUsize};
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

    unsafe { init_paging_and_identity_map_all_pages_from_page_allocator() };

    // make sure to identity map the framebuffer
    {
        assert_eq!(framebuffer.base() % 4096, 0);
        let base = framebuffer.base();
        for i in 0..framebuffer.size().div_ceil(4096) {
            let page = base + i * 4096;
            unsafe { map_page(page, page) };
        }
    }

    unsafe { enable_paging() };

    with_page_allocator(|alloc| {
        for i in 0..10 {
            let align = NonZeroUsize::MIN;
            let size = 4096;
            let addr = alloc.allocate(align, size);
            writeln!(text_writer, "Allocated: {:x?}", addr).unwrap();
            if i % 2 == 1
                && let Some(addr) = addr
            {
                unsafe { alloc.free(addr, size) };
            }
        }
    });

    hlt()
}
