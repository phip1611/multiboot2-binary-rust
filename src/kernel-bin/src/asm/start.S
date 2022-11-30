# This file specifies the x86 entry point into the kernel. The Multiboot2 bootloader, GRUB2, will
# jump to code specified in this file on both, UEFI and legacy BIOS systems.
#
# The code in this file is responsible for bringing the Bootstrap Processor (BSP) into 64-bit long
# mode. All further platform setup will be done in high-level Rust code.
#
# WHY IS THE KERNEL RELOCATABLE?
# The kernel is relocatable as the assembly code uses relative addressing and no absolute jumps.
# I do not actively set compiler settings for that, but it seems to be the case, that the compiler
# uses relative addressing when the symbols are close together and in the same section.
# This way, the bootloader can move the kernel around in memory. The assembly code sets up page
# tables so that the high-level code can run at it's expected link addresses within the linker
# script.

# start symbol must be globally available (linker must find it, don't discard it)
.global entry_mb2_32bit

# Strictly speaking, .extern is not relevant, as "GNU as" (and other assemblery I think as well)
# handle all unknown symbols as extern symbols. However, this is good to keep track of dependencies
# to other files.
#
# Entry into Rust code.
.extern entry_rust_x86_64_bsp
# Load address of the kernel.
.extern LOAD_ADDR
# Link address of the kernel.
.extern KERNEL_LINK_ADDR

# "ax": section flags. Required so that the linker actually puts this into a LOAD segment. Linker
#                      script magic alone is not enough.
#                      See: https://phip1611.de/blog/gnu-ld-discards-section-containing-code/
.section .init_asm, "ax"

.align 4
.code32
entry_mb2_32bit:
    M_ASSERT_MB2_HANDOFF

    # Find load address offset (are we relocated?).
    # We use the pointer to the MBI as temporary stack. Look into the macro description for details.
    M_CALC_LOAD_ADDR_OFFSET_32BIT REG_MB2_INFO, REG_LOAD_OFFSET

    # Setup a small stack.
    M_RESOLVE_RUNTIME_ADDR mem_boot_stack_end,  %esp

    # Print hello world via QEMU's debugcon device.
    M_DEBUGCON_PRINT_32 str_hello_world

    # Now, I want to make the switch from 32-bit protected mode without paging to paging enabled.
    # I start creating boot page table in preallocated memory for the low memory (where this code
    # is running).

    /* BEGIN: IDENTITY MAP FOR BOOT_LINK_ADDR (WITH RESPECT TO RELOCATION) */
        M_RESOLVE_RUNTIME_ADDR BOOT_LINK_ADDR,   %ebp
        # entry bits: present, read only, implicitly executable, 4 MiB huge page
        M_PAGE_TABLE_WRITE_ENTRY 2, %ebp, mem_lvl2l, BOOT_LINK_ADDR, 0x81
    /* END:   IDENTITY MAP FOR BOOT_LINK_ADDR (WITH RESPECT TO RELOCATION) */

    # init paging
    M_RESOLVE_RUNTIME_ADDR mem_lvl2l,   %eax
    mov     %eax,           %cr3

    # Enable huge page support for 32-bit paging.
    mov     %cr4,           %eax
    or      $0x10,          %eax    # set bit 4 (Page Size Extension (PSE))
    mov     %eax,           %cr4

    # set paging (PG) and protection (PE) bits in cr0
    mov     %cr0,           %eax
    or      $0x80000001,    %eax
    mov     %eax,           %cr0

    mov     $0xdeadbeef,    %ebx
    cli
    hlt


