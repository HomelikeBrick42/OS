use crate::framebuffer::FramebufferColor;

pub trait Screen: Send + Sync {
    fn width(&self) -> usize;
    fn height(&self) -> usize;

    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> FramebufferColor;
    unsafe fn set_pixel_unchecked(&mut self, x: usize, y: usize, color: FramebufferColor);

    fn present(&mut self);

    fn get_pixel(&self, x: usize, y: usize) -> Option<FramebufferColor> {
        if x < self.width() && y < self.height() {
            Some(unsafe { self.get_pixel_unchecked(x, y) })
        } else {
            None
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: FramebufferColor) {
        if x < self.width() && y < self.height() {
            unsafe { self.set_pixel_unchecked(x, y, color) };
        }
    }

    fn fill(
        &mut self,
        left: usize,
        top: usize,
        width: usize,
        height: usize,
        color: FramebufferColor,
    ) {
        let screen_width = self.width();
        let screen_height = self.height();

        let top = top.min(screen_height);
        let bottom = top.saturating_add(height).min(screen_height);
        let left = left.min(screen_width);
        let right = left.saturating_add(width).min(screen_width);

        for y in top..bottom {
            for x in left..right {
                unsafe { self.set_pixel_unchecked(x, y, color) };
            }
        }
    }

    fn copy(&mut self, source: &dyn Screen) {
        assert_eq!(self.width(), source.width());
        assert_eq!(self.height(), source.height());

        for y in 0..self.height() {
            for x in 0..self.width() {
                unsafe { self.set_pixel_unchecked(x, y, source.get_pixel_unchecked(x, y)) };
            }
        }
    }
}

pub struct NoopScreen;

impl NoopScreen {
    pub const fn get_static() -> &'static mut NoopScreen {
        unsafe { &mut *core::ptr::dangling_mut() }
    }
}

impl Screen for NoopScreen {
    fn width(&self) -> usize {
        0
    }

    fn height(&self) -> usize {
        0
    }

    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> FramebufferColor {
        _ = x;
        _ = y;
        unsafe { core::hint::unreachable_unchecked() }
    }

    unsafe fn set_pixel_unchecked(&mut self, x: usize, y: usize, color: FramebufferColor) {
        _ = x;
        _ = y;
        _ = color;
        unsafe { core::hint::unreachable_unchecked() }
    }

    fn present(&mut self) {}
}
