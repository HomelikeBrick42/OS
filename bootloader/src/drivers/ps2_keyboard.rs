use alloc::collections::vec_deque::VecDeque;
use arrayvec::ArrayVec;

use crate::{
    drivers::pic::pic1_end,
    idt::{InterruptStackFrame, with_disabled_interrupts},
    utils::{inb, io_wait},
};

#[derive(Debug, Clone)]
pub enum Key {
    Escape,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Minus,
    Equals,
    Backspace,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    OpenBracket,
    CloseBracket,
    Enter,
    LeftControl,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,
    Quote,
    Backtick,
    LeftShift,
    Backslash,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Period,
    Slash,
    RightShift,
    KeypadMultiply,
    LeftAlt,
    Space,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    NumberLock,
    ScrollLock,
    Keypad7,
    Keypad8,
    Keypad9,
    KeypadMinus,
    Keypad4,
    Keypad5,
    Keypad6,
    KeypadPlus,
    Keypad1,
    Keypad2,
    Keypad3,
    Keypad0,
    KeypadPeriod,
    F11,
    F12,

    PreviousTrack,
    NextTrack,
    KeypadEnter,
    RightControl,
    Mute,
    Calculator,
    Play,
    Stop,
    VolumeDown,
    VolumeUp,
    WWWHome,
    KeypadSlash,
    RightAlt,
    Home,
    CursorUp,
    PageUp,
    CursorLeft,
    CursorRight,
    End,
    CursorDown,
    PageDown,
    Insert,
    Delete,
    LeftGUI,
    RightGUI,
    Apps,
    Power,
    Sleep,
    Wake,
    WWWSearch,
    WWWFavorites,
    WWWRefresh,
    WWWStop,
    WWWForward,
    WWWBack,
    MyComputer,
    Email,
    MediaSelect,

    Unknown1([u8; 1]),
    Unknown2([u8; 2]),
    Unknown3([u8; 3]),
}

#[derive(Debug, Clone, Copy)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Key,
    pub state: KeyState,
}

pub struct KeyboardState {
    key_events: VecDeque<KeyEvent>,
    bytes: ArrayVec<u8, 4>,
}

impl KeyboardState {
    pub fn next_event(&mut self) -> Option<KeyEvent> {
        self.key_events.pop_front()
    }
}

pub unsafe fn setup_keyboard() {}

pub fn with_keyboard_state<R>(f: impl FnOnce(&mut KeyboardState) -> R) -> R {
    static KEYBOARD_STATE: spin::Mutex<KeyboardState> = spin::Mutex::new(KeyboardState {
        key_events: VecDeque::new(),
        bytes: ArrayVec::new_const(),
    });
    with_disabled_interrupts(|| f(&mut KEYBOARD_STATE.lock()))
}

pub unsafe extern "x86-interrupt" fn keyboard_handler(_: InterruptStackFrame) {
    let scancode = unsafe { inb::<0x60>() };
    io_wait();

    if scancode == 0xFA {
        // just an ACK message
    } else {
        with_keyboard_state(|keyboard| {
            keyboard.bytes.push(scancode);

            if keyboard.bytes[0] == 0xE0 {
                if keyboard.bytes.len() >= 2 {
                    assert_eq!(keyboard.bytes.len(), 2);

                    let bytes = core::mem::take(&mut keyboard.bytes);

                    let state = if bytes[1] & 0b1000_0000 == 0 {
                        KeyState::Pressed
                    } else {
                        KeyState::Released
                    };
                    let key = match bytes[1] & !0b1000_0000 {
                        0x10 => Key::PreviousTrack,
                        0x19 => Key::NextTrack,
                        0x1C => Key::Enter,
                        0x1D => Key::RightControl,
                        0x20 => Key::Mute,
                        0x21 => Key::Calculator,
                        0x22 => Key::Play,
                        0x24 => Key::Stop,
                        0x2E => Key::VolumeDown,
                        0x30 => Key::VolumeUp,
                        0x32 => Key::WWWHome,
                        0x35 => Key::KeypadSlash,
                        0x38 => Key::RightAlt,
                        0x47 => Key::Home,
                        0x48 => Key::CursorUp,
                        0x49 => Key::PageUp,
                        0x4B => Key::CursorLeft,
                        0x4D => Key::CursorRight,
                        0x4F => Key::End,
                        0x50 => Key::CursorDown,
                        0x51 => Key::PageDown,
                        0x52 => Key::Insert,
                        0x53 => Key::Delete,
                        0x5B => Key::LeftGUI,
                        0x5C => Key::RightGUI,
                        0x5D => Key::Apps,
                        0x5E => Key::Power,
                        0x5F => Key::Sleep,
                        0x63 => Key::Wake,
                        0x65 => Key::WWWSearch,
                        0x66 => Key::WWWFavorites,
                        0x67 => Key::WWWRefresh,
                        0x68 => Key::WWWStop,
                        0x69 => Key::WWWForward,
                        0x6A => Key::WWWBack,
                        0x6B => Key::MyComputer,
                        0x6C => Key::Email,
                        0x6D => Key::MediaSelect,

                        _ => Key::Unknown2(bytes.as_slice().try_into().unwrap()),
                    };
                    keyboard.key_events.push_back(KeyEvent { key, state });
                }
            } else if keyboard.bytes[0] == 0xE1 {
                if keyboard.bytes.len() >= 3 {
                    assert_eq!(keyboard.bytes.len(), 3);

                    let bytes = core::mem::take(&mut keyboard.bytes);

                    let state = if bytes[1] & 0b1000_0000 == 0 {
                        KeyState::Pressed
                    } else {
                        KeyState::Released
                    };
                    keyboard.key_events.push_back(KeyEvent {
                        key: Key::Unknown3(bytes.as_slice().try_into().unwrap()),
                        state,
                    });
                }
            } else {
                assert_eq!(keyboard.bytes.len(), 1);

                let bytes = core::mem::take(&mut keyboard.bytes);

                let state = if bytes[0] & 0b1000_0000 == 0 {
                    KeyState::Pressed
                } else {
                    KeyState::Released
                };
                let key = match bytes[0] & !0b1000_0000 {
                    0x01 => Key::Escape,
                    0x02 => Key::Num1,
                    0x03 => Key::Num2,
                    0x04 => Key::Num3,
                    0x05 => Key::Num4,
                    0x06 => Key::Num5,
                    0x07 => Key::Num6,
                    0x08 => Key::Num7,
                    0x09 => Key::Num8,
                    0x0A => Key::Num9,
                    0x0B => Key::Num0,
                    0x0C => Key::Minus,
                    0x0D => Key::Equals,
                    0x0E => Key::Backspace,
                    0x0F => Key::Tab,
                    0x10 => Key::Q,
                    0x11 => Key::W,
                    0x12 => Key::E,
                    0x13 => Key::R,
                    0x14 => Key::T,
                    0x15 => Key::Y,
                    0x16 => Key::U,
                    0x17 => Key::I,
                    0x18 => Key::O,
                    0x19 => Key::P,
                    0x1A => Key::OpenBracket,
                    0x1B => Key::CloseBracket,
                    0x1C => Key::Enter,
                    0x1D => Key::LeftControl,
                    0x1E => Key::A,
                    0x1F => Key::S,
                    0x20 => Key::D,
                    0x21 => Key::F,
                    0x22 => Key::G,
                    0x23 => Key::H,
                    0x24 => Key::J,
                    0x25 => Key::K,
                    0x26 => Key::L,
                    0x27 => Key::Semicolon,
                    0x28 => Key::Quote,
                    0x29 => Key::Backtick,
                    0x2A => Key::LeftShift,
                    0x2B => Key::Backslash,
                    0x2C => Key::Z,
                    0x2D => Key::X,
                    0x2E => Key::C,
                    0x2F => Key::V,
                    0x30 => Key::B,
                    0x31 => Key::N,
                    0x32 => Key::M,
                    0x33 => Key::Comma,
                    0x34 => Key::Period,
                    0x35 => Key::Slash,
                    0x36 => Key::RightShift,
                    0x37 => Key::KeypadMultiply,
                    0x38 => Key::LeftAlt,
                    0x39 => Key::Space,
                    0x3A => Key::CapsLock,
                    0x3B => Key::F1,
                    0x3C => Key::F2,
                    0x3D => Key::F3,
                    0x3E => Key::F4,
                    0x3F => Key::F5,
                    0x40 => Key::F6,
                    0x41 => Key::F7,
                    0x42 => Key::F8,
                    0x43 => Key::F9,
                    0x44 => Key::F10,
                    0x45 => Key::NumberLock,
                    0x46 => Key::ScrollLock,
                    0x47 => Key::Keypad7,
                    0x48 => Key::Keypad8,
                    0x49 => Key::Keypad9,
                    0x4A => Key::KeypadMinus,
                    0x4B => Key::Keypad4,
                    0x4C => Key::Keypad5,
                    0x4D => Key::Keypad6,
                    0x4E => Key::KeypadPlus,
                    0x4F => Key::Keypad1,
                    0x50 => Key::Keypad2,
                    0x51 => Key::Keypad3,
                    0x52 => Key::Keypad0,
                    0x53 => Key::KeypadPeriod,
                    0x57 => Key::F11,
                    0x58 => Key::F12,

                    _ => Key::Unknown1(bytes.as_slice().try_into().unwrap()),
                };
                keyboard.key_events.push_back(KeyEvent { key, state });
            }
        });
    }

    unsafe { pic1_end() };
}
