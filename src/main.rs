#![no_std]
#![no_main]
#![feature(try_trait_v2, arbitrary_self_types, sync_unsafe_cell)]
#![allow(
    clippy::too_many_arguments,
    clippy::missing_safety_doc,
    clippy::not_unsafe_ptr_arg_deref
)]
#![deny(rust_2018_idioms, unsafe_op_in_unsafe_fn)]

pub mod efi;
pub mod framebuffer;

use core::{arch::asm, cell::SyncUnsafeCell, ffi::c_void, panic::PanicInfo};
use framebuffer::Framebuffer;
use utf16_lit::utf16_null;

use crate::framebuffer::PixelFormat;

#[no_mangle]
pub unsafe extern "system" fn efi_main(
    _image_handle: efi::Handle,
    system_table: *mut efi::SystemTable,
) -> efi::Status {
    unsafe {
        let mut gop: *mut efi::Gop = core::ptr::null_mut();
        (*system_table).boottime.locate_protocol(
            &efi::Guid::GRAPHICS_OUTPUT_PROTOCOL,
            core::ptr::null_mut(),
            &mut gop as *mut *mut _ as *mut *mut c_void,
        )?;

        let mut info = core::ptr::null_mut();
        let mut size_of_info = 0;
        gop.query_mode(
            if !(*gop).mode.is_null() {
                (*(*gop).mode).mode
            } else {
                0
            },
            &mut size_of_info,
            &mut info,
        )?;
        gop.set_mode(if !(*gop).mode.is_null() {
            (*(*gop).mode).mode
        } else {
            0
        })?;

        let (width, height, pixels_per_scanline) = (
            (*info).width as usize,
            (*info).height as usize,
            (*info).pixels_per_scanline as usize,
        );
        let pixel_format = match (*info).pixel_format {
            efi::Gop::GOT_RGBA8 => PixelFormat::Rgb,
            efi::Gop::GOT_BGRA8 => PixelFormat::Bgr,
            _ => {
                (*system_table)
                    .console_out
                    .output_string(utf16_null!("Unsupported pixel format!\r\n").as_ptr())?;
                return efi::Status::UNSUPPORTED;
            }
        };

        // Sound, because nothing is trying to get a reference to the framebuffer before here
        FRAMEBUFFER.get().write(Framebuffer::new(
            (*(*gop).mode).fb_base,
            width,
            height,
            pixels_per_scanline,
            pixel_format,
        ));
    }

    let framebuffer = get_screen_framebuffer();
    framebuffer.draw_rect(
        0,
        0,
        framebuffer.width(),
        framebuffer.height(),
        (51, 51, 51),
    );

    unsafe {
        loop {
            asm!("hlt");
        }
    }
}

static FRAMEBUFFER: SyncUnsafeCell<Framebuffer> = unsafe {
    SyncUnsafeCell::new(Framebuffer::new(
        core::ptr::null_mut(),
        0,
        0,
        0,
        PixelFormat::Rgb,
    ))
};

/// this can be called after GOP is initialized, which is basically anytime except at the start of efi_main
pub fn get_screen_framebuffer() -> Framebuffer {
    unsafe { *FRAMEBUFFER.get() }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
