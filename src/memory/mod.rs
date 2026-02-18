use directory::PageDirectory;


pub mod pageflags;
mod pagetable;
pub mod define;
pub mod vmm;

pub mod directory;
pub mod physical;



extern "C" {
    pub fn clear_page1();
}

pub fn init() {
    unsafe {
        PAGING = Some(PageDirectory::new());
    }
    diagnose_page_directory();
    init_physical_memory();

    // Install recursive page directory mapping (PDE[1023] → PD itself).
    // This must happen before clear_page1() and before any map/unmap calls,
    // because the VMM uses the recursive mapping for all page table access.
    vmm::init();

    unsafe {
        #[cfg(feature = "verbose")]
        println!("Entry 0 : {}", *PAGING.as_mut().unwrap().get_entry(0));
        #[cfg(feature = "verbose")]
        println!("Cleaning identity map...");
        clear_page1();
        #[cfg(feature = "verbose")]
        println!("Entry 0 : {}", *PAGING.as_mut().unwrap().get_entry(0));
    }

    // Run VMM self-tests after everything is initialised
    vmm::test_virtual_memory();
}

// Diagnostic tool for paging infrastructure
// Add this to your memory module and call it from memory::init()

pub fn diagnose_page_directory() {
    println!("\n=== Page Directory Diagnostic ===\n");
    
    unsafe {
        use crate::memory::PAGING;
        use crate::memory::define::KERNEL_OFFSET;
        
        // Check CR3
        let cr3: u32;
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
        println!("CR3 (Page Directory Physical): {:#010x}", cr3);
        
        // Check our page directory address
        let pd_virt = &crate::memory::define::page_directory as *const _ as u32;
        let pd_phys = pd_virt - KERNEL_OFFSET as u32;
        println!("Page Directory Virtual:  {:#010x}", pd_virt);
        println!("Page Directory Physical: {:#010x}", pd_phys);
        
        if cr3 != pd_phys {
            println!("WARNING: CR3 doesn't match page_directory!");
        } else {
            println!("CR3 matches page_directory OK");
        }
        
        println!("\n--- Checking Key Entries ---\n");
        
        // Check entry 0 (should be cleared)
        let pde_0 = PAGING.as_mut().unwrap().get_entry(0);
        println!("PDE[0] (0x00000000-0x003FFFFF):");
        println!("  Raw value: {:#010x}", (*pde_0).value());
        println!("  Present:   {}", (*pde_0).present());
        if (*pde_0).present() {
            println!("  Address:   {:#010x}", (*pde_0).address());
            println!("  Writable:  {}", (*pde_0).writeable());
            println!("  User:      {}", (*pde_0).user());
        }
        
        // Check entry 768 (should be present for higher half)
        let pde_768 = PAGING.as_mut().unwrap().get_entry(768);
        println!("\nPDE[768] (0xC0000000-0xC03FFFFF):");
        println!("  Raw value: {:#010x}", (*pde_768).value());
        println!("  Present:   {}", (*pde_768).present());
        if (*pde_768).present() {
            println!("  Address:   {:#010x}", (*pde_768).address());
            println!("  Writable:  {}", (*pde_768).writeable());
            println!("  User:      {}", (*pde_768).user());
            
            // Check if it points to page_table1
            extern "C" {
                static page_table1: [u32; 1024];
            }
            let pt1_virt = &page_table1 as *const _ as u32;
            let pt1_phys = pt1_virt - KERNEL_OFFSET as u32;
            println!("  page_table1 phys: {:#010x}", pt1_phys);
            
            if (*pde_768).address() == pt1_phys {
                println!("  Points to page_table1 OK");
            } else {
                println!("  Does NOT point to page_table1!");
            }
        } else {
            println!("  NOT PRESENT - This is the problem!");
        }
        
        // Scan for all present entries
        println!("\n--- All Present Entries ---\n");
        let mut count = 0;
        for i in 0..1024 {
            let pde = PAGING.as_mut().unwrap().get_entry(i);
            if (*pde).present() {
                let virt_start = i * 0x400000; // Each PDE covers 4MB
                println!("PDE[{}] -> {:#010x} (maps {:#010x}-{:#010x})", 
                    i, 
                    (*pde).address(),
                    virt_start,
                    virt_start + 0x3FFFFF);
                count += 1;
            }
        }
        println!("\nTotal present entries: {}", count);
        
        println!("\n=== End Diagnostic ===\n");
    }
}

// src/memory/mod.rs - Updated init_physical_memory function
fn init_physical_memory() {
    // Get memory map from multiboot (now returns a slice reference)
    if let Some(memory_map) = crate::multiboot2::meminfo::get_memory_map() {
        #[cfg(feature = "verbose")]
        println!("Initializing physical memory allocator with {} memory regions...", memory_map.len());

        physical::init_frame_allocator(memory_map);

        #[cfg(feature = "verbose")]
        println!("Physical memory allocator initialized");
    } else {
        panic!("Failed to get memory map from multiboot!");
    }
    test_paging_infrastructure()
}



// pub static mut PAGING: PageDirectory = PageDirectory::default();

pub static mut PAGING: Option<PageDirectory> = None;

pub fn paging() -> &'static mut PageDirectory {
    unsafe {
        PAGING.as_mut().unwrap()
    }
}

// Test module for paging infrastructure
// This can be called from memory::init() with #[cfg(feature = "paging_test")]

// Test module for paging infrastructure
// This can be called from memory::init() with #[cfg(feature = "paging_test")]

pub fn test_paging_infrastructure() {
    use crate::memory::{pageflags::PageFlags, PAGING};
    use crate::memory::directory::PageDirectoryEntry;
    use crate::memory::pagetable::{PageTable, PageTableEntry};
    
    println!("\n=== Testing Paging Infrastructure ===");
    
    // Test 1: PageDirectoryEntry manipulation
    println!("\n[Test 1] PageDirectoryEntry operations");
        let mut pde = PageDirectoryEntry::new(0x1000, PageFlags::PRESENT.value());
        assert!(pde.present(), "PDE should be present");
        assert_eq!(pde.address(), 0x1000, "PDE address mismatch");
        
        pde.set(0x2000, (PageFlags::PRESENT | PageFlags::WRITABLE).value());
        assert!(pde.present(), "PDE should still be present");
        assert!(pde.writeable(), "PDE should be writable");
        assert_eq!(pde.address(), 0x2000, "PDE address should be updated");
        
        pde.clear();
        assert!(!pde.present(), "PDE should not be present after clear");
        
        println!("  PageDirectoryEntry tests passed");
    
    // Test 2: PageTableEntry manipulation
    println!("\n[Test 2] PageTableEntry operations");
    let mut pte = PageTableEntry::new(0x1000, PageFlags::PRESENT.value());
    assert!(pte.present(), "PTE should be present");
    assert_eq!(pte.address(), 0x1000, "PTE address mismatch");
    
    pte.set(0x3000, (PageFlags::PRESENT | PageFlags::WRITABLE | PageFlags::USER).value());
    assert!(pte.present(), "PTE should still be present");
    assert!(pte.writeable(), "PTE should be writable");
    assert!(pte.user(), "PTE should be user-accessible");
    assert_eq!(pte.address(), 0x3000, "PTE address should be updated");
    
    pte.clear();
    assert!(!pte.present(), "PTE should not be present after clear");
    
    println!("  PageTableEntry tests passed");
    
    // Test 3: PageFlags operations
    println!("\n[Test 3] PageFlags operations");
    {
        let flags1 = PageFlags::PRESENT | PageFlags::WRITABLE;
        assert!(flags1.is_present(), "Flags should have PRESENT");
        assert!(flags1.is_writable(), "Flags should have WRITABLE");
        assert!(!flags1.is_user(), "Flags should not have USER");
        
        let flags2 = PageFlags::PRESENT | PageFlags::USER;
        let flags3 = flags1 | flags2;
        assert!(flags3.is_present(), "Combined flags should have PRESENT");
        assert!(flags3.is_writable(), "Combined flags should have WRITABLE");
        assert!(flags3.is_user(), "Combined flags should have USER");
        
        println!("  PageFlags tests passed");
    }
    
    // Test 4: Read existing page directory entries
    println!("\n[Test 4] Reading current page directory");
    unsafe {
        // Entry 0 should NOT be present (identity map was cleared)
        let pde_0 = PAGING.as_mut().unwrap().get_entry(0);
        println!("  PDE[0] (0x00000000): {}", *pde_0);
        if !(*pde_0).present() {
            println!("    Identity mapping correctly cleared");
        } else {
            println!("    Warning: Identity mapping still present");
            println!("      Value: {:#010x}", (*pde_0).value());
        }
        
        // Entry 768 - check if present (maps 0xC0000000)
        let pde_768 = PAGING.as_mut().unwrap().get_entry(768);
        println!("  PDE[768] (0xC0000000): {}", *pde_768);
        
        if (*pde_768).present() {
            println!("    PDE[768] is present for higher-half kernel");
            println!("      Points to physical: {:#010x}", (*pde_768).address());
            println!("      Writable: {}", (*pde_768).writeable());
        } else {
            println!("    CRITICAL: PDE[768] is NOT present!");
            println!("      Value: {:#010x}", (*pde_768).value());
            println!("      This means 0xC0000000 region is not mapped!");
            
            // Scan for any present entries
            println!("\n    Scanning for present entries:");
            for i in 0..1024 {
                let pde = PAGING.as_mut().unwrap().get_entry(i);
                if (*pde).present() {
                    println!("      PDE[{}] = {:#010x}", i, (*pde).value());
                }
            }
        }
    }
    
    // Test 5: Page directory physical address
    println!("\n[Test 5] Page directory physical address");
    unsafe {
        let phys_addr = PAGING.as_mut().unwrap().physical_address();
        println!("  Page directory physical address: {:#010x}", phys_addr);
        
        // Should match the address loaded into CR3
        let cr3: u32;
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
        println!("  CR3 register value: {:#010x}", cr3);
        
        if phys_addr == cr3 {
            println!("  Physical address matches CR3");
        } else {
            println!("  WARNING: Physical address doesn't match CR3!");
            println!("    This could indicate the PAGING global is pointing to wrong memory");
        }
    }
    
    println!("\n=== Paging Infrastructure Tests Complete ===\n");
}