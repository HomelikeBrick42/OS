use core::arch::asm;
use core::fmt::Write;

use crate::{
    get_screen_framebuffer, halt,
    io::{input_byte, output_byte, PIC1_COMMAND_PORT, PIC2_COMMAND_PORT, PIC_EOI},
    text_writer::TextWriter,
};

macro_rules! interrupt_handler {
    ($name:ident, $to_call:path) => {
        const _: unsafe extern "win64" fn() = $to_call; // make sure the signature is correct

        #[naked]
        pub unsafe extern "C" fn $name() {
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
sub rsp, 8
call {to_call}
add rsp, 8
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
                    to_call = sym $to_call,
                    options(noreturn),
                );
            }
        }
    };
}

interrupt_handler!(page_fault_handler, page_fault);
unsafe extern "win64" fn page_fault() {
    _ = try_with_writer((255, 0, 0), Some((0, 0, 0)), |w| {
        write!(w, "page fault detected")
    });
    halt()
}

interrupt_handler!(double_fault_handler, double_fault);
unsafe extern "win64" fn double_fault() {
    _ = try_with_writer((255, 0, 0), Some((0, 0, 0)), |w| {
        write!(w, "double fault detected")
    });
    halt()
}

interrupt_handler!(general_protection_fault_handler, general_protection);
unsafe extern "win64" fn general_protection() {
    _ = try_with_writer((255, 0, 0), Some((0, 0, 0)), |w| {
        write!(w, "general protection fault detected")
    });
    halt()
}

interrupt_handler!(keyboard_handler, keyboard);
unsafe extern "win64" fn keyboard() {
    unsafe {
        let scancode = input_byte(0x60);

        _ = try_with_writer((255, 255, 255), Some((0, 0, 0)), |w| {
            write!(w, "Keyboard input: {scancode}                      ")
        });

        // reset/end pic interrupt
        output_byte(PIC2_COMMAND_PORT, PIC_EOI);
        output_byte(PIC1_COMMAND_PORT, PIC_EOI);
    }
}

fn try_with_writer(
    foreground_color: (u8, u8, u8),
    background_color: Option<(u8, u8, u8)>,
    f: impl FnOnce(&mut TextWriter<&[u8]>) -> core::fmt::Result,
) -> core::fmt::Result {
    if let Ok(font) = psf2::Font::new(include_bytes!("./zap-light24.psf") as &[u8]) {
        let framebuffer = get_screen_framebuffer();
        let mut writer = TextWriter {
            framebuffer,
            font,
            cursor_x: 0,
            cursor_x_begin: 0,
            cursor_x_end: Some(framebuffer.width()),
            cursor_y: 0,
            foreground_color,
            background_color,
        };
        f(&mut writer)
    } else {
        Ok(())
    }
}
