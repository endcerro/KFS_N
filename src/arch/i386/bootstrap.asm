; =============================================================================
; bootstrap.asm - Kernel entry point (runs in physical address space)
;
; This is the very first code that executes after GRUB hands off control.
; At entry, we are in 32-bit protected mode with paging OFF:
;   - EAX = multiboot2 magic (0x36d76289)
;   - EBX = physical address of multiboot2 info structure
;   - No valid stack
;   - No paging
;
; We need to:
;   - Set up a temporary physical stack
;   - Save the multiboot pointer for later use by Rust
;   - Set up identity + higher-half page tables
;   - Enable paging
;   - Jump to the higher-half virtual address and hand off to boot.asm
;
; All symbols defined in .bss (page_directory, page_table1, stack) have
; virtual addresses (>= 0xC0000000) at link time, so we must subtract
; KERNEL_VBASE when accessing them before paging is enabled.
; =============================================================================

; ---- Constants ----
KERNEL_VBASE equ 0xC0000000              ; Higher-half base virtual address
PDE_KERNEL_INDEX equ (KERNEL_VBASE >> 22) ; = 768, PDE slot for 0xC0000000
PTE_FLAGS equ 0x3                         ; Present + Read/Write

; ---- Exports ----
global start                ; Entry point (referenced by linker)
global stack_top            ; Top of kernel stack (used by GDT/TSS)
global stack_bottom         ; Bottom of kernel stack
global page_directory       ; Page directory (used by Rust memory subsystem)
global page_table1          ; Boot page table (maps first 4 MB)
global setup_paging         ; Called from start, also exported for reference
global multiboot_ptr        ; Multiboot2 info pointer for use in rust

; ---- Imports ----
extern higher_half_start    ; Defined in boot.asm - runs in virtual space


; =============================================================================
; .boot section - executes before paging, linked at low physical addresses
; =============================================================================
section .boot

; -----------------------------------------------------------------------------
; start - Kernel entry point
;
; Sets up a physical stack, saves the multiboot pointer into a global
; variable, enables paging, then jumps to the higher-half entry.
; -----------------------------------------------------------------------------
start:
	; --- Step 1: Set up a temporary stack using physical addresses ---
	; stack_top is a virtual address (>= 0xC0000000), convert to physical
	mov esp, stack_top
	sub esp, KERNEL_VBASE

	; --- Step 2: Save multiboot2 info pointer ---
	; GRUB passes it in EBX as a physical address.
	; We'll convert to virtual now - only used after paging is on.
	; Store in a global so Rust can read it via extern "C"
	add ebx, KERNEL_VBASE
	mov dword [multiboot_ptr - KERNEL_VBASE], ebx

	; --- Step 3: Set up paging (identity map + higher-half) ---
	call setup_paging

	; --- Step 4: Jump to higher-half code ---
	; Paging is now enabled
	; Both identity mapping (PDE[0]) and higher-half mapping (PDE[768])
	; point to the same physical memory right now, so we can execute at
	; either address. Load the virtual address of th higher-half trampoline and jump there.
	lea ecx, [.trampoline_to_higher_half]
	jmp ecx

.trampoline_to_higher_half:
	; Now executing at a virtual address (>= 0xC0000000).
	; Fix the stack pointer to use the virtual address.
	add esp, KERNEL_VBASE

	; Hand off to boot.asm which sets up the rest and calls rust_main.
	jmp higher_half_start


; -----------------------------------------------------------------------------
; setup_paging - Identity-map and higher-half-map the first 4 MB
;
; Creates a minimal page directory with two entries:
;   PDE[0]   → page_table1 (identity map:   0x00000000–0x003FFFFF)
;   PDE[768] → page_table1 (higher-half:    0xC0000000–0xC03FFFFF)
;
; Both point to the same page table, which maps physical 0–4 MB linearly.
; The identity map is needed so the `ret` after enabling paging doesn't
; fault; it will be cleared later by clear_page1() in boot.asm.
; -----------------------------------------------------------------------------
setup_paging:
	; -- Prepare the page directory --
	mov edi, page_directory
	sub edi, KERNEL_VBASE               ; Convert to physical address
	push edi                            ; Save PD physical base for later

	mov ecx, 1024                       ; 1024 entries × 4 bytes = 4 KB
	xor eax, eax
	rep stosd                           ; Fill with zeros (all not-present)

	pop edi                             ; EDI = page_directory physical address

	; -- Point PDE[0] and PDE[768] at page_table1 --
	mov eax, page_table1
	sub eax, KERNEL_VBASE               ; Physical address of page_table1
	or  eax, PTE_FLAGS                  ; Present + Read/Write

	mov dword [edi], eax                ; PDE[0]   → identity map
	mov dword [edi + PDE_KERNEL_INDEX * 4], eax  ; PDE[768] → higher-half

	; -- Fill page_table1: linearly map physical 0x00000000–0x003FFFFF --
	mov edi, page_table1
	sub edi, KERNEL_VBASE               ; Physical address of page_table1
	mov eax, PTE_FLAGS                  ; Start at phys 0x00000000 + flags
	mov ecx, 1024                       ; 1024 pages × 4 KB = 4 MB

.fill_page_table:
	stosd                               ; Write PTE, advance EDI by 4
	add eax, 4096                       ; Next 4 KB page frame
	loop .fill_page_table

	; -- Load page directory into CR3 and enable paging --
	mov eax, page_directory
	sub eax, KERNEL_VBASE
	mov cr3, eax

	cli                                 ; Disable interrupts during mode switch
	mov eax, cr0
	or  eax, 0x80000000                 ; Set PG (paging enable) bit
	mov cr0, eax

	; Paging is now ON.  The next instruction fetch uses the identity map
	; (we're still at a low physical address), which is why PDE[0] exists.
	ret


; =============================================================================
; .bss section - Zeroed at load globals
; =============================================================================
section .bss

; Multiboot2 info structure pointer (virtual address).
; Written by start (pre-paging), read by Rust via extern "C".
alignb 4
multiboot_ptr:
	resd 1

; Kernel stack - 16 KB (4 pages).  Grows downward: stack_bottom is the
; lowest address, stack_top is where ESP starts.
alignb 16
stack_bottom:
	resb 4096 * 4
stack_top:

; Page directory - 1024 entries × 4 bytes = 4 KB, must be 4 KB alignbed.
alignb 4096
page_directory:
	resb 4096

; Boot page table - maps the first 4 MB of physical memory.
; Shared by PDE[0] (identity) and PDE[768] (higher-half) during boot.
alignb 4096
page_table1:
	resb 4096