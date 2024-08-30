const PAGE_SIZE: usize = 4096;
const ENTRIES_PER_TABLE: usize = 1024;
const BITMAP_SIZE: usize = 1024 * 1024 / PAGE_SIZE / 8; // Assuming 1 GiB of physical memory

type PageDirectory = [u32; ENTRIES_PER_TABLE];
type PageTable = [u32; ENTRIES_PER_TABLE];

#[derive(Clone, Copy)]
#[repr(align(4096))]
struct AlignedPageDirectory(PageDirectory);

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct AlignedPageTable(PageTable);

static mut PAGE_DIRECTORY: AlignedPageDirectory = AlignedPageDirectory([0; ENTRIES_PER_TABLE]);
static mut PAGE_TABLES: [AlignedPageTable; ENTRIES_PER_TABLE] = [AlignedPageTable([0; ENTRIES_PER_TABLE]); ENTRIES_PER_TABLE];

struct FrameAllocator {
    bitmap: [u8; BITMAP_SIZE],
}

impl FrameAllocator {
    fn new() -> Self {
        Self {
            bitmap: [0; BITMAP_SIZE],
        }
    }

    fn set_bit(&mut self, frame: usize) {
        let byte = frame / 8;
        let bit = frame % 8;
        self.bitmap[byte] |= 1 << bit;
    }

    fn clear_bit(&mut self, frame: usize) {
        let byte = frame / 8;
        let bit = frame % 8;
        self.bitmap[byte] &= !(1 << bit);
    }

    fn is_set(&self, frame: usize) -> bool {
        let byte = frame / 8;
        let bit = frame % 8;
        self.bitmap[byte] & (1 << bit) != 0
    }

    fn allocate_frame(&mut self) -> Option<usize> {
        for (i, byte) in self.bitmap.iter().enumerate() {
            if *byte != 0xFF {
                for bit in 0..8 {
                    if byte & (1 << bit) == 0 {
                        let frame = i * 8 + bit;
                        self.set_bit(frame);
                        return Some(frame);
                    }
                }
            }
        }
        None
    }

    fn free_frame(&mut self, frame: usize) {
        self.clear_bit(frame);
    }
}

static mut FRAME_ALLOCATOR: FrameAllocator = FrameAllocator {
    bitmap: [0; BITMAP_SIZE],
};

fn initialize_frame_allocator() {
    unsafe {
        FRAME_ALLOCATOR = FrameAllocator::new();
    }
}

fn allocate_frame() -> Option<usize> {
    unsafe { FRAME_ALLOCATOR.allocate_frame() }
}

fn free_frame(frame: usize) {
    unsafe { FRAME_ALLOCATOR.free_frame(frame) }
}

fn initialize_paging() {
    unsafe {
        for i in 0..ENTRIES_PER_TABLE {
            let frame = allocate_frame().expect("Out of memoryq");
            PAGE_DIRECTORY.0[i] = (frame * PAGE_SIZE) as u32 | 0x3; // Present and writable
            for j in 0..ENTRIES_PER_TABLE {
                let frame = allocate_frame().expect("Out of memoryn");
                PAGE_TABLES[i].0[j] = (frame * PAGE_SIZE) as u32 | 0x3; // Present and writable
            }
        }
    }
}

fn enable_paging() {
    unsafe {
        println!("Page Directory Address: {:#x}", PAGE_DIRECTORY.0.as_ptr() as u32);
        for i in 0..ENTRIES_PER_TABLE {
            println!("Page Directory Entry {}: {:#x}", i, PAGE_DIRECTORY.0[i]);
        }

        core::arch::asm!(
            "mov cr3, {0}",
            "mov eax, cr0",
            "or eax, 0x80000000",
            "mov cr0, eax",
            in(reg) PAGE_DIRECTORY.0.as_ptr()
        );
    }
}

pub fn doa(multiboot_info_addr: usize) {
    initialize_frame_allocator();
    initialize_paging();
    enable_paging();

    // Continue with the rest of your kernel initialization
}
