#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![no_main]
#[macro_use]
pub mod vga;
pub mod gdt;
pub mod keyboard;
//pub mod memory;
pub mod multiboot2;
pub mod utils;
pub mod interrupts;
pub mod serial;
pub mod shell;
pub mod commands;

use core::{panic::PanicInfo, ptr::addr_of};

use keyboard::{get_next_key_event, ControlKey, KeyCode, KeyEvent, CTRL, KEYBOARD};
use vga::{ColorCode, Direction, WRITER};


// use keyboard::{KeyCode, KEYBOARD_BUFFER};

extern "C" {
    static _kernel_start : u8;
    static _kernel_end : u8;
}

#[no_mangle]
pub extern "C" fn rust_main(_multiboot_struct_ptr: *const multiboot2::MultibootInfoHeader) -> ! {
    init();
    // unsafe {
    let size = addr_of!(_kernel_end) as u32 - addr_of!(_kernel_start) as u32 ;
    //     // size = size /8;
    println!("The size of this kernel is {} kbytes", size / (1024));
    println!("The size of this kernel is {} mbytes", (size / (1024 * 1024)));
    serial_println!("Hello from serial port!");
    serial_println!("Kernel size: {} kbytes", size / 1024);
    shell();
//    test_move_cursors(); 
    
    

}

fn shell() -> !
{
    let mut paint :bool;
    loop {
        paint = false;
        shell::processor::hello_shell();
        loop {
        if let Some(event) = get_next_key_event() {
            if event.pressed == true {
                    // println!("{event}");
                match event.code {
                    KeyCode::Control(ControlKey::Enter) => break,
                    KeyCode::Char(c) => 
                    {
                        if event.modifiers == CTRL && c == '2'{
                            paint = true;
                            break;
                        }
                        print!("{c}")
                    },
                    KeyCode::Control(ControlKey::Backspace) => {
                        if !keyboard::input_buffer_empty()
                        {
                            WRITER.lock().delete_char();
                        }
                    },
                    _ => ()}}}
        }
        if paint {
            test_move_cursors();
        } else {
            let len = keyboard::get_input_string();
            shell::processor::process_command(len);
        }

    }
}

fn test_move_cursors()
{
    vga::clear_screen();
    static mut PAINT_BUFFER : vga::Buffer = vga::Buffer{
        chars : [[vga::ScreenCharacter {
            ascii_value : b' ', 
            color : ColorCode::new(vga::Color::White, vga::Color::Black)
        }; vga::VGA_BUFFER_WIDTH]; vga::VGA_BUFFER_HEIGHT] };
        unsafe {
            utils::enable_interrupts(false);

            WRITER.lock().buffer.copy_from(addr_of!(PAINT_BUFFER).as_ref().unwrap());
            utils::enable_interrupts(true);
        }
    WRITER.lock().cursor.x = 0;
    WRITER.lock().cursor.y = 0;
    WRITER.lock().cursor.update_cursor(0, 0);
    let mut done : bool = false;
    while done == false {
        loop {
                if let Some(event) = get_next_key_event() {
                if event.pressed == true {

                    match event.code {
                        KeyCode::Control(ControlKey::UpArrow) => WRITER.lock().cursor.move_cursors(Direction::Top),
                        KeyCode::Control(ControlKey::DownArrow) => WRITER.lock().cursor.move_cursors(Direction::Down),
                        KeyCode::Control(ControlKey::LeftArrow) => WRITER.lock().cursor.move_cursors(Direction::Left),
                        KeyCode::Control(ControlKey::RightArrow) => WRITER.lock().cursor.move_cursors(Direction::Right),
                        KeyCode::Char(c) =>  {
                            if event.modifiers == CTRL && c == '1'{
                                done = true;
                                break;
                            }
                            WRITER.lock().write_byte_at_cursor(c as u8 );
                        }
                        _ => ()
                    }
                }
            }
        }
    }
    unsafe {
        utils::enable_interrupts(false);
        PAINT_BUFFER.copy_from(WRITER.lock().buffer);
        WRITER.lock().clear_screen();
        let key : KeyEvent = KeyEvent {
            code : KeyCode::Control(ControlKey::Enter),
            modifiers : 0,
            pressed : true,
        };
        KEYBOARD.update_input_buffer(&key);
        let key : KeyEvent = KeyEvent {
            code : KeyCode::Control(ControlKey::Enter),
            modifiers : 0,
            pressed : false,
        };
        KEYBOARD.update_input_buffer(&key);

        utils::enable_interrupts(true);
    }
    // memcpy(addr_of!(paint_buffer.chars) as *mut u8, VGA_BUFFER_ADDR as *const u8, size_of::<vga::Buffer>());
}


fn init() {
    WRITER.lock().change_color(Some(vga::Color::White), Some(vga::Color::Black));
    WRITER.lock().cursor.enable_cursor(1, 10);
    serial::init();
    vga::clear_screen();
    vga::print_ft();
    
    gdt::init();
    interrupts::init();

}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}
