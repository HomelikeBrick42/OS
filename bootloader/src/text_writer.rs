use core::fmt::Write;

use crate::framebuffer::{Color, Framebuffer, FramebufferColor};
use font::Font;

pub struct TextWriter<'a> {
    pub x: &'a mut usize,
    pub y: &'a mut usize,
    pub left_margin: usize,
    pub text_color: Color,
    pub background: Color,
    pub font: &'a Font<'a>,
    pub framebuffer: &'a Framebuffer,
}

impl Write for TextWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.chars().try_for_each(|c| self.write_char(c))
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        if let Ok(char_index) = self
            .font
            .chars
            .binary_search_by_key(&(c as u32), |char| char.id)
        {
            let char = &self.font.chars[char_index];
            let page = &self.font.pages[char.page as usize];

            for yoffset in 0..char.height as usize {
                for xoffset in 0..char.width as usize {
                    let brightness = page.brightnesses[(char.x as usize + xoffset)
                        + (char.y as usize + yoffset) * page.width as usize];
                    let color =
                        FramebufferColor::new(self.background.lerp(self.text_color, brightness));
                    self.framebuffer.set_pixel(
                        *self.x + xoffset + char.xoffset as usize,
                        *self.y + yoffset + char.yoffset as usize,
                        color,
                    );
                }
            }

            *self.x += char.xadvance as usize;
        }

        if c == '\n' {
            *self.x = self.left_margin;
            *self.y += self.font.common.line_height as usize;
        }

        Ok(())
    }
}
