; Add these to your existing global declarations
global stack_top
global stack_bottom
global setup_paging
global start
extern higher_half_start

section .boot

kernel_hello:
	mov dword [0xb8000], 0x4f524f45 ; "ER"
	mov dword [0xb8004], 0x4f3a4f52 ; "R:"
	mov dword [0xb8008], 0x4f204f20 ; "  "
	mov dword [0xb800a], 0x2f322f34
	hlt

start :
    mov esp, stack_top
    sub esp, 0xC0000000
	; mov esp, edx
    ; mov esp, stack_top

    ; Store GRUB parameters
    push ebx                    ; Multiboot structure - we'll need this later


    call setup_paging

    lea ecx, [.higher_half]    ; Get the physical address of higher_half label
    jmp ecx                    ; Jump to physical address

    ; jmp kernel_hello
    ; call rust_main

.higher_half:
    ; Now paging is enabled, switch to virtual addresses
    add esp, 0xC0000000
    ; Switch to virtual stack

    pop ebx                    ; Restore multiboot structure pointer
    push ebx                   ; Push it again for rust_main

    jmp higher_half_start      ; Jump to higher half kernel


setup_paging:
    ; Initialize page directory
    mov edi, page_directory
    sub edi, 0xC0000000

    push edi

    mov ecx, 1024                ; 1024 entries in page directory
    xor eax, eax                 ; Clear eax
    rep stosd                    ; Fill page directory with zeros

    pop edi


    ; Set up identity mapping for first 4MB
    ; mov edx, page_directory;
    ; sub edx, 0xC0000000
    mov eax, page_table1
    sub eax, 0xC0000000
    or eax, 3

    mov dword [edi], eax  ; Present + R/W
    mov dword [edi + 768 * 4], eax  ; Present + R/W

    mov eax, page_table2
    sub eax, 0xC0000000
    or eax, 3

    mov dword [edi + 4], eax  ; Present + R/W
    mov dword [edi + 769 * 4], eax  ; Present + R/W


    ; mov dword [edi + 4], page_table2 + 3  ; Present + R/W


    ; mov dword [edi + 768 * 4], page_table1 + 3  ; Present + R/W
    ; mov dword [edi + 769 * 4], page_table2 + 3  ; Present + R/W

    ; Set up page table (identity map first 4MB)
    mov edi, page_table1
    sub edi, 0xC0000000
    mov eax, 3                   ; Present + R/W
    mov ecx, 1024                ; 1024 entries in page table
.set_entry1:
    stosd
    add eax, 4096                ; Next page (4KB)
    loop .set_entry1

    ; Set up second page table (identity map second 4MB)
    mov edi, page_table2
    sub edi, 0xC0000000
    mov eax, 0x400003         ; Start at 4MB + flags
    mov ecx, 1024
.set_entry2:
    stosd
    add eax, 4096                ; Next page (4KB)
    
    loop .set_entry2

    ; Load page directory
    mov eax, page_directory
    sub eax, 0xC0000000
    mov cr3, eax
.enable_paging:
    ; Enable paging
    cli
    mov eax, cr0
    or eax, 0x80000000           ; Set PG bit
    mov cr0, eax
    ret

    ; Set up stack

    ; Jump to higher half start
    ; jmp higher_half_start

    ; Halt the CPU

.halt:
    hlt
    jmp .halt

global page_directory
global identity_page_table
global higher_half_page_table

section .bss

align 16
stack_bottom:
		resb 4096*4
stack_top:

align 4096
page_directory:
    resb 4096

align 4096
page_table1 :
    resb 4096

align 4096
page_table2 :
    resb 4096
; align 4096
; higher_half_page_table:
;     times 1024 dd 0
