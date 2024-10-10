; Add these to your existing global declarations
global setup_paging
extern higher_half_start

section .text

setup_paging:
    ; Initialize page directory
    mov edi, page_directory
    mov ecx, 1024                ; 1024 entries in page directory
    xor eax, eax                 ; Clear eax
    rep stosd                    ; Fill page directory with zeros

    ; Set up identity mapping for first 4MB
    mov dword [page_directory], page_table1 + 3  ; Present + R/W
    mov dword [page_directory + 4], page_table2 + 3  ; Present + R/W

    
    ; Set up page table (identity map first 4MB)
    mov edi, page_table1
    mov eax, 3                   ; Present + R/W
    mov ecx, 1024                ; 1024 entries in page table

.set_entry1:
    stosd
    add eax, 4096                ; Next page (4KB)
    loop .set_entry1

    ; Set up second page table (identity map second 4MB)
    mov edi, page_table2
    mov ecx, 1024  
   .set_entry2:
    stosd
    add eax, 4096                ; Next page (4KB)
    loop .set_entry2

    ; Load page directory
    mov eax, page_directory
    mov cr3, eax

    ; Enable paging
    mov eax, cr0
    or eax, 0x80000000           ; Set PG bit
    mov cr0, eax

    ; Set up stack

    ; Jump to higher half start
    jmp higher_half_start

    ; Halt the CPU
    cli
.halt:
    hlt
    jmp .halt
global page_directory
global identity_page_table
global higher_half_page_table
section .bss
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