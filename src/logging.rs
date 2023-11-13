use crate::text_writer::TextWriter;
use core::fmt::{Arguments, Result, Write};
use spin::Once;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::logging::_print(::core::format_args!($($arg)*)).unwrap();
    }};
}
pub use print;

#[macro_export]
macro_rules! println {
    () => {{
        $crate::logging::print!("\n")
    }};
    ($($arg:tt)*) => {{
        $crate::logging::_print(::core::format_args!($($arg)*)).unwrap();
        $crate::logging::print!("\n")
    }};
}
pub use println;

static TEXT_WRITER: Once<TextWriter<&'static [u8]>> = Once::new();

pub(crate) unsafe fn init_text_writer(writer: TextWriter<&'static [u8]>) {
    TEXT_WRITER.call_once(|| writer);
}

pub fn _print(args: Arguments<'_>) -> Result {
    TEXT_WRITER
        .get()
        .expect("the text writer should already be initialized")
        .write_fmt(args)?;
    Ok(())
}
