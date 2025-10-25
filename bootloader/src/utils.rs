use crate::{
    framebuffer::{Color, framebuffer},
    text_writer::TextWriter,
};
use core::arch::asm;
use font::SPACE_MONO;

pub fn hlt() {
    unsafe { asm!("hlt", options(nomem, nostack)) };
}

pub unsafe fn inb<const PORT: u16>() -> u8 {
    let value;
    unsafe {
        asm!(
            "in al, {port}",
            port = const PORT,
            out("al") value,
            options(nomem, nostack)
        );
    }
    value
}

pub unsafe fn outb<const PORT: u16>(value: u8) {
    unsafe {
        asm!(
            "out {port}, al",
            port = const PORT,
            in("al") value,
            options(nomem, nostack)
        );
    }
}

pub fn io_wait() {
    unsafe { outb::<0x80>(0) };
}

pub fn get_flags() -> u64 {
    let flags;
    unsafe {
        asm!(
            "pushf",
            "pop rax",
            out("rax") flags,
            options(nomem)
        );
    }
    flags
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
