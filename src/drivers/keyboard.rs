use crate::{
    interrupts::interrupt_handler,
    io::{input_byte, io_wait, output_byte},
};
use crossbeam::queue::SegQueue;
use derive_more::Display;
use spin::Mutex;

#[derive(Display)]
pub enum Side {
    #[display(fmt = "left")]
    Left,
    #[display(fmt = "right")]
    Right,
}

#[derive(Display)]
pub enum Keypad {
    #[display(fmt = "+")]
    Plus,
    #[display(fmt = "-")]
    Minus,
    #[display(fmt = "*")]
    Asterisk,
    #[display(fmt = "/")]
    Slash,
    #[display(fmt = "1")]
    Num1,
    #[display(fmt = "2")]
    Num2,
    #[display(fmt = "3")]
    Num3,
    #[display(fmt = "4")]
    Num4,
    #[display(fmt = "5")]
    Num5,
    #[display(fmt = "6")]
    Num6,
    #[display(fmt = "7")]
    Num7,
    #[display(fmt = "8")]
    Num8,
    #[display(fmt = "9")]
    Num9,
    #[display(fmt = "0")]
    Num0,
    #[display(fmt = ".")]
    Period,
}

#[derive(Display)]
pub enum Keycode {
    #[display(fmt = "esc")]
    Escape,
    #[display(fmt = "1")]
    Num1,
    #[display(fmt = "2")]
    Num2,
    #[display(fmt = "3")]
    Num3,
    #[display(fmt = "4")]
    Num4,
    #[display(fmt = "5")]
    Num5,
    #[display(fmt = "6")]
    Num6,
    #[display(fmt = "7")]
    Num7,
    #[display(fmt = "8")]
    Num8,
    #[display(fmt = "9")]
    Num9,
    #[display(fmt = "0")]
    Num0,
    #[display(fmt = "-")]
    Minus,
    #[display(fmt = "=")]
    Equals,
    #[display(fmt = "backspace")]
    Backspace,

    #[display(fmt = "tab")]
    Tab,
    #[display(fmt = "q")]
    Q,
    #[display(fmt = "w")]
    W,
    #[display(fmt = "e")]
    E,
    #[display(fmt = "r")]
    R,
    #[display(fmt = "t")]
    T,
    #[display(fmt = "y")]
    Y,
    #[display(fmt = "u")]
    U,
    #[display(fmt = "i")]
    I,
    #[display(fmt = "o")]
    O,
    #[display(fmt = "p")]
    P,
    #[display(fmt = "[")]
    OpenSquareBracket,
    #[display(fmt = "]")]
    CloseSquareBracket,
    #[display(fmt = "enter")]
    Enter,

    #[display(fmt = "a")]
    A,
    #[display(fmt = "s")]
    S,
    #[display(fmt = "d")]
    D,
    #[display(fmt = "f")]
    F,
    #[display(fmt = "g")]
    G,
    #[display(fmt = "h")]
    H,
    #[display(fmt = "j")]
    J,
    #[display(fmt = "k")]
    K,
    #[display(fmt = "l")]
    L,
    #[display(fmt = ";")]
    Semicolon,
    #[display(fmt = "'")]
    Quote,
    #[display(fmt = "`")]
    Backtick,
    #[display(fmt = "\\")]
    Backslash,

    #[display(fmt = "z")]
    Z,
    #[display(fmt = "x")]
    X,
    #[display(fmt = "c")]
    C,
    #[display(fmt = "v")]
    V,
    #[display(fmt = "b")]
    B,
    #[display(fmt = "n")]
    N,
    #[display(fmt = "m")]
    M,
    #[display(fmt = ",")]
    Comma,
    #[display(fmt = ".")]
    Period,
    #[display(fmt = "/")]
    Slash,

    #[display(fmt = "space")]
    Space,

    #[display(fmt = "f1")]
    F1,
    #[display(fmt = "f2")]
    F2,
    #[display(fmt = "f3")]
    F3,
    #[display(fmt = "f4")]
    F4,
    #[display(fmt = "f5")]
    F5,
    #[display(fmt = "f6")]
    F6,
    #[display(fmt = "f7")]
    F7,
    #[display(fmt = "f8")]
    F8,
    #[display(fmt = "f9")]
    F9,
    #[display(fmt = "f10")]
    F10,
    #[display(fmt = "f11")]
    F11,
    #[display(fmt = "f12")]
    F12,

    #[display(fmt = "caps lock")]
    CapsLock,
    #[display(fmt = "number lock")]
    NumberLock,
    #[display(fmt = "scroll lock")]
    ScrollLock,

    #[display(fmt = "keypad {_0}")]
    Keypad(Keypad),

    #[display(fmt = "{_0} control")]
    Control(Side),
    #[display(fmt = "{_0} shift")]
    Shift(Side),
    #[display(fmt = "{_0} alt")]
    Alt(Side),

    #[display(fmt = "unknown")]
    Unknown,
}

#[derive(Display)]
pub enum Action {
    #[display(fmt = "pressed")]
    Pressed,
    #[display(fmt = "released")]
    Released,
}

pub struct KeyboardEvent {
    pub key: Keycode,
    pub action: Action,
}

pub static KEYBOARD_EVENTS: SegQueue<KeyboardEvent> = SegQueue::new();

pub const PIC1_COMMAND_PORT: u16 = 0x20;
pub const PIC1_DATA_PORT: u16 = 0x21;
pub const PIC2_COMMAND_PORT: u16 = 0xA0;
pub const PIC2_DATA_PORT: u16 = 0xA1;

pub const PS2_DATA_PORT: u16 = 0x60;
pub const PS2_COMMAND_PORT: u16 = 0x64;
pub const PS2_LED: u8 = 0xED;

pub const PIC_EOI: u8 = 0x20;
pub const ICW1_INIT: u8 = 0x10;
pub const ICW1_ICW4: u8 = 0x01;
pub const ICW1_8086: u8 = 0x01;

pub unsafe fn remap_pic() {
    unsafe {
        let a1 = input_byte(PIC1_DATA_PORT);
        io_wait();
        let a2 = input_byte(PIC2_DATA_PORT);
        io_wait();

        output_byte(PIC1_COMMAND_PORT, ICW1_INIT | ICW1_ICW4);
        io_wait();
        output_byte(PIC2_COMMAND_PORT, ICW1_INIT | ICW1_ICW4);
        io_wait();

        output_byte(PIC1_DATA_PORT, 0x20);
        io_wait();
        output_byte(PIC2_DATA_PORT, 0x28);
        io_wait();

        output_byte(PIC1_DATA_PORT, 4);
        io_wait();
        output_byte(PIC2_DATA_PORT, 2);
        io_wait();

        output_byte(PIC1_DATA_PORT, ICW1_8086);
        io_wait();
        output_byte(PIC2_DATA_PORT, ICW1_8086);
        io_wait();

        output_byte(PIC1_DATA_PORT, a1);
        io_wait();
        output_byte(PIC2_DATA_PORT, a2);
        io_wait();
    }
}

enum State {
    Idle,
    FirstByteOfMultibyteScancode(u8),
}

static STATE: Mutex<State> = Mutex::new(State::Idle);

interrupt_handler!(keyboard_handler, keyboard_interrupt);
unsafe extern "win64" fn keyboard_interrupt() {
    unsafe {
        let byte = input_byte(PS2_DATA_PORT);
        io_wait();

        let mut state = STATE.lock();
        *state = if let Some(key) = byte_to_keycode(byte) {
            let action = if (byte & 0b1000_0000) == 0 {
                Action::Pressed
            } else {
                Action::Released
            };
            KEYBOARD_EVENTS.push(KeyboardEvent { key, action });
            State::Idle
        } else if [0xE0, 0xE1].contains(&byte) {
            State::FirstByteOfMultibyteScancode(byte)
        } else {
            match *state {
                State::Idle => State::Idle,
                State::FirstByteOfMultibyteScancode(0xE0) => todo!(),
                State::FirstByteOfMultibyteScancode(0xE1) => todo!(),
                State::FirstByteOfMultibyteScancode(_) => unreachable!(),
            }
        };

        ready_for_next_keyboard_event();
    }
}

unsafe fn byte_to_keycode(byte: u8) -> Option<Keycode> {
    if (0x01..=0xD8).contains(&byte) {
        Some(match byte & 0b0111_1111 {
            0x01 => Keycode::Escape,
            0x02 => Keycode::Num1,
            0x03 => Keycode::Num2,
            0x04 => Keycode::Num3,
            0x05 => Keycode::Num4,
            0x06 => Keycode::Num5,
            0x07 => Keycode::Num6,
            0x08 => Keycode::Num7,
            0x09 => Keycode::Num8,
            0x0A => Keycode::Num9,
            0x0B => Keycode::Num0,
            0x0C => Keycode::Minus,
            0x0D => Keycode::Equals,
            0x0E => Keycode::Backspace,

            0x0F => Keycode::Tab,
            0x10 => Keycode::Q,
            0x11 => Keycode::W,
            0x12 => Keycode::E,
            0x13 => Keycode::R,
            0x14 => Keycode::T,
            0x15 => Keycode::Y,
            0x16 => Keycode::U,
            0x17 => Keycode::I,
            0x18 => Keycode::O,
            0x19 => Keycode::P,
            0x1A => Keycode::OpenSquareBracket,
            0x1B => Keycode::CloseSquareBracket,
            0x1C => Keycode::Enter,

            0x1D => Keycode::Control(Side::Left),
            0x1E => Keycode::A,
            0x1F => Keycode::S,
            0x20 => Keycode::D,
            0x21 => Keycode::F,
            0x22 => Keycode::G,
            0x23 => Keycode::H,
            0x24 => Keycode::J,
            0x25 => Keycode::K,
            0x26 => Keycode::L,
            0x27 => Keycode::Semicolon,
            0x28 => Keycode::Quote,
            0x29 => Keycode::Backtick,
            0x2A => Keycode::Shift(Side::Left),
            0x2B => Keycode::Backslash,

            0x2C => Keycode::Z,
            0x2D => Keycode::X,
            0x2E => Keycode::C,
            0x2F => Keycode::V,
            0x30 => Keycode::B,
            0x31 => Keycode::N,
            0x32 => Keycode::M,
            0x33 => Keycode::Comma,
            0x34 => Keycode::Period,
            0x35 => Keycode::Slash,
            0x36 => Keycode::Shift(Side::Right),
            0x37 => Keycode::Keypad(Keypad::Asterisk),

            0x38 => Keycode::Alt(Side::Left),
            0x39 => Keycode::Space,

            0x3B => Keycode::F1,
            0x3C => Keycode::F2,
            0x3D => Keycode::F3,
            0x3E => Keycode::F4,
            0x3F => Keycode::F5,
            0x40 => Keycode::F6,
            0x41 => Keycode::F7,
            0x42 => Keycode::F8,
            0x43 => Keycode::F9,
            0x44 => Keycode::F10,
            0x57 => Keycode::F11,
            0x58 => Keycode::F12,

            0x3A => Keycode::CapsLock,
            0x45 => Keycode::NumberLock,
            0x46 => Keycode::ScrollLock,

            0x47 => Keycode::Keypad(Keypad::Num7),
            0x48 => Keycode::Keypad(Keypad::Num8),
            0x49 => Keycode::Keypad(Keypad::Num9),
            0x4A => Keycode::Keypad(Keypad::Minus),
            0x4B => Keycode::Keypad(Keypad::Num4),
            0x4C => Keycode::Keypad(Keypad::Num5),
            0x4D => Keycode::Keypad(Keypad::Num6),
            0x4E => Keycode::Keypad(Keypad::Plus),
            0x4F => Keycode::Keypad(Keypad::Num1),
            0x50 => Keycode::Keypad(Keypad::Num2),
            0x51 => Keycode::Keypad(Keypad::Num3),
            0x52 => Keycode::Keypad(Keypad::Num0),
            0x53 => Keycode::Keypad(Keypad::Period),

            _ => Keycode::Unknown,
        })
    } else {
        None
    }
}

unsafe fn ready_for_next_keyboard_event() {
    unsafe {
        output_byte(PIC2_COMMAND_PORT, PIC_EOI);
        io_wait();
        output_byte(PIC1_COMMAND_PORT, PIC_EOI);
        io_wait();
    }
}
