
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

/*
	This file contains helper functions to allow us to print to the screen in an easy
	manner.
	Please not that this only uses character mode and needs another implmentation for a
	pixel buffer
 */

pub const VGA_BUFFER_ADDR : u32 = 0xb8000;

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
	pub const fn new(foreground : Color, background: Color) -> ColorCode {
		ColorCode((background as u8) << 4 | (foreground as u8))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenCharacter {
	pub ascii_value : u8,
	pub color : ColorCode
}


pub const VGA_BUFFER_HEIGHT : usize = 25;
pub const VGA_BUFFER_WIDTH : usize = 80;

#[repr(transparent)]
pub struct Buffer {
	pub chars: [[ScreenCharacter; VGA_BUFFER_WIDTH]; VGA_BUFFER_HEIGHT]
}

impl Buffer {
	pub fn copy_from(&mut self, other : &Buffer) {
		for i in 0..VGA_BUFFER_HEIGHT {
			for j in 0..VGA_BUFFER_WIDTH {
				self.chars[i][j] = other.chars[i][j]
			}
		}
	}
}

pub struct Writer {
	column_position : usize,
	row_position : usize,
	color_code : ColorCode,
	pub background : Color,
	pub foreground : Color,
	pub cursor : Cursor,
	pub buffer : &'static mut Buffer,
}
use spin::Mutex;
use lazy_static::lazy_static;

use crate::utils::{Cursor, Direction};

lazy_static! {
	pub static ref WRITER : Mutex<Writer> = Mutex::new(Writer::new());
}

impl Writer {
	pub const fn new() -> Self {
		Self {
			column_position : 0,
			row_position: 0,
			color_code : ColorCode::new(Color::White, Color::Black),
			background : Color::Black,
			foreground : Color::White,
			cursor : Cursor::new(),
			buffer : unsafe { &mut *(VGA_BUFFER_ADDR as *mut Buffer)}
		}
	}
	pub fn write_byte(&mut self, byte: u8) {
		match byte {
			b'\n' => self.new_line(),
			byte => {
				if self.column_position >= VGA_BUFFER_WIDTH {
					self.new_line()
				}
				self.buffer.chars[self.row_position][self.column_position] = ScreenCharacter {
					ascii_value: byte,
					color : self.color_code
				};
				self.column_position +=1;
			}
		}
		if self.column_position > 0
		{
			self.cursor.update_cursor(self.column_position, self.row_position);
		}
	}
	pub fn write_byte_at_pos(&mut self, byte: u8, x : usize, y : usize){
		match byte {
			byte => {
				if self.column_position >= VGA_BUFFER_WIDTH {
					self.new_line()
				}
				self.buffer.chars[x][y] = ScreenCharacter {
					ascii_value: byte,
					color : self.color_code
				};
			}
		}
	}
	pub fn write_byte_at_cursor(&mut self, byte: u8)
	{
		match byte {
			byte => {
				self.buffer.chars[self.cursor.y][self.cursor.x] = ScreenCharacter {
					ascii_value: byte,
					color : self.color_code
				};
			}
		}
	}
	pub fn write_char(&mut self, char : char) {
		match char {
			'\t' => for _ in 0..4 {self.write_byte(b' ');}
			_ => self.write_byte(char as u8)
		}
	}
	pub fn write_char_at_cursor(&mut self, char : char) {
		match char {
			_ => self.write_byte(char as u8)
		}
	}
	pub fn write_string(&mut self, str : &str) {
		for char in str.bytes() {
			match char {
				0x20..=0x7e | 0x0A => self.write_char(char as char),
				_ => self.write_byte(0xfe)
			}
		}
	}
	fn new_line(&mut self) {
		if self.row_position < VGA_BUFFER_HEIGHT - 1{
			self.row_position += 1;
			self.column_position = 0;
			return;
		}
		for row in 1..VGA_BUFFER_HEIGHT {
			for col in 0..VGA_BUFFER_WIDTH {
				let character = self.buffer.chars[row][col];
				self.buffer.chars[row - 1][col] = character;
			}
		}
		self.clear_row(self.row_position);
		self.column_position = 0;
		// self.row_position = 0;

	}
	fn clear_row(&mut self, index : usize){
		for col in 0..VGA_BUFFER_WIDTH {
			self.buffer.chars[index][col] = ScreenCharacter {
				ascii_value : 0x20,
				color : self.color_code
			};
		}
	}
	pub fn clear_screen(&mut self){
		for row in 0..VGA_BUFFER_HEIGHT {
			for col in 0..VGA_BUFFER_WIDTH {
				self.buffer.chars[row][col] = ScreenCharacter {
					ascii_value : 0x20,
					color : self.color_code
				};
			}
		}
		self.column_position = 0;
		self.row_position = 0;
	}
	pub fn delete_char(&mut self){
	{
		if self.column_position > 0
		{
			self.column_position -= 1;
		}
		self.write_byte(b' ');
		self.column_position -= 1;
		if self.column_position > 0
		{
			self.cursor.update_cursor(self.column_position, self.row_position);
		}
			}
		}
	pub fn change_color(&mut self, foreground : Option<Color>,  background : Option<Color>){
		match background {
			Some(c) => self.background = c,
			_ => ()
		}
		match foreground {
			Some(c) => self.foreground = c,
			_ => ()
		}
		self.color_code = ColorCode::new(self.foreground, self.background);
	}
}


impl core::fmt::Write for Writer {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		self.write_string(s);
		Ok(())
	}
}


pub fn delete_char() {
	WRITER.lock().delete_char();
}

pub fn clear_screen() {
	WRITER.lock().clear_screen();
}

pub fn print_ft() {

	let old_foreground = WRITER.lock().foreground;
	let mut foreground_color = old_foreground;
	for c in HEADER_42.bytes() {
		match c {
			b'\n' => {
				WRITER.lock().new_line();
				foreground_color = foreground_color.cycle();
				while foreground_color == WRITER.lock().background  {
					foreground_color = foreground_color.cycle();
				 }
				WRITER.lock().change_color(Some(foreground_color), None);

			},
			c => WRITER.lock().write_char(c as char)
		}
	}
	WRITER.lock().change_color(Some(old_foreground), None);
	WRITER.lock().write_char('\n');
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
pub fn _print(args: core::fmt::Arguments) {
	use core::fmt::Write;
	WRITER.lock().write_fmt(args).unwrap();
}




impl Cursor {
	pub const fn new() -> Self
	{
		Self {
			x : 0,
			y : 0
		}
	}
	pub fn enable_cursor(&mut self, start :u8, end :u8)
	{
		crate::utils::outb(0x3D4, 0x0A);
		crate::utils::outb(0x3D5, (crate::utils::inb(0x3D5) & 0xC0) | start);

		crate::utils::outb(0x3D4, 0x0B);
		crate::utils::outb(0x3D5, (crate::utils::inb(0x3D5) & 0xE0) | end);
		self.update_cursor(0,0);
	}
	pub fn disable_cursor()
	{
		crate::utils::outb(0x3D4, 0x0A);
		crate::utils::outb(0x3D5, 0x20);
	}
	pub fn update_cursor(&mut self, x : usize,  y : usize)
	{
		let pos = y * VGA_BUFFER_WIDTH + x;

		crate::utils::outb(0x3D4, 0x0F);
		crate::utils::outb(0x3D5,  (pos & 0xFF) as u8);
		crate::utils::outb(0x3D4, 0x0E);
		crate::utils::outb(0x3D5, ((pos >> 8) & 0xFF)as u8);
	}
	pub fn move_cursors(&mut self, dir : Direction) {
		match dir {
			Direction::Top => if self.y > 0 {self.y -= 1},
			Direction::Down => if self.y < VGA_BUFFER_HEIGHT - 1  {self.y += 1},
			Direction::Left => if self.x > 0 {self.x -= 1},
			Direction::Right => if self.x < VGA_BUFFER_WIDTH - 1 {self.x += 1}
		}
		// serial_print!("Moving cursor to {}, {}", self.x,self.y);
		self.update_cursor(self.x, self.y);
	}
}