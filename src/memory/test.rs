// Define the memory region struct
#[repr(C)]
pub struct MemoryRegion {
    start: u32,
    end: u32,
}

// Constants for page sizes and bits
const PAGE_SIZE: usize = 4096;
const PAGE_TABLE_ENTRIES: usize = 1024;
const PAGE_DIRECTORY_ENTRIES: usize = 1024;

// Page table entry structure
#[repr(transparent)]
struct PageTableEntry(u32);

impl PageTableEntry {
    const PRESENT: u32 = 1 << 0;
    const WRITABLE: u32 = 1 << 1;
    const USER_ACCESSIBLE: u32 = 1 << 2;

    fn new(page_frame: u32, flags: u32) -> Self {
        PageTableEntry((page_frame << 12) | flags)
    }
}

// Page directory entry structure
#[repr(transparent)]
struct PageDirectoryEntry(u32);

impl PageDirectoryEntry {
    const PRESENT: u32 = 1 << 0;
    const WRITABLE: u32 = 1 << 1;
    const USER_ACCESSIBLE: u32 = 1 << 2;

    fn new(page_table_addr: u32, flags: u32) -> Self {
        PageDirectoryEntry((page_table_addr & !0xFFF) | flags)
    }
}

// Frame allocator
struct FrameAllocator {
    next_free_frame: u32,
    memory_regions: &'static [MemoryRegion],
}

impl FrameAllocator {
    fn new(memory_regions: &'static [MemoryRegion]) -> Self {
        let mut allocator = FrameAllocator {
            next_free_frame: 0,
            memory_regions,
        };
        allocator.next_free_frame = allocator.find_next_free_frame();
        allocator
    }

    fn allocate_frame(&mut self) -> Option<u32> {
        let frame = self.next_free_frame;
        self.next_free_frame = self.find_next_free_frame();
        Some(frame)
    }

    fn find_next_free_frame(&self) -> u32 {
        let mut frame = self.next_free_frame + 1;
        while !self.is_frame_available(frame) {
            frame += 1;
        }
        frame
    }

    fn is_frame_available(&self, frame: u32) -> bool {
        let addr = frame as u32 * PAGE_SIZE as u32;
        self.memory_regions.iter().any(|region| {
            addr >= region.start && addr + PAGE_SIZE as u32 <= region.end
        })
    }
}

// Paging structure
struct Paging {
    page_directory: &'static mut [PageDirectoryEntry; PAGE_DIRECTORY_ENTRIES],
    frame_allocator: FrameAllocator,
}

impl Paging {
    fn new(page_directory: &'static mut [PageDirectoryEntry; PAGE_DIRECTORY_ENTRIES], memory_regions: &'static [MemoryRegion]) -> Self {
        Paging {
            page_directory,
            frame_allocator: FrameAllocator::new(memory_regions),
        }
    }

    fn map_page(&mut self, virtual_addr: u32, physical_addr: u32, flags: u32) {
        let pd_index = (virtual_addr >> 22) & 0x3FF;
        let pt_index = (virtual_addr >> 12) & 0x3FF;

        let pd_entry = &mut self.page_directory[pd_index as usize];

        if pd_entry.0 & PageDirectoryEntry::PRESENT == 0 {
            let pt_frame = self.frame_allocator.allocate_frame().expect("Out of memory");
            *pd_entry = PageDirectoryEntry::new(pt_frame * PAGE_SIZE as u32, PageDirectoryEntry::PRESENT | PageDirectoryEntry::WRITABLE);

            // Clear the new page table
            let page_table = unsafe { &mut *(((pt_frame * PAGE_SIZE as u32) as *mut PageTableEntry).as_mut().unwrap()) };
            for entry in page_table.iter_mut() {
                *entry = PageTableEntry(0);
            }
        }

        let pt_addr = pd_entry.0 & !0xFFF;
        let page_table = unsafe { &mut *((pt_addr as *mut PageTableEntry).as_mut().unwrap()) };

        page_table[pt_index as usize] = PageTableEntry::new(physical_addr >> 12, flags | PageTableEntry::PRESENT);
    }

    fn enable_paging(&self) {
        unsafe {
            asm!(
                "mov eax, {}",
                "mov cr3, eax",
                "mov eax, cr0",
                "or eax, 0x80000000",
                "mov cr0, eax",
                in(reg) self.page_directory.as_ptr() as u32,
                out("eax") _,
            );
        }
    }

    fn test_paging(&mut self) {
        println!("Testing paging implementation...");

        // Test 1: Allocate and map a page
        let virt_addr = 0x400000; // 4MB
        let phys_addr = match self.frame_allocator.allocate_frame() {
            Some(frame) => frame * PAGE_SIZE as u32,
            None => {
                println!("Test 1 Failed: Unable to allocate frame");
                return;
            }
        };

        self.map_page(virt_addr, phys_addr, PageTableEntry::WRITABLE);
        println!("Test 1 Passed: Page mapped successfully");

        // Test 2: Write to the mapped page
        let test_value: u32 = 0xdeadbeef;
        unsafe {
            *(virt_addr as *mut u32) = test_value;
        }

        // Test 3: Read from the mapped page
        let read_value = unsafe { *(virt_addr as *const u32) };
        if read_value == test_value {
            println!("Test 2 & 3 Passed: Successfully wrote to and read from mapped page");
        } else {
            println!("Test 2 & 3 Failed: Read value does not match written value");
        }

        // Test 4: Try to access an unmapped page (should cause a page fault)
        println!("Test 4: Attempting to access unmapped page (should cause page fault)...");
        unsafe {
            *(0x800000 as *mut u32) = 0; // This should cause a page fault
        }

        println!("If you see this, Test 4 Failed: No page fault occurred");
    }
}

pub fn doa(memory_regions: &'static [MemoryRegion]) -> ! {
    println!("Kernel started. Initializing paging...");

    // Allocate space for the page directory
    let page_directory: &'static mut [PageDirectoryEntry; PAGE_DIRECTORY_ENTRIES] = 
        unsafe { &mut *(0x100000 as *mut [PageDirectoryEntry; PAGE_DIRECTORY_ENTRIES]) };

    // Initialize paging
    let mut paging = Paging::new(page_directory, memory_regions);

    println!("Mapping first 1MB...");
    // Identity map the first 1MB
    for addr in (0..0x100000).step_by(PAGE_SIZE) {
        paging.map_page(addr, addr, PageTableEntry::WRITABLE);
    }

    println!("Mapping kernel...");
    // Map kernel
    for addr in (0x100000..0x400000).step_by(PAGE_SIZE) {
        paging.map_page(addr, addr, PageTableEntry::WRITABLE);
    }

    println!("Enabling paging...");
    // Enable paging
    paging.enable_paging();

    println!("Paging enabled. Running tests...");
    // Run paging tests
    paging.test_paging();

    println!("All tests completed.");

    loop {}
}

