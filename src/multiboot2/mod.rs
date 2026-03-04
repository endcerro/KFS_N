// multiboot2/mod.rs - Multiboot2 info structure parsing
//
// The multiboot2 info pointer is stored by bootstrap.asm into the global
// `multiboot_ptr` symbol.  We read it here via `extern "C"` - no raw
// pointer parameter passing needed.

use core::ptr::null;

use meminfo::{MemoryInfo, MemoryInfoHeader};

pub mod meminfo;

// ---------------------------------------------------------------------------
// Multiboot pointer - written by bootstrap.asm at boot, read-only from Rust
// ---------------------------------------------------------------------------

extern "C" {
    /// Virtual address of the multiboot2 info structure.
    /// Set by bootstrap.asm before calling rust_main.
    static multiboot_ptr: u32;
}

/// Global cache of the multiboot header pointer.
/// Initialised once by `init()`, then read by all subsystems.
static mut MBOOT_HEADER: *const MultibootInfoHeader = null();

/// Read the multiboot pointer from the assembly global and cache it.
/// Must be called once at the very start of rust_main, before anything
/// that needs multiboot data (memory map, etc.).
pub fn init() {
    unsafe {
        let raw = multiboot_ptr;
        println!("Multiboot2: raw pointer value = {:#x}", raw);

        let ptr = raw as *const MultibootInfoHeader;
        if ptr.is_null() {
            panic!("Multiboot2 info pointer is null!");
        }

        // Sanity check: read the total_size field - should be reasonable
        // (typically a few hundred bytes, never zero or gigabytes)
        let header = &*ptr;
        println!("Multiboot2: total_size = {}, reserved = {}",
            header.total_size, header.reserverd);

        if header.total_size == 0 || header.total_size > 0x10000 {
            panic!("Multiboot2: total_size looks invalid ({:#x}), pointer likely corrupt",
                header.total_size);
        }

        MBOOT_HEADER = ptr;
    }
    println!("Multiboot2: info structure parsed OK");
}

// ---------------------------------------------------------------------------
// Multiboot2 structures
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct MultibootInfo {
    pub header: *const MultibootInfoHeader,
    pub tag: MultibootInfoTagIterator,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MultibootInfoHeader {
    total_size: u32,
    reserverd: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct MultibootInfoTagIterator {
    pub tag: *const MultibootInfoTag,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MultibootInfoTag {
    pub typee: u32,
    pub size: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Tag {
    pub tag_type: u32,
    pub size: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapTag {
    pub tag_type: u32,
    pub size: u32,
    pub entry_size: u32,
    pub entry_version: u32,
    // Entries follow immediately after
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryInfoEntry {
    pub base_addr: u64,
    pub length: u64,
    pub typee: u32,
    pub _reserved: u32,
}

// ---------------------------------------------------------------------------
// MultibootInfo - tag iteration and accessors
// ---------------------------------------------------------------------------

impl MultibootInfo {
    pub fn new(ptr: *const MultibootInfoHeader) -> MultibootInfo {
        MultibootInfo {
            header: ptr,
            // First tag starts immediately after the 8-byte header
            tag: MultibootInfoTagIterator {
                tag: unsafe { ptr.offset(1) as *const MultibootInfoTag },
            },
        }
    }

    pub fn display(&mut self) {
        let mut id_collected: [u32; 100] = [0; 100];
        let mut current_idx = 0;
        loop {
            match self.tag.next() {
                Some(i) => {
                    id_collected[current_idx] = unsafe { (*i).typee };
                    current_idx += 1;
                    println!("{:#?}", i)
                }
                None => break,
            }
        }
        print!("We collected : ");
        for a in 0..current_idx {
            match a {
                0 => (),
                _n => print!("{},", id_collected[a]),
            }
        }
        println!();
    }

    pub fn get_memory_info(&mut self) -> Option<MemoryInfo> {
        loop {
            match self.tag.next() {
                Some(i) => {
                    if unsafe { (*i).typee } == 6 {
                        return Some(MemoryInfo::new(i as *const MemoryInfoHeader));
                    }
                }
                None => return None,
            }
        }
    }

    pub fn print_memory_info() {
        meminfo::print_meminfo();
    }
}

impl MultibootInfoHeader {
    pub fn display(&self) {
        print!("{:#?}", self);
    }
}

impl MultibootInfoTag {
    pub fn display(&self) {
        print!("{:#?}", self);
    }
}

impl Iterator for MultibootInfoTagIterator {
    type Item = *const MultibootInfoTag;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let tag: *const MultibootInfoTag = self.tag;
            // End tag: type == 0 and size == 8
            if (*tag).typee == 0 && (*tag).size == 8 {
                return None;
            } else {
                // Tags are 8-byte aligned in the multiboot2 info structure
                let offset: u32 = match (*tag).size {
                    s if s % 8 == 0 => s,
                    s => (s & !0x7) + 8,
                };
                self.tag = ((self.tag as usize) + offset as usize) as *const MultibootInfoTag;
                return Some(tag);
            }
        }
    }
}