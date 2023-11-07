#![no_std]
#![no_main]
#![feature(try_trait_v2, arbitrary_self_types)]
#![allow(
    clippy::too_many_arguments,
    clippy::missing_safety_doc,
    clippy::not_unsafe_ptr_arg_deref
)]
#![deny(rust_2018_idioms, unsafe_op_in_unsafe_fn)]

pub mod efi;

use core::{arch::asm, ffi::c_void, panic::PanicInfo};
use utf16_lit::utf16_null;

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

        #[derive(Clone, Copy)]
        enum PixelFormat {
            Rgb,
            Bgr,
        }

        let (width, height) = ((*info).width as usize, (*info).height as usize);
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

        #[inline]
        unsafe fn set_pixel(
            gop: *mut efi::Gop,
            x: usize,
            y: usize,
            pixel_format: PixelFormat,
            color: (u8, u8, u8),
        ) {
            unsafe {
                let mode = (*gop).mode;
                let info = (*mode).info;
                let pixel = ((*mode).fb_base as *mut [u8; 4])
                    .add(x + y * (*info).pixels_per_scanline as usize);
                *pixel = match pixel_format {
                    PixelFormat::Rgb => [color.0, color.1, color.2, 0x00],
                    PixelFormat::Bgr => [color.2, color.1, color.0, 0x00],
                };
            }
        }

        for x in 0..width {
            for y in 0..height {
                set_pixel(gop, x, y, pixel_format, (255, 0, 0));
            }
        }

        loop {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
