extern "C" {
    pub static page_directory: [u32; 1024];
    // static page_table1: [u32; 1024];
    static _kernel_start : u8;
    static _kernel_end : u8;
}

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_TABLE_ENTRIES: usize = 1024;
pub const PAGE_DIRECTORY_ENTRIES: usize = 1024;
pub const KERNEL_OFFSET: usize = 0xC0000000; // Higher half kernel offset

// ---------------------------------------------------------------------------
// Kernel heap region
//
// Placed at 0xC1000000 - well above the kernel image and frame-allocator
// bitmap, but below the recursive mapping region (0xFFC00000).
//
// 16 MB is generous for the kernel heap; the initial mapping only covers
// KERNEL_HEAP_INITIAL_SIZE and the allocator can grow on demand up to
// KERNEL_HEAP_END by calling vmm::map_alloc().
// ---------------------------------------------------------------------------
pub const KERNEL_HEAP_START: usize        = 0xC100_0000;
pub const KERNEL_HEAP_END: usize          = 0xC200_0000; // 16 MB total capacity
pub const KERNEL_HEAP_SIZE: usize         = KERNEL_HEAP_END - KERNEL_HEAP_START;
/// How much of the heap to pre-map at boot (128 KB - 32 pages).
/// The rest is mapped lazily as the allocator grows.
pub const KERNEL_HEAP_INITIAL_SIZE: usize = 128 * 1024;