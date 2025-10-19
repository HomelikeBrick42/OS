#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

use crate::{
    framebuffer::{Color, framebuffer, init_framebuffer},
    kernel::kernel_main,
    page_allocator::{init_page_allocator, with_page_allocator},
    text_writer::TextWriter,
    utils::{disable_interrupts, hlt},
};
use core::{arch::asm, fmt::Write};
use core::{num::NonZeroUsize, panic::PanicInfo};
use font::SPACE_MONO;

pub mod efi;
pub mod framebuffer;
pub mod gdt;
pub mod kernel;
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

    unsafe {
        init_page_allocator(
            memory_map,
            memory_map_size,
            memory_descriptor_size,
            &mut text_writer,
        );
    }

    unsafe {
        let stack_size = 4 * 1024 * 1024;
        let stack = with_page_allocator(|alloc| {
            alloc
                .allocate(const { NonZeroUsize::new(16).unwrap() }, stack_size)
                .expect("allocating the stack should succeed")
        });

        let stack_start = stack + stack_size;
        let _: extern "win64" fn() -> ! = kernel_main;
        asm!(
            "mov rsp, {stack_start}",
            "call {kernel_main}",
            stack_start = in(reg) stack_start,
            kernel_main = sym kernel_main,
            options(noreturn)
        )
    }
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
