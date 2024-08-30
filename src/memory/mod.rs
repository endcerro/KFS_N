// use directory::PagingDirectory;


mod test;
mod physical;
extern "C" {
    static mut _kernel_end: u32;
    static mut _kernel_start: u32;
    static mut _section_bss_start: u32;
    static mut _common_bss_sep: u32;
    static mut _section_bss_end: u32;
    static mut page_directory_first_entry: * mut u32;
    static mut page_table_first_entry: * mut u32;

    fn loadpagedirectory(pagedir :* const u32);
    fn enablepaging();

}
pub fn paging() {

     test2();
     //  unsafe {

     //       for i in 0..1024 {
     //            let page_directory: *mut u32 = page_directory_first_entry.add(i);
     //            *page_directory = 0x00000002; 
     //       }
           
     //       for i in 0..1024
     //       {
     //            let page_table = page_table_first_entry.add(i);
     //            *page_table = ((i * 0x1000) | 3) as u32
     //       }
     //       *page_directory_first_entry = page_table_first_entry as u32 | 3;
     //       loadpagedirectory(page_directory_first_entry);
     //       enablepaging();
     //  }
      
}

pub fn test2()
{
     test::doa(10);


     

     // if let Some(page) = test::allocate_page() {
     //      println!("Allocated page at address: {:#x}", page);
     // } else {
     //      println!("Failed to allocate page");
     // }
     
      
}