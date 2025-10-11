#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]

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

    framebuffer.fill(0, 0, 100, 300, framebuffer.color(255, 0, 0));
    framebuffer.fill(100, 0, 100, 300, framebuffer.color(0, 255, 0));
    framebuffer.fill(200, 0, 100, 300, framebuffer.color(0, 0, 255));

    hlt()
}

#[panic_handler]
fn panic(#[expect(unused)] info: &PanicInfo<'_>) -> ! {
    hlt()
}

fn hlt() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}
