# Assembly macro helpers. Compatible with GNU As (GAS) and LLVM's assembler
# - https://llvm.org/doxygen/AsmParser_8cpp_source.html

.set MULTIBOOT2_MAGIC, 0x36d76289

# Verifies that there was a valid handoff from a multiboot2 bootloader by checking the
# magic value in %ecx.
#
# Input Registers:
#   - %eax - the Multiboot2 magic number
# Clobbers: rflags
.macro M_ASSERT_MB2_HANDOFF
    cmp     $MULTIBOOT2_MAGIC,  REG_MB2_MAGIC
    # relative jump to following code
    je      0f

    # signal error and hlt
    mov     $0xbadb001, %ecx
    mov     $0xbadb001, %edx
    cli
    hlt
    0:
    # flow continues..
.endm

# Calculates the offset between link address and load address during runtime and put it into the
# provided register. This offset can be added to known link addresses of certain symbols to find
# them during runtime.
#
# Useful in 32-bit code. In 64-bit x86, an approach using the lea instruction should be used
# instead.
#
# Input Registers:
# - None
# Parameters:
# - reg_tmp_addr: Register (distinct from the clobbered ones down below) with the begin address
#   of a 4-byte long memory area that can be used as temporal storage. Must be valid and in physical
#   ram. The original content will be restored after this code is done. Valid options depend on the
#   platform. One valid option is to pass the pointer to the Multiboot Information Structure (MBI).
# - out_reg: Register for the output value.
# Outputs:
# - Provided parameter 'out_reg' will contain the offset
# Clobbers: %eax, %esp, parameter out_reg
.macro M_LOAD_ADDR_OFFSET_IN_EDX_32BIT reg_tmp_addr out_reg
        # prepare minimal stack for the next call
        mov     (\reg_tmp_addr),   %eax    # save original memory content
        mov     \reg_tmp_addr,     %esp

        call 1f
    1:  # After the call, the stack contains the old eip (which points to runtime address of local
        # symbol 1:). We pop the eip from the stack intp register %edx.
        pop     %edx
        # Substracting the link address of label "1" from the instruction pointer in %edx.
        # %eax now contains the offset between link address and load address (=runtime address).
        sub     $1b,            \out_reg

        # restore memory
        mov     %eax,           (\reg_tmp_addr)
.endm

# Calculates the real load address from the link address of a known symbol and stores
# it in the provided register %eax. Assumes that %edx contains the offset.
#
# Operates on 32 bit registers.
#
# Inputs:
# - offset_reg: The 32-bit register that contains the relocation offset).
# - link_addr: Link address as immediate or by its symbol name.
# - out_reg: Register where the output should be stored.
# Outputs:
# - %eax - The real load address of the symbol
# Clobbers: %eax
.macro M_LINK_TO_LOAD_ADDR_IN_EAX_32 offset_reg link_addr out_reg
    mov     \offset_reg,    \out_reg
    add     $\link_addr,    \out_reg
.endm

# Computes the page table index for an address depending on the level.
#
# Inputs:
#  - level: 1, 2, 3, or 4
#  - vaddr: virtual address
#  - working_register: register to use for the output value
# Outputs:
#   - %rax - the index in the page table
# Clobbers: The provided working register
.macro M_PAGE_TABLE_ENTRY_INDEX level, vaddr, working_register
    # We bit-shift to the right:
    # * 9 as each level is indexed by 9 bits (with x86_64 4 level paging)
    # + 12 used as index into the page
    shr     $((\level - 1) * 9 + 12), \working_register
    # Select the 9 bits which are the index.
    and     $0x1ff,             \working_register
.endm

# Writes a page table entry.
# Inputs:
#  - level: 1, 2, 3, or 4
#  - vaddr: the virtual address used for this mapping
#  - phys_src_table: symbol name of backing storage for source table (the one that needs an entry)
#  - phys_dest_table: symbol name of backing storage for next level page table
#  - permissions: page table permissions (bits 11..0)
# Outputs:
#   - None.
# Clobbers: %eax, %ebx, %ecx
.macro M_PAGE_TABLE_WRITE_ENTRY level, vaddr, phys_src_table, phys_dest_table, permissions
    # when this code is done, the following applies:
    # %eax => physical address of page table entry in the corresponding table
    # %ebx => index into the page table for the entry
    # %ecx => the page table entry

    M_LINK_TO_LOAD_ADDR_IN_RAX_64 \phys_dest_table # runtime physical address of table of next level
    M_PAGE_TABLE_ENTRY_INDEX    4,  \vaddr,     %ebx
    add     $\permissions,   %eax    # entry: add bits: present, read only, implicitly executable
    mov     %eax,   %ecx    # save page table entry in ecx
    M_LINK_TO_LOAD_ADDR_IN_RAX_64 \phys_src_table # runtime physical address of lvl4 table
    add     %ebx,   %eax    # location to write entry to = base address + index
    mov     %ecx,   (%eax)  # write entry in %ecx to address behind %eax
.endm



