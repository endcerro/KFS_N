
// const HEADER : &str =
// "/* ************************************************************************** */\n\
// /*                                                                            */\n\
// /*                                                        :::      ::::::::   */\n\
// /*   kfs.rs                                             :+:      :+:    :+:   */\n\
// /*                                                    +:+ +:+         +:+     */\n\
// /*   By: edal--ce <edal--ce@student.42.fr>          +#+  +:+       +#+        */\n\
// /*                                                +#+#+#+#+#+   +#+           */\n\
// /*   Created: 2023/11/04 14:44:15 by edal--ce          #+#    #+#             */\n\
// /*   Updated: 2019/12/28 08:17:21 by edal--ce         ###   ########.fr       */\n\
// /*                                                                            */\n\
// /* ************************************************************************** */\n";

pub const HEADER_42 : &str = 
"         :::        ::::::::
       :+:        :+:    :+:
     +:+ +:+           +:+
   +#+  +:+         +#+
 +#+#+#+#+#+     +#+
      #+#      #+#
     ###     ##########";

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15
    
}
impl Color {
    fn cycle(&self) -> Self {
        use Color::*;
        match *self {
            Black => Blue,
            Blue => Green,
            Green => Cyan,
            Cyan => Red,
            Red => Magenta,
            Magenta => Brown,
            Brown => LightGray,
            LightGray => DarkGray,
            DarkGray => LightBlue,
            LightBlue => LightGreen,
            LightGreen => LightCyan,
            LightCyan => LightRed,
            LightRed => Pink,
            Pink => Yellow,
            Yellow => White,
            White => Black
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground : Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenCharacter {
    ascii_value : u8,
    color : ColorCode
}

const BUFFER_HEIGHT : usize = 25;
const BUFFER_WIDTH : usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenCharacter; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    column_position : usize,
    _row_position : usize,
    color_code : ColorCode,
    buffer : &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line()
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col] = ScreenCharacter {
                    ascii_value: byte,
                    color : color_code
                };
                self.column_position +=1;
            }
        }
    }
    pub fn write_string(&mut self, str : &str) {
        for char in str.bytes() {
            match char {
                0x20..=0x7e | 0x0A => self.write_byte(char),
                _ => self.write_byte(0xfe)
            }
        }
    }
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col];
                self.buffer.chars[row - 1][col] = character;
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }
    fn clear_row(&mut self, index : usize){
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[index][col] = ScreenCharacter {
                ascii_value : 0x20,
                color : ColorCode::new(Color::Black, Color::Black)
            };
        }
    }
    pub fn clear_screen(&mut self){
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col] = ScreenCharacter {
                    ascii_value : 0x20,
                    color : ColorCode::new(Color::Black, Color::Black)
                };
            }
        }
        
    }
}

use core::fmt;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn _print_test() {
    let mut writer = Writer {
        column_position: 0,
        _row_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Red),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer)},
    };
    writer.write_string("HELLO WORLD !");
    writer.new_line();
    writer.write_byte(b'4');
    writer.write_byte(b'2');
    writer.new_line();
    writer.write_string(HEADER_42);
}

pub fn clear_screen() {
    let mut writer = Writer {
        column_position: 0,
        _row_position: 0,
        color_code: ColorCode::new(Color::Black, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer)},
    };
    writer.clear_screen();
}

pub fn print_ft() {
    let mut current_color = Color::Blue;
    let mut writer = Writer {
        column_position: 0,
        _row_position: 0,
        color_code: ColorCode::new(current_color, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer)},
    };
    for c in HEADER_42.bytes() {
        match c {
            b'\n' => {
                writer.new_line();
                current_color = current_color.cycle();
                writer.color_code = ColorCode::new(current_color, Color::Black);
            },
            c => writer.write_byte(c)
        }
    }
}

use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref WRITER : Mutex<Writer> = Mutex::new(Writer {
        column_position : 0,
        _row_position : 0,
        color_code : ColorCode::new(Color::LightGreen, Color::Black),
        buffer : unsafe {
            &mut *(0xb8000 as *mut Buffer)
        },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}