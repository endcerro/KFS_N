use core::ptr::NonNull;
use crate::multiboot2::meminfo::MemoryInfoEntry;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysFrame {
    pub number: usize,
}

impl PhysFrame {
    pub fn containing_address(address: usize) -> PhysFrame {
        PhysFrame { number: address / PAGE_SIZE }
    }

    pub fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    pub fn end_address(&self) -> usize {
        (self.number + 1) * PAGE_SIZE
    }
}

pub struct FrameAllocator {
    // Bitmap for frame allocation status. Each bit represents one frame
    bitmap: NonNull<[u8]>,
    // Total number of frames being managed
    total_frames: usize,
    // Next frame to check when allocating (simple optimization)
    next_free_frame: usize,
    // Statistics
    used_frames: usize,
}

#[derive(Debug)]
pub enum AllocationError {
    NoFramesAvailable,
    InvalidFrame,
    FrameInUse,
}

impl FrameAllocator {
    // Initialize the frame allocator from multiboot memory map
    pub fn new(memory_map: &[MemoryInfoEntry], bitmap_addr: usize) -> Self {
        #[cfg(feature = "verbose")]
        {
            println!("Initializing frame allocator at bitmap address: {:#x}", bitmap_addr);
            println!("Frame allocator bitmap: phys {:#x}, virt {:#x}",
                bitmap_addr, bitmap_addr + super::define::KERNEL_OFFSET);
        }
        let mut highest_addr = 0;

        // Find highest address to determine number of frames needed
        for entry in memory_map {
            let end_addr = (entry.base_addr + entry.length) as usize;
            if end_addr > highest_addr {
                highest_addr = end_addr;
            }
        }

        let total_frames = (highest_addr + PAGE_SIZE - 1) / PAGE_SIZE; // Round up
        let bitmap_size = (total_frames + 7) / 8; // Round up to nearest byte

        #[cfg(feature = "verbose")]
        println!("Total frames to manage: {} ({} MB)", total_frames, (total_frames * PAGE_SIZE) / (1024 * 1024));
        #[cfg(feature = "verbose")]
        println!("Bitmap size: {} bytes", bitmap_size);

        // Create bitmap accessor.
        let bitmap = unsafe {
            let virt_addr = bitmap_addr + super::define::KERNEL_OFFSET;
            let ptr = virt_addr as *mut u8;
            // Initialize bitmap to all 1s (all frames marked as used initially)
            for i in 0..bitmap_size {
                ptr.add(i).write(0xFF);
            }
            NonNull::new_unchecked(core::slice::from_raw_parts_mut(ptr, bitmap_size))
        };

        let mut allocator = FrameAllocator {
            bitmap,
            total_frames,
            next_free_frame: 0,
            used_frames: total_frames,// Start with all frames marked as used
        };

        // Mark available regions as free
        for entry in memory_map {
            if entry.typee == 1 { // Available memory
                let start_frame = PhysFrame::containing_address(entry.base_addr as usize);
                let end_frame = PhysFrame::containing_address((entry.base_addr + entry.length) as usize);
                #[cfg(feature = "verbose")]
                println!("Marking frames {}-{} as free (region {:#x}-{:#x})", start_frame.number, end_frame.number, entry.base_addr, entry.base_addr + entry.length);
                for frame in start_frame.number..end_frame.number {
                    allocator.mark_frame_free(frame);
                }
            }
        }

        allocator.protect_kernel_region();
        allocator.protect_bitmap_region(bitmap_addr, bitmap_size);
        #[cfg(feature = "verbose")]
        println!("Frame allocator initialized: {} free frames available",
            allocator.total_frames - allocator.used_frames);
        allocator
    }

    fn protect_kernel_region(&mut self) {
    extern "C" {
            static _kernel_start: u8;
            static _kernel_end: u8;
        }

        unsafe {
            let kernel_start_phys = (&_kernel_start as *const u8 as usize).saturating_sub(super::define::KERNEL_OFFSET);
            let kernel_end_phys = (&_kernel_end as *const u8 as usize).saturating_sub(super::define::KERNEL_OFFSET);

            let start_frame = PhysFrame::containing_address(kernel_start_phys);
            let end_frame = PhysFrame::containing_address(kernel_end_phys);

            #[cfg(feature = "verbose")]
            println!("Protecting kernel frames {}-{} (phys {:#x}-{:#x})",
                start_frame.number, end_frame.number,
                kernel_start_phys, kernel_end_phys);

            for frame_num in start_frame.number..=end_frame.number {
                if frame_num < self.total_frames && !self.is_frame_used(frame_num) {
                    self.mark_frame_used(frame_num);
                }
            }
        }
    }

    // Protect bitmap memory region from allocation
    fn protect_bitmap_region(&mut self, bitmap_addr: usize, bitmap_size: usize) {
        let start_frame = PhysFrame::containing_address(bitmap_addr);
        let end_frame = PhysFrame::containing_address(bitmap_addr + bitmap_size - 1);

        #[cfg(feature = "verbose")]
        println!("Protecting bitmap frames {}-{} (addr {:#x}, size {})",
            start_frame.number, end_frame.number, bitmap_addr, bitmap_size);

        for frame_num in start_frame.number..=end_frame.number {
            if frame_num < self.total_frames && !self.is_frame_used(frame_num) {
                self.mark_frame_used(frame_num);
            }
        }
    }
    // Allocate a physical frame
    pub fn allocate_frame(&mut self) -> Result<PhysFrame, AllocationError> {
        // Start searching from next_free_frame
        for offset in 0..self.total_frames {
            let frame = (self.next_free_frame + offset) % self.total_frames;
            
            if !self.is_frame_used(frame) {
                self.mark_frame_used(frame);
                self.next_free_frame = (frame + 1) % self.total_frames;
                
                #[cfg(feature = "verbose")]
                println!("Allocated frame {} at phys addr {:#x}", frame, PhysFrame { number: frame }.start_address());
                
                return Ok(PhysFrame { number: frame });
            }
        }

        Err(AllocationError::NoFramesAvailable)
    }

        // Allocate a specific physical frame (useful for DMA or memory-mapped I/O)
    pub fn allocate_specific_frame(&mut self, frame: PhysFrame) -> Result<(), AllocationError> {
        if frame.number >= self.total_frames {
            return Err(AllocationError::InvalidFrame);
        }
        
        if self.is_frame_used(frame.number) {
            return Err(AllocationError::FrameInUse);
        }
        
        self.mark_frame_used(frame.number);
        
        #[cfg(feature = "verbose")]
        println!("Allocated specific frame {} at phys addr {:#x}", frame.number, frame.start_address());
        
        Ok(())
    }
    // Deallocate a physical frame
    pub fn deallocate_frame(&mut self, frame: PhysFrame) -> Result<(), AllocationError> {
        if frame.number >= self.total_frames {
            return Err(AllocationError::InvalidFrame);
        }

        self.mark_frame_free(frame.number);

        // Update next_free_frame if this frame is earlier
        if frame.number < self.next_free_frame {
            self.next_free_frame = frame.number;
            #[cfg(feature = "debug")]
            println!("Warning: Attempting to free already-free frame {}", frame.number);
        }

        Ok(())
    }

    // Check if a frame is marked as used
    fn is_frame_used(&self, frame: usize) -> bool {
        let byte_index = frame / 8;
        let bit_index = frame % 8;
        unsafe {
            let bitmap = self.bitmap.as_ref();
            (bitmap[byte_index] & (1 << bit_index)) != 0
        }
    }

    // Mark a frame as used in the bitmap
    fn mark_frame_used(&mut self, frame: usize) {
        if !self.is_frame_used(frame) {
            let byte_index = frame / 8;
            let bit_index = frame % 8;
            unsafe {
                let bitmap = self.bitmap.as_mut();
                bitmap[byte_index] |= 1 << bit_index;
            }
            self.used_frames += 1;
        }
    }

    // Mark a frame as free in the bitmap
    fn mark_frame_free(&mut self, frame: usize) {
        let byte_index = frame / 8;
        let bit_index = frame % 8;
        unsafe {
            let bitmap = self.bitmap.as_mut();
            bitmap[byte_index] &= !(1 << bit_index);
        }
        //TODO This is a bug waiting to happen as it can underflow
        self.used_frames -= 1;
    }

    // Get the total number of frames
    pub fn total_frames(&self) -> usize {
        self.total_frames
    }
    
    // Get the number of used frames
    pub fn used_frames(&self) -> usize {
        self.used_frames
    }

    // Get the number of free frames
    pub fn free_frames(&self) -> usize {
        self.total_frames - self.used_frames
    }

    // Get memory statistics in bytes
    pub fn memory_stats(&self) -> (usize, usize, usize) {
        let total_bytes = self.total_frames * PAGE_SIZE;
        let used_bytes = self.used_frames * PAGE_SIZE;
        let free_bytes = self.free_frames() * PAGE_SIZE;
        (total_bytes, used_bytes, free_bytes)
    }

    // Print memory statistics
    pub fn print_stats(&self) {
        let (total, used, free) = self.memory_stats();
        println!("Physical Memory Statistics:");
        println!("  Total: {} MB ({} frames)", total / (1024 * 1024), self.total_frames);
        println!("  Used:  {} MB ({} frames)", used / (1024 * 1024), self.used_frames);
        println!("  Free:  {} MB ({} frames)", free / (1024 * 1024), self.free_frames());
    }
}

pub static mut FRAME_ALLOCATOR: Option<FrameAllocator> = None;

pub fn init_frame_allocator(memory_map: &[MemoryInfoEntry]) {
    extern "C" {
        static _kernel_end: u8;
    }
    
    let bitmap_addr = unsafe {
        let kernel_end_phys = (&_kernel_end as *const u8 as usize).saturating_sub(super::define::KERNEL_OFFSET);
        // Align to next page boundary
        (kernel_end_phys + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
    };
    
    unsafe {
        FRAME_ALLOCATOR = Some(FrameAllocator::new(memory_map, bitmap_addr));
        
        #[cfg(feature = "verbose")]
        if let Some(ref allocator) = FRAME_ALLOCATOR {
            allocator.print_stats();
        }
    }
}