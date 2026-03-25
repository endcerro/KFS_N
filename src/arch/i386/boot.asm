; =============================================================================
; boot.asm - Higher-half entry point and multiboot2 header
;
; This file contains:
;	- The multiboot2 header (must be early in the binary for GRUB to find it)
;	- higher_half_start - called after paging is enabled, goes to Rust
;	- clear_page1 - removes the identity mapping once we're safely in the
;		higher half
;	- early_panic - minimal VGA error display for pre-Rust failures
;
; At the point higher_half_start runs:
;	- Paging is ON
;	- ESP points to the virtual kernel stack
;	- The multiboot pointer is stored in the global `multiboot_ptr`
;	- Identity map (PDE[0]) is still active (cleared by Rust)
; =============================================================================

; ---- Imports ----
extern rust_main 			; Rust kernel entry point (lib.rs)
extern page_directory		; Page directory (bootstrap.asm, .bss)

; ---- Exports ----
global higher_half_start	; Called by bootstrap.asm after enabling paging
global clear_page1			; Called by Rust to remove the identity map
global early_panic			; Emergency VGA erro


; =============================================================================
; Multiboot2 header
; =============================================================================
section .boot.multiboot2header

MULTIBOOT2_MAGIC equ 0xe85250d6			 ; Multiboot2 magic number
ARCHITECTURE	 equ 0					 ; 0 = 32-bit protected mode (i386)

header_start:
	dd MULTIBOOT2_MAGIC
	dd ARCHITECTURE
	dd header_end - header_start		 ; Header length
	dd 0x100000000 - (MULTIBOOT2_MAGIC + ARCHITECTURE + (header_end - header_start))

	; -- End tag (type=0, flags=0, size=8) --
	; Required terminator; additional tags (framebuffer, module alignment,
	; etc.) can be inserted before this if needed in the future.
	dw 0 ; Type
	dw 0 ; Flags
	dd 8 ; Size
header_end:


; =============================================================================
; .boot section - code that may run before paging or during early boot
; =============================================================================
section .boot

; -----------------------------------------------------------------------------
; early_panic - Last-resort error display
;
; Writes "ERR:" to the top-left of the VGA text buffer in white-on-red,
; then halts.  Useful for debugging failures before the Rust panic handler
; is available.  No stack or paging required.
; -----------------------------------------------------------------------------
early_panic:
	mov dword [0xb8000], 0x4f524f45		; "ER" (white on red)
	mov dword [0xb8004], 0x4f3a4f52		; "R:"
	mov dword [0xb8008], 0x4f204f20		; "  "
	cli
	hlt


; =============================================================================
; .text section - runs in the higher half (virtual addresses >= 0xC0000000)
; =============================================================================
section .text

; -----------------------------------------------------------------------------
; higher_half_start - Bridge from assembly to Rust
;
; Called by bootstrap.asm after paging is enabled and ESP is virtual.
; Calls rust_main() which takes no arguments - the multiboot pointer is
; read from the `multiboot_ptr` global instead.
; -----------------------------------------------------------------------------
higher_half_start:
	call rust_main

	; rust_main should never return, but if it does, halt cleanly.
	cli
	hlt


; -----------------------------------------------------------------------------
; clear_page1 - Remove the identity mapping (PDE[0])
;
; After boot, PDE[0] maps virtual 0x00000000–0x003FFFFF to physical memory.
; This is only needed during the paging transition; once we're firmly in the
; higher half, we clear it so null pointer dereferences and other accesses
; to low memory correctly page-fault.
;
; Called from Rust: extern "C" { fn clear_page1(); }
; -----------------------------------------------------------------------------
clear_page1:
	mov edi, page_directory				  ; Virtual address (paging is on)
	mov dword [edi], 0						 ; Clear PDE[0]

	; Flush TLB so the CPU stops using the stale identity mapping.
	; Reloading CR3 invalidates all TLB entries.
	mov eax, cr3
	mov cr3, eax

	ret