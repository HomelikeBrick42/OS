use crate::efi;
use core::{arch::asm, cell::SyncUnsafeCell};
use utf16_literal::utf16;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn add(self, other: Self) -> Self {
        Self {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
        }
    }

    pub const fn lerp(self, other: Self, t: u8) -> Self {
        self.scale(u8::MAX - t).add(other.scale(t))
    }

    pub const fn multiply(self, other: Self) -> Self {
        Self {
            r: ((self.r as u16 * other.r as u16) / u8::MAX as u16) as u8,
            g: ((self.g as u16 * other.g as u16) / u8::MAX as u16) as u8,
            b: ((self.b as u16 * other.b as u16) / u8::MAX as u16) as u8,
        }
    }

    pub const fn scale(self, brightness: u8) -> Self {
        self.multiply(Color {
            r: brightness,
            g: brightness,
            b: brightness,
        })
    }
}

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
    pub fn color(&self, color: Color) -> FramebufferColor {
        FramebufferColor(u32::from_ne_bytes(match self.format {
            FrameBufferFormat::Rgb => [color.r, color.g, color.b, 0x00],
            FrameBufferFormat::Bgr => [color.b, color.g, color.r, 0x00],
        }))
    }

    pub fn base(&self) -> usize {
        self.pixels_base.addr()
    }

    pub fn size(&self) -> usize {
        self.pixels_height * self.pixels_per_scanline
    }

    pub fn width(&self) -> usize {
        self.pixels_width
    }

    pub fn height(&self) -> usize {
        self.pixels_height
    }

    unsafe fn set_pixel_unchecked(&self, x: usize, y: usize, color: FramebufferColor) {
        let pixel = unsafe { self.pixels_base.add(x + y * self.pixels_per_scanline) };
        unsafe { pixel.write(color) };
    }

    pub fn set_pixel(&self, x: usize, y: usize, color: FramebufferColor) {
        if x < self.pixels_width && y < self.pixels_height {
            unsafe { self.set_pixel_unchecked(x, y, color) };
            unsafe { asm!("/* {0} */", in(reg) self.pixels_base, options(nostack)) };
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
        unsafe { asm!("/* {0} */", in(reg) self.pixels_base, options(nostack)) };
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
        efi::GraphicsPixelFormat::RedGreenBlueReserved8BitPerColor => FrameBufferFormat::Rgb,
        efi::GraphicsPixelFormat::BlueGreenRedReserved8BitPerColor => FrameBufferFormat::Bgr,
        efi::GraphicsPixelFormat::BitMask => unsafe {
            system_table
                .con_out_print(utf16!("bitmask pixel format is not supported\r\n\0").as_ptr())?;
            for _ in 0..999999999 {
                asm!("nop");
            }
            return Err(efi::Error::UNSUPPORTED);
        },
        efi::GraphicsPixelFormat::BltOnly => unsafe {
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
