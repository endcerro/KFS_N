

extern "C"
{
  fn kernel_hello();
}

#[no_mangle]
#[link_section = ".boot"]
pub unsafe extern "C" fn boot() -> ! {
  
  // Set up identity mapping for first 4
  loop {
    kernel_hello();
  }
}