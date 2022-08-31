default: kernel

.PHONY: clean code_style default kernel run

kernel: | code_style
	$(MAKE) -C src/kernel-bin
	cp "target/x86_64-unknown-none/release/kernel-bin" "build/multiboot2-kernel_x86_64.elf"
	grub-file --is-x86-multiboot2 "build/multiboot2-kernel_x86_64.elf"

code_style:
	$(MAKE) -C src/kernel-lib code_style
	$(MAKE) -C src/kernel-bin code_style

clean:
	$(MAKE) -C src/kernel-lib clean
	$(MAKE) -C src/kernel-bin clean

run:
	./.run_qemu.sh
