
pub unsafe extern "x86-interrupt" fn divide_by_zero() {
    println!("Divide by zero error!");
    loop {}
}

pub unsafe extern "x86-interrupt" fn page_fault() {
    println!("Page fault error!");
    loop {}
}
pub unsafe extern "x86-interrupt" fn default() {
    println!("Default handler error!");
    loop {}
}