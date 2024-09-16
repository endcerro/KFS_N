use crate::vga;
use crate::vga::Color;
use crate::WRITER;
use crate::ColorCode;


pub fn run(mut args: core::str::SplitWhitespace<'_>) {

    if let Some(background ) = args.next() {
        if let Some(foreground) = args.next() {
            
            let back = Color::from_string(background);
            let fore = Color::from_string(foreground);

            vga::clear_screen();
            vga::print_ft();
            WRITER.lock().change_color(ColorCode::new(back, fore));
        }
        else {
            let foreground = "white";
        }

    }
    else {
        let background = "black";
        let foreground: &str = "white";
    }
}