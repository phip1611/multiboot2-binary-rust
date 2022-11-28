# We need to apply certain Rust flags to the WHOLE build to enable the kernel code-model of the
# System V ABI. Otherwise we get stuck with ugly build problems during link time such as
# "relocation R_X86_64_32 out of range". See https://github.com/rust-lang/rust/issues/101209
#
# - code-model=kernel       : We want to link the kernel to the upper half of the address space
# - link-arg=-Tlink.l       : We want to use a custom linker script.
# - relocation-model=static : We want to produce an executable ELF and not a dynamic ELF.
#                             Bootloaders such as ELF can load executable ELF files easy into
#                             memory but do not support dynamic ELF files out of the box, as this
#                             requires additional policies in the bootloader.
export RUSTFLAGS=-C code-model=kernel -C link-arg=-Tlink.ld -C relocation-model=static

default: kernel

.PHONY: clean code_style default kernel kernel_build verify_elf

# TODO UN-COMMENT AS AS THE BOOT CODE WORKS!
kernel: | kernel_build # verify_elf

kernel_build: | code_style
	# I build both so that I can regularly switch. The debug build is useful as it has debug
	# symbols. This accelerates debugging when something fails .
	cargo build
	cargo build --release

code_style:
	cargo fmt
	# cargo fmt -- --check
	# cargo clippy

clean:
	cargo clean

# Checks that the ELF file contains the correct sections in the correct segments with the right
# virtual addresses (link addresses) and the right physical addresses (load addresses).
# This may change from time to time as sections get bigger. However, it helps to find bugs
# when making changes to the linker script, such as a missing section.
verify_elf: | kernel_build
	KERNEL_BINARY=../../target/x86_64-unknown-none/release/kernel-bin test/check_kernel_elf.sh
