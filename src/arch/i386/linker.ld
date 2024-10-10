/* https://wiki.osdev.org/Linker_Scripts */
/* Output format and architecture */
OUTPUT_FORMAT(elf32-i386)
OUTPUT_ARCH(i386)

/* Entry point of the kernel */
ENTRY(start)

KERNEL_VMA = 0xC0000000;
KERNEL_LMA = 0x00100000;  /* Typically 1MB */

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
    . = KERNEL_LMA;

    _kernel_start = .;

    /* First put the multiboot header, as it is required to be put very early
       in the image or the bootloader won't recognize the file format */
    .boot ALIGN(4K) : {
        KEEP(*(.multiboot_header))
        *(.boot .boot.*)
    } :boot

    /* . += KERNEL_VMA; */

    _kernel_start = .;

    /* Then the text section */
    .text ALIGN(4K) : /*AT(ADDR(.text) - KERNEL_VMA)*/ {
        *(.text .text.*)
    } :text

    /* Read-only data */
    .rodata ALIGN(4K) : /*AT(ADDR(.rodata) - KERNEL_VMA)*/ {
        *(.rodata .rodata.*)
    } :rodata

    /* Read-write data (initialized) */
    .data ALIGN(4K) : /*AT(ADDR(.data) - KERNEL_VMA)*/ {
        *(.data .data.*)
    } :data

    /* Read-write data (uninitialized) and stack */
    .bss ALIGN(4K) : /*AT(ADDR(.bss) - KERNEL_VMA)*/ {
        *(COMMON)
        *(.bss .bss.*)
    } :data

    /* Add a symbol that indicates the end address of the kernel */
    . = ALIGN(4K);
    _kernel_end = .;

    /* Debugging info */
    /DISCARD/ : { *(.comment) *(.eh_frame) *(.note.gnu.build-id) }
}