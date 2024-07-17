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
	push eax ; magic value for MultiBoot2 should be 0x36d76289
    call rust_main
;     jmp kernel_hello    
    hlt
kernel_hello:
    mov dword [0xb8000], 0x2f322f34
    hlt

global gdtflush
gdtflush : 
    mov eax, [esp + 4]
    lgdt [eax]
    mov eax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    jmp 0x08:.flush
.flush :
    ret
