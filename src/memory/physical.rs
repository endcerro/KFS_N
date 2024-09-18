use core::ptr::addr_of;

use crate::{multiboot2::{meminfo::MemoryInfoEntry, MultibootInfo}, serial_println};

const PAGE_DIRECTORY_SIZE : usize = 1024;
const PAGE_TABLE_SIZE : usize = 1024;
const MAX_BITMAP_SIZE : usize = PAGE_DIRECTORY_SIZE * PAGE_TABLE_SIZE / 8 / 1024;

pub struct FrameAllocator<const BITMAP_SIZE: usize = MAX_BITMAP_SIZE> {
    bitmap: [u8; BITMAP_SIZE],
    total_frames: u64,
}

impl<const BITMAP_SIZE: usize> FrameAllocator<BITMAP_SIZE> {
    pub const fn new() -> Self {
        FrameAllocator {
            bitmap: [0; BITMAP_SIZE],
            total_frames: 0,
        }
    }

    pub fn init(&mut self, multiboot_info: &mut MultibootInfo) {
        let (start_addr, end_addr) = self.get_memory_bounds(multiboot_info);
        self.total_frames = end_addr.saturating_sub(start_addr) as u64;
    

        assert!((BITMAP_SIZE * 8  ) as u64 >= self.total_frames, "Bitmap size is too small for the available memory");
 
        for byte in self.bitmap.iter_mut() {
            *byte = 0xFF;
        }

        self.mark_memory_regions(multiboot_info);
    }

    fn mark_memory_regions(&mut self, multiboot_info: &mut MultibootInfo) {
        if let Some(mem_info) = multiboot_info.get_memory_info() {
            let mut mem_iterator = mem_info.entry;
            loop {
                match mem_iterator.next() {
                    None => break, // No more memory regions
                    Some(entry) => unsafe {
                        let entry: &MemoryInfoEntry = &*entry;
                        let start_frame = (entry.base_addr / 4096) as usize ;
                            let end_frame =(((entry.base_addr + entry.length + 4095) / 4096) as u64)
                            .min(self.total_frames);

                        match entry.typee {
                            1 => { if start_frame >= 256 {self.mark_region_available(start_frame, end_frame);}},
                            2 => self.mark_region_unavailable(start_frame, end_frame),
                            _ => {
                                serial_println!("Unknown memory region type {}: start={:#x}, end={:#x}", 
                                entry.typee, entry.base_addr, entry.base_addr + entry.length);
                                    self.mark_region_unavailable(start_frame, end_frame);
                                panic!("Unknown memory region type {}: start={:#x}, end={:#x}", 
                                entry.typee, entry.base_addr, entry.base_addr + entry.length)}
                    }
                }
            }
        }
        self.mark_kernel_region_unavailable();
    }
}

    fn mark_region_available(&mut self, start_frame: u64, end_frame: u64) {
        for frame in start_frame..end_frame {
            if frame < self.total_frames {
                let (byte_index, bit_index) = ((frame / 8 )as usize, (frame % 8) as usize);
                if byte_index < BITMAP_SIZE {
                    self.bitmap[byte_index] &= !(1 << bit_index);
                }
            }
        }
    }

    fn mark_region_unavailable(&mut self, start_frame: usize, end_frame: usize) {
        for frame in start_frame..end_frame {
            if frame < self.total_frames {
                let (byte_index, bit_index) = (frame / 8, frame % 8);
                if byte_index < BITMAP_SIZE {
                    self.bitmap[byte_index] |= 1 << bit_index;
                }
            }
        }
    }

    fn mark_kernel_region_unavailable(&mut self) {
        extern "C" {
            static _kernel_start : u8;
            static _kernel_end : u8;
        }
        let start_frame = (addr_of!(_kernel_start) as usize) / 4096;
        let end_frame = ((addr_of!(_kernel_end) as usize) + 4095) / 4096;
        self.mark_region_unavailable(start_frame, end_frame);
    }
    // Other methods (mark_available_regions, mark_kernel_region_used, etc.) ...
    fn get_memory_bounds(&self, multiboot_info: &mut MultibootInfo) -> (u64, u64) {
        let mut start_addr: u64 = u64::MAX;
        let mut end_addr: u64 = 0;

        if let Some(mem_info) = multiboot_info.get_memory_info() {
            let mut mem_iterator = mem_info.entry;
            
            loop {
                match mem_iterator.next() {
                    None => break,
                    Some(entry) => unsafe {
                        let entry: &MemoryInfoEntry = &*entry;
                        let entry_start = entry.base_addr;
                        let entry_end = entry.base_addr + entry.length;
                        
                        start_addr = start_addr.min(entry_start);
                        end_addr = end_addr.max(entry_end);
                    }
                }
            }
        }

        // Convert to frame numbers, ensuring we don't exceed usize capacity
        let start_frame = (start_addr / 4096) as usize;
        let end_frame = ((end_addr + 4095) / 4096) as usize;

        // Handle potential overflow
        let max_addressable_frame = usize::MAX / 4096;
        (start_frame as u64, end_frame.min(max_addressable_frame) as u64)
    }
    pub fn print_bitmap_state(&self) {
        serial_println!("Bitmap State (0 = Available, 1 = Used):");
        serial_println!("Total Frames: {}", self.total_frames);

        let mut line_buffer = [0u8; 65]; // 64 chars + null terminator
        let mut buffer_index = 0;
        let mut current_frame: u64 = 0;

        for (i, &byte) in self.bitmap.iter().enumerate() {
            for bit in 0..8 {
                if (i * 8 + bit) as u64 >= self.total_frames {
                    break;
                }
                line_buffer[buffer_index] = if byte & (1 << bit) != 0 { b'1' } else { b'0' };
                buffer_index += 1;

                if buffer_index == 64 || (i * 8 + bit + 1 ) as u64 == self.total_frames {
                    // Print the line
                    line_buffer[buffer_index] = 0; // Null-terminate the string
                    serial_println!("{:016x}: {}", current_frame * 4096, 
                                    core::str::from_utf8(&line_buffer[..buffer_index]).unwrap());
                    buffer_index = 0;
                    current_frame += 64;
                }
            }
        }

        // Print summary
        let mut used_frames: u64 = 0;
        for (i, &byte) in self.bitmap.iter().enumerate() {
            let frame_count = (self.total_frames - i * 8).min(8) as i32;
            used_frames += (byte.count_ones() as u64).min(frame_count as u64);
        }
        let available_frames = self.total_frames as u64 - used_frames;

        serial_println!("Summary:");
        serial_println!("  Used Frames: {} ({} KB)", used_frames, used_frames * 4);
        serial_println!("  Available Frames: {} ({} KB)", available_frames, available_frames * 4);
        serial_println!("  Total Memory: {} KB", self.total_frames as u64 * 4);
    }
}


// Static allocator
pub static mut FRAME_ALLOCATOR: FrameAllocator = FrameAllocator::new();

static ALLOCATOR_INITIALIZED: bool = false;


// pub fn init_frame_allocator(multiboot_info: &MultibootInfo) {
//     if ALLOCATOR_INITIALIZED.load(Ordering::Relaxed) {
//         return;
//     }

//     unsafe {
//         FRAME_ALLOCATOR.init(multiboot_info);
//     }

//     ALLOCATOR_INITIALIZED.store(true, Ordering::Relaxed);
// }

// pub fn allocate_frame() -> Option<usize> {
//     unsafe { FRAME_ALLOCATOR.allocate() }
// }

// pub fn deallocate_frame(frame: usize) {
//     unsafe { FRAME_ALLOCATOR.deallocate(frame) }
// }