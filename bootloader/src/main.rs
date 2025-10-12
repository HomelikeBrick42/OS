#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

use font::{Font, SPACE_MONO};

use crate::framebuffer::{Color, Framebuffer, framebuffer, init_framebuffer};
use core::{arch::asm, panic::PanicInfo};

pub mod efi;
pub mod framebuffer;

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

    let (mut x, mut y) = (0, 0);

    let font = &SPACE_MONO;
    let mut char = |c: char| {
        draw_char(
            &mut x,
            &mut y,
            c,
            Color {
                r: 255,
                g: 255,
                b: 255,
            },
            background,
            font,
            framebuffer,
        )
    };

    char('H');
    char('e');
    char('l');
    char('l');
    char('o');
    char(',');
    char(' ');
    char('W');
    char('o');
    char('r');
    char('l');
    char('d');
    char('!');

    hlt()
}

fn draw_char(
    x: &mut usize,
    y: &mut usize,
    c: char,
    color: Color,
    background: Color,
    font: &Font<'_>,
    framebuffer: &Framebuffer,
) {
    let Ok(char_index) = font.chars.binary_search_by_key(&(c as u32), |char| char.id) else {
        return;
    };
    let char = &font.chars[char_index];
    let page = &font.pages[char.page as usize];

    for yoffset in 0..char.height as usize {
        for xoffset in 0..char.width as usize {
            let brightness = page.brightnesses
                [(char.x as usize + xoffset) + (char.y as usize + yoffset) * page.width as usize];
            let color = framebuffer.color(background.lerp(color, brightness));
            framebuffer.set_pixel(
                *x + xoffset + char.xoffset as usize,
                *y + yoffset + char.yoffset as usize,
                color,
            );
        }
    }
    *x += char.xadvance as usize;
}

#[panic_handler]
fn panic(#[expect(unused)] info: &PanicInfo<'_>) -> ! {
    let framebuffer = framebuffer();
    framebuffer.fill(
        0,
        0,
        framebuffer.width(),
        framebuffer.height(),
        framebuffer.color(Color { r: 255, g: 0, b: 0 }),
    );
    hlt()
}

fn hlt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}
