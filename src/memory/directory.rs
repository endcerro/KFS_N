const PRESENT: u32 = 1 << 0;
const READ_WRITE: u32 = 1 << 1;
const USER_SUPERVISOR: u32 = 1 << 2;
const SUPERVISOR: u32 = 0 << 2;
const WRITE_THROUGH: u32 = 1 << 3;
const CACHE_DISABLE: u32 = 1 << 4;
const ACCESSED: u32 = 1 << 5;
const DIRTY: u32 = 1 << 6;
const PAGE_SIZE_4MB: u32 = 1 << 7;
const PAGE_SIZE_4KB: u32 = 0 << 7;

const DIRSIZE: usize = 1024;
//PageDirectoryEntry points to TableDirectoryEntry
struct PageDirectoryEntry(u32);
pub struct PagingDirectory([PageDirectoryEntry; 1024]);

impl PagingDirectory {
    pub fn init(&mut self) {
        for i in 0..DIRSIZE {
            self.0[i].0 = 2;
        }
    }
}
