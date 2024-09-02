MULTIBOOT_MAGIC equ 0xe85250d6        ; Magic number for multiboot 2.
ARCHITECTURE    equ 0                 ; Protected mode i386 architecture.
SCREEN_BASE     equ 0xb8000           ; VGA Buffer address
section .multiboot_header
header_start:
		dd MULTIBOOT_MAGIC            ; Magic.
		dd ARCHITECTURE               ; Architecture.
		dd header_end - header_start  ; Length.
		;; Checksum.
		dd 0x100000000 - (MULTIBOOT_MAGIC + ARCHITECTURE + (header_end - header_start))

		;; End tag.
		dw 0                          ; Type.
		dw 0                          ; Flags.
		dd 8                          ; Size.
header_end:

section .bss

global page_directory_first_entry
global page_table_first_entry
page_directory_first_entry:
	resb 4096
page_table_first_entry:
	resb 4096
global stack_bottom
global stack_top
stack_bottom:
		resb 4096*4
stack_top:

section .text
global start
extern rust_main
start:

	mov esp, stack_top               ; Use our stack.

	; push arguments https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Boot-information-format
	push ebx ; address of Multiboot2 information structure
	;push eax ; magic value for MultiBoot2 should be 0x36d76289
	call rust_main
	;jmp kernel_hello    
	hlt
kernel_hello:
	mov dword [0xb8000], 0x2f322f34
	hlt

global gdtflush
gdtflush : 
	mov eax, [esp + 4]
	lgdt [eax]
	mov eax, 0x10
	mov ds, ax ; kdata segment

	mov eax, 0x18
	mov ss, ax ; kstack segment

	mov eax, 0x20
	mov es, ax ; ucode segment

	mov eax, 0x28
	mov fs, ax ; udata segment

	mov eax, 0x30
	mov gs, ax ; ustack segment

	jmp 0x08:.flush ; Set CS, kcode segment
.flush :
	ret

global tssflush
tssflush: 
	mov ax, 0x38
	ltr ax
	ret

global loadpagedirectory
loadpagedirectory:
    push ebp
    mov ebp, esp
    mov eax, [ebp+8]  ; Get the page directory address from the stack
    mov cr3, eax      ; Load the page directory address into CR3
    mov esp, ebp
    pop ebp
    ret

global enablepaging
enablepaging : 
    push ebp
    mov ebp, esp
    mov eax, cr0
    or eax, 0x80000000  ; Set the paging bit in CR0
    mov cr0, eax
    mov esp, ebp
    pop ebp
    ret

