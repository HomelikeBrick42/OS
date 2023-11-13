use core::{
    fmt::Write,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::framebuffer::Framebuffer;

pub struct TextWriter<Data> {
    pub framebuffer: Framebuffer,
    pub font: psf2::Font<Data>,
    pub cursor_x: AtomicUsize,
    pub cursor_x_begin: usize,
    pub cursor_x_end: Option<usize>,
    pub cursor_y: AtomicUsize,
    pub foreground_color: (u8, u8, u8),
    pub background_color: Option<(u8, u8, u8)>,
}

impl<Data> Write for TextWriter<Data>
where
    Data: AsRef<[u8]>,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        (&*self).write_str(s)
    }

    #[inline]
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        (&*self).write_char(c)
    }

    #[inline]
    fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        (&*self).write_fmt(args)
    }
}

impl<Data> Write for &TextWriter<Data>
where
    Data: AsRef<[u8]>,
{
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            (*self).write_char(c)?;
        }
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        let glyph = if c.is_ascii() {
            unsafe { self.font.get_ascii(c as u8).unwrap_unchecked() }
        } else {
            unsafe { self.font.get_ascii(b'?').unwrap_unchecked() }
        };

        match c {
            '\n' => {
                self.cursor_x.store(self.cursor_x_begin, Ordering::Relaxed);
                self.cursor_y
                    .fetch_add(self.font.height() as usize, Ordering::Relaxed);
            }
            '\r' => self.cursor_x.store(0, Ordering::Relaxed),
            _ => {
                if self.cursor_x_end.is_some_and(|cursor_x_end| {
                    self.cursor_x.load(Ordering::Relaxed) >= cursor_x_end
                }) {
                    self.cursor_x.store(self.cursor_x_begin, Ordering::Relaxed);
                    self.cursor_y
                        .fetch_add(self.font.height() as usize, Ordering::Relaxed);
                }
                for (y_offset, row) in glyph.into_iter().enumerate() {
                    for (x_offset, pixel) in row.into_iter().enumerate() {
                        if pixel {
                            if !self.framebuffer.draw_pixel(
                                self.cursor_x
                                    .load(Ordering::Relaxed)
                                    .saturating_add(x_offset),
                                self.cursor_y
                                    .load(Ordering::Relaxed)
                                    .saturating_add(y_offset),
                                self.foreground_color,
                            ) {
                                break;
                            }
                        } else if let Some(background_color) = self.background_color {
                            if !self.framebuffer.draw_pixel(
                                self.cursor_x
                                    .load(Ordering::Relaxed)
                                    .saturating_add(x_offset),
                                self.cursor_y
                                    .load(Ordering::Relaxed)
                                    .saturating_add(y_offset),
                                background_color,
                            ) {
                                break;
                            }
                        }
                    }
                }
                self.cursor_x
                    .fetch_add(self.font.width() as usize, Ordering::Relaxed);
            }
        }

        Ok(())
    }
}
