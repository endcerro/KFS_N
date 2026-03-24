// shell/commands/vmalloc.rs
//
// Shell command: `vmalloc`
//
// Runs memory::vmm::demo_virt_alloc() — a self-contained demonstration of
// the full virtual-memory allocation pipeline:
//
//   1. Check address is free
//   2. Allocate a physical frame + map it at virtual 0xD0CAF000
//   3. Verify translate() agrees with the allocated frame
//   4. Write four u32 values spread across the 4 KB page
//   5. Read them back and compare
//   6. Unmap and free the physical frame
//
// Usage: vmalloc
// Can be called multiple times safely (stale mapping is cleaned up).

use crate::memory::vmm;

pub fn run(_args: &[&str]) {
    vmm::demo_virt_alloc();
}