pub const VGA_BUFFER_ADDR: u32 = 0xC00b8000;

pub const HEADER_42: &str = "         :::        ::::::::
	   :+:        :+:    :+:
	 +:+ +:+           +:+
   +#+  +:+         +#+
 +#+#+#+#+#+     +#+
	  #+#      #+#
	 ###     ##########";

// How many spaces a tab character expands to.
const TAB_WIDTH: usize = 4;

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
    White = 15,
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
            White => Black,
        }
    }

    pub fn all() -> impl Iterator<Item = Color> {
        (0..=15).map(|i| unsafe { core::mem::transmute(i as u8) })
    }
}

impl core::str::FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "black" => Ok(Color::Black),
            "blue" => Ok(Color::Blue),
            "green" => Ok(Color::Green),
            "cyan" => Ok(Color::Cyan),
            "red" => Ok(Color::Red),
            "magenta" => Ok(Color::Magenta),
            "brown" => Ok(Color::Brown),
            "lightgray" => Ok(Color::LightGray),
            "darkgray" => Ok(Color::DarkGray),
            "lightblue" => Ok(Color::LightBlue),
            "lightgreen" => Ok(Color::LightGreen),
            "lightcyan" => Ok(Color::LightCyan),
            "lightred" => Ok(Color::LightRed),
            "pink" => Ok(Color::Pink),
            "yellow" => Ok(Color::Yellow),
            "white" => Ok(Color::White),
            _ => Err(()),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenCharacter {
    pub ascii_value: u8,
    pub color: ColorCode,
}

pub const VGA_BUFFER_HEIGHT: usize = 25;
pub const VGA_BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    pub chars: [[ScreenCharacter; VGA_BUFFER_WIDTH]; VGA_BUFFER_HEIGHT],
}

impl Buffer {
    pub fn copy_from(&mut self, other: &Buffer) {
        for i in 0..VGA_BUFFER_HEIGHT {
            for j in 0..VGA_BUFFER_WIDTH {
                self.chars[i][j] = other.chars[i][j];
            }
        }
    }
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    pub background: Color,
    pub foreground: Color,
    pub cursor: Cursor,
    pub buffer: &'static mut Buffer,
}

use crate::utils::{Cursor, Direction};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
}

impl Writer {
    pub const fn new() -> Self {
        Self {
            column_position: 0,
            row_position: 0,
            color_code: ColorCode::new(Color::White, Color::Black),
            background: Color::Black,
            foreground: Color::White,
            cursor: Cursor::new(),
            buffer: unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer) },
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= VGA_BUFFER_WIDTH {
                    self.new_line();
                }
                self.buffer.chars[self.row_position][self.column_position] = ScreenCharacter {
                    ascii_value: byte,
                    color: self.color_code,
                };
                self.column_position += 1;
            }
        }
        self.cursor
            .update_cursor(self.column_position, self.row_position);
    }

    pub fn write_at(&mut self, col: usize, row: usize, byte: u8) {
        if col < VGA_BUFFER_WIDTH && row < VGA_BUFFER_HEIGHT {
            self.buffer.chars[row][col] = ScreenCharacter {
                ascii_value: byte,
                color: self.color_code,
            };
        }
    }

    pub fn write_byte_at_cursor(&mut self, byte: u8) {
        let (x, y) = (self.cursor.x, self.cursor.y);
        if x < VGA_BUFFER_WIDTH && y < VGA_BUFFER_HEIGHT {
            self.buffer.chars[y][x] = ScreenCharacter {
                ascii_value: byte,
                color: self.color_code,
            };
        }
    }

    pub fn write_char(&mut self, char: char) {
        match char {
            '\t' => {
                for _ in 0..TAB_WIDTH {
                    self.write_byte(b' ');
                }
            }
            _ => self.write_byte(char as u8),
        }
    }

    pub fn write_char_at_cursor(&mut self, char: char) {
        self.write_byte_at_cursor(char as u8);
    }

    pub fn write_string(&mut self, str: &str) {
        for char in str.bytes() {
            match char {
                0x20..=0x7e | 0x0A => self.write_char(char as char),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_position < VGA_BUFFER_HEIGHT - 1 {
            self.row_position += 1;
            self.column_position = 0;
            return;
        }
        for row in 1..VGA_BUFFER_HEIGHT {
            for col in 0..VGA_BUFFER_WIDTH {
                self.buffer.chars[row - 1][col] = self.buffer.chars[row][col];
            }
        }
        self.clear_row(VGA_BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, index: usize) {
        let blank = ScreenCharacter {
            ascii_value: 0x20,
            color: self.color_code,
        };
        for col in 0..VGA_BUFFER_WIDTH {
            self.buffer.chars[index][col] = blank;
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..VGA_BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.row_position = 0;
        self.column_position = 0;
        self.cursor.update_cursor(0, 0);
    }

    pub fn delete_char(&mut self) {
        if self.column_position == 0 {
            return;
        }
        self.column_position -= 1;

        self.buffer.chars[self.row_position][self.column_position] = ScreenCharacter {
            ascii_value: b' ',
            color: self.color_code,
        };
        self.cursor
            .update_cursor(self.column_position, self.row_position);
    }

    pub fn change_color(&mut self, foreground: Option<Color>, background: Option<Color>) {
        if let Some(c) = background {
            self.background = c;
        }
        if let Some(c) = foreground {
            self.foreground = c;
        }
        self.color_code = ColorCode::new(self.foreground, self.background);
    }

    pub fn get_color(&self) -> (Color, Color) {
        (self.foreground, self.background)
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Public helpers - each acquires the lock exactly once per call.
// ---------------------------------------------------------------------------

pub fn delete_char() {
    WRITER.lock().delete_char();
}

pub fn clear_screen() {
    WRITER.lock().clear_screen();
}

pub fn print_ft() {
    let flags: u32;
    //Save the state of the interrupts and disable them
    unsafe {
        core::arch::asm!("pushfd; pop {}", out(reg) flags, options(nomem));
        core::arch::asm!("cli", options(nomem, nostack));
    }

    {
        let mut writer = WRITER.lock();
        let old_foreground = writer.foreground;
        let mut foreground_color = old_foreground;

        for c in HEADER_42.bytes() {
            if c == b'\n' {
                writer.new_line();
                foreground_color = foreground_color.cycle();
                // Skip colors that match the background so text is readable
                while foreground_color == writer.background {
                    foreground_color = foreground_color.cycle();
                }
                writer.change_color(Some(foreground_color), None);
            } else {
                writer.write_char(c as char);
            }
        }
        writer.change_color(Some(old_foreground), None);
        writer.write_char('\n');
    } // lock released here

    // Restore the interrupt flag
    unsafe {
        core::arch::asm!("push {}; popfd", in(reg) flags, options(nomem));
    }
}

// ---------------------------------------------------------------------------
// Macros
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! print {
	($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
	() => ($crate::print!("\n"));
	($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! colored_println {
	() => ($crate::print!("\n"));
	(($fg:expr, $bg:expr), $($arg:tt)*) => {{
		$crate::colored_print!(($fg, $bg), $($arg)*);
		$crate::print!("\n");
	}};
}

macro_rules! colored_print {
	(($fg:expr, $bg:expr), $($arg:tt)*) => {{
		$crate::vga::_print_color(format_args!($($arg)*), ($fg, $bg));
	}};
}

// ---------------------------------------------------------------------------
// Internal print helpers - each acquires the lock exactly once.
// ---------------------------------------------------------------------------

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _print_color(args: core::fmt::Arguments, colors: (Option<Color>, Option<Color>)) {
    use core::fmt::Write;
    let mut writer = WRITER.lock();
    let (oldfg, oldbg) = writer.get_color();
    writer.change_color(colors.0, colors.1);
    writer.write_fmt(args).unwrap();
    writer.change_color(Some(oldfg), Some(oldbg));
    // Lock released here - color is restored inside the same critical section
}

pub fn get_current_colors() -> (Color, Color) {
    let writer = WRITER.lock();
    writer.get_color()
}
// Clear a region of VGA row with spaces.
pub fn vga_clear_region(row: usize, col_start: usize, col_end: usize, color: ColorCode) {
    let buf = unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer) };
    for c in col_start..col_end {
        if c >= VGA_BUFFER_WIDTH {
            break;
        }
        buf.chars[row][c] = ScreenCharacter {
            ascii_value: b' ',
            color,
        };
    }
}

// Write a string to VGA at (row, start_col) with the given color.
// Does not move the cursor or affect Writer state.
pub fn vga_write_at(row: usize, col: usize, s: &[u8], color: ColorCode) {
    let buf = unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer) };
    for (i, &byte) in s.iter().enumerate() {
        let c = col + i;
        if c >= VGA_BUFFER_WIDTH {
            break;
        }
        buf.chars[row][c] = ScreenCharacter {
            ascii_value: byte,
            color,
        };
    }
}

// ---------------------------------------------------------------------------
// Cursor
// ---------------------------------------------------------------------------

impl Cursor {
    pub const fn new() -> Self {
        Self { x: 0, y: 0 }
    }

    pub fn enable_cursor(&mut self, start: u8, end: u8) {
        crate::utils::outb(0x3D4, 0x0A);
        crate::utils::outb(0x3D5, (crate::utils::inb(0x3D5) & 0xC0) | start);
        crate::utils::outb(0x3D4, 0x0B);
        crate::utils::outb(0x3D5, (crate::utils::inb(0x3D5) & 0xE0) | end);
        self.update_cursor(0, 0);
    }

    pub fn disable_cursor() {
        crate::utils::outb(0x3D4, 0x0A);
        crate::utils::outb(0x3D5, 0x20);
    }

    pub fn update_cursor(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
        let pos = y * VGA_BUFFER_WIDTH + x;
        crate::utils::outb(0x3D4, 0x0F);
        crate::utils::outb(0x3D5, (pos & 0xFF) as u8);
        crate::utils::outb(0x3D4, 0x0E);
        crate::utils::outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
    }

    pub fn move_cursors(&mut self, dir: Direction) {
        match dir {
            Direction::Top => {
                if self.y > 0 {
                    self.y -= 1;
                }
            }
            Direction::Down => {
                if self.y < VGA_BUFFER_HEIGHT - 1 {
                    self.y += 1;
                }
            }
            Direction::Left => {
                if self.x > 0 {
                    self.x -= 1;
                }
            }
            Direction::Right => {
                if self.x < VGA_BUFFER_WIDTH - 1 {
                    self.x += 1;
                }
            }
        }
        self.update_cursor(self.x, self.y);
    }
}
