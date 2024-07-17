
ASM_SRC = src/arch/i386
SRC = src/
all : boot_basic

# configure  :
# 	rustup toolchain install nightly --allow-downgrade
# 	rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

kvm : iso_basic
	kvm -name kfs -m 8000 -cdrom ./os.iso -boot c
qemu : iso_basic
	qemu-system-x86_64 -cdrom os.iso

rust_files :
	cargo build 

asm_files :
	mkdir -p obj
	nasm -f elf32 $(ASM_SRC)/boot.asm -o obj/boot.o

kernel_basic : asm_files rust_files
	ld -m elf_i386 -o obj/kernel.bin -T $(ASM_SRC)/linker.ld obj/boot.o target/i686-unknown-none/debug/libkfs_1.a

iso_basic : kernel_basic
	mkdir -p isofiles
	mkdir -p isofiles/boot
	mkdir -p isofiles/boot/grub
	cp $(SRC)/grub/grub.cfg isofiles/boot/grub
	cp obj/kernel.bin isofiles/boot/kernel.bin
	grub-mkrescue -d /usr/lib/grub/i386-pc -o os.iso isofiles

boot_basic : qemu

clean : 
	cargo clean
	rm -rf obj
	rm -rf isofiles
	rm -rf os.iso