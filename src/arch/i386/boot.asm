;https://www.gnu.org/software/grub/manual/multiboot2/html_node/boot_002eS.html#boot_002eS
section .multiboot_header
MULTIBOOT_MAGIC equ 0xe85250d6        ; Magic number for multiboot 2.
ARCHITECTURE    equ 0                 ; Protected mode i386 architecture.
SCREEN_BASE     equ 0xb8000           ; VGA Buffer address
header_start:
		dd MULTIBOOT_MAGIC            ; Magic.
		dd ARCHITECTURE               ; Architecture.
		dd header_end - header_start  ; Length.
		;; Checksum.
		dd 0x100000000 - (MULTIBOOT_MAGIC + ARCHITECTURE + (header_end - header_start))

		;; Insert tags here as requiered;

		;; End tag.
		dw 0                          ; Type.
		dw 0                          ; Flags.
		dd 8                          ; Size.
header_end:

; global page_directory_first_entry
; global page_table_first_entry
; page_directory_first_entry:
; 	resb 4096
; page_table_first_entry:
; 	resb 4096

global stack_top
global stack_bottom
section .bss
align 16
stack_bottom:
		resb 4096*4
stack_top:
;https://en.wikipedia.org/wiki/.bss
;Allocate some space for the stack since there is none yet

section .text
global start
extern rust_main
start:

	mov esp, stack_top               ; Enable the stack.

	; push arguments https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Boot-information-format
	push ebx ; address of Multiboot2 information structure
	;push eax ; magic value for MultiBoot2 should be 0x36d76289
	call rust_main
	mov dword [0xb8000], 0x4f524f45 ; "ER"
	mov dword [0xb8004], 0x4f3a4f52 ; "R:"
	mov dword [0xb8008], 0x4f204f20 ; "  "
	mov byte  [0xb800a], al
	;jmp kernel_hello
	hlt
kernel_hello:
	mov dword [0xb8000], 0x2f322f34
	hlt


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