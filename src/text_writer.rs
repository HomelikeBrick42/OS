use core::fmt::Write;

use crate::framebuffer::Framebuffer;

pub struct TextWriter<Data> {
    pub framebuffer: Framebuffer,
    pub font: psf2::Font<Data>,
    pub cursor_x: usize,
    pub cursor_x_begin: usize,
    pub cursor_x_end: Option<usize>,
    pub cursor_y: usize,
}

impl<Data> Write for TextWriter<Data>
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
                self.cursor_x = self.cursor_x_begin;
                self.cursor_y += self.font.height() as usize;
            }
            '\r' => self.cursor_x = 0,
            _ => {
                if self
                    .cursor_x_end
                    .is_some_and(|cursor_x_end| self.cursor_x >= cursor_x_end)
                {
                    self.cursor_x = self.cursor_x_begin;
                    self.cursor_y += self.font.height() as usize;
                }
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
