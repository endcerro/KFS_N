use crate::print;

// External functions and variables defined in assembly or linker script
extern "C" {
    // Start address of the kernel in memory
    static mut _kernel_start: u32;
    // End address of the kernel in memory
    static mut _kernel_end: u32;
    // Assembly function to load the page directory address into CR3
    fn loadpagedirectory(page_directory: *const PageDirectory);
    // Assembly function to set the paging bit in CR0, enabling paging
    fn enablepaging();
}

// Constants for paging structure sizes
const PAGE_SIZE: usize = 4096;  // Size of a single page (4 KiB)
const PAGE_TABLE_ENTRIES: usize = 1024;  // Number of entries in a page table
const PAGE_DIRECTORY_ENTRIES: usize = 1024;  // Number of entries in a page directory

// Represents the flags that can be set for a page table entry
#[repr(transparent)]
struct PageFlags(u32);

impl PageFlags {
    // Page is present in memory
    const PRESENT: u32 = 1 << 0;
    // Page is writable
    const WRITABLE: u32 = 1 << 1;
    // Page can be accessed by user-mode processes
    const USER_ACCESSIBLE: u32 = 1 << 2;
    // Write-through caching is enabled
    const WRITE_THROUGH: u32 = 1 << 3;
    // Caching is disabled for this page
    const CACHE_DISABLE: u32 = 1 << 4;
    // Page has been accessed (set by CPU)
    const ACCESSED: u32 = 1 << 5;
    // Page has been written to (set by CPU)
    const DIRTY: u32 = 1 << 6;
    // Page size extension (4 MiB pages if set)
    const PAGE_SIZE: u32 = 1 << 7;
    // Page is global (not flushed from TLB on task switch)
    const GLOBAL: u32 = 1 << 8;

    // Create a new PageFlags with the given value
    fn new(value: u32) -> Self {
        PageFlags(value)
    }

    // Get the raw value of the flags
    fn value(&self) -> u32 {
        self.0
    }
}

// Represents a page table (1024 entries, each mapping to a 4 KiB page)
#[repr(C, align(4096))]
#[derive(Clone, Copy)]
struct PageTable([u32; PAGE_TABLE_ENTRIES]);

// Represents a page directory (1024 entries, each pointing to a page table)
#[repr(C, align(4096))]
#[derive(Clone, Copy)]
struct PageDirectory([u32; PAGE_DIRECTORY_ENTRIES]);

// Static allocations for the page directory and initial page tables
static mut PAGE_DIRECTORY: PageDirectory = PageDirectory([0; PAGE_DIRECTORY_ENTRIES]);
static mut PAGE_TABLES: [PageTable; 4] = [PageTable([0; PAGE_TABLE_ENTRIES]); 4];

pub fn init_paging() {
    print!("Initializing paging...\n");

    // Get the start and end addresses of the kernel
    let kernel_start = unsafe { &_kernel_start as *const u32 as u32 };
    let kernel_end = unsafe { &_kernel_end as *const u32 as u32 };

    print!("Kernel start: 0x{:x}, Kernel end: 0x{:x}\n", kernel_start, kernel_end);

    // Identity map first 16MB (which should cover our kernel and initial needs)
    for i in 0..4 {
        unsafe {
            // Set up the page directory entry to point to the corresponding page table
            PAGE_DIRECTORY.0[i] = (&PAGE_TABLES[i] as *const PageTable as u32) | PageFlags::PRESENT | PageFlags::WRITABLE;
            
            // Set up each entry in the page table
            for j in 0..PAGE_TABLE_ENTRIES {
                let addr = (i * PAGE_TABLE_ENTRIES + j) * PAGE_SIZE;
                PAGE_TABLES[i].0[j] = (addr as u32) | PageFlags::PRESENT | PageFlags::WRITABLE;
            }
        }
    }

    unsafe {
        print!("Loading page directory...\n");
        // Load the page directory address into CR3
        loadpagedirectory(&PAGE_DIRECTORY as *const PageDirectory);
        
        print!("Enabling paging...\n");
        // Set the paging bit in CR0, enabling paging
        enablepaging();
    }

    print!("Paging initialized.\n");

    // Test write to ensure paging is working
    unsafe {
        // Write a test value to a mapped address
        *(0x1000 as *mut u32) = 0xdeadbeef;
        // Read back the test value
        let value = *(0x1000 as *const u32);
        print!("Test write and read at 0x1000: 0x{:x}\n", value);
    }
}

// Map a physical address to a virtual address with specified flags
pub fn map_page(physical_addr: u32, virtual_addr: u32, flags: PageFlags) {
    // Calculate the indices into the page directory and page table
    let pd_index: usize = ((virtual_addr >> 22) & 0x3FF) as usize;
    let pt_index: usize = ((virtual_addr >> 12) & 0x3FF)  as usize;

    unsafe {
        // Check if the page table for this address exists
        if PAGE_DIRECTORY.0[pd_index] & PageFlags::PRESENT == 0 {
            // Page table doesn't exist, create a new one
            let new_table = alloc_page_table();
            PAGE_DIRECTORY.0[pd_index] = new_table as u32 | PageFlags::PRESENT | PageFlags::WRITABLE;
        }

        // Get a pointer to the page table
        let page_table = &mut *((PAGE_DIRECTORY.0[pd_index] & 0xFFFFF000) as *mut PageTable);
        // Set the page table entry to map the virtual address to the physical address
        page_table.0[pt_index] = (physical_addr & 0xFFFFF000) | flags.value();
    }
}

// Unmap a page at the given virtual address
pub fn unmap_page(virtual_addr: u32) {
    // Calculate the indices into the page directory and page table
    let pd_index: usize = ((virtual_addr >> 22) & 0x3FF) as usize;
    let pt_index : usize= ((virtual_addr >> 12) & 0x3FF) as usize;

    unsafe {
        // Check if the page table for this address exists
        if PAGE_DIRECTORY.0[pd_index] & PageFlags::PRESENT != 0 {
            // Get a pointer to the page table
            let page_table = &mut *((PAGE_DIRECTORY.0[pd_index] & 0xFFFFF000) as *mut PageTable);
            // Clear the page table entry to unmap the page
            page_table.0[pt_index] = 0;
        }
    }
}

// Allocate a new page table
// Note: This is a very basic allocation method and should be replaced with a proper allocator in a real kernel
fn alloc_page_table() -> *mut PageTable {
    // Keep track of the next available page table
    static mut NEXT_PAGE_TABLE: usize = 4;
    unsafe {
        // Check if we've run out of pre-allocated page tables
        if NEXT_PAGE_TABLE >= PAGE_TABLES.len() {
            panic!("Out of page tables!");
        }
        // Get a pointer to the next available page table
        let table = &mut PAGE_TABLES[NEXT_PAGE_TABLE];
        NEXT_PAGE_TABLE += 1;
        table as *mut PageTable
    }
}

// Add more memory management functions as needed...