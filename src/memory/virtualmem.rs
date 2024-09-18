// src/paging/mod.rs

// Constants
const PAGE_SIZE: usize = 4096;
const PAGE_TABLE_ENTRIES: usize = 1024;
const PAGE_DIRECTORY_SIZE: usize = 1024;

// Page table entry flags
const PRESENT: u32 = 1 << 0;
const WRITABLE: u32 = 1 << 1;

// src/paging/mod.rs

use core::fmt;

#[derive(Clone, Copy)]
pub struct PageEntry(u32);

impl PageEntry {
    pub const fn new() -> Self {
        PageEntry(0)
    }

    pub fn set_address(&mut self, address: u32) {
        self.0 = (address & 0xFFFFF000) | (self.0 & 0xFFF);
    }

    pub fn address(&self) -> u32 {
        self.0 & 0xFFFFF000
    }

    pub fn set_flags(&mut self, flags: u32) {
        self.0 = (self.0 & 0xFFFFF000) | (flags & 0xFFF);
    }

    pub fn flags(&self) -> u32 {
        self.0 & 0xFFF
    }

    pub fn is_present(&self) -> bool {
        self.0 & 1 != 0
    }

    pub fn set_present(&mut self, present: bool) {
        if present {
            self.0 |= 1;
        } else {
            self.0 &= !1;
        }
    }

    pub fn is_writable(&self) -> bool {
        self.0 & 2 != 0
    }

    pub fn set_writable(&mut self, writable: bool) {
        if writable {
            self.0 |= 2;
        } else {
            self.0 &= !2;
        }
    }

}

impl fmt::Debug for PageEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PageEntry(address: {:#x}, flags: {:#x})", self.address(), self.flags())
    }
}

#[derive(Clone, Copy)]
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageEntry; PAGE_DIRECTORY_SIZE],
}

impl PageTable {
    pub const fn new() -> Self {
        PageTable {
            entries: [PageEntry::new(); PAGE_DIRECTORY_SIZE],
        }
    }

    pub fn entry(&self, index: usize) -> &PageEntry {
        &self.entries[index]
    }

    pub fn entry_mut(&mut self, index: usize) -> &mut PageEntry {
        &mut self.entries[index]
    }
}

// The PageDirectory can now use the same PageTable structure
pub type PageDirectory = PageTable;

static mut PAGE_DIRECTORY : [PageDirectory; PAGE_DIRECTORY_SIZE] = [PageDirectory::new(); PAGE_DIRECTORY_SIZE];
static mut FIRST_PAGE_TABLE : PageTable = PageTable::new();
// Example usage in your paging initialization


pub unsafe fn enable_paging() {
    core::arch::asm!(
        "mov eax, cr0",
        "or eax, 0x80000000",
        "mov cr0, eax",
        out("eax") _
    );
}

pub fn is_paging_enabled() -> bool {
    let cr0: u32;
    unsafe {
        core::arch::asm!("mov {}, cr0", out(reg) cr0);
    }
    cr0 & 0x80000000 != 0
}