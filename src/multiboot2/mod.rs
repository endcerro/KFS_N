// use meminfo::{MemoryInfo, MemoryInfoHeader};


use core::ptr::null;

use meminfo::{MemoryInfo, MemoryInfoHeader};

pub mod meminfo;
#[derive(Debug, Copy, Clone)]
pub struct MultibootInfo { //Base strtuct to init
    pub header : *const MultibootInfoHeader,
    pub tag : MultibootInfoTagIterator
}

static mut MBOOT_HEADER : *const MultibootInfoHeader = null();

pub fn bind_header(ptr : *const MultibootInfoHeader) {
    unsafe  {
        MBOOT_HEADER = ptr;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MultibootInfoHeader { //Base content of the multiboot struct
    total_size : u32,
    reserverd : u32
}

#[derive(Debug, Copy, Clone)]
pub struct MultibootInfoTagIterator {
    pub tag : *const MultibootInfoTag
}
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MultibootInfoTag {
    pub typee : u32,
    pub size : u32,
}

impl MultibootInfo {
    pub fn new (ptr : *const MultibootInfoHeader) -> MultibootInfo {
        MultibootInfo {
            header : ptr,
            tag : MultibootInfoTagIterator {tag: unsafe { ptr.offset(1) as *const MultibootInfoTag } } //The tag points to the first tag available
        }
    }
    pub fn display (&mut self) {
            // (*self.header).display();
        let mut id_collected : [u32; 100] = [0;100];
        let mut current_idx = 0;
        loop {
            match self.tag.next() {
                Some(i) => {
                    id_collected[current_idx] = unsafe { (*i).typee };
                    current_idx +=1 ;
                    println!("{:#?}", i)
                },
                None => break
            }
        }
        print!("We collected : ");
        for a in 0..current_idx
        {
            match a {
                0 => (),
                _n => print!("{},", id_collected[a])
            }
        }
        println!();
    }
    pub fn get_memory_info(&mut self) -> Option<MemoryInfo> {
        loop {
            match self.tag.next() {
                Some(i ) => {
                    if unsafe {(*i).typee} == 6 {
                        return Some(MemoryInfo::new(i as *const MemoryInfoHeader))
                    }
                },
                None => { return None}
            }
        }
    }
    pub fn print_memory_info() {
        // let mut meminfo = self.get_memory_info().unwrap();
        meminfo::print_meminfo();
    }
}

impl MultibootInfoHeader {
    pub fn display (&self) {
        print!("{:#?}", self);
    }
}

impl MultibootInfoTag {
    pub fn display (&self) {
        print!("{:#?}", self);
    }
}

impl Iterator for MultibootInfoTagIterator {
    type Item = *const MultibootInfoTag;
    fn next(&mut self) -> Option<Self::Item> //Return the current and move ptr to next
    {
        unsafe {
            let tag: *const MultibootInfoTag = self.tag ;
            if (*tag).typee == 0 && (*tag).size == 8 {
                return None
            }
            else {
                let offset: u32 = match (*tag).size {
                    s if s % 8 == 0 => s,
                    s => (s & !0x7) + 8
                };
                self.tag = ((self.tag as usize) + offset as usize) as *const MultibootInfoTag;
                return Some(tag);
            }
        }
    }
}


