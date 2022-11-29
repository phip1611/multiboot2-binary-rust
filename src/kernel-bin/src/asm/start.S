# This file specifies the x86 entry point into the kernel. The Multiboot2 bootloader, GRUB2, will
# jump to code specified in this file on both, UEFI and legacy BIOS systems.
#
# The bootstrap processor (BSP) will undergo the following steps:
# 1. prepare CPU state to use common code accross different boot flows.
#   - BIOS/Legacy x86: create UEFI-like setup with identity mapped memory; go to 2
#   - UEFI x86_64    : go to 2
# 2. set up page tables, switch to high level code
#
# The kernel is relocatable as the assembly code uses relative addressing and no absolute jumps.
# This way, the bootloader can move the kernel around in memory. The assembly code sets up page
# tables so that the high-level code can run at it's expected link addresses within the linker
# script.

# start symbol must be globally available (linker must find it, don't discard it)
.global entry_mb2_32bit_default

# Strictly speaking, .extern is not relevant, as "GNU as" (and other assemblery I think as well)
# handle all unknown symbols as extern symbols. However, this is good to keep track of dependencies
# to other files.
#
# Entry into Rust code.
.extern entry_rust_x86_64_bsp
# Load address of the kernel.
.extern LOAD_ADDR
# Link address of the kernel.
.extern LINK_ADDR

# "ax": section flags. Required so that the linker actually puts this into a LOAD segment. Linker
#                      script magic alone is not enough.
#                      See: https://phip1611.de/blog/gnu-ld-discards-section-containing-code/
.section .init_asm, "ax"

.code32
entry_mb2_32bit_default:
    M_ASSERT_MB2_HANDOFF
    mov     $0xbadb001,     %eax
    # TODO
    cli
    hlt

.code64
entry_mb2_64_bit_efi:
    M_ASSERT_MB2_HANDOFF

    # setup stack
    M_LINK_TO_LOAD_ADDR_IN_RAX_64 boot_stack_end
    mov     %rax,   %rsp

    M_LINK_TO_LOAD_ADDR_IN_RAX_64 str_hello_world
    # (Compiler produces a relative call -> it works with relocations)
    call debugcon_print_string_until_null_64bit

    mov  $0xdeadbeef,    %rbx
    cli
    hlt

# Static backing storage for a minimal 512 byte stack.
# Enough to call multiple levels of code.
.align 4
boot_stack_begin:
.fill 512, 1, 0
boot_stack_end:
