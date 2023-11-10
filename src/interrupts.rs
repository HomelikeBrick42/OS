use core::arch::asm;
use core::fmt::Write;

use crate::{get_screen_framebuffer, text_writer::TextWriter};

#[naked]
pub unsafe extern "C" fn page_fault_thunk() {
    unsafe {
        asm!(
            "
sub rsp, 128
push rax
push rcx
push rdx
push r8
push r9
push r10
push r11
push 0
call {page_fault}
pop r11 // not a mistake
pop r11
pop r10
pop r9
pop r8
pop rdx
pop rcx
pop rax
add rsp, 128
iretq
",
            page_fault = sym page_fault,
            options(noreturn),
        );
    }
}

unsafe extern "win64" fn page_fault() {
    if let Ok(font) = psf2::Font::new(include_bytes!("./zap-light24.psf")) {
        let framebuffer = get_screen_framebuffer();
        let mut writer = TextWriter {
            framebuffer,
            font,
            cursor_x: 0,
            cursor_x_begin: 0,
            cursor_x_end: Some(framebuffer.width()),
            cursor_y: 0,
            foreground_color: (255, 0, 0),
            background_color: Some((0, 0, 0)),
        };
        _ = writeln!(writer, "page fault");
    }

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
