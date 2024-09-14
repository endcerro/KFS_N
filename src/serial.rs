use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::utils::{outb, inb};

const PORT: u16 = 0x3F8; // COM1

pub struct SerialPort {
    port: u16,
}

impl SerialPort {
    pub const fn new(port: u16) -> SerialPort {
        SerialPort { port }
    }

    pub fn init(&mut self) {
        unsafe {
            // Disable all interrupts
            outb(self.port + 1, 0x00);
            // Enable DLAB (set baud rate divisor)
            outb(self.port + 3, 0x80);
            // Set divisor to 3 (lo byte) 38400 baud
            outb(self.port + 0, 0x03);
            // (hi byte)
            outb(self.port + 1, 0x00);
            // 8 bits, no parity, one stop bit
            outb(self.port + 3, 0x03);
            // Enable FIFO, clear them, with 14-byte threshold
            outb(self.port + 2, 0xC7);
            // IRQs enabled, RTS/DSR set
            outb(self.port + 4, 0x0B);
        }
    }

    fn is_transmit_empty(&mut self) -> bool {
        unsafe { inb(self.port + 5) & 0x20 != 0 }
    }

    pub fn write_byte(&mut self, byte: u8) {
        while !self.is_transmit_empty() {}
        unsafe { outb(self.port, byte) }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = Mutex::new(SerialPort::new(PORT));
}

pub fn init() {
    SERIAL1.lock().init();
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

// These functions should be defined in your utils.rs file
