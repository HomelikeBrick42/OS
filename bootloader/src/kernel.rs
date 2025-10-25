use crate::{
    drivers::{
        pic::{PIC1_DATA, PIC2_DATA, remap_pic},
        ps2_keyboard::{keyboard_handler, setup_keyboard, with_keyboard_state},
        ps2_mouse::{mouse_handler, setup_mouse, with_mouse_state},
    },
    framebuffer::{Color, FramebufferColor},
    gdt::setup_gdt,
    idt::{InterruptType, disable_interrupts, enable_interrupts, setup_idt, with_idt_entry},
    print::with_global_printer,
    screen::{NoopScreen, Screen},
    utils::{hlt, io_wait, outb},
};
use alloc::{vec, vec::Vec};
use core::cell::SyncUnsafeCell;

struct BufferedScreen {
    width: usize,
    height: usize,
    screen: &'static mut dyn Screen,
    buffer: Vec<FramebufferColor>,
}

impl Screen for BufferedScreen {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> FramebufferColor {
        unsafe { *self.buffer.get_unchecked(x + y * self.width) }
    }

    unsafe fn set_pixel_unchecked(&mut self, x: usize, y: usize, color: FramebufferColor) {
        unsafe { *self.buffer.get_unchecked_mut(x + y * self.width) = color }
    }

    fn present(&mut self) {
        let screen = core::mem::replace(&mut self.screen, NoopScreen::get_static());
        screen.copy(self);
        screen.present();
        self.screen = screen;
    }
}

pub unsafe extern "win64" fn kernel_main() -> ! {
    unsafe { disable_interrupts() };

    with_global_printer(|printer| {
        let screen = core::mem::replace(&mut printer.screen, NoopScreen::get_static());
        let buffer = vec![
            FramebufferColor::new(Color {
                r: 50,
                g: 50,
                b: 50,
            });
            screen.width() * screen.height()
        ];

        static BUFFERED_SCREEN: SyncUnsafeCell<BufferedScreen> =
            SyncUnsafeCell::new(BufferedScreen {
                width: 0,
                height: 0,
                screen: NoopScreen::get_static(),
                buffer: vec![],
            });
        unsafe {
            *BUFFERED_SCREEN.get() = BufferedScreen {
                width: screen.width(),
                height: screen.height(),
                screen,
                buffer,
            }
        };
        printer.screen = unsafe { &mut *BUFFERED_SCREEN.get() };
    });

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

    let mut mouse_x = 0usize;
    let mut mouse_y = 0usize;
    loop {
        clear_screen();

        with_keyboard_state(|keyboard| {
            while let Some(event) = keyboard.next_event() {
                _ = event;
            }
        });

        with_mouse_state(|mouse| {
            while let Some(event) = mouse.next_event() {
                mouse_x = mouse_x.saturating_add_signed(event.x_offset as isize);
                mouse_y = mouse_y.saturating_add_signed(-(event.y_offset as isize));
            }
        });

        with_global_printer(|printer| {
            mouse_x = mouse_x.min(printer.screen.width());
            mouse_y = mouse_y.min(printer.screen.height());

            printer.screen.fill(
                mouse_x.saturating_sub(5),
                mouse_y.saturating_sub(5),
                10,
                10,
                FramebufferColor::new(Color {
                    r: 255,
                    g: 255,
                    b: 255,
                }),
            );

            printer.screen.present();
        });
        hlt();
    }
}

fn clear_screen() {
    with_global_printer(|printer| {
        printer.x = printer.left_margin;
        printer.y = 0;

        let width = printer.screen.width();
        let height = printer.screen.height();
        printer.screen.fill(
            0,
            0,
            width,
            height,
            FramebufferColor::new(printer.background_color),
        );
    });
}
