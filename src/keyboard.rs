use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    None,
    Shift,
    Ctrl,
    Alt,
}
impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Modifier::Shift => write!(f, "shift"),
            Modifier::Ctrl => write!(f, "ctrl"),
            Modifier::Alt => write!(f, "alt"),
            Modifier::None => write!(f, ""),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub character: Option<char>,
    pub modifier: Modifier,
    pub scancode: u8,
}

impl fmt::Display for KeyEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.character, self.modifier) {
            (Some(ch), Modifier::None) => write!(f, "{}", ch),
            (Some(ch), modifier) => write!(f, "{:?}+{}", modifier, ch),
            (None, modifier) => write!(f, "{:?}", modifier),
        }
    }
}
pub fn scancode_to_key_event(scancode: u8, shift: bool) -> KeyEvent {
    println!("{}", scancode);
    let modifier = if shift { Modifier::Shift } else { Modifier::None };
    let character = match scancode {
        0x02..=0x0A => Some((b'1'..=b'9').nth((scancode - 0x02) as usize).unwrap() as char),
        0x0B => Some(if shift { ')' } else { '0' }),
        0x0C => Some(if shift { '_' } else { '-' }),
        0x0D => Some(if shift { '+' } else { '=' }),
        0x10..=0x19 => {
            let base = if shift { b'Q' } else { b'q' };
            Some((base + (scancode - 0x10)) as char)
        }
        0x1E..=0x26 => {
            let base = if shift { b'A' } else { b'a' };
            Some((base + (scancode - 0x1E)) as char)
        }
        0x2C..=0x32 => {
            let base = if shift { b'Z' } else { b'z' };
            Some((base + (scancode - 0x2C)) as char)
        }
        0x27 => Some(if shift { ':' } else { ';' }),
        0x28 => Some(if shift { '"' } else { '\'' }),
        0x29 => Some(if shift { '~' } else { '`' }),
        0x2B => Some(if shift { '|' } else { '\\' }),
        0x33 => Some(if shift { '<' } else { ',' }),
        0x34 => Some(if shift { '>' } else { '.' }),
        0x35 => Some(if shift { '?' } else { '/' }),
        0x39 => Some(' '),
        _ => None,
    };

    KeyEvent {
        character,
        modifier,
        scancode,
    }
}
pub fn handle_keyboard_interrupt(scancode: u8) {
    let ch = scancode_to_key_event(scancode, false);
    match ch.character {
        Some(c) => println!("{} {}",c, ch.modifier),
        _ => ()
    }

}