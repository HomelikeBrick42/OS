use crate::{
    drivers::pic::pic2_end,
    idt::{InterruptStackFrame, with_disabled_interrupts},
    utils::{inb, io_wait, outb},
};
use alloc::collections::vec_deque::VecDeque;
use enum_map::{Enum, EnumMap, enum_map};

pub fn mouse_wait() {
    for _ in 0..100000 {
        if unsafe { inb::<0x64>() } & 0b10 == 0 {
            return;
        }
        core::hint::spin_loop();
    }
}

pub fn mouse_wait_input() {
    for _ in 0..100000 {
        if unsafe { inb::<0x64>() } & 0b1 == 0 {
            return;
        }
        core::hint::spin_loop();
    }
}

pub unsafe fn mouse_write(value: u8) {
    mouse_wait();
    unsafe { outb::<0x64>(0xD4) };
    mouse_wait();
    unsafe { outb::<0x60>(value) };
}

pub unsafe fn mouse_read() -> u8 {
    mouse_wait_input();
    unsafe { inb::<0x60>() }
}

pub unsafe fn setup_mouse() {
    unsafe { outb::<0x64>(0xA4) };

    mouse_wait();
    unsafe { outb::<0x64>(0x20) };
    mouse_wait_input();
    let mut status = unsafe { inb::<0x60>() };
    status |= 0b10;
    mouse_wait();
    unsafe { outb::<0x64>(0x60) };
    mouse_wait();
    unsafe { outb::<0x60>(status) };

    unsafe { mouse_write(0xF6) };
    assert_eq!(unsafe { mouse_read() }, 0xFA);

    unsafe { mouse_write(0xF4) };
    assert_eq!(unsafe { mouse_read() }, 0xFA);
}

enum MouseDataState {
    None,
    First([u8; 1]),
    Second([u8; 2]),
}

#[derive(Enum, Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButtonState {
    Pressed,
    Released,
}

#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub mouse_buttons: EnumMap<MouseButton, MouseButtonState>,
    pub x_offset: i16,
    pub y_offset: i16,
}

pub struct MouseState {
    mouse_events: VecDeque<MouseEvent>,
    data_state: MouseDataState,
}

impl MouseState {
    pub fn next_event(&mut self) -> Option<MouseEvent> {
        self.mouse_events.pop_front()
    }
}

pub fn with_mouse_state<R>(f: impl FnOnce(&mut MouseState) -> R) -> R {
    static MOUSE_STATE: spin::Mutex<MouseState> = spin::Mutex::new(MouseState {
        data_state: MouseDataState::None,
        mouse_events: VecDeque::new(),
    });
    with_disabled_interrupts(|| f(&mut MOUSE_STATE.lock()))
}

pub unsafe extern "x86-interrupt" fn mouse_handler(_: InterruptStackFrame) {
    let mouse_data = unsafe { inb::<0x60>() };
    io_wait();

    with_mouse_state(|mouse| {
        mouse.data_state = match mouse.data_state {
            MouseDataState::None => {
                if mouse_data == 0xFA {
                    MouseDataState::None
                } else {
                    MouseDataState::First([mouse_data])
                }
            }

            MouseDataState::First([first]) => MouseDataState::Second([first, mouse_data]),

            MouseDataState::Second([first, x_axis]) => {
                // TODO: this doesnt yet handle the overflow bit
                let x_axis = x_axis as i8 as i16;
                let y_axis = mouse_data as i8 as i16;

                let x_offset = x_axis - ((first << 4) as i16 & 0x100);
                let y_offset = y_axis - ((first << 3) as i16 & 0x100);

                let mut mouse_buttons = enum_map! {
                    _ => MouseButtonState::Released,
                };
                for (mouse_button, state) in mouse_buttons.iter_mut() {
                    *state = match mouse_button {
                        MouseButton::Left => {
                            if first & 0b001 != 0 {
                                MouseButtonState::Pressed
                            } else {
                                MouseButtonState::Released
                            }
                        }

                        MouseButton::Right => {
                            if first & 0b010 != 0 {
                                MouseButtonState::Pressed
                            } else {
                                MouseButtonState::Released
                            }
                        }

                        MouseButton::Middle => {
                            if first & 0b100 != 0 {
                                MouseButtonState::Pressed
                            } else {
                                MouseButtonState::Released
                            }
                        }
                    };
                }

                mouse.mouse_events.push_back(MouseEvent {
                    mouse_buttons,
                    x_offset,
                    y_offset,
                });

                MouseDataState::None
            }
        };
    });

    unsafe { pic2_end() };
}
