fn scancode_to_char(scancode: u8) -> Option<char> {
    match scancode {
        0x02..=0x0A => Some((scancode as u8 + 0x2E) as char), // 1-9
        0x0B => Some('0'),
        0x0C => Some('-'),
        0x0D => Some('='),
        0x0E => Some('\x08'), // Backspace
        0x0F => Some('\t'),   // Tab
        0x10 => Some('q'),
        0x11 => Some('w'),
        0x12 => Some('e'),
        0x13 => Some('r'),
        0x14 => Some('t'),
        0x15 => Some('y'),
        0x16 => Some('u'),
        0x17 => Some('i'),
        0x18 => Some('o'),
        0x19 => Some('p'),
        0x1A => Some('['),
        0x1B => Some(']'),
        0x1C => Some('\n'),   // Enter
        0x1E => Some('a'),
        0x1F => Some('s'),
        0x20 => Some('d'),
        0x21 => Some('f'),
        0x22 => Some('g'),
        0x23 => Some('h'),
        0x24 => Some('j'),
        0x25 => Some('k'),
        0x26 => Some('l'),
        0x27 => Some(';'),
        0x28 => Some('\''),
        0x29 => Some('`'),
        0x2B => Some('\\'),
        0x2C => Some('z'),
        0x2D => Some('x'),
        0x2E => Some('c'),
        0x2F => Some('v'),
        0x30 => Some('b'),
        0x31 => Some('n'),
        0x32 => Some('m'),
        0x33 => Some(','),
        0x34 => Some('.'),
        0x35 => Some('/'),
        0x39 => Some(' '),    // Space
        // Keypad keys
        0x52 => Some('0'),
        0x4F => Some('1'),
        0x50 => Some('2'),
        0x51 => Some('3'),
        0x4B => Some('4'),
        0x4C => Some('5'),
        0x4D => Some('6'),
        0x47 => Some('7'),
        0x48 => Some('8'),
        0x49 => Some('9'),
        0x37 => Some('*'),    // Keypad *
        0x4A => Some('-'),    // Keypad -
        0x4E => Some('+'),    // Keypad +
        0x53 => Some('.'),    // Keypad .
        _ => None,
    }
}

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub caps_lock: bool,
}

impl Modifiers {
    pub const fn new() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            caps_lock: false,
        }
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        macro_rules! write_mod {
            ($cond:expr, $name:expr) => {
                if $cond {
                    if !first {
                        write!(f, "+")?;
                    }
                    write!(f, "{}", $name)?;
                    first = false;
                }
            };
        }
        write_mod!(self.shift, "Shift");
        write_mod!(self.ctrl, "Ctrl");
        write_mod!(self.alt, "Alt");
        write_mod!(self.caps_lock, "CapsLock");
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    // Add other special keys as needed
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyCode::Char(c) => write!(f, "{}", c),
            KeyCode::Enter => write!(f, "Enter"),
            KeyCode::Backspace => write!(f, "Backspace"),
            KeyCode::Delete => write!(f, "Delete"),
            KeyCode::Escape => write!(f, "Escape"),
            KeyCode::ArrowUp => write!(f, "UP"),
            KeyCode::ArrowDown => write!(f, "DOWN"),
            KeyCode::ArrowLeft => write!(f, "LEFT"),
            KeyCode::ArrowRight => write!(f, "RIGHT"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: Modifiers,
    pub scancode: u8,
}

impl KeyEvent {
    pub const fn new(code: KeyCode, modifiers: Modifiers, scancode: u8) -> Self {
        Self { code, modifiers, scancode }
    }
}


pub fn scancode_to_keycode(scancode: u8, is_extended: bool) -> Option<KeyCode> {
    match (scancode, is_extended) {
        (0x1C, _) => Some(KeyCode::Enter),
        (0x0E, _) => Some(KeyCode::Backspace),
        (0x01, _) => Some(KeyCode::Escape),
        (0x48, true) => Some(KeyCode::ArrowUp),
        (0x50, true) => Some(KeyCode::ArrowDown),
        (0x4B, true) => Some(KeyCode::ArrowLeft),
        (0x4D, true) => Some(KeyCode::ArrowRight),
        (0x53, true) => Some(KeyCode::Delete),
        (code, false) if code < 0x80 => scancode_to_char(code).map(KeyCode::Char),
        _ => None,
    }
}


const KEY_BUFFER_SIZE: usize = 16;

pub struct KeyboardBuffer {
    events: [KeyEvent; KEY_BUFFER_SIZE],
    write_index: usize,
    read_index: usize,
}

impl KeyboardBuffer {
    pub const fn new() -> Self {
        const EMPTY_EVENT: KeyEvent = KeyEvent {
            code: KeyCode::Char('\0'),
            modifiers: Modifiers::new(),
            scancode: 0,
        };
        Self {
            events: [EMPTY_EVENT; KEY_BUFFER_SIZE],
            write_index: 0,
            read_index: 0,
        }
    }

    pub fn push(&mut self, event: KeyEvent) {
        self.events[self.write_index] = event;
        self.write_index = (self.write_index + 1) % KEY_BUFFER_SIZE;
        if self.write_index == self.read_index {
            self.read_index = (self.read_index + 1) % KEY_BUFFER_SIZE;
        }
    }

    pub fn pop(&mut self) -> Option<KeyEvent> {
        if self.read_index == self.write_index {
            None
        } else {
            let event = self.events[self.read_index];
            self.read_index = (self.read_index + 1) % KEY_BUFFER_SIZE;
            Some(event)
        }
    }
}

pub static mut KEYBOARD_BUFFER: KeyboardBuffer = KeyboardBuffer::new();

pub fn handle_keyboard_interrupt(scancode: u8) {
    static mut IS_EXTENDED: bool = false;
    static mut CURRENT_MODIFIERS: Modifiers = Modifiers::new();

    unsafe {
        if scancode == 0xE0 {
            IS_EXTENDED = true;
            return;
        }

        let is_release = scancode >= 0x80;
        let actual_scancode = if is_release { scancode - 0x80 } else { scancode };

        // Update modifiers
        match actual_scancode {
            0x2A | 0x36 => CURRENT_MODIFIERS.shift = !is_release,
            0x1D => CURRENT_MODIFIERS.ctrl = !is_release,
            0x38 => CURRENT_MODIFIERS.alt = !is_release,
            0x3A if !is_release => CURRENT_MODIFIERS.caps_lock = !CURRENT_MODIFIERS.caps_lock,
            _ => {}
        }

        if !is_release {
            if let Some(keycode) = scancode_to_keycode(actual_scancode, IS_EXTENDED) {
                let event = KeyEvent::new(keycode, CURRENT_MODIFIERS, actual_scancode);
                KEYBOARD_BUFFER.push(event);
                // You can add your print logic here if needed
            }
        }

        IS_EXTENDED = false;
    }
}

impl fmt::Display for KeyEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut has_modifier = false;
        macro_rules! write_mod {
            ($cond:expr, $name:expr) => {
                if $cond {
                    if has_modifier {
                        write!(f, "+")?;
                    }
                    write!(f, "{}", $name)?;
                    has_modifier = true;
                }
            };
        }

        write_mod!(self.modifiers.shift, "Shift");
        write_mod!(self.modifiers.ctrl, "Ctrl");
        write_mod!(self.modifiers.alt, "Alt");
        write_mod!(self.modifiers.caps_lock, "CapsLock");

        if has_modifier {
            write!(f, "+")?;
        }

        write!(f, "{}", self.code)
    }
}
// fn process()
// {
//     unsafe {
//         if let Some(event) = KEYBOARD_BUFFER.pop() {
//             // Process the event
//         }
//     }
// }