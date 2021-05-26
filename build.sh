#!/usr/bin/env bash

set -e
set -x

# nice "hack" which make the script work, even if not executed from "./"
DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

# corresponds to x86-none-bare_metal-multiboot2.json without file extension
TARGET_NAME="x86-none-bare_metal-multiboot2";

# destination directory
BUILD_DIR="./build"
# final path/name of multiboot2 rust binary
FINAL_MULTIBOOT2_ELF="${BUILD_DIR}/multiboot2-rust-binary.elf"

rm -rf "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}"

# rust standalone binary to ELF
cargo build

# multiboot2 header to ELF
# nasm -f elf64 multiboot2_header.asm -o ./build/multiboot2_header.o
nasm -f elf32 "multiboot2_header.asm" -o "${BUILD_DIR}/multiboot2_header.o"
nasm -f elf32 "start.asm" -o "${BUILD_DIR}/start.o"

# link both together using the linker script (multiboot header will be first in binary)
# note that we want to build a 32bit elf file with 32 bit code
ld -n -o "${FINAL_MULTIBOOT2_ELF}" \
  -T linker.ld \
  -m elf_i386 \
  "${BUILD_DIR}/multiboot2_header.o" \
  "${BUILD_DIR}/start.o" \
  "target/${TARGET_NAME}/debug/rust-multiboot-binary"
# btw: valid options for "-m" can be found via "ld --verbose"
# e.g.   elf_x86_64, elf32_x86_64, elf_i386, ...

# check if grub could boot the file as multiboot2
grub-file --is-x86-multiboot2 "${FINAL_MULTIBOOT2_ELF}"
