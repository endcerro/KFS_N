use core::fmt;

use crate::multiboot2::{MultibootInfo, MBOOT_HEADER};

#[derive(Debug, Copy, Clone)]
pub struct MemoryInfo {
    pub header : *const MemoryInfoHeader,
    pub entry : MemoryInfoIterator
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MemoryInfoHeader {
    typee : u32,
    size : u32,
    entry_size : u32,
    entry_version : u32
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryInfoIterator {
    pub entry : *const MemoryInfoEntry,
    endpoint : *const MemoryInfoEntry
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MemoryInfoEntry {
    pub base_addr : u64,
    pub length : u64,
    pub typee : u32,
    reserved : u32
}

impl Default for MemoryInfoEntry {
    fn default() -> Self {
        Self {
            base_addr :  0,
            length : 0,
            typee : 0,
            reserved : 0
        }
    }
}

// Memory region type constants
const MULTIBOOT_MEMORY_AVAILABLE: u32 = 1;
const MULTIBOOT_MEMORY_RESERVED: u32 = 2;
const MULTIBOOT_MEMORY_ACPI_RECLAIMABLE: u32 = 3;
const MULTIBOOT_MEMORY_NVS: u32 = 4;
const MULTIBOOT_MEMORY_BADRAM: u32 = 5;

// Function to format bytes into human readable format
fn format_size(bytes: u64) -> (f64, &'static str) {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        ((bytes as f64) / (GB as f64), "GB")
    } else if bytes >= MB {
        ((bytes as f64) / (MB as f64), "MB")
    } else if bytes >= KB {
        ((bytes as f64) / (KB as f64), "KB")
    } else {
        (bytes as f64, "B")
    }
}

fn get_region_type_str(typee: u32) -> &'static str {
    match typee {
        MULTIBOOT_MEMORY_AVAILABLE => "Available",
        MULTIBOOT_MEMORY_RESERVED => "Reserved",
        MULTIBOOT_MEMORY_ACPI_RECLAIMABLE => "ACPI Reclaimable",
        MULTIBOOT_MEMORY_NVS => "ACPI NVS",
        MULTIBOOT_MEMORY_BADRAM => "Bad RAM",
        _ => "Unknown"
    }
}

impl fmt::Display for MemoryInfoEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (size, unit) = format_size(self.length);
        let region_type = get_region_type_str(self.typee);
        write!(f, "Base: 0x{:08x}, Size: {:.2} {}, Type: {}", 
            self.base_addr,
            size,
            unit,
            region_type
        )
    }
}

impl MemoryInfo {
    pub fn new(ptr: *const MemoryInfoHeader) -> MemoryInfo
    {
        MemoryInfo {
            header : ptr,
            entry : MemoryInfoIterator::new(unsafe { ptr.offset(1) as *const MemoryInfoEntry },
            unsafe { (*ptr).size })
        }
    }
}

impl MemoryInfoIterator {
    pub fn new(ptr : *const MemoryInfoEntry, size : u32) -> MemoryInfoIterator {
        MemoryInfoIterator {
            entry : ptr,
            endpoint : (unsafe { ptr.offset(-1) } as usize + size as usize ) as *const MemoryInfoEntry
        }
    }
}


impl Iterator for MemoryInfoIterator {
    type Item = *const MemoryInfoEntry;
    fn next(&mut self) -> Option<Self::Item> {
        if self.entry as usize >= self.endpoint as usize{
            return None;
        }
        // let ret: MemoryInfoEntry = unsafe {*self.entry};
        let ret = self.entry;
        unsafe { self.entry = self.entry.offset(1);}
        return Some(ret);
    }
}

// pub fn meminfo(multiboot_struct_ptr: *const MultibootInfoHeader) {
pub fn print_meminfo() {
    let mut meminfo;
    unsafe {
        meminfo = MultibootInfo::new(MBOOT_HEADER).get_memory_info().unwrap();
    }
    let mut entries = [MemoryInfoEntry::default(); 128];
    let mut i: usize = 0;
    let mut total_available: u64 = 0;
    let mut total_reserved: u64 = 0;

    // Collect memory entries
    loop {
        match meminfo.entry.next() {
            Some(meminfo) => {
                entries[i] = unsafe { *meminfo };
                match entries[i].typee {
                    MULTIBOOT_MEMORY_AVAILABLE => total_available += entries[i].length,
                    _ => total_reserved += entries[i].length,
                }
                i += 1;
            }
            None => break
        }
    }

    // Print memory map header
    println!("\nMemory Map:");
    println!("----------------------------------------");
    // Print each memory region
    for j in 0..i {
        println!("Region {}: {}", j, entries[j]);
    }
    // Print summary
    println!("----------------------------------------");
    let (avail_size, avail_unit) = format_size(total_available);
    let (resv_size, resv_unit) = format_size(total_reserved);
    let (total_size, total_unit) = format_size(total_reserved + total_available);
    println!("Total Available: {:.2} {}", avail_size, avail_unit);
    println!("Total Reserved:  {:.2} {}", resv_size, resv_unit);
    println!("Total Memory:    {:.2} {}", total_size, total_unit);
    println!("----------------------------------------\n");
}