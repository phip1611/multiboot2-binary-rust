#!/usr/bin/env bash

set -e
set -x

#########################################################################
# nice "hack" which make the script work, even if not executed from "./"
DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit
#########################################################################

# change this to fit your environment; it requires a valid EDK2 environment
# and requires the "OVMF"-files to be built.
EDK2_PATH="../edk2"
OVMF_BUILD_ARTIFACT_PATH="${EDK2_PATH}/Build/OvmfX64/DEBUG_GCC5/FV"
OVMF_FW_PATH="${OVMF_BUILD_ARTIFACT_PATH}/OVMF_CODE.fd"
OVMF_VARS_PATH="${OVMF_BUILD_ARTIFACT_PATH}/OVMF_VARS.fd"

# this directory contains the volumes for QEMU testing
# + additional config files for grub and more (if necessary)
QEMU_DIR="./qemu"
QEMU_VOLUME_DIR="${QEMU_DIR}/.vm-volume"

BUILD_DIR="./build"
FINAL_ELF="${BUILD_DIR}/multiboot2-binary.elf"

fn_main() {
  rm -rf "${QEMU_VOLUME_DIR}"

  fn_prepare_grub_installation

  fn_start_qemu
}

# Function: Starts QEMU with proper parameters (e.g. local directories will be mapped as volumes into the VM).
fn_start_qemu() {
  QEMU_ARGS=(
          # Disable default devices
          # QEMU by default enables a ton of devices which slow down boot.
          "-nodefaults"

          # Use a standard VGA for graphics
          "-vga"
          "std"

          # Use a modern machine, with acceleration if possible.
          "-machine"
          # "q35" # also works, but slower
          # Interesting to see how this changes CPU-ID
          # Without KVM the Hypervisor is QEMU, else its KVM
          "q35,accel=kvm:tcg"

          # Allocate some memory
          "-m"
          "128M"

          # Set up OVMF
          "-drive"
          "if=pflash,format=raw,readonly,file=${OVMF_FW_PATH}"
          "-drive"
          "if=pflash,format=raw,file=${OVMF_VARS_PATH}"

          # Mount a local directory as a FAT partition
          "-drive"
          "format=raw,file=fat:rw:${QEMU_VOLUME_DIR}"

          # Enable serial
          #
          # Connect the serial port to the host. OVMF is kind enough to connect
          # the UEFI stdout and stdin to that port too.
          "-serial"
          "stdio"

          # https://qemu-project.gitlab.io/qemu/system/invocation.html
          # using this, the program can write to X86 I/O port 0xe9 and talk
          # to qemu => debug output
          "-debugcon"
          # or "/dev/stdout" => it appears in terminal window
          # this is poorly documented! I found out by coincidence, that I can use a file like this
          "file:qemu/debugcon.txt"

          # Setup monitor
          "-monitor"
          "vc:1024x768"
  )

  echo "Executing: qemu-system-x86_64 " "${QEMU_ARGS[@]}"
  qemu-system-x86_64 "${QEMU_ARGS[@]}"
}

# Function: Prepares the local grub installation inside the folder which will be a volume in the QEMU VM
# The grub config and the final ELF file, are built into the "memdisk" of grub for the sake of simplicity.
fn_prepare_grub_installation() {
  mkdir -p "${QEMU_VOLUME_DIR}/EFI/BOOT"
  # create a standalone GRUB installation for x86_64-efi platform in a local directory
  # UEFI spec: BOOT/EFI/BOOTX64.EFI will be automatically bootet
  #
  grub-mkstandalone -O x86_64-efi -o "${QEMU_VOLUME_DIR}/EFI/BOOT/BOOTX64.EFI" \
      "/boot/grub/grub.cfg=${QEMU_DIR}/grub.cfg" \
      "/boot/multiboot2-binary.elf=$FINAL_ELF"
      # this is poorly documented, but the tool allows to specify key-value
      # pairs where the value on the right, a file, will be built into the "(memdisk)"
      # volume inside the grub image
}

#########################################
# invoke function main
fn_main
