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
    #[expect(unused)] image_handle: efi::Handle,
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

    writeln!(text_writer, "Hello, World!").unwrap();
    writeln!(text_writer, "Some more text.").unwrap();
    writeln!(text_writer, "And a number: {}", 1 + 2).unwrap();

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
