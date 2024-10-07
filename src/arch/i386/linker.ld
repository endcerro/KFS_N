/* https://wiki.osdev.org/Linker_Scripts */
/* Output format and architecture */
OUTPUT_FORMAT(elf32-i386)
OUTPUT_ARCH(i386)

/* Entry point of the kernel */
ENTRY(start)

/* Define the program headers we want in our kernel */
PHDRS
{
    boot    PT_LOAD FLAGS(5) ; /* Read + Execute */
    text    PT_LOAD FLAGS(5) ; /* Read + Execute */
    rodata  PT_LOAD FLAGS(4) ; /* Read only */
    data    PT_LOAD FLAGS(6) ; /* Read + Write */
}

/* Define the sections in our kernel */
SECTIONS {
    /* The kernel will be loaded at 1MB by GRUB */
    . = 1M;

    _kernel_start = .;

    /* First put the multiboot header, as it is required to be put very early
       in the image or the bootloader won't recognize the file format */
    .boot ALIGN(4K) : {
        KEEP(*(.multiboot_header))
        *(.boot .boot.*)
    } :boot

    _kernel_start = .;

    /* Then the text section */
    .text ALIGN(4K) : {
        *(.text .text.*)
    } :text

    /* Read-only data */
    .rodata ALIGN(4K) : {
        *(.rodata .rodata.*)
    } :rodata

    /* Read-write data (initialized) */
    .data ALIGN(4K) : {
        *(.data .data.*)
    } :data

    /* Read-write data (uninitialized) and stack */
    .bss ALIGN(4K) : {
        *(COMMON)
        *(.bss .bss.*)
    } :data

    /* Add a symbol that indicates the end address of the kernel */
    . = ALIGN(4K);
    _kernel_end = .;

    /* Debugging info */
    /DISCARD/ : { *(.comment) *(.eh_frame) *(.note.gnu.build-id) }
}