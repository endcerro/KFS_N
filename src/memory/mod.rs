use directory::PagingDirectory;

mod directory;
mod paging;

extern "C" {
    static mut kernel_end: u32;
}
pub fn paging() {
    let test: PagingDirectory;
    unsafe {
        println!("Kernel end {}", kernel_end);
    } // test.init();
      // paging::test_paging();
}
