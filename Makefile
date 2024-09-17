# Directories
ASM_SRC := src/arch/i686
SRC := src
OBJ_DIR := obj
ISO_DIR := isofiles
TARGET_DIR := target/i686-unknown-none/debug

# Files
KERNEL_BIN := $(OBJ_DIR)/kernel.bin
ISO_FILE := os.iso
RUST_LIB := $(TARGET_DIR)/libkfs_1.a

# Commands
RUSTC := cargo build --target $(ASM_SRC)/i686-unknown-none.json
NASM := nasm -f elf32
LD := ld -m elf_i386 -z noexecstack
GRUB_MKRESCUE := grub-mkrescue -d /usr/lib/grub/i386-pc
QEMU := qemu-system-i386
KVM := kvm

# Flags
RUST_FLAGS := #--features gdt_test --features verbose

# Phony targets
.PHONY: all kvm qemu qemu_dbg dbg rust_dbg clean

all: $(ISO_FILE)

kvm: $(ISO_FILE)
	@echo "Starting with KVM"
	$(KVM) -name kfs -cdrom $(ISO_FILE) -boot c

qemu: $(ISO_FILE)
	@echo "Starting with QEMU"
	$(QEMU) -cdrom $(ISO_FILE) -m 1G -serial stdio

qemu_dbg: $(ISO_FILE)
	@echo "Starting with QEMU in debug mode"
	$(QEMU) -cdrom $(ISO_FILE) -s -S -serial stdio

dbg:
	gdb "$(ISO_DIR)/boot/kernel.bin" -ex "1234"

rust_dbg: $(KERNEL_BIN)
	$(QEMU) -cdrom $(ISO_FILE) -serial stdio

# Build rules
$(OBJ_DIR)/boot.o: $(ASM_SRC)/boot.asm | $(OBJ_DIR)
	@echo "Building ASM"
	$(NASM) $< -o $@

$(RUST_LIB):
	@echo "Building Rust"
	$(RUSTC) $(RUST_FLAGS)

$(KERNEL_BIN): $(OBJ_DIR)/boot.o $(RUST_LIB) | $(OBJ_DIR)
	@echo "Linking kernel"
	$(LD) -o $@ -T $(ASM_SRC)/linker.ld $^

$(ISO_FILE): $(KERNEL_BIN) | $(ISO_DIR)/boot/grub
	@echo "Making ISO"
	cp $(SRC)/grub/grub.cfg $(ISO_DIR)/boot/grub
	cp $(KERNEL_BIN) $(ISO_DIR)/boot/kernel.bin
	$(GRUB_MKRESCUE) -o $@ $(ISO_DIR) 2> /dev/null

# Directory creation
$(OBJ_DIR) $(ISO_DIR)/boot/grub:
	mkdir -p $@

clean:
	cargo clean
	rm -rf $(OBJ_DIR) $(ISO_DIR) $(ISO_FILE)

# Run targets
run-bochs: $(ISO_FILE)
	bochs -q -f bochsrc.txt