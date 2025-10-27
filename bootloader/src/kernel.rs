use crate::{
    drivers::{
        pic::{PIC1_DATA, PIC2_DATA, remap_pic},
        ps2_keyboard::{KEYBOARD_STATE, Key, keyboard_handler, setup_keyboard},
        ps2_mouse::{MOUSE_STATE, mouse_handler, setup_mouse},
    },
    framebuffer::{FramebufferColor, framebuffer},
    gdt::setup_gdt,
    idt::{InterruptType, disable_interrupts, enable_interrupts, setup_idt, with_idt_entry},
    print::{GLOBAL_PRINTER, println},
    utils::{hlt, io_wait, outb},
};

pub unsafe extern "win64" fn kernel_main() -> ! {
    unsafe { disable_interrupts() };

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

    loop {
        KEYBOARD_STATE.with(|keyboard| {
            while let Some(event) = keyboard.next_event() {
                println!("{event:?}");
                if matches!(event.key, Key::Backspace) {
                    clear_screen();
                }
            }
        });

        MOUSE_STATE.with(|mouse| {
            while let Some(event) = mouse.next_event() {
                clear_screen();
                println!("{event:#?}");
            }
        });

        hlt();
    }
}

fn clear_screen() {
    let background_color = GLOBAL_PRINTER.with(|printer| {
        printer.x = 0;
        printer.y = 0;
        printer.left_margin = 0;
        printer.background_color
    });

    let framebuffer = framebuffer();
    framebuffer.fill(
        0,
        0,
        framebuffer.width(),
        framebuffer.height(),
        FramebufferColor::new(background_color),
    );
}
