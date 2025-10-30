use crate::framebuffer::{Color, FramebufferColor};
use alloc::{vec, vec::Vec};

pub trait Screen {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    unsafe fn set_pixel_unchecked(&mut self, x: usize, y: usize, color: Color);
    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> Color;

    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x < self.width() && y < self.height() {
            unsafe { self.set_pixel_unchecked(x, y, color) };
        }
    }

    fn get_pixel(&self, x: usize, y: usize) -> Option<Color> {
        if x < self.width() && y < self.height() {
            Some(unsafe { self.get_pixel_unchecked(x, y) })
        } else {
            None
        }
    }

    fn fill(&mut self, left: usize, top: usize, width: usize, height: usize, color: Color) {
        let pixels_width = self.width();
        let pixels_height = self.height();

        let top = top.min(pixels_height);
        let bottom = top.saturating_add(height).min(pixels_height);
        let left = left.min(pixels_width);
        let right = left.saturating_add(width).min(pixels_width);

        for y in top..bottom {
            for x in left..right {
                unsafe { self.set_pixel_unchecked(x, y, color) };
            }
        }
    }

    fn copy(&mut self, screen: &dyn Screen, left: usize, top: usize) {
        let width = self.width();
        let height = self.height();
        let screen_width = screen.width();
        let screen_height = screen.height();

        let top = top.min(height);
        let bottom = top.saturating_add(screen_height).min(height);
        let left = left.min(width);
        let right = left.saturating_add(screen_width).min(width);

        for y in top..bottom {
            for x in left..right {
                unsafe {
                    self.set_pixel_unchecked(x, y, screen.get_pixel_unchecked(x - left, y - top));
                }
            }
        }
    }
}

pub struct Pixels {
    pixels: Vec<Color>,
    width: usize,
    height: usize,
}

impl Pixels {
    pub const fn zero_size() -> Self {
        Self {
            pixels: vec![],
            width: 0,
            height: 0,
        }
    }

    pub fn new(color: Color, width: usize, height: usize) -> Self {
        Self {
            pixels: vec![color; width * height],
            width,
            height,
        }
    }
}

impl Screen for Pixels {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    unsafe fn set_pixel_unchecked(&mut self, x: usize, y: usize, color: Color) {
        unsafe {
            *self.pixels.get_unchecked_mut(x + y * self.width) = color;
        }
    }

    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> Color {
        unsafe { *self.pixels.get_unchecked(x + y * self.width) }
    }
}

pub struct FramebufferColorPixels {
    pixels: Vec<FramebufferColor>,
    width: usize,
    height: usize,
}

impl FramebufferColorPixels {
    pub const fn zero_size() -> Self {
        Self {
            pixels: vec![],
            width: 0,
            height: 0,
        }
    }

    pub fn new(color: FramebufferColor, width: usize, height: usize) -> Self {
        Self {
            pixels: vec![color; width * height],
            width,
            height,
        }
    }

    pub unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> FramebufferColor {
        unsafe { *self.pixels.get_unchecked(x + y * self.width) }
    }
}

impl Screen for FramebufferColorPixels {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    unsafe fn set_pixel_unchecked(&mut self, x: usize, y: usize, color: Color) {
        unsafe {
            *self.pixels.get_unchecked_mut(x + y * self.width) = FramebufferColor::new(color);
        }
    }

    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> Color {
        unsafe { self.pixels.get_unchecked(x + y * self.width).color() }
    }
}
