// use crate::m_print;
// use crate::m_println;

// Number of bytes displayed per hexdump row.
const HEXDUMP_ROW: usize = 16;

// Emit one hexdump row to both VGA and serial.
// Format:  0xADDR  XX XX XX XX XX XX XX XX  XX XX XX XX XX XX XX XX  |ASCII....|
//
// `addr`  - virtual address of the first byte in this row
// `bytes` - slice of exactly HEXDUMP_ROW bytes (caller guarantees this)
fn print_hexdump_row(addr: usize, bytes: &[u8]) {
    // --- address column ---
    m_print!("0x{:08x}  ", addr);
    // serial_println!("0x{:08x}  ", addr);  // serial gets its own full line below

    // --- hex columns (split into two groups of 8 with an extra space) ---
    for (i, b) in bytes.iter().enumerate() {
        if i == 8 {
            m_print!(" ");
        } // visual gap between the two halves
        m_print!("{:02x} ", b);
    }

    // --- ASCII preview ---
    m_print!(" |");
    for b in bytes.iter() {
        // Printable ASCII range: 0x20 (' ') through 0x7E ('~')
        let ch = if *b >= 0x20 && *b <= 0x7e {
            *b as char
        } else {
            '.'
        };
        m_print!("{}", ch);
    }
    m_println!("|");

    // Duplicate the full formatted row to the serial port for capture
    // serial_println!("0x{:08x}  {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}  {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}  |{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}|",
    //     addr,
    //     bytes[0],  bytes[1],  bytes[2],  bytes[3],
    //     bytes[4],  bytes[5],  bytes[6],  bytes[7],
    //     bytes[8],  bytes[9],  bytes[10], bytes[11],
    //     bytes[12], bytes[13], bytes[14], bytes[15],
    //     if bytes[0]  >= 0x20 && bytes[0]  <= 0x7e { bytes[0]  as char } else { '.' },
    //     if bytes[1]  >= 0x20 && bytes[1]  <= 0x7e { bytes[1]  as char } else { '.' },
    //     if bytes[2]  >= 0x20 && bytes[2]  <= 0x7e { bytes[2]  as char } else { '.' },
    //     if bytes[3]  >= 0x20 && bytes[3]  <= 0x7e { bytes[3]  as char } else { '.' },
    //     if bytes[4]  >= 0x20 && bytes[4]  <= 0x7e { bytes[4]  as char } else { '.' },
    //     if bytes[5]  >= 0x20 && bytes[5]  <= 0x7e { bytes[5]  as char } else { '.' },
    //     if bytes[6]  >= 0x20 && bytes[6]  <= 0x7e { bytes[6]  as char } else { '.' },
    //     if bytes[7]  >= 0x20 && bytes[7]  <= 0x7e { bytes[7]  as char } else { '.' },
    //     if bytes[8]  >= 0x20 && bytes[8]  <= 0x7e { bytes[8]  as char } else { '.' },
    //     if bytes[9]  >= 0x20 && bytes[9]  <= 0x7e { bytes[9]  as char } else { '.' },
    //     if bytes[10] >= 0x20 && bytes[10] <= 0x7e { bytes[10] as char } else { '.' },
    //     if bytes[11] >= 0x20 && bytes[11] <= 0x7e { bytes[11] as char } else { '.' },
    //     if bytes[12] >= 0x20 && bytes[12] <= 0x7e { bytes[12] as char } else { '.' },
    //     if bytes[13] >= 0x20 && bytes[13] <= 0x7e { bytes[13] as char } else { '.' },
    //     if bytes[14] >= 0x20 && bytes[14] <= 0x7e { bytes[14] as char } else { '.' },
    //     if bytes[15] >= 0x20 && bytes[15] <= 0x7e { bytes[15] as char } else { '.' },
    // );
}

pub fn print_kernel_stack() {
    let stack_top_addr: usize;
    let esp: usize;
    unsafe {
        core::arch::asm!(
            "lea {top}, [stack_top]",
            "mov {esp}, esp",
            top = out(reg) stack_top_addr,
            esp = out(reg) esp,
        );
    }

    // The stack grows downward: ESP (stack_bottom of live data) → stack_top symbol.
    // We walk from ESP up to stack_top, inclusive.
    let start = esp;
    let end = stack_top_addr;

    if start >= end {
        m_println!(
            "Stack: nothing to dump (esp={:#x} >= stack_top={:#x})",
            start,
            end
        );
        return;
    }

    let total = end - start;
    m_println!(
        "\nKernel stack dump  ESP={:#010x}  TOP={:#010x}  ({} bytes)\n",
        start,
        end,
        total
    );
    // serial_println!("\nKernel stack dump  ESP={:#010x}  TOP={:#010x}  ({} bytes)\n",
    //     start, end, total);

    // Align the dump start down to a HEXDUMP_ROW boundary so the address
    // column always shows a multiple of 16 (classic hexdump convention).
    let aligned_start = start & !(HEXDUMP_ROW - 1);

    let mut row_addr = aligned_start;
    let mut row_buf = [0u8; HEXDUMP_ROW];

    while row_addr < end {
        for i in 0..HEXDUMP_ROW {
            let byte_addr = row_addr + i;
            row_buf[i] = if byte_addr >= start && byte_addr < end {
                // Live stack byte - safe to read
                unsafe { *(byte_addr as *const u8) }
            } else {
                // Padding before the first live byte in the first row
                0x00
            };
        }
        print_hexdump_row(row_addr, &row_buf);
        row_addr += HEXDUMP_ROW;
    }

    m_println!();
}

pub fn memcpy(dest: *mut u8, src: *const u8, size: usize) {
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

pub fn outw(port: u16, value: u16) {
    unsafe {
        core::arch::asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") value,
            options(nostack, nomem)
        );
    }
}

pub fn inw(port: u16) -> u16 {
    let result: u16;
    unsafe {
        core::arch::asm!(
            "in dx, ax",
            out("ax") result,
            in("dx") port,
            options(nostack, nomem)
        );
    }
    result
}

pub fn send_eoi(irq: u8) {
    if irq >= 8 {
        outb(0xA0, 0x20);
    }
    outb(0x20, 0x20);
}

pub unsafe fn enable_interrupts(enable: bool) {
    if enable {
        core::arch::asm!("sti", options(nomem, nostack));
    } else {
        core::arch::asm!("cli", options(nomem, nostack));
    }
}

pub struct Cursor {
    pub x: usize,
    pub y: usize,
}
pub enum Direction {
    Top,
    Down,
    Left,
    Right,
}
