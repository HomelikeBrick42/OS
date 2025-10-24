use crate::{
    drivers::{
        pic::{PIC1_DATA, PIC2_DATA, remap_pic},
        ps2_keyboard::{keyboard_handler, with_keyboard_state},
    },
    framebuffer::framebuffer,
    gdt::setup_gdt,
    idt::{InterruptType, disable_interrupts, enable_interrupts, setup_idt, with_idt_entry},
    print::{println, with_global_printer},
    utils::{hlt, io_wait, outb},
};

pub unsafe extern "win64" fn kernel_main() -> ! {
    unsafe { disable_interrupts() };

    let framebuffer = framebuffer();

    let background_color = with_global_printer(|printer| {
        printer.x = 0;
        printer.y = 0;
        printer.left_margin = 0;
        printer.background_color
    });
    framebuffer.fill(
        0,
        0,
        framebuffer.width(),
        framebuffer.height(),
        framebuffer.color(background_color),
    );

    unsafe { setup_gdt() };
    unsafe { setup_idt() };

    unsafe { remap_pic(0x20, 0x28) };

    unsafe {
        with_idt_entry(0x21, |entry| {
            entry.set_handler(keyboard_handler, InterruptType::Interrupt);
        });
    }
    unsafe { outb::<PIC1_DATA>(0b11111101) };
    io_wait();
    unsafe { outb::<PIC2_DATA>(0b11111111) };
    io_wait();

    unsafe { enable_interrupts() };

    loop {
        with_keyboard_state(|keyboard| {
            while let Some(event) = keyboard.next_event() {
                println!("{event:?}");
            }
        });
        hlt();
    }
}
