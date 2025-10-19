#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

use crate::{
    framebuffer::{Color, framebuffer, init_framebuffer},
    gdt::setup_gdt,
    page_allocator::{init_page_allocator, with_page_allocator},
    paging::{enable_paging, init_paging_and_identity_map_all_pages_from_page_allocator, map_page},
    text_writer::TextWriter,
    utils::{disable_interrupts, hlt},
};
use core::panic::PanicInfo;
use core::{fmt::Write, num::NonZeroUsize};
use font::SPACE_MONO;

pub mod efi;
pub mod framebuffer;
pub mod gdt;
pub mod page_allocator;
pub mod paging;
pub mod text_writer;
pub mod utils;

#[unsafe(no_mangle)]
unsafe extern "efiapi" fn efi_main(
    image_handle: efi::Handle,
    system_table: efi::SystemTable,
) -> efi::Status {
    unsafe { init_framebuffer(system_table)? };

    let framebuffer = framebuffer();
    let width = framebuffer.width();
    let height = framebuffer.height();

    let background = Color {
        r: 20,
        g: 20,
        b: 20,
    };
    framebuffer.fill(0, 0, width, height, framebuffer.color(background));

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

    // get memory map
    let mut memory_map_size = 0;
    let mut memory_map = core::ptr::null_mut();
    let mut map_key = 0;
    let mut memory_descriptor_size = 0;
    let mut memory_descritor_version = 0;
    loop {
        let old_size = memory_map_size;
        match unsafe {
            system_table.get_memory_map(
                &mut memory_map_size,
                memory_map,
                &mut map_key,
                &mut memory_descriptor_size,
                &mut memory_descritor_version,
            )
        } {
            Err(efi::Error::BUFFER_TOO_SMALL) => {
                if !memory_map.is_null() {
                    unsafe { system_table.free_pages(memory_map.cast(), old_size.div_ceil(4096))? };
                }
                memory_map = unsafe {
                    system_table
                        .allocate_pages(
                            efi::AllocateType::AnyPages,
                            efi::MemoryType::LoaderData,
                            memory_map_size.div_ceil(4096),
                        )?
                        .cast()
                };
                continue;
            }
            result => break result?,
        }
    }
    assert!(memory_map_size.is_multiple_of(memory_descriptor_size));

    // exit boot services
    unsafe { system_table.exit_boot_services(image_handle, map_key)? };

    unsafe { disable_interrupts() };
    unsafe { setup_gdt() };

    unsafe {
        init_page_allocator(
            memory_map,
            memory_map_size,
            memory_descriptor_size,
            &mut text_writer,
        )
    };

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

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    let framebuffer = framebuffer();

    let background = Color { r: 255, g: 0, b: 0 };
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

    if let Some(location) = info.location() {
        _ = write!(text_writer, "{}: ", location);
    }
    _ = writeln!(text_writer, "{}", info.message());

    hlt()
}
