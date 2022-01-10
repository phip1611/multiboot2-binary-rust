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
FINAL_ELF="${BUILD_DIR}/multiboot2-kernel_x86_64.elf"

function fn_main() {
  ./src/rust/build.sh
  fn_prepare_build_dir
  fn_copy_bin
  fn_test_grub_multiboot2
}

function fn_prepare_build_dir() {
  rm -rf "${BUILD_DIR}"
  mkdir -p "${BUILD_DIR}"
}

function fn_copy_bin() {
  # symlink doesn't work, when GRUB makes a standalone image
  # ln -s "$(pwd)/src/rust/rust-kernel/target/x86_64-none-bare_metal/debug/rust-multiboot2-64-bit-kernel" \
  # "${BUILD_DIR}/multiboot2-kernel_x86_64.elf"
  cp "$(pwd)/src/rust/kernel-bin/target/x86_64-none-bare_metal/debug/kernel-bin" \
   "${BUILD_DIR}/multiboot2-kernel_x86_64.elf"
}

function fn_test_grub_multiboot2 {
  # check if grub could boot the file as multiboot2
  grub-file --is-x86-multiboot2 "$FINAL_ELF"
}

# invoke main function
fn_main
