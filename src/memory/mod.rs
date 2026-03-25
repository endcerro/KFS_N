use paging::PageDirectory;

pub mod allocator;
pub mod define;
pub mod heap;
pub mod pageflags;
pub mod paging;
pub mod physical;
pub mod vmm;

extern "C" {
    pub fn clear_page1();
}

pub fn init() {
    unsafe {
        PAGING = Some(PageDirectory::new());
    }

    init_physical_memory();

    // Install recursive page directory mapping (PDE[1023] -> PD itself).
    // This must happen before clear_page1() and before any map/unmap calls,
    // because the VMM uses the recursive mapping for all page table access.
    vmm::init();

    unsafe {
        #[cfg(feature = "verbose")]
        println!("Cleaning identity map...");
        clear_page1();
    }
    // diagnose_page_directory();
    // Run VMM self-tests after everything is initialised
    // vmm::test_virtual_memory();

    // Map the initial kernel heap region.
    // Must come after vmm::init() and the frame allocator.
    heap::init();
    // heap::test_heap();
    heap::print_stats();

    // Run GlobalAlloc tests if the feature is enabled.
    // These exercise Box, Vec, String through the #[global_allocator].
    #[cfg(feature = "alloc_test")]
    allocator::test_global_alloc();
}

pub fn diagnose_page_directory() {
    println!("\n=== Page Directory Diagnostic ===\n");

    unsafe {
        use crate::memory::define::KERNEL_OFFSET;
        use crate::memory::PAGING;

        let cr3: u32;
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
        println!("CR3 (Page Directory Physical): {:#010x}", cr3);

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

        let pde_0 = PAGING.as_mut().unwrap().get_entry(0);
        println!("PDE[0] (0x00000000-0x003FFFFF):");
        println!("  Raw value: {:#010x}", (*pde_0).value());
        println!("  Present:   {}", (*pde_0).present());
        if (*pde_0).present() {
            println!("  Address:   {:#010x}", (*pde_0).address());
            println!("  Writable:  {}", (*pde_0).writeable());
            println!("  User:      {}", (*pde_0).user());
        }

        let pde_768 = PAGING.as_mut().unwrap().get_entry(768);
        println!("\nPDE[768] (0xC0000000-0xC03FFFFF):");
        println!("  Raw value: {:#010x}", (*pde_768).value());
        println!("  Present:   {}", (*pde_768).present());
        if (*pde_768).present() {
            println!("  Address:   {:#010x}", (*pde_768).address());
            println!("  Writable:  {}", (*pde_768).writeable());
            println!("  User:      {}", (*pde_768).user());

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

        println!("\n--- All Present Entries ---\n");
        let mut count = 0;
        for i in 0..1024 {
            let pde = PAGING.as_mut().unwrap().get_entry(i);
            if (*pde).present() {
                let virt_start = i * 0x400000;
                println!(
                    "PDE[{}] -> {:#010x} (maps {:#010x}-{:#010x})",
                    i,
                    (*pde).address(),
                    virt_start,
                    virt_start + 0x3FFFFF
                );
                count += 1;
            }
        }
        println!("\nTotal present entries: {}", count);

        println!("\n=== End Diagnostic ===\n");
    }
}

fn init_physical_memory() {
    if let Some(memory_map) = crate::multiboot2::meminfo::get_memory_map() {
        #[cfg(feature = "verbose")]
        println!(
            "Initializing physical memory allocator with {} memory regions...",
            memory_map.len()
        );

        physical::init_frame_allocator(memory_map);

        #[cfg(feature = "verbose")]
        println!("Physical memory allocator initialized");
    } else {
        panic!("Failed to get memory map from multiboot!");
    }
    // test_paging_infrastructure()
}

pub static mut PAGING: Option<PageDirectory> = None;

pub fn paging() -> &'static mut PageDirectory {
    unsafe { PAGING.as_mut().unwrap() }
}

// ---------------------------------------------------------------------------
// Paging infrastructure tests
// ---------------------------------------------------------------------------

pub fn test_paging_infrastructure() {
    use crate::memory::pageflags::PageFlags;
    use crate::memory::paging::PageEntry;

    println!("\n=== Testing Paging Infrastructure ===");

    // Test 1: PageEntry manipulation (same type for PDE and PTE)
    println!("\n[Test 1] PageEntry operations");
    {
        let mut entry = PageEntry::new(0x1000, PageFlags::PRESENT);
        assert!(entry.present(), "Entry should be present");
        assert_eq!(entry.address(), 0x1000, "Address mismatch");

        entry.set(0x2000, PageFlags::PRESENT | PageFlags::WRITABLE);
        assert!(entry.present(), "Entry should still be present");
        assert!(entry.writeable(), "Entry should be writable");
        assert_eq!(entry.address(), 0x2000, "Address should be updated");

        entry.set(
            0x3000,
            PageFlags::PRESENT | PageFlags::WRITABLE | PageFlags::USER,
        );
        assert!(entry.user(), "Entry should be user-accessible");
        assert_eq!(entry.address(), 0x3000, "Address should be updated");

        entry.clear();
        assert!(!entry.present(), "Entry should not be present after clear");

        println!("  PageEntry tests passed");
    }

    // Test 2: PageFlags operations
    println!("\n[Test 2] PageFlags operations");
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

        assert!(
            flags3.contains(PageFlags::PRESENT | PageFlags::USER),
            "contains() should detect combined flags"
        );
        assert!(
            !flags1.contains(PageFlags::USER),
            "contains() should reject missing flags"
        );

        println!("  PageFlags tests passed");
    }

    // Test 3: Read existing page directory entries
    println!("\n[Test 3] Reading current page directory");
    unsafe {
        let pde_0 = PAGING.as_mut().unwrap().get_entry(0);
        println!("  PDE[0] (0x00000000): {}", *pde_0);
        if !(*pde_0).present() {
            println!("    Identity mapping correctly cleared");
        } else {
            println!("    Warning: Identity mapping still present");
            println!("      Value: {:#010x}", (*pde_0).value());
        }

        let pde_768 = PAGING.as_mut().unwrap().get_entry(768);
        println!("  PDE[768] (0xC0000000): {}", *pde_768);

        if (*pde_768).present() {
            println!("    PDE[768] is present for higher-half kernel");
            println!("      Points to physical: {:#010x}", (*pde_768).address());
            println!("      Writable: {}", (*pde_768).writeable());
        } else {
            println!("    CRITICAL: PDE[768] is NOT present!");
            println!("      Value: {:#010x}", (*pde_768).value());

            println!("\n    Scanning for present entries:");
            for i in 0..1024 {
                let pde = PAGING.as_mut().unwrap().get_entry(i);
                if (*pde).present() {
                    println!("      PDE[{}] = {:#010x}", i, (*pde).value());
                }
            }
        }
    }

    // Test 4: Page directory physical address
    println!("\n[Test 4] Page directory physical address");
    unsafe {
        let phys_addr = PAGING.as_mut().unwrap().physical_address();
        println!("  Page directory physical address: {:#010x}", phys_addr);

        let cr3: u32;
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
        println!("  CR3 register value: {:#010x}", cr3);

        if phys_addr == cr3 {
            println!("  Physical address matches CR3");
        } else {
            println!("  WARNING: Physical address doesn't match CR3!");
        }
    }

    println!("\n=== Paging Infrastructure Tests Complete ===\n");
}
