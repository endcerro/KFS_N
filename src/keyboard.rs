use core::{fmt, iter::Scan};

use crate::{gdt::print, serial_println};



const KEY_BUFF_SIZE : usize = 1024;




pub struct KeyboardEvents {
    events : [KeyEvent; KEY_BUFF_SIZE],
    idx : usize,
    waiting : usize
}

pub static mut KEYBOARD_EVENTS : KeyboardEvents = KeyboardEvents::new();

pub static mut CHAR_BUFFER : [char; KEY_BUFF_SIZE] = ['\0'; KEY_BUFF_SIZE];

impl KeyboardEvents {
    pub const fn new() -> Self {
        Self {
            events : [KeyEvent::new(); KEY_BUFF_SIZE],
            idx : 0,
            waiting : 0
        }
    }
    fn increment_idx(&mut self) {
        self.idx += 1;
        if self.idx == KEY_BUFF_SIZE{
            self.idx = 0;
        }
     }
    pub fn getEvent(&mut self) -> Option<KeyEvent> {
        
        let curr_idx = self.idx;
        if self.waiting > 0 {
            self.increment_idx();
            self.waiting -= 1;
            return Some(self.events[curr_idx])
        }
        None
    }
    pub fn pushEvent(&mut self,event : KeyEvent) {
        self.waiting += 1;
        let pushed_index;
        pushed_index = (self.waiting + self.idx) % KEY_BUFF_SIZE;
        self.events[pushed_index] = event; 
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifier {
    None : bool,
    RShift: bool,
    RCtrl: bool,
    RAlt: bool,
    LShift: bool,
    LCtrl: bool,
    LAlt: bool,
    CapsLock :bool, 
    LEFT_SIDE_MOD : bool
}

impl Modifier {
    pub const fn new() -> Self
    {
        Self {
            None : true,
            RShift : false,
            RCtrl : false,
            RAlt : false,
            LShift : false,
            LCtrl : false,
            LAlt: false,
            CapsLock : false,
            LEFT_SIDE_MOD : false
        }
    }
    
}


// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum ModifierId {
//     None,
//     RShift,
//     RCtrl,
//     RAlt,
//     LShift,
//     LCtrl,
//     LAlt,
// }
impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        write!(f, "RShift {} RCtrl {} RAlt {}", self.LShift, self.LCtrl, self.LAlt)
        
        // match self {
        //     _ => write!(f, "tmp")
        //     // Modifier::Shift => write!(f, "shift"),
        //     // Modifier::Ctrl => write!(f, "ctrl"),
        //     // Modifier::Alt => write!(f, "alt"),
        //     // Modifier::None => write!(f, ""),
        // }
    }
}

impl fmt::Display for KeyEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.character != None
        {
            return write!(f, "{}", self.character.unwrap());
        }
        write!(f, "")
        // match self {
        //     _ => write!(f, "tmp")
        //     // Modifier::Shift => write!(f, "shift"),
        //     // Modifier::Ctrl => write!(f, "ctrl"),
        //     // Modifier::Alt => write!(f, "alt"),
        //     // Modifier::None => write!(f, ""),
        // }
    }
}

static mut CURRENT_MODIFIER : Modifier = Modifier::new();

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub character: Option<char>,
    pub modifier: Modifier,
    pub scancode: u8,
}

impl KeyEvent{
    pub const fn new() -> Self {
        Self {
            character : None,
            modifier : Modifier::new(),
            scancode : 0
        }
    }
    pub fn to_char(&self) -> char {
        let mut c = self.character.unwrap();
        if (self.modifier.LShift)
        {
            c = c.to_uppercase().next().unwrap();
        }
        c
    }
}

// impl fmt::Display for KeyEvent {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match (self.character, self.modifier) {
//             // (Some(ch), Modifier::None) => write!(f, "{}", ch),
//             (Some(ch), modifier) => write!(f, "{:?}+{}", modifier, ch),
//             (None, modifier) => write!(f, "{:?}", modifier),
//         }
//     }
// }

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

pub fn update_modifier(scancode: u8) -> bool {
    // // if (scancode != 0xe0)
    // {
        // serial_println!("Current code is {:x}", scancode);
    // }
    let mut ret = true;
    unsafe {
       match scancode {
           0x1d => CURRENT_MODIFIER.LCtrl = true,
           0x9d => CURRENT_MODIFIER.LCtrl = false,
           0x38 => CURRENT_MODIFIER.LAlt = true,
           0xb8 => CURRENT_MODIFIER.LAlt = false, 
           0x2a => CURRENT_MODIFIER.LShift = true, 
           0xaa => CURRENT_MODIFIER.LShift = false,
           _ => ret = false
        }

    }
    return  ret;

}
pub fn scancode_to_key_event(scancode: u8) -> KeyEvent {
    
    unsafe {
    let mut event = KeyEvent::new();
    // event.modifier = scancode_to_modifier();
    event.character = scancode_to_char(scancode);
    if event.character == None {
        if update_modifier(scancode)
        {
            // println!("{}", CURRENT_MODIFIER);
        }
    }
    else {
        // println!("{}",event);        
    }
    // update_modifier(scancode);
    // match event.character {
    //     Some(c) => serial_println!("We read {} for {:x}", c, scancode),
    //     _ => ()//serial_println!("Not in map {:x}", scancode)        
    // }
    event.modifier = CURRENT_MODIFIER;
    event
    }

}

// pub fn scancode_to_key_event(scancode: u8, shift: bool) -> KeyEvent {
//     println!("{}", scancode);
//     let modifier = if shift { Modifier::Shift } else { Modifier::None };
//     let character = match scancode {
//         0x02..=0x0A => Some((b'1'..=b'9').nth((scancode - 0x02) as usize).unwrap() as char),
//         0x0B => Some(if shift { ')' } else { '0' }),
//         0x0C => Some(if shift { '_' } else { '-' }),
//         0x0D => Some(if shift { '+' } else { '=' }),
//         0x10..=0x19 => {
//             let base = if shift { b'Q' } else { b'q' };
//             Some((base + (scancode - 0x10)) as char)
//         }
//         0x1E..=0x26 => {
//             let base = if shift { b'A' } else { b'a' };
//             Some((base + (scancode - 0x1E)) as char)
//         }
//         0x2C..=0x32 => {
//             let base = if shift { b'Z' } else { b'z' };
//             Some((base + (scancode - 0x2C)) as char)
//         }
//         0x27 => Some(if shift { ':' } else { ';' }),
//         0x28 => Some(if shift { '"' } else { '\'' }),
//         0x29 => Some(if shift { '~' } else { '`' }),
//         0x2B => Some(if shift { '|' } else { '\\' }),
//         0x33 => Some(if shift { '<' } else { ',' }),
//         0x34 => Some(if shift { '>' } else { '.' }),
//         0x35 => Some(if shift { '?' } else { '/' }),
//         0x39 => Some(' '),
//         _ => None,
//     };

//     KeyEvent {
//         character,
//         modifier,
//         scancode,
//     }
// }

pub fn fill_char_buffer() {

}
pub fn handle_keyboard_interrupt(scancode: u8) {
    // println!("ScanCode is {}", scancode);
    let ch = scancode_to_key_event(scancode);
    if (ch.character != None)
    {
        let c : char;
        c = ch.to_char();
        print!("{}",c);

    }
    unsafe {
        KEYBOARD_EVENTS.pushEvent(ch);
    }

    // match ch.character {
    //     Some(c) => println!("{} {}",c, ch.modifier),
    //     _ => ()
    // }

}