//use physical::FRAME_ALLOCATOR;

use core::{ops::Index, ptr::addr_of};

use crate::{multiboot2::{meminfo::{self, MemoryInfoEntry}, MultibootInfo, MultibootInfoHeader}, serial, serial_println, utils};

//pub mod physical;
//pub mod virtualmem;

const PAGE_ENTRIES: usize = 1024;
const PAGE_SIZE : usize = 4096; //4K


#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PageEntry(u32);

pub type PageDirectoryEntry = PageEntry;
pub type PageTableEntry = PageEntry;

#[repr(transparent)]
pub struct PageTable([PageEntry;PAGE_ENTRIES]);

pub type PageDirectory = PageTable;


impl PageEntry {
    pub const fn new() -> Self {
        PageEntry(0)
    }
    pub fn set_flags(&mut self, present : bool, writeable : bool) {
        self.set_present(present);
        self.set_writable(writeable);
    }
    pub fn is_present(&self) -> bool{
        self.0 & 1 != 0
    }
    pub fn set_present(&mut self, present: bool){
        if present{
            self.0 |= 1 << 0;
        } else {
            self.0 |= 0 << 0;
        }
    }
    pub fn is_writable(&self) -> bool {
        self.0 & 1 << 1 != 0
    }
    pub fn set_writable(&mut self, writeable : bool){
        if writeable {
            self.0 |= 1 << 1;
        } else {
            self.0 |= 0 << 1;
        }
    }
    pub fn address(&self) -> usize {
       (self.0 & 0xFFFFF000) as usize
    }
    pub fn set_address(&mut self, address : usize)  {
        self.0 = self.0 & 0xFFF | (address as u32 & 0xFFFFF000);
    }
}

impl PageTable {
     pub const fn new() -> Self {
        PageTable([PageEntry::new(); PAGE_ENTRIES])
    }
    pub fn entry(&self, index: usize) -> &PageEntry {
        &self.0[index]
    }
    pub fn entry_mut(&mut self, index: usize) -> &mut PageEntry {
        &mut self.0[index]
    }
}

// Static Page Directory
pub static mut PAGE_DIRECTORY: PageDirectory = PageTable([PageEntry(0); PAGE_ENTRIES]);


// Static initial Page Table (for kernel space)
pub static mut KERNEL_PAGE_TABLE : PageTable = PageTable([PageEntry(0); PAGE_ENTRIES]);


pub unsafe fn init(_ptr : *mut MultibootInfoHeader) {
    utils::enable_interrupts(false);
   // Identity map the first 4MB of memory in KERNEL_PAGE_TABLE
   for i in 0..PAGE_ENTRIES {
    KERNEL_PAGE_TABLE.entry_mut(i).set_address(i * PAGE_SIZE);
    KERNEL_PAGE_TABLE.entry_mut(i).set_flags(true, true);
}

// Set up the first entry in the Page Directory
PAGE_DIRECTORY.entry_mut(0).set_address(addr_of!(KERNEL_PAGE_TABLE) as usize);
PAGE_DIRECTORY.entry_mut(0).set_flags(true, true);

// Ensure all other page directory entries are marked as not present
for i in 1..PAGE_ENTRIES {
    PAGE_DIRECTORY.entry_mut(i).set_flags(false, false);
}

serial_println!("Page structures set up");

// Load PAGE_DIRECTORY address into CR3
let pd_addr = addr_of!(PAGE_DIRECTORY) as usize;
core::arch::asm!("mov cr3, {}", in(reg) pd_addr, options(nomem, nostack));

serial_println!("Paging loaded");

// Enable paging
core::arch::asm!(
    "mov eax, cr0",
    "or eax, 0x80000000",
    "mov cr0, eax",
    "hlt",
    options(noreturn)
);

serial_println!("Paging enabled");

// Flush TLB
core::arch::asm!("invlpg [0]");

serial_println!("TLB flushed");
utils::enable_interrupts(true);

}



pub fn calculate_available_memory_bytes(multiboot_info: &mut MultibootInfo) -> u64 {
    let mut total_available = 0;
    
    if let Some(mem_info) = multiboot_info.get_memory_info() {
        let mut mem_iterator = mem_info.entry;
        
        loop {
            match mem_iterator.next() {
                None => break,
                Some(entry) => unsafe {
                    let entry: &MemoryInfoEntry = &*entry;
                    if entry.typee == 1 {  // Assuming type 1 is available memory
                        total_available += entry.length;
                    }
                }
            }
        }
    }
    serial_println!("There is {}b", total_available);
    serial_println!("There is {}kb", total_available / 1024);
    serial_println!("There is {}mb", total_available / 1024 /1024);
    total_available
}