use alloc::vec;

use crate::{
    drivers::{
        pic::{PIC1_DATA, PIC2_DATA, remap_pic},
        ps2_keyboard::{KEYBOARD_STATE, Key, keyboard_handler, setup_keyboard},
        ps2_mouse::{MOUSE_STATE, mouse_handler, setup_mouse},
    },
    framebuffer::{Color, framebuffer},
    gdt::setup_gdt,
    idt::{
        InterruptType, disable_interrupts, enable_interrupts, setup_idt, with_disabled_interrupts,
        with_idt_entry,
    },
    print::{GLOBAL_PRINTER, println},
    screen::Pixels,
    utils::{hlt, io_wait, outb},
};
use core::cell::SyncUnsafeCell;

pub unsafe extern "win64" fn kernel_main() -> ! {
    unsafe { disable_interrupts() };

    let mut framebuffer = framebuffer();
    {
        static PIXELS: SyncUnsafeCell<Pixels> = SyncUnsafeCell::new(Pixels::zero_size());
        let pixels = unsafe { &mut *PIXELS.get() };
        *pixels = Pixels::new(
            Color { r: 0, g: 0, b: 0 },
            framebuffer.width(),
            framebuffer.height(),
        );
        GLOBAL_PRINTER.with(|printer| printer.screen = Some(pixels));
    }

    clear_screen();

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
            with_disabled_interrupts(|| {
                clear_screen();

                for event in &events {
                    println!("{event:?}");
                }

                GLOBAL_PRINTER.with(|printer| {
                    let framebuffer_ = framebuffer;
                    let screen = printer.screen.as_deref_mut().unwrap_or(&mut framebuffer);
                    screen.fill(
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
                    framebuffer_.copy(screen, 0, 0);
                });
            });
        } else {
            hlt();
        }
    }
}

fn clear_screen() {
    GLOBAL_PRINTER.with(|printer| {
        printer.x = 0;
        printer.y = 0;
        printer.left_margin = 0;

        let mut framebuffer = framebuffer();
        let screen = printer.screen.as_deref_mut().unwrap_or(&mut framebuffer);
        screen.fill(
            0,
            0,
            screen.width(),
            screen.height(),
            printer.background_color,
        );
    });
}
