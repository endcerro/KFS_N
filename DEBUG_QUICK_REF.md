# Quick Reference Card - i386 Kernel Debugging

## VSCode Shortcuts
| Key | Action |
|-----|--------|
| `F5` | Start Debugging |
| `F9` | Toggle Breakpoint |
| `F10` | Step Over |
| `F11` | Step Into |
| `Shift+F11` | Step Out |
| `Ctrl+Shift+B` | Build |
| `Ctrl+Shift+F5` | Restart Debugging |
| `Shift+F5` | Stop Debugging |

## GDB Commands
| Command | Description |
|---------|-------------|
| `c` or `continue` | Continue execution |
| `n` or `next` | Next line (step over) |
| `s` or `step` | Step into function |
| `si` | Step one instruction |
| `ni` | Next instruction (step over) |
| `bt` or `backtrace` | Show stack trace |
| `info registers` | Show all registers |
| `x/10xw $esp` | Examine 10 words at stack |
| `break rust_main` | Set breakpoint |
| `delete` | Delete all breakpoints |
| `print $eax` | Print register value |

## Custom GDB Commands
| Command | Description |
|---------|-------------|
| `show-kernel-state` | Complete CPU/memory state |
| `show-paging` | Paging status |
| `show-control-regs` | CR0-CR4 + EFLAGS |
| `show-segments` | All segment registers |
| `vtophys 0xC0000000` | Translate virt→phys |
| `decode-pte 0x123` | Decode page table entry |

## Memory Layout
```
Virtual:
0x00000000 - User space (not implemented)
0xC0000000 - Kernel start
0xC0000800 - GDT location

Physical:
0x00100000 - Kernel load (1MB)
```

## Important Addresses
```gdb
# Page directory
x/256xw $cr3

# GDT
x/8gx 0xC0000800

# Kernel entry
break *0xC0100000

# Stack top
print/x $esp
```

## QEMU Monitor (Ctrl+Alt+2)
```
info registers    # All registers
info mem         # Page mappings  
info tlb         # TLB entries
info pic         # PIC state
```

## Common Issues
| Problem | Solution |
|---------|----------|
| Triple fault | `qemu-system-i386 -d int,cpu_reset` |
| No symbols | Check `obj/kernel.bin` exists |
| Can't connect | QEMU must be running with `-s -S` |
| Page fault | Check `$cr2` for fault address |
