use crate::{
    framebuffer::{Color, framebuffer},
    text_writer::TextWriter,
};
use core::fmt::Write;
use font::{Font, SPACE_MONO};

#[allow(unused)]
macro_rules! print {
    ($($tokens:tt)*) => {
        match ::core::format_args!($($tokens)*) {
            fmt => $crate::print::with_global_printer(|printer| {
                ::core::fmt::Write::write_fmt(printer, fmt)
            }).unwrap(),
        }
    };
}
#[allow(unused)]
pub(crate) use print;

#[allow(unused)]
macro_rules! println {
    ($($tokens:tt)*) => {
        match ::core::format_args_nl!($($tokens)*) {
            fmt => $crate::print::with_global_printer(|printer| {
                ::core::fmt::Write::write_fmt(printer, fmt)
            }).unwrap(),
        }
    };
}
#[allow(unused)]
pub(crate) use println;

pub struct GlobalPrinter {
    pub x: usize,
    pub y: usize,
    pub left_margin: usize,
    pub text_color: Color,
    pub background_color: Color,
    pub font: &'static Font<'static>,
}

impl Write for GlobalPrinter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        TextWriter {
            x: &mut self.x,
            y: &mut self.y,
            left_margin: self.left_margin,
            text_color: self.text_color,
            background: self.background_color,
            font: self.font,
            framebuffer: framebuffer(),
        }
        .write_str(s)
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        TextWriter {
            x: &mut self.x,
            y: &mut self.y,
            left_margin: self.left_margin,
            text_color: self.text_color,
            background: self.background_color,
            font: self.font,
            framebuffer: framebuffer(),
        }
        .write_char(c)
    }

    fn write_fmt(&mut self, args: core::fmt::Arguments<'_>) -> core::fmt::Result {
        TextWriter {
            x: &mut self.x,
            y: &mut self.y,
            left_margin: self.left_margin,
            text_color: self.text_color,
            background: self.background_color,
            font: self.font,
            framebuffer: framebuffer(),
        }
        .write_fmt(args)
    }
}

static GLOBAL_PRINTER: spin::Mutex<GlobalPrinter> = spin::Mutex::new(GlobalPrinter {
    x: 0,
    y: 0,
    left_margin: 0,
    text_color: Color {
        r: 255,
        g: 255,
        b: 255,
    },
    background_color: Color {
        r: 50,
        g: 50,
        b: 50,
    },
    font: &SPACE_MONO,
});

pub fn with_global_printer<R>(f: impl FnOnce(&mut GlobalPrinter) -> R) -> R {
    f(&mut GLOBAL_PRINTER.lock())
}
