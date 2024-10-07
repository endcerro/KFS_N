
#[no_mangle]
#[link_section = ".boot"]
pub unsafe extern "C" fn boot() -> ! {

  println!("Sample text from boot");
  loop {
      
  }
}