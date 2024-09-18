use core::ptr;
use core::arch::asm;

const PAGE_SIZE: usize = 4096; // 4ko par page
const ENTRIES_PER_TABLE: usize = 1024; // 1024 entrées par Page Table et Page Directory
struct PageTableEntry(u32);
const TEST: usize = 3;


/*
|--------------------------------------|
| Bits  | Description                  |
|--------------------------------------|
| 0     | Presence                     |
| 1     | Lecture / ecriture           |
| 2     | Niveau utilisateur /kernel   |
| 3-11  | Proprietes diverse           |
| 12-31 | addresse de la page physique |
|--------------------------------------|
*/

#[repr(align(4096))] // specifie l'alignement de la memoire sur 4ko
#[derive(Copy, Clone)] // Cela permet de rendre PageTable copiable
struct PageTable {
    entries: [u32; ENTRIES_PER_TABLE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)] // Cela permet de rendre PageTable copiable
struct PageDirectory {
    entries: [PageTable; TEST],
}

static mut PAGE_DIR : PageDirectory = PageDirectory{entries:[PageTable{entries: [0; ENTRIES_PER_TABLE]}; TEST]}; 

impl PageTable {
    fn new() -> PageTable {
        PageTable {
            entries: [0; ENTRIES_PER_TABLE],
        }
    }
}

impl PageDirectory {
    fn map_page(&mut self, virtual_addr: u32, physical_addr: u32, flags: u32) {
        // Calcule l'index du directory et de la table de pages à partir de l'adresse virtuelle
        let dir_index = (virtual_addr >> 22) & 0x3FF;  // Les 10 bits supérieurs
        let table_index = (virtual_addr >> 12) & 0x3FF; // Les 10 bits intermédiaires

        // Vérifie si la table de pages à cet index est présente, sinon alloue-en une nouvelle
        if self.entries[dir_index as usize].entries[0] == 0 {
            // Alloue une nouvelle table de pages
            self.entries[dir_index as usize] = PageTable::new();
        }

        // Crée l'entrée dans la table de pages
        let entry = physical_addr | flags; // Les flags contiennent les permissions et autres informations

        // Remplit l'entrée de la table de pages avec l'adresse physique et les permissions
        self.entries[dir_index as usize].entries[table_index as usize] = entry;
    }
}

unsafe fn enable_paging(page_directory_addr: u32) {
    asm!(
        "mov cr3, {0}",          // Charge l'adresse du directory dans CR3
        "mov eax, cr0",
        "or eax, 0x80000000",    // Active le bit de paging dans CR0
        "mov cr0, eax",
        in(reg) page_directory_addr
    );
}

fn init_identity_mapping(page_dir: &mut PageDirectory) {
    let flags: u32 = 0x3; // Présent et Lecture/Écriture

    for i in 0..1024 {
        let addr = (i * PAGE_SIZE) as u32;
        page_dir.map_page(addr, addr, flags); // Mappage identique (virtuel = physique)
    }
}

pub fn testmain() {
    unsafe {
        // Étape 1: Initialiser un mappage identique pour les premières pages (par exemple, 4 Mo)
        println!("Initialisation du mappage identique...");
        init_identity_mapping(&mut PAGE_DIR);

        // Étape 2: Vérifier l'adresse du PageDirectory
        let page_directory_addr = &PAGE_DIR as *const _ as u32; // Adresse physique de PAGE_DIR
        println!("Adresse physique du Page Directory: {:#x}", page_directory_addr);

        // Étape 3: Activer le paging
        println!("Activation du paging...");
        enable_paging(page_directory_addr);

        // Étape 4: Test d'accès mémoire à différentes adresses
        println!("Test d'accès à la mémoire mappée...");

        // Test d'une adresse virtuelle identique (par exemple, 0x1000)
        let test_addr: *mut u32 = 0x1000 as *mut u32;  // Adresse virtuelle mappée identiquement
        println!("Essai d'écriture à l'adresse virtuelle: {:#x}", test_addr as u32);

        *test_addr = 42;  // Écrire à cette adresse
        let value = *test_addr;  // Lire depuis cette adresse
        println!("Valeur lue depuis {:#x}: {}", test_addr as u32, value);

        // Vérification de la valeur écrite et lue
        if value == 42 {
            println!("Paging et mappage réussi à l'adresse 0x1000 !");
        } else {
            println!("Erreur dans le mappage ou le paging à l'adresse 0x1000.");
        }
    }

} 



/* 




impl PageDirectory {
    fn new() -> PageDirectory {
        PageDirectory {
            entries: [0; ENTRIES_PER_TABLE],
        }
    }

}

unsafe fn load_page_directory(page_directory: &PageDirectory) {
    // Obtenir l'adresse physique du Page Directory
    let pd_address = page_directory as *const _ as u32;

    // Charger cette adresse dans CR3
    asm!(
        "mov cr3, {0}",
        in(reg) pd_address,
    );
}

unsafe fn enable_paging() {
    let mut cr0: u32;
    asm!("mov {0}, cr0", out(reg) cr0);
    cr0 |= 0x80000000;  // Activer le bit de paging (bit 31)
    asm!("mov cr0, {0}", in(reg) cr0);
}

extern "C" fn page_fault_handler() {
    let faulting_address: u32;

    unsafe {
        asm!("mov {0}, cr2", out(reg) faulting_address);  // L'adresse qui a causé la faute est dans CR2
    }

    // Ici, tu pourrais soit allouer dynamiquement la page, soit tuer le processus
    println!("Page fault at address: {:#x}", faulting_address);
}


static mut NEXT_FREE_PAGE: u32 = 0x100000; // Point de départ de l’allocation (par exemple 1 Mo)

fn alloc_page() -> u32 {
    unsafe {
        let page = NEXT_FREE_PAGE;
        NEXT_FREE_PAGE += PAGE_SIZE as u32; // Augmenter pour la prochaine allocation
        page
    }
}

unsafe fn map_virtual_to_physical(
    page_directory: *mut PageDirectory,
    virtual_address: u32,
    physical_address: u32
) {
    let page_directory_index = (virtual_address >> 22) as usize; // 10 bits pour le Page Directory
    let page_table_index = ((virtual_address >> 12) & 0x03FF) as usize; // 10 bits pour la Page Table

    // Vérifier si la Page Table existe dans le Page Directory
    if (*page_directory).entries[page_directory_index] == 0 {
        // Allouer une nouvelle Page Table si elle n'existe pas
        let new_page_table = alloc_page_table();
        (*page_directory).entries[page_directory_index] = (new_page_table as u32) | 0x3; // Présent + Lecture/Écriture
    }

    // Récupérer l'adresse de la Page Table
    let page_table_ptr = ((*page_directory).entries[page_directory_index] & 0xFFFFF000) as *mut PageTable;
    let page_table = &mut *page_table_ptr;

    // Mettre à jour l'entrée dans la Page Table pour mapper la page physique
    page_table.entries[page_table_index] = physical_address | 0x3; // Présent + Lecture/Écriture
}

fn init_paging() {
    unsafe {
        // Allouer un Page Directory
        let page_directory = alloc_page_directory();

        // Mapper quelques pages (par exemple, 10 pages virtuelles à partir de 0x400000)
        for i in 0..10 {
            let virtual_address = 0x400000 + i * PAGE_SIZE as u32;
            let physical_address = alloc_page() as u32; // Allouer une page physique
            map_virtual_to_physical(page_directory, virtual_address, physical_address);
        }

        // Charger le Page Directory dans CR3 et activer le paging
        load_page_directory(&page_directory);
        enable_paging();
    }
}



fn test_memory() {
    // Adresse virtuelle de test
    let virtual_address: *mut u32 = 0x400000 as *mut u32;

    unsafe {
        // Écriture dans la mémoire mappée
        *virtual_address = 42;

        // Lecture dans la mémoire mappée
        let value = *virtual_address;
        println!("Valeur lue depuis la mémoire virtuelle : {}", value);
    }
}

pub fn maintest() {
    // Initialiser et activer le paging
    init_memory();

    // Tester l'accès à la mémoire virtuelle mappée
    test_memory();
}

/* 
fn init_paging() -> PageDirectory {
    let mut page_directory = PageDirectory::new();
    let mut page_table = PageTable::new();

    // Mapper les page physiques a des pages virtuelles dans page_table

    for i in 0..ENTRIES_PER_TABLE {
        let address = (i * PageTable) as u32; // address physique
        page_table.entries[i] = address | 0x3; // present (bit0) + lecture/ecriture (bit1)
    }

    page_directory
}

*/
*/