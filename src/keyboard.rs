pub fn scancode_to_ascii(scancode: u8) -> Option<char> {
    match scancode {
        0x1E => Some('a'),
        0x30 => Some('b'),
        // ... more mappings ...
        _ => Some('X'),
    }
}

pub fn handle_keyboard_interrupt(scancode: u8) {
    if let Some(ch) = scancode_to_ascii(scancode) {
        println!("{}", ch);
    }
}