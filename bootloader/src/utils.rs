use crate::{
    framebuffer::{Color, framebuffer},
    text_writer::TextWriter,
};
use core::arch::asm;
use font::SPACE_MONO;

pub fn hlt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}

pub unsafe fn inb(port: u16) -> u8 {
    let value;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack)
        );
    }
    value
}

pub unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack)
        );
    }
}

pub fn io_wait() {
    unsafe {
        asm!(
            "out 0x80, al",
            in("al") 0u8,
            options(nomem, nostack)
        );
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
