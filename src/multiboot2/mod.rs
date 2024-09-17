// use meminfo::{MemoryInfo, MemoryInfoHeader};

use meminfo::{MemoryInfo, MemoryInfoEntry, MemoryInfoHeader};

pub mod meminfo;
#[derive(Debug, Copy, Clone)]
pub struct MultibootInfo { //Base strtuct to init
    pub header : *const MultibootInfoHeader,
    pub tag : MultibootInfoTagIterator
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

pub fn init_mem(multiboot_struct_ptr: *const MultibootInfoHeader) {
    let mut meminfo = MultibootInfo::new(multiboot_struct_ptr).get_memory_info().unwrap();
    let mut entries = [MemoryInfoEntry::default(); 128];
    let mut i : usize = 0;
    loop {
        match meminfo.entry.next() {
            Some(meminfo) => {
                entries[i] = unsafe {*meminfo};
                i += 1;
            }
            None => break
        }
    }
    for j in 0..i {
        println!("{} : {}",j, entries[j]);
    }
}