use core::ptr::addr_of;

use crate::vga::{self, *};
use crate::utils::{self, *};
use crate::keyboard::*;

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

            vga::WRITER.lock().buffer.copy_from(addr_of!(PAINT_BUFFER).as_ref().unwrap());
            crate::utils::enable_interrupts(true);
        }
    vga::WRITER.lock().cursor.x = 0;
    vga::WRITER.lock().cursor.y = 0;
    vga::WRITER.lock().cursor.update_cursor(0, 0);
    let mut done : bool = false;
    while done == false {
        loop {
                if let Some(event) = get_next_key_event() {
                if event.pressed == true {

                    match event.code {
                        KeyCode::Control(ControlKey::UpArrow) => vga::WRITER.lock().cursor.move_cursors(Direction::Top),
                        KeyCode::Control(ControlKey::DownArrow) => vga::WRITER.lock().cursor.move_cursors(Direction::Down),
                        KeyCode::Control(ControlKey::LeftArrow) => vga::WRITER.lock().cursor.move_cursors(Direction::Left),
                        KeyCode::Control(ControlKey::RightArrow) => vga::WRITER.lock().cursor.move_cursors(Direction::Right),
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
        PAINT_BUFFER.copy_from(vga::WRITER.lock().buffer);
        vga::WRITER.lock().clear_screen();
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