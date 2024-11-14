# Directories
ASM_SRC := src/arch/i386
SRC := src
OBJ_DIR := obj
ISO_DIR := isofiles
TARGET_DIR := target/i386-unknown-none/debug

# Files
KERNEL_BIN := $(OBJ_DIR)/kernel.bin
ISO_FILE := os.iso
RUST_LIB := $(TARGET_DIR)/libkfs.a

# Find all Rust source files
RUST_SRCS := $(shell find $(SRC) -name '*.rs')

# Commands
RUSTC := cargo build --target $(ASM_SRC)/i386-unknown-none.json
NASM := nasm -g -f elf32
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
	$(KVM) -name kfs -cdrom $(ISO_FILE) -boot c -cpu host

qemu: $(ISO_FILE)
	@echo "Starting with QEMU"
	$(QEMU) -cdrom $(ISO_FILE) -m 8M -serial stdio

qemu_dbg: $(ISO_FILE)
	@echo "Starting with QEMU in debug mode"
	$(QEMU) -cdrom $(ISO_FILE) -s -S -serial file:/tmp/qemu_serial.log

dbg:
	$(QEMU) -cdrom $(ISO_FILE) -s -S -serial stdio &
	gdb -x gdb_init.gdb isofiles/kernel.bin
	pkill qemu

# Build rules
$(OBJ_DIR)/boot.o: $(ASM_SRC)/boot.asm | $(OBJ_DIR)
	@echo "Building ASM"
	$(NASM) $< -o $@

	# Build rules
$(OBJ_DIR)/bootstrap.o: $(ASM_SRC)/bootstrap.asm | $(OBJ_DIR)
	@echo "Building ASM"
	$(NASM) $< -o $@

$(RUST_LIB): $(RUST_SRCS) | $(TARGET_DIR)
	@echo "Building Rust"
	$(RUSTC) $(RUST_FLAGS)
	@touch $(RUST_LIB)

$(KERNEL_BIN): $(OBJ_DIR)/boot.o $(OBJ_DIR)/bootstrap.o $(RUST_LIB) | $(OBJ_DIR)
	@echo "Linking kernel"
	$(LD) -o $@ -T $(ASM_SRC)/linker.ld $^

$(ISO_FILE): $(KERNEL_BIN) | $(ISO_DIR)/boot/grub
	@echo "Making ISO"
	cp $(SRC)/grub/grub.cfg $(ISO_DIR)/boot/grub
	cp $(KERNEL_BIN) $(ISO_DIR)/boot/kernel.bin
	$(GRUB_MKRESCUE) -o $@ $(ISO_DIR) 2> /dev/null

# Directory creation
$(OBJ_DIR) $(ISO_DIR)/boot/grub $(TARGET_DIR):
	mkdir -p $@

clean:
	cargo clean -p kfs
	rm -rf $(OBJ_DIR) $(ISO_DIR) $(ISO_FILE)

fclean: clean
	cargo clean

re: fclean all

# Run targets
run-bochs: $(ISO_FILE)
	bochs -q -f bochsrc.txt