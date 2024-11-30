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
}

pub struct FrameAllocator {
    // Bitmap for frame allocation status. Each bit represents one frame
    bitmap: NonNull<[u8]>,
    // Total number of frames being managed
    total_frames: usize,
    // Next frame to check when allocating (simple optimization)
    next_free_frame: usize,
}

#[derive(Debug)]
pub enum AllocationError {
    NoFramesAvailable,
    InvalidFrame,
}

impl FrameAllocator {
    /// Initialize the frame allocator from multiboot memory map
    pub fn new(memory_map: &[MemoryInfoEntry], bitmap_addr: usize) -> Self {
        let mut highest_addr = 0;
    
        // Find highest address to determine number of frames needed
        for entry in memory_map {
            let end_addr = (entry.base_addr + entry.length) as usize;
            if end_addr > highest_addr {
                highest_addr = end_addr;
            }
        }

        let total_frames = highest_addr / PAGE_SIZE + 1;
        let bitmap_size = (total_frames + 7) / 8; // Round up to nearest byte

        // Mark all frames as used initially
        let bitmap = unsafe { NonNull::new_unchecked(core::slice::from_raw_parts_mut(
            bitmap_addr as *mut u8,
            bitmap_size
        ))};
        
        let mut allocator = FrameAllocator {
            bitmap,
            total_frames,
            next_free_frame: 0,
        };

        // Mark available regions as free
        for entry in memory_map {
            if entry.typee == 1 { // Available memory
                let start_frame = PhysFrame::containing_address(entry.base_addr as usize);
                let end_frame = PhysFrame::containing_address((entry.base_addr + entry.length) as usize);
                
                for frame in start_frame.number..=end_frame.number {
                    allocator.mark_frame_free(frame);
                }
            }
        }

        allocator
    }

    /// Allocate a physical frame
    pub fn allocate_frame(&mut self) -> Result<PhysFrame, AllocationError> {
        // Start searching from next_free_frame
        let mut frame = self.next_free_frame;

        while frame < self.total_frames {
            if !self.is_frame_used(frame) {
                self.mark_frame_used(frame);
                self.next_free_frame = frame + 1;
                return Ok(PhysFrame { number: frame });
            }
            frame += 1;
        }

        // If we didn't find a frame, start from beginning
        frame = 0;
        while frame < self.next_free_frame {
            if !self.is_frame_used(frame) {
                self.mark_frame_used(frame);
                self.next_free_frame = frame + 1;
                return Ok(PhysFrame { number: frame });
            }
            frame += 1;
        }

        Err(AllocationError::NoFramesAvailable)
    }

    /// Deallocate a physical frame
    pub fn deallocate_frame(&mut self, frame: PhysFrame) -> Result<(), AllocationError> {
        if frame.number >= self.total_frames {
            return Err(AllocationError::InvalidFrame);
        }

        self.mark_frame_free(frame.number);
        
        // Update next_free_frame if this frame is earlier
        if frame.number < self.next_free_frame {
            self.next_free_frame = frame.number;
        }

        Ok(())
    }

    /// Check if a frame is marked as used
    fn is_frame_used(&self, frame: usize) -> bool {
        let byte_index = frame / 8;
        let bit_index = frame % 8;
        unsafe {
            let bitmap = self.bitmap.as_ref();
            (bitmap[byte_index] & (1 << bit_index)) != 0
        }
    }

    /// Mark a frame as used in the bitmap
    fn mark_frame_used(&mut self, frame: usize) {
        let byte_index = frame / 8;
        let bit_index = frame % 8;
        unsafe {
            let bitmap = self.bitmap.as_mut();
            bitmap[byte_index] |= 1 << bit_index;
        }
    }

    /// Mark a frame as free in the bitmap
    fn mark_frame_free(&mut self, frame: usize) {
        let byte_index = frame / 8;
        let bit_index = frame % 8;
        unsafe {
            let bitmap = self.bitmap.as_mut();
            bitmap[byte_index] &= !(1 << bit_index);
        }
    }

    /// Get the total number of frames
    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    /// Get the number of free frames
    pub fn free_frames(&self) -> usize {
        let mut count = 0;
        for frame in 0..self.total_frames {
            if !self.is_frame_used(frame) {
                count += 1;
            }
        }
        count
    }
}