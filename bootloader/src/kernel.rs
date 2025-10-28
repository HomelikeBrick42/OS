use crate::{
    drivers::{
        pic::{PIC1_DATA, PIC2_DATA, remap_pic},
        ps2_keyboard::{KEYBOARD_STATE, Key, keyboard_handler, setup_keyboard},
        ps2_mouse::{MOUSE_STATE, mouse_handler, setup_mouse},
    },
    framebuffer::{Color, framebuffer},
    gdt::setup_gdt,
    idt::{InterruptType, disable_interrupts, enable_interrupts, setup_idt, with_idt_entry},
    screen::{Pixels, Screen},
    text_writer::TextWriter,
    utils::{io_wait, outb},
};
use alloc::vec;
use core::fmt::Write;
use font::SPACE_MONO;

pub unsafe extern "win64" fn kernel_main() -> ! {
    unsafe { disable_interrupts() };

    let framebuffer = framebuffer();
    let mut pixels = Pixels::new(
        Color { r: 0, g: 0, b: 0 },
        framebuffer.width(),
        framebuffer.height(),
    );

    unsafe { setup_gdt() };
    unsafe { setup_idt() };

    unsafe { remap_pic(0x20, 0x28) };

    unsafe {
        with_idt_entry(0x21, |entry| {
            entry.set_handler(keyboard_handler, InterruptType::Interrupt);
        });
    }
    unsafe { setup_keyboard() };

    unsafe {
        with_idt_entry(0x2C, |entry| {
            entry.set_handler(mouse_handler, InterruptType::Interrupt);
        });
    }
    unsafe { setup_mouse() };

    unsafe { outb::<PIC1_DATA>(0b11111001) };
    io_wait();
    unsafe { outb::<PIC2_DATA>(0b11101111) };
    io_wait();

    unsafe { enable_interrupts() };

    let mut changed = true;
    let mut events = vec![];

    let mut mouse_x = 0usize;
    let mut mouse_y = 0usize;
    loop {
        KEYBOARD_STATE.with(|keyboard| {
            while let Some(event) = keyboard.next_event() {
                changed = true;
                if matches!(event.key, Key::Backspace) {
                    events.clear();
                }
                events.push(event);
            }
        });

        MOUSE_STATE.with(|mouse| {
            while let Some(event) = mouse.next_event() {
                changed = true;
                mouse_x = mouse_x
                    .saturating_add_signed(event.x_offset as isize)
                    .min(framebuffer.width());
                mouse_y = mouse_y
                    .saturating_add_signed(-(event.y_offset as isize))
                    .min(framebuffer.height());
            }
        });

        if changed {
            let background = Color {
                r: 50,
                g: 50,
                b: 50,
            };
            pixels.fill(0, 0, pixels.width(), pixels.height(), background);

            {
                let mut writer = TextWriter {
                    x: &mut 0,
                    y: &mut 0,
                    left_margin: 0,
                    text_color: Color {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                    background,
                    font: &SPACE_MONO,
                    screen: &mut pixels,
                };
                for event in &events {
                    writeln!(writer, "{event:?}").unwrap();
                }
            }

            pixels.fill(
                mouse_x.saturating_sub(5),
                mouse_y.saturating_sub(5),
                10,
                10,
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            );
            framebuffer.copy(&pixels, 0, 0);
        }
    }
}
