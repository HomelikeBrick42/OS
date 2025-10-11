#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

use font::{Font, SPACE_MONO};

use crate::framebuffer::{framebuffer, init_framebuffer};
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
    framebuffer.fill(0, 0, width, height, framebuffer.color(0, 0, 0));

    draw_char(0, 0, 'a', &SPACE_MONO);

    hlt()
}

fn draw_char(x: usize, y: usize, c: char, font: &Font<'_>) {
    let framebuffer = framebuffer();

    let Ok(char_index) = font.chars.binary_search_by_key(&(c as u32), |char| char.id) else {
        return;
    };
    let char = &font.chars[char_index];
    let page = &font.pages[char.page as usize];

    for yoffset in 0..char.height as usize {
        for xoffset in 0..char.width as usize {
            let brightness = page.brightnesses
                [(char.x as usize + xoffset) + (char.y as usize + yoffset) * page.width as usize];
            if brightness != 0 {
                let color = framebuffer.color(255, 255, 255);
                framebuffer.set_pixel(x + xoffset, y + yoffset, color);
            }
        }
    }
}

#[panic_handler]
fn panic(#[expect(unused)] info: &PanicInfo<'_>) -> ! {
    let framebuffer = framebuffer();
    framebuffer.fill(
        0,
        0,
        framebuffer.width(),
        framebuffer.height(),
        framebuffer.color(255, 0, 0),
    );
    hlt()
}

fn hlt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}
