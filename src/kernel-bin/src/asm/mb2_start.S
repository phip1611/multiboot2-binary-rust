# This file specifies the entry point for the boot hand off from GRUB2 on UEFI and legacy BIOS
# systems.
#
# The bootstrap processor (BSP) will undergo the boot steps:
# 1. prepare CPU state to use common code accross different boot flows.
#   - UEFI x86_64       : disable paging, switch to 32-bit protected mode
#   - BIOS/Legacy x86_64: continue with 2
# 2. set up page tables, switch to 64-bit long mode

# start symbol must be globally available (linker must find it, don't discard it)
.global mb2_32bit_default_entry

# Strictly speaking, .extern is not relevant, as GNU as (and others I think as well) handle all
# unknown symbols as extern symbols. However, this is good to keep track of dependencies to
# other files.
#
# Entry into Rust code.
.extern entry_rust
# Load address of the kernel.
.extern LOAD_ADDR
# Link address of the kernel.
.extern LINK_ADDR

# Magic value in EAX after Multiboot2 dispatch.
.set MULTIBOOT2_MAGIC, 0x36d76289


.code64

# "ax": section flags. Required so that the linker actually puts this into a LOAD segment. Linker
#                      script magic alone is not enough.
#                      See: https://phip1611.de/blog/gnu-ld-discards-section-containing-code/
.section .init_asm, "ax"

hello_world_str2: .asciz "Hello World"

    # Entry specified for the desired machine state in Multiboot2 header
    entry_mb2_64_bit_efi:
        # save MB2 header pointer for later use
        mov     %rbx,   %r15
        xor     %rbp,   %rbp
        xor     %rsp,   %rsp
        M_SETUP_STACK boot_stack_end
        movl    $hello_world_str2,    %eax
        call    print_debugcon_string_until_nul

        # M_SETUP_STACK
        mov     $MULTIBOOT2_MAGIC,  %eax
        # ecx contains Multiboot2 mgaic, according to spec
        cmp     %ecx,               %eax
        je      no_mb2_64bit

        xor     %rbx,               %rbx
        xor     %rcx,               %rcx

        # movabs  $LOAD_ADDR,         %rbx
        # movabs  $LINK_ADDR,         %rcx

        movl    $hello_world_str,    %eax
        call    print_debugcon_string_until_nul
        cli
        hlt
        ud2

.code32
    # Entry of the ELF file. Used as the entry by the Multiboot2 bootloader if none of the
    # specified/requested machine states in the Multiboot2 header is available.
    mb2_32bit_default_entry:
        # TODO
        mov     $0xbadb001,  %eax
        mov     $0xbadb001,  %ebx
        mov     $0xbadb001,  %ecx
        mov     $0xbadb001,  %edx
        cli
        hlt
        ud2

.code32
    no_mb2_32bit:
        mov     $0xbadb001,  %eax
        mov     $0xbadb001,  %ebx
        mov     $0xbadb001,  %ecx
        mov     $0xbadb001,  %edx
        cli
        hlt
        ud2

.code64
    no_mb2_64bit:
        mov     $0xbadb001,  %rax
        mov     $0xbadb001,  %rbx
        mov     $0xbadb001,  %rcx
        mov     $0xbadb001,  %rdx
        cli
        hlt
        ud2

# Reset at the end (documented in Rust Documentation)
# Because of this, currently I can only support x86_64 and not x86, I think
.code64
