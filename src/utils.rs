use crate::serial_println;

pub fn print_kernel_stack() {
    let stack_top : *const usize;
    let stack_bottom : *const usize;
    unsafe {
        core::arch::asm!("lea {}, [stack_top]
            mov {}, esp",
            out(reg) stack_top, out(reg) stack_bottom,
        );
    }
    let mut current : *const usize = stack_bottom;
    while current != stack_top {
        unsafe  {
            println!("{:p}:{:20x}", current,*current);
            serial_println!("{:p}:{:20x}", current,*current);
            current = current.offset(1);
        }
    }
    unsafe {
        println!("{:p}:{:20x}", current, *current);
        serial_println!("{:p}:{:20x}", current, *current);
    }

}

pub fn memcpy(dest : *mut u8, src : *const u8, size : usize) {
    if dest.is_null() || src.is_null() {
        panic!("memcpy called with null pointers");
    }
    for i in 0..size {
        unsafe {
            *((dest.wrapping_add(i)) as *mut u8) = *(src.wrapping_add(i));
        }
    }
}

pub fn outb(port: u16, value: u8) {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") port, in("al") value);
    }
}

pub fn inb(port: u16) -> u8 {
    let result: u8;
    unsafe {
        core::arch::asm!("in al, dx", out("al") result, in("dx") port);
    }
    result
}

pub fn send_eoi(irq: u8) {
        if irq >= 8 {
            outb(0xA0, 0x20);
        }
        outb(0x20, 0x20);
}

pub unsafe fn enable_interrupts(enable : bool) {
    if enable {
        core::arch::asm!("sti", options(nomem, nostack));
    } else {
        core::arch::asm!("cli", options(nomem, nostack));
    }
}

pub struct Cursor {
	pub x : usize,
	pub y : usize
}
pub enum Direction {
	Top,
	Down,
	Left,
	Right
}