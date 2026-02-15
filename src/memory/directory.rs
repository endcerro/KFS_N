use core::{fmt, ptr::NonNull};


// use crate::define;

const PRESENT :u32 = 1<<0;
const WRITABLE :u32 = 1<<1;
const USER :u32 = 1<<2;
const PWT: u32 = 1<<3;
const CACHE_DISABLE: u32 = 1 << 4;
const ACCESSED: u32 = 1 << 5;
const AVAILABLE: u32 = 1 << 6;
const PAGESIZE4MB: u32 = 1 << 7;

#[repr(C, align(4096))]
pub struct PageDirectory {
    pub entries: NonNull<[PageDirectoryEntry; super::define::PAGE_DIRECTORY_ENTRIES]>,
}
impl PageDirectory {
    pub fn new() -> Self {
        unsafe {
            PageDirectory {
                entries: NonNull::new_unchecked((
                    &super::define::page_directory as *const [u32; 1024]) as *mut [PageDirectoryEntry; 1024])
            }
        }
    }
    pub const fn default() -> Self {
        unsafe {
            PageDirectory {
                entries: NonNull::new_unchecked((
                    &super::define::page_directory as *const [u32; 1024]) as *mut [PageDirectoryEntry; 1024])
            }

        }
    }

    pub fn set_entry(&mut self, index: usize, page_table_phys_addr: u32, flags: super::pageflags::PageFlags) {
        assert!(index < super::define::PAGE_DIRECTORY_ENTRIES, "PDE index out of bounds");
        assert!(page_table_phys_addr & 0xFFF == 0, "Page table address must be 4KB aligned");

        unsafe {
            (*self.get_entry(index)).set(page_table_phys_addr, flags.value());
        }

        #[cfg(feature = "verbose")]
        println!("PDE[{}] set to phys {:#x} with flags {:#x}", index, page_table_phys_addr, flags.value());
    }
    pub fn clear_entry(&mut self, index: usize) {
        assert!(index < super::define::PAGE_DIRECTORY_ENTRIES, "PDE index out of bounds");

        unsafe {
            (*self.get_entry(index)).clear();
        }

        #[cfg(feature = "verbose")]
        println!("PDE[{}] cleared", index);
    }

    pub fn get_entry(&mut self, index:usize) -> *mut PageDirectoryEntry {
         unsafe {
            // CRITICAL: Cast to *mut PageDirectoryEntry FIRST, then add index
            // If we add to the array pointer, we jump by array size, not element size!
            let base = self.entries.as_ptr() as *mut PageDirectoryEntry;
            base.add(index)
        }
    }
    pub fn physical_address(&self) -> u32 {
        (self.entries.as_ptr() as u32).wrapping_sub(super::define::KERNEL_OFFSET as u32)
    }
}

pub struct PageDirectoryEntry(u32);
impl PageDirectoryEntry {

    //Why32 ? Use Struct pageflags
    pub fn new(phys_addr: u32, flags: u32) -> Self {
        debug_assert!(phys_addr & 0xFFF == 0, "PDE address must be 4KB aligned");
        PageDirectoryEntry((phys_addr & 0xFFFFF000) | (flags & 0xFFF))
    }

    pub const fn empty() -> Self {
        PageDirectoryEntry(0)
    }

    pub fn set(&mut self, phys_addr: u32, flags: u32) {
        debug_assert!(phys_addr & 0xFFF == 0, "PDE address must be 4KB aligned");
        self.0 = (phys_addr & 0xFFFFF000) | (flags & 0xFFF);
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }

    #[inline] pub fn value(&self) -> u32 {
        self.0
    }
    #[inline] pub fn present(&self) -> bool {
        self.0 & PRESENT > 0
    }
    #[inline] pub fn writeable(&self) -> bool {
        self.0 & WRITABLE > 0
    }
    #[inline] pub fn user(&self) -> bool {
        self.0 & USER > 0
    }
    #[inline] pub fn pwt(&self) -> bool {
        self.0 & PWT > 0
    }
    #[inline] pub fn cache_disable(&self) -> bool {
        self.0 & CACHE_DISABLE > 0
    }
    #[inline] pub fn accessed(&self) -> bool {
        self.0 & ACCESSED > 0
    }
    #[inline] pub fn available(&self) -> bool {
        self.0 & AVAILABLE > 0
    }
    #[inline] pub fn pagesize4mb(&self) -> bool {
        self.0 & PAGESIZE4MB > 0
    }
    #[inline] pub fn page_frame_number(&self) -> u32 {
        self.0 >> 12
    }
    #[inline] pub fn address(&self) -> u32 {
        self.0 & 0xFFFFF000
    }
}

impl fmt::Display for PageDirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Present {},writeable {},user {},pwt {},cache disable {},accessed {},available {},4mb {}, address {:x}",
        self.present(), self.writeable(), self.user(), self.pwt(), self.cache_disable(), self.accessed(), self.available(), self.pagesize4mb(), self.address())
    }
}