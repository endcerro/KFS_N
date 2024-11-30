use crate::multiboot2;

pub fn run(_args: &[&str]) {

    multiboot2::meminfo::print_meminfo();
    // if  _args.len() == 1 && (_args[0] == "-h" || _args[0] == "--help") || _args.len() != 2 {
    //         usage();
    //         return;
    // }

    // let colors: (Result<Color, _>, Result<Color, _>) = (_args[0].parse(), _args[1].parse());

    // match colors {
    //     (Ok(fg), Ok(bg)) if fg != bg => {
    //         WRITER.lock().change_color(Some(fg), Some(bg));
    //     },
    //     (Ok(_), Ok(_)) => {
    //         println!("\nForeground and background colors must be different");
    //     },
    //     _ => println!("\nInvalid colors")
    // }
}

fn usage() {
    print!("\nDisplays meminfo");
    // for color in Color::all() {
    //     WRITER.lock().change_color(Some(color), None);
    //     print!("\n{:?}", color);
    // }
}
