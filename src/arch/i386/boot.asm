;https://www.gnu.org/software/grub/manual/multiboot2/html_node/boot_002eS.html#boot_002eS
extern rust_main
extern boot
extern page_directory
global higher_half_start

section .boot.multboot2header
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

section .boot
extern setup_paging

; global kernel_hello
kernel_hello:
	mov dword [0xb8000], 0x4f524f45 ; "ER"
	mov dword [0xb8004], 0x4f3a4f52 ; "R:"
	mov dword [0xb8008], 0x4f204f20 ; "  "
	mov dword [0xb800a], 0x2f322f34
	hlt
section .text

global clear_page1
clear_page1 :

	mov edi, page_directory     ; Using virtual address now
    mov dword [edi], 0          ; Clear first PDE (0-4MB)
    ; mov dword [edi + 4], 0      ; Clear second PDE (4-8MB)

	mov eax, cr3
    mov cr3, eax
	ret

higher_half_start:

	push eax ; magic value for MultiBoot2 should be 0x36d76289
	push ebx ; address of Multiboot2 information structure
	call rust_main
	; jmp kernel_hello
	hlt
