/* https://wiki.osdev.org/Linker_Scripts */
OUTPUT_FORMAT(elf32-i386)
OUTPUT_ARCH(i386)
ENTRY(start)

SECTIONS {
	. = 1M;          /* the code should be loaded at 1 MB since grub and multiboot will be present here */
	_kernel_start = .;
	.multiboot_header ALIGN(4K) :
	{
		KEEP(*(.multiboot_header))
	}

	.text ALIGN (4K) :   /* align at 4 KB */
	{
		_TEXT_START_ = .;
		*(.text*)            /* all text sections from all files */
		_TEXT_END_ = .;
	}

	.data ALIGN (4K) :    /* align at 4 KB */
	{
		_DATA_START_ = .;
		common_data_sep = .;
		*(.data*)              /* all data sections from all files */
		_DATA_END_ = .;
	}

	.bss ALIGN (4K) :    /* align at 4 KB */
	{
		_BSS_START_ = .;
		*(COMMON)            /* all COMMON sections from all files */
		_BSS_SEP_ = .;
		*(.bss*)              /* all bss sections from all files */
		_BSS_END_ = .;
	}
	. = ALIGN(4K);
	_kernel_end = .;
}