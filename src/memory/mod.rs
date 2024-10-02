use crate::vga::Color;

pub mod paging;

pub fn init() {
    unsafe {
        paging::PAGING.init();
    }
    paging::Paging::enable_paging();
    colored_print!((Some(Color::Green), Some(Color::Black)), "\nPAGING OK");
}