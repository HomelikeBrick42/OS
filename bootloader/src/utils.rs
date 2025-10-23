use core::arch::asm;
use font::SPACE_MONO;
use crate::{framebuffer::{framebuffer, Color}, text_writer::TextWriter};

pub fn hlt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}

pub fn error_screen<R>(f: impl FnOnce(&mut TextWriter<'_>) -> R) -> R {
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
        x: &mut 0,
        y: &mut 0,
        left_margin: 0,
        text_color: Color {
            r: 255,
            g: 255,
            b: 255,
        },
        background,
        font: &SPACE_MONO,
        framebuffer,
    };

    f(&mut text_writer)
}
