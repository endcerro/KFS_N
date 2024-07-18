ENTRY(start)                /* the name of the entry label */

SECTIONS {
	. = 1M;          /* the code should be loaded at 1 MB */
	kernel_start = .;
	.text ALIGN (4K) :   /* align at 4 KB */
	{
		section_text_start = .;
		KEEP(*(.multiboot_header))
		*(.text*)             /* all text sections from all files */
		section_text_end = .;
	}

	.bss ALIGN (4K) :    /* align at 4 KB */
	{
		section_bss_start = .;
		*(COMMON)            /* all COMMON sections from all files */
		common_bss_sep = .;
		*(.bss)              /* all bss sections from all files */
		section_bss_end = .;
	}
	kernel_end = .;

	
}