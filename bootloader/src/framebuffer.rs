use crate::efi;
use core::{arch::asm, cell::SyncUnsafeCell};
use utf16_literal::utf16;

enum FrameBufferFormat {
    Rgb,
    Bgr,
}

pub struct Framebuffer {
    format: FrameBufferFormat,
    pixels_base: *mut FramebufferColor,
    pixels_width: usize,
    pixels_height: usize,
    pixels_per_scanline: usize,
}

unsafe impl Send for Framebuffer {}
unsafe impl Sync for Framebuffer {}

impl Framebuffer {
    pub fn color(&self, r: u8, g: u8, b: u8) -> FramebufferColor {
        FramebufferColor(u32::from_ne_bytes(match self.format {
            FrameBufferFormat::Rgb => [r, g, b, 0x00],
            FrameBufferFormat::Bgr => [b, g, r, 0x00],
        }))
    }

    pub fn width(&self) -> usize {
        self.pixels_width
    }

    pub fn height(&self) -> usize {
        self.pixels_height
    }

    /// # Safety
    /// `x` and `y` must be in-bounds
    pub unsafe fn set_pixel_unchecked(&self, x: usize, y: usize, color: FramebufferColor) {
        let pixel = unsafe { self.pixels_base.add(x + y * self.pixels_per_scanline) };
        unsafe { pixel.write_volatile(color) };
    }

    pub fn set_pixel(&self, x: usize, y: usize, color: FramebufferColor) {
        if x < self.pixels_width && y < self.pixels_height {
            unsafe { self.set_pixel_unchecked(x, y, color) };
        }
    }

    pub fn fill(
        &self,
        left: usize,
        top: usize,
        width: usize,
        height: usize,
        color: FramebufferColor,
    ) {
        let top = top.min(self.pixels_height);
        let bottom = top.saturating_add(height).min(self.pixels_height);
        let left = left.min(self.pixels_width);
        let right = left.saturating_add(width).min(self.pixels_width);
        for y in top..bottom {
            for x in left..right {
                unsafe { self.set_pixel_unchecked(x, y, color) };
            }
        }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct FramebufferColor(u32);

static FRAMEBUFFER: SyncUnsafeCell<Framebuffer> = SyncUnsafeCell::new(Framebuffer {
    format: FrameBufferFormat::Rgb,
    pixels_base: core::ptr::null_mut(),
    pixels_width: 0,
    pixels_height: 0,
    pixels_per_scanline: 0,
});

pub fn framebuffer() -> &'static Framebuffer {
    unsafe { &*FRAMEBUFFER.get() }
}

pub unsafe fn init_framebuffer(system_table: efi::SystemTable) -> efi::Status {
    let gop = unsafe { system_table.locate_gop()? };
    let (_, info) =
        unsafe { gop.query_mode(gop.mode().map(|mode| (*mode.as_ptr()).mode).unwrap_or(0))? };

    let format = match info.pixel_format {
        efi::GraphicsPixelFormat::PixelRedGreenBlueReserved8BitPerColor => FrameBufferFormat::Rgb,
        efi::GraphicsPixelFormat::PixelBlueGreenRedReserved8BitPerColor => FrameBufferFormat::Bgr,
        efi::GraphicsPixelFormat::PixelBitMask => unsafe {
            system_table
                .con_out_print(utf16!("bitmask pixel format is not supported\r\n\0").as_ptr())?;
            for _ in 0..999999999 {
                asm!("nop");
            }
            return Err(efi::Error::UNSUPPORTED);
        },
        efi::GraphicsPixelFormat::PixelBltOnly => unsafe {
            system_table
                .con_out_print(utf16!("blt pixel format is not supported\r\n\0").as_ptr())?;
            for _ in 0..999999999 {
                asm!("nop");
            }
            return Err(efi::Error::UNSUPPORTED);
        },
    };

    unsafe { gop.set_mode(gop.mode().map(|mode| (*mode.as_ptr()).mode).unwrap_or(0))? };
    let mode = unsafe { gop.mode().unwrap_unchecked().read() };

    unsafe {
        *FRAMEBUFFER.get() = Framebuffer {
            format,
            pixels_base: mode.frame_buffer_base.cast(),
            pixels_width: info.horizontal_resolution as _,
            pixels_height: info.vertical_resolution as _,
            pixels_per_scanline: info.pixels_per_scan_line as _,
        };
    }
    Ok(())
}
