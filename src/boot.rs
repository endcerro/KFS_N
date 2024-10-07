

extern "C"
{
  fn kernel_hello();
}

#[no_mangle]
#[link_section = ".boot"]
pub unsafe extern "C" fn boot() -> ! {
  
  unsafe {
    kernel_hello();

  }
  // println!("Sample text from boot");
  loop {
      
  }
}