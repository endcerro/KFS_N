// keyboard.rs

use core::fmt;


// Constants
const BUFFER_SIZE: usize = 256;

// Bitflags for modifiers
pub const SHIFT: u8 = 0b0000_0001;
pub const CTRL: u8 = 0b0000_0010;
pub const ALT: u8 = 0b0000_0100;
pub const CAPS_LOCK: u8 = 0b0000_1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Function(u8),  // F1-F12
    Control(ControlKey),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlKey {
    Enter,
    Backspace,
    Escape,
    Tab,
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    LeftAlt,
    RightAlt,
    CapsLock,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,

}
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: u8,
    pub pressed: bool,
}
impl ControlKey {
    fn as_bytes(&self) -> &'static [u8] {
        match self {
            ControlKey::Enter => b"Enter",
            ControlKey::Backspace => b"Backspace",
            ControlKey::Escape => b"Escape",
            ControlKey::Tab => b"Tab",
            ControlKey::LeftShift => b"LeftShift",
            ControlKey::RightShift => b"RightShift",
            ControlKey::LeftCtrl => b"LeftCtrl",
            ControlKey::RightCtrl => b"RightCtrl",
            ControlKey::LeftAlt => b"LeftAlt",
            ControlKey::RightAlt => b"RightAlt",
            ControlKey::CapsLock => b"CapsLock",
            ControlKey::Delete => b"Delete",
            ControlKey::Home => b"Home",
            ControlKey::End => b"End",
            ControlKey::PageUp => b"PageUp",
            ControlKey::PageDown => b"PageDown",
            ControlKey::LeftArrow => b"LeftArrow",
            ControlKey::RightArrow => b"RightArrow",
            ControlKey::UpArrow => b"UpArrow",
            ControlKey::DownArrow => b"DownArrow",
        }
    }
}

pub struct Keyboard {
    event_buffer: [KeyEvent; BUFFER_SIZE],
    write_index: usize,
    read_index: usize,
    modifiers: u8,
    is_extended: bool,
    input_buffer: [u8; BUFFER_SIZE],
    input_len: usize,
    finished_buffer: [u8; BUFFER_SIZE],
    finished_len: usize,
    just_deleted : bool
}

impl Keyboard {
    pub const fn new() -> Self {
        const EMPTY_EVENT: KeyEvent = KeyEvent {
            code: KeyCode::Char('\0'),
            modifiers: 0,
            pressed: false,
        };
        Self {
            event_buffer: [EMPTY_EVENT; BUFFER_SIZE],
            write_index: 0,
            read_index: 0,
            modifiers: 0,
            is_extended: false,
            input_buffer: [b'\0'; BUFFER_SIZE],
            input_len: 0,
            finished_buffer: [b'\0'; BUFFER_SIZE],
            finished_len: 0,
            just_deleted : false
        }
    }

pub fn handle_scancode(&mut self, scancode: u8) {
        if scancode == 0xE0 {
            self.is_extended = true;
            return;
        }

        let pressed = scancode < 0x80;
        let actual_scancode = if pressed { scancode } else { scancode - 0x80 };

        if let Some(key_event) = self.scancode_to_key_event(actual_scancode, pressed) {
            self.update_modifiers(&key_event);
            self.push_event(key_event);
            self.update_input_buffer(&key_event);
        }

        self.is_extended = false;
    }

    fn scancode_to_key_event(&self, scancode: u8, pressed: bool) -> Option<KeyEvent> {
        let code = match (scancode, self.is_extended) {
            (0x01, false) => KeyCode::Control(ControlKey::Escape),
            (0x0E, false) => KeyCode::Control(ControlKey::Backspace),
            (0x0F, false) => KeyCode::Control(ControlKey::Tab),
            (0x1C, false) => KeyCode::Control(ControlKey::Enter),
            (0x1D, false) => KeyCode::Control(ControlKey::LeftCtrl),
            (0x1D, true) => KeyCode::Control(ControlKey::RightCtrl),
            (0x2A, false) => KeyCode::Control(ControlKey::LeftShift),
            (0x36, false) => KeyCode::Control(ControlKey::RightShift),
            (0x38, false) => KeyCode::Control(ControlKey::LeftAlt),
            (0x38, true) => KeyCode::Control(ControlKey::RightAlt),
            (0x3A, false) => KeyCode::Control(ControlKey::CapsLock),
            (0x47, true) => KeyCode::Control(ControlKey::Home),
            (0x48, true) => KeyCode::Control(ControlKey::UpArrow),
            (0x49, true) => KeyCode::Control(ControlKey::PageUp),
            (0x4B, true) => KeyCode::Control(ControlKey::LeftArrow),
            (0x4D, true) => KeyCode::Control(ControlKey::RightArrow),
            (0x4F, true) => KeyCode::Control(ControlKey::End),
            (0x50, true) => KeyCode::Control(ControlKey::DownArrow),
            (0x51, true) => KeyCode::Control(ControlKey::PageDown),
            (0x53, true) => KeyCode::Control(ControlKey::Delete),
            (0x3B..=0x44, false) => KeyCode::Function(scancode - 0x3A),
            (0x57..=0x58, false) => KeyCode::Function(scancode - 0x4E),
            _ => self.scancode_to_char(scancode).map(KeyCode::Char)?,
        };

        Some(KeyEvent {
            code,
            modifiers: self.modifiers,
            pressed,
        })
    }

    fn scancode_to_char(&self, scancode: u8) -> Option<char> {
        let qwerty_layout = [
            '\0', '\x1B', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\x08',
            '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
            '\0', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
            '\0', '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
            '\0', '*', '\0', ' '
        ];
        qwerty_layout.get(scancode as usize).copied().filter(|&c| c != '\0')
    }

    fn update_modifiers(&mut self, event: &KeyEvent) {
        let modifier = match event.code {
            KeyCode::Control(ControlKey::LeftShift) | KeyCode::Control(ControlKey::RightShift) => Some(SHIFT),
            KeyCode::Control(ControlKey::LeftCtrl) | KeyCode::Control(ControlKey::RightCtrl) => Some(CTRL),
            KeyCode::Control(ControlKey::LeftAlt) | KeyCode::Control(ControlKey::RightAlt) => Some(ALT),
            KeyCode::Control(ControlKey::CapsLock) if event.pressed => {
                self.modifiers ^= CAPS_LOCK;
                None
            }
            _ => None,
        };

        if let Some(modifier) = modifier {
            if event.pressed {
                self.modifiers |= modifier;
            } else {
                self.modifiers &= !modifier;
            }
        }
    }

    fn push_event(&mut self, event: KeyEvent) {
        self.event_buffer[self.write_index] = event;
        self.write_index = (self.write_index + 1) % BUFFER_SIZE;
        if self.write_index == self.read_index {
            self.read_index = (self.read_index + 1) % BUFFER_SIZE;
        }
    }

    pub fn pop_event(&mut self) -> Option<KeyEvent> {
        if self.read_index == self.write_index {
            None
        } else {
            let event = self.event_buffer[self.read_index];
            self.read_index = (self.read_index + 1) % BUFFER_SIZE;
            Some(event)
        }
    }

    pub fn update_input_buffer(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char(c) if event.pressed => {
                if self.input_len < BUFFER_SIZE {
                    let shift_on = event.modifiers & SHIFT != 0;
                    let caps_on = event.modifiers & CAPS_LOCK != 0;
                    let output_char = if shift_on ^ caps_on {
                        c.to_ascii_uppercase()
                    } else {
                        c
                    };
                    self.input_buffer[self.input_len] = output_char as u8;
                    self.input_len += 1;
                    self.just_deleted = false;
                }
            }
            KeyCode::Control(ControlKey::Backspace) if event.pressed => {
                if self.input_len > 0 {
                    self.input_len -= 1;
                    self.just_deleted = true;
                }else {
                    self.just_deleted = false;

                }
            }
            KeyCode::Control(ControlKey::Enter) if event.pressed => {
                // Move current input to finished buffer
                self.finished_buffer[..self.input_len].copy_from_slice(&self.input_buffer[..self.input_len]);
                self.finished_len = self.input_len;
                self.input_len = 0;
                self.just_deleted = false;
            }
            _ => {}
        }
    }

    pub fn input_buffer_empty(&self) -> bool{
        if self.input_len > 0 || self.just_deleted {
            // serial_println!("Not empty size is {}, {}", self.input_len, self.just_deleted);
                return false
            }
        true
    }

    pub fn get_input_string(&self) -> &str {
        // Return the finished buffer as a str
        unsafe { core::str::from_utf8_unchecked(&self.finished_buffer[..self.finished_len]) }
    }
    pub fn clear_input(&mut self) {
        self.finished_len = 0;
    }
}

// Global keyboard instance
pub static mut KEYBOARD: Keyboard = Keyboard::new();

// Public interface
pub fn handle_keyboard_interrupt(scancode: u8) {
    // serial_println!("{scancode}");
    unsafe {
        KEYBOARD.handle_scancode(scancode);
    }
}

pub fn get_next_key_event() -> Option<KeyEvent> {
    unsafe {
        KEYBOARD.pop_event()
    }
}

pub fn get_input_string() -> &'static str {
    unsafe {
        KEYBOARD.get_input_string()
    }
}
pub fn clear_input() {
    unsafe {
        KEYBOARD.clear_input();
    }
}

pub fn input_buffer_empty() -> bool{
    unsafe {

        KEYBOARD.input_buffer_empty()
    }
}


// Implement Display traits
impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyCode::Char(c) => write!(f, "{}", c),
            KeyCode::Function(n) => write!(f, "F{}", n),
            KeyCode::Control(ctrl) => write!(f, "{:?}", ctrl),
        }
    }
}

impl fmt::Display for KeyEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = [0u8; 64];
        let mut writer = WriterWithoutAlloc::new(&mut buffer);
        
        if self.modifiers & SHIFT != 0 { let _ = writer.write(b"Shift+"); }
        if self.modifiers & CTRL != 0 { let _ = writer.write(b"Ctrl+"); }
        if self.modifiers & ALT != 0 { let _ = writer.write(b"Alt+"); }
        if self.modifiers & CAPS_LOCK != 0 { let _ = writer.write(b"CapsLock+"); }

        match self.code {
            KeyCode::Char(c) => { let _ = writer.write(&[c as u8]); }
            KeyCode::Function(n) => {
                let _ = writer.write(b"F");
                let _ = writer.write_number(n as usize);
            }
            KeyCode::Control(ctrl) => {
                let _ = writer.write(ctrl.as_bytes());
            }
        }

        let _ = writer.write(if self.pressed { b" (pressed)" } else { b" (released)" });
        f.write_str(core::str::from_utf8(writer.as_slice()).unwrap_or("Error"))
    }
}

// Helper struct for writing without allocation
struct WriterWithoutAlloc<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> WriterWithoutAlloc<'a> {
    fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, position: 0 }
    }

    fn write(&mut self, bytes: &[u8]) -> fmt::Result {
        if self.position + bytes.len() > self.buffer.len() {
            return Err(fmt::Error);
        }
        self.buffer[self.position..self.position + bytes.len()].copy_from_slice(bytes);
        self.position += bytes.len();
        Ok(())
    }

    fn write_number(&mut self, mut n: usize) -> fmt::Result {
        let mut digits = [0u8; 20];
        let mut i = 0;
        loop {
            digits[i] = (n % 10) as u8 + b'0';
            n /= 10;
            i += 1;
            if n == 0 { break; }
        }
        for digit in digits[..i].iter().rev() {
            self.write(&[*digit])?;
        }
        Ok(())
    }

    fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.position]
    }
}