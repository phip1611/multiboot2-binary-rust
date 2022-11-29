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
.extern LINK_ADDR

# "ax": section flags. Required so that the linker actually puts this into a LOAD segment. Linker
#                      script magic alone is not enough.
#                      See: https://phip1611.de/blog/gnu-ld-discards-section-containing-code/
.section .init_asm, "ax"

.code32
entry_mb2_32bit:
    M_ASSERT_MB2_HANDOFF

    # find load address offset (are we relocated?)
    M_LOAD_ADDR_OFFSET_IN_EDX_32BIT REG_MB2_INFO REG_LOAD_OFFSET
    # setup stack
    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_boot_stack_end %esp

    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET str_hello_world %eax
    call debugcon_print_string_until_null_32bit

    # I now start creating our boot page table in preallocated memory. I need five page tables
    # (Level 4, 2x Level 3, 2x Level 2) whereras each level 2 table creates one 2 MiB huge page
    # mapping for the kernel in high memory and one in low memory. After that, I jump into high
    # level Rust code and do all further setup.

    # Create PML4 entry in %rax

    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_lvl3h %eax
    M_PAGE_TABLE_ENTRY_INDEX    4,  $LINK_ADDR,     %ebx
    add     $0x1,   %eax    # entry: add bits: present, read only, implicitly executable
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_lvl4 %eax # runtime physical address of lvl3 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax

    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_lvl2h %eax
    M_PAGE_TABLE_ENTRY_INDEX    4,  $LINK_ADDR,     %ebx
    add     $0x1,   %eax    # entry: add bits: present, read only, implicitly executable
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_lvl3h %eax # runtime physical address of lvl3 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax

    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET LOAD_ADDR %eax
    M_PAGE_TABLE_ENTRY_INDEX    4,  $LINK_ADDR,     %ebx
    add     $0x1,   %eax    # entry: add bits: present, read only, implicitly executable
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_lvl2h %eax # runtime physical address of lvl3 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax

    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_lvl3l %eax
    M_PAGE_TABLE_ENTRY_INDEX    4,  $LINK_ADDR,     %ebx
    add     $0x1,   %eax    # entry: add bits: present, read only, implicitly executable
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_lvl4 %eax # runtime physical address of lvl3 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax

    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_lvl2l %eax
    M_PAGE_TABLE_ENTRY_INDEX    4,  $LINK_ADDR,     %ebx
    add     $0x1,   %eax    # entry: add bits: present, read only, implicitly executable
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_lvl3l %eax # runtime physical address of lvl3 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax

    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET LOAD_ADDR %eax
    M_PAGE_TABLE_ENTRY_INDEX    4,  $LINK_ADDR,     %ebx
    add     $0x1,   %eax    # entry: add bits: present, read only, implicitly executable
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_lvl2l %eax # runtime physical address of lvl3 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax

     /*
     * Enable the following features in IA32_EFER:
     *
     * - the SYSCALL/SYSRET instructions,
     * - the page table NX bits,
     * - IA-32e (64-bit) mode.
     */
    or      $0x901, %eax
    # wrmsr
    #mov     $0x6b8, %eax
    #mov     %eax, %cr4
    #M_LINK_TO_LOAD_ADDR_IN_EAX_32 REG_LOAD_OFFSET mem_lvl4 %eax
    #mov     %eax, %cr3
    #mov     $0x80010023, %eax
    #mov     %eax, %cr0

/*


    # Create PML4 entry in %rax

    M_LINK_TO_LOAD_ADDR_IN_RAX_64 mem_lvl3h # runtime physical address of lvl3 table
    M_PAGE_TABLE_ENTRY_INDEX    4,  $LINK_ADDR,     %ebx
    add     $0x1,   %eax    # entry: add bits: present, read only, implicitly executable
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_RAX_64 mem_lvl4 # runtime physical address of lvl4 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax

    M_LINK_TO_LOAD_ADDR_IN_RAX_64 mem_lvl2h # runtime physical address of lvl2 table
    M_PAGE_TABLE_ENTRY_INDEX    3,  $LINK_ADDR,     %ebx
    add     $0x1,   %eax    # entry: add bits: present, read only, implicitly executable
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_RAX_64 mem_lvl3h # runtime physical address of lvl3 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax

    M_LINK_TO_LOAD_ADDR_IN_RAX_64 LOAD_ADDR # runtime physical address of kernel code
    M_PAGE_TABLE_ENTRY_INDEX    2,  $LINK_ADDR,     %ebx
    add     $0x81,  %eax    # entry: add bits: present, read only, implicitly executable, 2 Mib huge page
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_RAX_64 mem_lvl2h # runtime physical address of lvl4 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax




    // now map the currently executing code

    M_LINK_TO_LOAD_ADDR_IN_RAX_64 mem_lvl3l # runtime physical address of lvl3 table
    M_PAGE_TABLE_ENTRY_INDEX    4,  $LOAD_ADDR,     %ebx
    add     $0x1,   %eax    # entry: add bits: present, read only, implicitly executable
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_RAX_64 mem_lvl4 # runtime physical address of lvl4 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax

    M_LINK_TO_LOAD_ADDR_IN_RAX_64 mem_lvl2l # runtime physical address of lvl2 table
    M_PAGE_TABLE_ENTRY_INDEX    3,  $LOAD_ADDR,     %ebx
    add     $0x1,   %eax    # entry: add bits: present, read only, implicitly executable
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_RAX_64 mem_lvl3l # runtime physical address of lvl3 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax

    M_LINK_TO_LOAD_ADDR_IN_RAX_64 LOAD_ADDR # runtime physical address of kernel code
    M_PAGE_TABLE_ENTRY_INDEX    2,  $LOAD_ADDR,     %ebx
    add     $0x81,  %eax    # entry: add bits: present, read only, implicitly executable, 2 Mib huge page
    mov     %eax,   %ecx    # save entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_RAX_64 mem_lvl2l # runtime physical address of lvl4 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax


    # refresh page table
    M_LINK_TO_LOAD_ADDR_IN_RAX_64 mem_lvl4
    mov     %rax, %cr3
*/
    mov  $0xdeadbeef,    %ebx
    cli
    hlt


