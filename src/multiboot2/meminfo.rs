use core::fmt;

#[derive(Debug, Copy, Clone)]
pub struct MemoryInfo
{
    pub header : *const MemoryInfoHeader,
    pub entry : MemoryInfoIterator
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MemoryInfoHeader
{
    typee : u32,
    size : u32,
    entry_size : u32,
    entry_version : u32
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryInfoIterator
{
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

impl fmt::Display for MemoryInfoEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Start: {:x}, length: {:x}, type {}", self.base_addr, self.length, self.typee)
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