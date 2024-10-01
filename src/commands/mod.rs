use core::ptr::addr_of;

use crate::{keyboard::{self, get_next_key_event, ControlKey, KeyCode, KeyEvent, CTRL, KEYBOARD}, shell::processor::{hello_shell, SHELL}, utils, vga::{self, ColorCode, Direction, WRITER}};
#[allow(static_mut_refs)]
pub mod echo;
pub mod clear;
pub mod credits;
pub mod custom;
pub mod print_ft_42;
pub mod print_stack;

pub fn shell() -> ! {
    let mut paint_mode :bool;
    loop {
        paint_mode = false;
        hello_shell();
        loop {
        if let Some(event) = get_next_key_event() {
            if event.pressed == true {
                    // println!("{event}");
                match event.code {
                    KeyCode::Control(ControlKey::Enter) => break,
                    KeyCode::Char(c) =>
                    {
                        if event.modifiers == CTRL && c == '2'{
                            paint_mode = true;
                            break;
                        }
                        print!("{c}")
                    },
                    KeyCode::Control(ControlKey::Backspace) => {
                        if !keyboard::input_buffer_empty() {
                            WRITER.lock().delete_char();
                        }},
                    _ => ()}}}}
        if paint_mode {
            paint();
        } else {
            let len = keyboard::get_input_string();
            unsafe {
                SHELL.run_command(len);
            }
        }

    }
}

pub fn paint()
{
    vga::clear_screen();
    #[allow(static_mut_refs)]
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
}