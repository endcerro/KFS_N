use crate::vga;
use crate::vga::Color;
use crate::WRITER;


pub fn run(mut args: core::str::SplitWhitespace<'_>) {

    let background;
    let foreground;

    background = match args.next() {
       Some(bg) => bg,
       None => {
        describe();
        return;
       }
    };

    foreground = match args.next() {
        Some(fg) => fg,
        None => {
            describe();
            return;
        }
    };

    let back = Color::from_string(background);
    let mut fore = Color::from_string(foreground);

    if back == fore && back != Color::White {
        fore = Color::White;
    }
    else if back == fore {
        fore = Color::Black;
    }

    vga::clear_screen();
    WRITER.lock().change_color(Some(fore),Some( back));
    vga::clear_screen();

}

pub fn describe() {
    print!("\n This function give you the possibility to change background 
        and foreground color\n Here is a list of all color available
        - black,
        - blue,
        - green,
        - cyan,
        - red,
        - magenta,
        - brown,
        - lightgray,
        - garkgray,
        - lightblue,
        - lightgreen,
        - lightcyan,
        - lightred,
        - pink,
        - yellow,
        - white
        Usage : custom background foreground");
}