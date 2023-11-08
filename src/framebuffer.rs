#[derive(Clone, Copy)]
pub enum PixelFormat {
    Rgb,
    Bgr,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Framebuffer {
    pixels: *mut (),
    width: usize,
    height: usize,
    pixels_per_scanline: usize,
    pixel_format: PixelFormat,
}

unsafe impl Sync for Framebuffer {}
unsafe impl Send for Framebuffer {}

impl Framebuffer {
    #[inline]
    pub const unsafe fn new(
        pixels: *mut (),
        width: usize,
        height: usize,
        pixels_per_scanline: usize,
        pixel_format: PixelFormat,
    ) -> Framebuffer {
        Self {
            pixels,
            width,
            height,
            pixels_per_scanline,
            pixel_format,
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn draw_pixel(&self, x: usize, y: usize, color: (u8, u8, u8)) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }

        unsafe {
            // hopefully the compiler cant optimise this out
            core::ptr::write_volatile(
                self.pixels
                    .cast::<[u8; 4]>()
                    .add(x + y * self.pixels_per_scanline)
                    .cast::<[u8; 3]>(),
                match self.pixel_format {
                    PixelFormat::Rgb => [color.0, color.1, color.2],
                    PixelFormat::Bgr => [color.2, color.1, color.0],
                },
            );
        }

        true
    }

    #[inline]
    pub fn draw_rect(&self, x: usize, y: usize, width: usize, height: usize, color: (u8, u8, u8)) {
        for y in y..y.saturating_add(height).min(height) {
            for x in x..x.saturating_add(width).min(width) {
                self.draw_pixel(x, y, color);
            }
        }
    }
}
