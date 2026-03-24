
pub fn shutdown() -> ! {
    // QEMU ACPI shutdown
    crate::utils::outw(0x604, 0x2000);
    // Fallback: halt loop if ACPI didn't work
    loop {
        unsafe { core::arch::asm!("hlt", options(nostack, nomem)); }
    }
}

pub fn run(_args: &[&str]) {
	shutdown();
}