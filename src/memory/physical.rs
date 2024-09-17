#[derive(Debug)]
struct MemoryRegion {
    start_address: usize,
    end_address: usize,
}

const PAGE_SIZE: usize = 4096; // 4 KiB pages

pub struct FrameAllocator {
    bitmap: &'static mut [u8],
    total_frames: usize,
    free_frames: usize,
}

impl FrameAllocator {
    /// Initialize the frame allocator with the given memory regions.
    pub fn init(memory_regions: &[MemoryRegion]) -> Self {
        // Calculate the total number of frames
        let mut total_frames = 0;
        for region in memory_regions {
            total_frames += (region.end_address - region.start_address) / PAGE_SIZE;
        }

        // Allocate a bitmap to track used/unused frames
        let bitmap_size = (total_frames + 7) / 8; // 1 bit per frame
        let bitmap = unsafe {
            let ptr = memory_regions.last().unwrap().end_address as *mut u8;
            core::slice::from_raw_parts_mut(ptr, bitmap_size)
        };

        // Initialize the bitmap to mark all frames as free
        for byte in bitmap.iter_mut() {
            *byte = 0;
        }

        FrameAllocator {
            bitmap,
            total_frames,
            free_frames: total_frames,
        }
    }

    /// Allocate a frame and return its physical address.
    pub fn allocate_frame(&mut self) -> Option<usize> {
        for (i, byte) in self.bitmap.iter_mut().enumerate() {
            if *byte != 0xFF {
                for bit in 0..8 {
                    if *byte & (1 << bit) == 0 {
                        *byte |= 1 << bit;
                        self.free_frames -= 1;
                        return Some((i * 8 + bit) * PAGE_SIZE);
                    }
                }
            }
        }
        None // No free frames available
    }

    /// Deallocate a frame given its physical address.
    pub fn deallocate_frame(&mut self, addr: usize) {
        let frame_number = addr / PAGE_SIZE;
        let byte_index = frame_number / 8;
        let bit_index = frame_number % 8;

        if self.bitmap[byte_index] & (1 << bit_index) != 0 {
            self.bitmap[byte_index] &= !(1 << bit_index);
            self.free_frames += 1;
        }
    }

    /// Get the number of free frames.
    pub fn free_frame_count(&self) -> usize {
        self.free_frames
    }
}

fn main() {
    // Example memory regions
    let memory_regions = [
        MemoryRegion { start_address: 0x1000, end_address: 0x9000 },
        MemoryRegion { start_address: 0xA000, end_address: 0xF000 },
    ];

    let mut frame_allocator = FrameAllocator::init(&memory_regions);

    let frame: Option<usize> = frame_allocator.allocate_frame();
    // Allocate a frame
    if frame.is_some() {
        println!("Allocated frame at address: {:#x}", frame.expect("a"));
    } else {
        println!("No free frames available");
    }

    // Deallocate the frame
    frame_allocator.deallocate_frame(frame.expect("aa"));

    println!("Free frames: {}", frame_allocator.free_frame_count());
}
