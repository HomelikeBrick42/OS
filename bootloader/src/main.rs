#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

use crate::{
    framebuffer::{Color, framebuffer, init_framebuffer},
    text_writer::TextWriter,
};
use core::fmt::Write;
use core::{arch::asm, panic::PanicInfo};
use font::SPACE_MONO;

pub mod efi;
pub mod framebuffer;
pub mod text_writer;

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

    let mut available_memory = 0;

    let memory_map_count = memory_map_size / memory_descriptor_size;
    for i in 0..memory_map_count {
        let memory_descriptor = unsafe { &*memory_map.byte_add(i * memory_descriptor_size) };
        if let efi::MemoryType::ConventionalMemory
        | efi::MemoryType::BootServicesCode
        | efi::MemoryType::BootServicesData = memory_descriptor.memory_type
        {
            available_memory += memory_descriptor.number_of_pages as usize * 4096;
        }
    }

    writeln!(
        text_writer,
        "Available Memory: {}kb",
        available_memory.div_ceil(1024)
    )
    .unwrap();

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

fn hlt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}
