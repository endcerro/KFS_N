# GDB initialization script for kernel debugging

# Tell GDB that we're working with a kernel
set architecture i386
set disassembly-flavor intel

# Disable pagination and set a reasonable print limit
set pagination off
set print pretty on
set print array-indexes on
set print elements 100

# Don't stop on SIGSEGV, as this is common during kernel development
# handle SIGSEGV nostop noprint pass

# Load symbols from the kernel binary
file isofiles/boot/kernel.bin

# Connect to QEMU's GDB stub (assuming QEMU is running with -s option)
target remote localhost:1234

# Prefer Rust source when available
set language rust
set disassemble-next-line auto

# Set a breakpoint at the kernel entry point
break start

# Set a breakpoint at the Rust entry point
break rust_main

# Useful functions for kernel debugging
define dump_page_table
    set $pde = $arg0
    set $i = 0
    while $i < 1024
        set $pte = *($pde + 4 * $i)
        if $pte & 1
            printf "PDE[%d] = %08X: ", $i, $pte
            if $pte & 0x80
                printf "PS "
            end
            if $pte & 4
                printf "U "
            else
                printf "S "
            end
            if $pte & 2
                printf "RW"
            else
                printf "RO"
            end
            printf "\n"
        end
        set $i = $i + 1
    end
end
directory src/

# Continue execution
continue