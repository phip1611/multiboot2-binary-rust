#!/usr/bin/env bash

# Builds everything inside "src"-directory.
# Uses "./build"-directory for intermediate and final artefacts.

set -e
set -x

# nice "hack" which make the script work, even if not executed from "./"
DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

# destination directory
BUILD_DIR="./build"
FINAL_ELF="${BUILD_DIR}/multiboot2-binary.elf"

RUST_64_BIT_BIN="src/rust/rust-64-bit/target/x86_64-none-bare_metal/debug/librust_multiboot2_64_bit_kernel.a"

function fn_main() {
  ./src/rust/build.sh
  fn_prepare_build_dir
  fn_compile_nasm
  fn_link_final_elf
  fn_test_grub_multiboot2
}

function fn_prepare_build_dir() {
  rm -rf "${BUILD_DIR}"
  mkdir -p "${BUILD_DIR}"
}

# compiles all assembler files
function fn_compile_nasm() {
  # btw: for multiboot2_header.asm it's irrelevant, whether it is compiled as elf32 or elf64 (it doesn't contain code)
  nasm -f elf64 "src/multiboot2_header.asm" -o "${BUILD_DIR}/multiboot2_header.o"
  nasm -f elf64 "src/start.asm" -o "${BUILD_DIR}/start.o"
}


# Builds the final ELF64-x86_64, that is multiboot2-compatible and contains
# the code from all object files using GNU ld.
function fn_link_final_elf() {
  # Link all object files together using the linker script.
  ld -n \
    -o "$FINAL_ELF" \
    -T "src/linker.ld" \
    -m elf_x86_64 \
    "${BUILD_DIR}/multiboot2_header.o" \
    "${BUILD_DIR}/start.o" \
    "${RUST_64_BIT_BIN}"

  # btw: valid options for "-m" can be found via "ld --verbose"
  # e.g.   elf_x86_64, elf32_x86_64, elf_i386, ...
}

function fn_test_grub_multiboot2 {
  # check if grub could boot the file as multiboot2
  grub-file --is-x86-multiboot2 "$FINAL_ELF"
}

# invoke main function
fn_main
