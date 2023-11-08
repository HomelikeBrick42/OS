#![no_std]
#![no_main]
#![feature(
    try_trait_v2,
    arbitrary_self_types,
    sync_unsafe_cell,
    panic_info_message
)]
#![allow(
    clippy::too_many_arguments,
    clippy::missing_safety_doc,
    clippy::not_unsafe_ptr_arg_deref
)]
#![deny(rust_2018_idioms, unsafe_op_in_unsafe_fn)]

pub mod efi;
pub mod framebuffer;

use crate::framebuffer::PixelFormat;
use core::{arch::asm, cell::SyncUnsafeCell, ffi::c_void, panic::PanicInfo};
use framebuffer::Framebuffer;
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
fn panic(info: &PanicInfo<'_>) -> ! {
    if let Some(message) = info.message() {
        if let Ok(font) = psf2::Font::new(include_bytes!("./zap-light24.psf")) {
            use core::fmt::Write;

            struct Writer<Data> {
                framebuffer: Framebuffer,
                font: psf2::Font<Data>,
                cursor_x: usize,
                cursor_y: usize,
            }

            impl<Data> Write for Writer<Data>
            where
                Data: AsRef<[u8]>,
            {
                fn write_str(&mut self, s: &str) -> core::fmt::Result {
                    for c in s.chars() {
                        self.write_char(c)?;
                    }
                    Ok(())
                }

                fn write_char(&mut self, c: char) -> core::fmt::Result {
                    let glyph = if c.is_ascii() {
                        unsafe { self.font.get_ascii(c as u8).unwrap_unchecked() }
                    } else {
                        unsafe { self.font.get_ascii(b'?').unwrap_unchecked() }
                    };

                    match c {
                        '\n' => {
                            self.cursor_x = 0;
                            self.cursor_y += self.font.height() as usize;
                        }
                        '\r' => self.cursor_x = 0,
                        _ => {
                            for (y_offset, row) in glyph.into_iter().enumerate() {
                                for (x_offset, pixel) in row.into_iter().enumerate() {
                                    if !self.framebuffer.draw_pixel(
                                        self.cursor_x.saturating_add(x_offset),
                                        self.cursor_y.saturating_add(y_offset),
                                        if pixel { (255, 255, 255) } else { (0, 0, 0) },
                                    ) {
                                        break;
                                    }
                                }
                            }
                            self.cursor_x += self.font.width() as usize
                        }
                    }

                    Ok(())
                }
            }

            let mut writer = Writer {
                framebuffer: get_screen_framebuffer(),
                font,
                cursor_x: 0,
                cursor_y: 0,
            };
            if let Some(location) = info.location() {
                _ = write!(writer, "{}: ", location);
            }
            _ = write!(writer, "{}", message);
        }
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
