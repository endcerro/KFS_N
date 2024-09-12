
ASM_SRC = src/arch/i686
SRC = src/
all : boot_basic

# configure  :
# 	rustup toolchain install nightly --allow-downgrade
# 	rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

kvm : iso_basic
	@echo Starting with KVM
	@kvm -name kfs -cdrom ./os.iso -boot c
qemu : iso_basic
	@echo Staring with qemu
	@qemu-system-x86_64 -cdrom os.iso -m 7M

qemu_dbg : iso_basic
	@echo Staring with qemu in debug mode
	@qemu-system-x86_64 -cdrom os.iso -s -S

dbg : 
	gdb "isofiles/boot/kernel.bin" -ex "1234"

rust_files :
	@echo "Building rust"
	@cargo build --target src/arch/i686/i686-unknown-none.json

asm_files :
	@mkdir -p obj
	@echo "Building ASM"
	@nasm -f elf32 $(ASM_SRC)/boot.asm -o obj/boot.o
	@nasm -f elf32 $(ASM_SRC)/interrupt.asm -o obj/interrupt.o

kernel_basic : asm_files rust_files
	@echo "Linking kernel"
	@ld -m elf_i386 -z noexecstack -o obj/kernel.bin -T $(ASM_SRC)/linker.ld obj/*.o target/i686-unknown-none/debug/libkfs_1.a

iso_basic : kernel_basic
	@mkdir -p isofiles
	@mkdir -p isofiles/boot
	@mkdir -p isofiles/boot/grub
	@cp $(SRC)/grub/grub.cfg isofiles/boot/grub
	@cp obj/kernel.bin isofiles/boot/kernel.bin
	@echo "Making ISO"
	@grub-mkrescue -d /usr/lib/grub/i386-pc -o os.iso isofiles 2> /dev/null

boot_basic : qemu

clean :
	@cargo clean
	@rm -rf obj
	@rm -rf isofiles
	@rm -rf os.iso