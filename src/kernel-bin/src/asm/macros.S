# Assembly macro helpers. Compatible with GNU As (GAS) and LLVM's assembler
# - https://llvm.org/doxygen/AsmParser_8cpp_source.html

.set MULTIBOOT2_MAGIC, 0x36d76289

# Verifies that there was a valid handoff from a multiboot2 bootloader by checking the
# magic value in %ecx.
#
# Input Registers:
#  - %eax - the Multiboot2 magic number
# Parameters:
#  - None
# Outputs:
#  - None
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
#  - None
# Parameters:
#  - reg_tmp_addr: Register (distinct from the clobbered ones down below) with the begin address
#    of a 4-byte long memory area that can be used as temporal storage. Must be valid and in physical
#    ram. The original content will be restored after this code is done. Valid options depend on the
#    platform. One valid option is to pass the pointer to the Multiboot Information Structure (MBI).
#  - out_reg: Register where the output is stored.
# Outputs:
#  - The register behind the parameter 'out_reg' will contain the result.
# Clobbers: %eax, %esp, out_reg
.macro M_CALC_LOAD_ADDR_OFFSET_32BIT reg_tmp_addr out_reg
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
# it in the provided register. It relies on that the relocation offset was already set
# in REG_LOAD_OFFSET.
#
# The function operates on 32 bit registers.
#
# Input Registers:
#  - REG_LOAD_OFFSET - must contain the load offset
# Parameters:
#  - link_addr: Link address by its symbol name.
#  - out_reg: Register where the output is stored.
# Inputs:
#  - None
# Outputs:
#  - The register behind the parameter 'out_reg' will contain the result.
# Clobbers: out_reg
.macro M_RESOLVE_RUNTIME_ADDR link_addr out_reg
    mov     REG_LOAD_OFFSET,    \out_reg
    add     $\link_addr,        \out_reg
.endm

# Computes the page table index for an address depending on the level.
# TODO clarify whether this returns the index or the offset
#
# Inputs:
#  - level: 1, 2, 3, or 4 (specifies which part of the virtual address is used).
#  - vaddr_reg: Register that contains the virtual address used for this calculation.
#  - out_reg: Register where the output is stored.
# Outputs:
#   - The register behind the parameter 'out_reg' will contain the result.
# Clobbers: out_reg
.macro M_CALC_PAGE_TABLE_ENTRY_INDEX level, vaddr_reg, out_reg
    mov     \vaddr_reg,     \out_reg
    # We bit-shift the virtual address to the right.
    # * 9 as each level is indexed by 9 bits (with x86_64 4 level paging)
    # + 12 used as index into the page
    shr     $((\level - 1) * 10 + 12),   \out_reg
    # Now we select the 10 bits which are the index into the page table of the selected level.
    and     $0x3ff,         \out_reg
    imul    $4,             \out_reg # index * 4 as each entry is 4 byte long with 32-bit paging
.endm

# Writes a page table entry.
#
# Input registers:
#  - None
# Parameters:
#  - level: 1, 2, 3, or 4
#  - vaddr_reg: Register that contains the virtual address used for the mapping.
#  - phys_src_table: Symbol name of backing storage for source table (the one that needs an entry).
#  - phys_dest: Symbol name of physical memory to point to. Next table or (super) page.
#  - entry_flags: page table entry flags (huge page, permission bits, ...) (bits 11..0)
# Outputs:
#   - None.
# Clobbers: %eax, %ebp, %ecx
.macro M_PAGE_TABLE_WRITE_ENTRY level, vaddr_reg, phys_src_table, phys_dest, entry_flags
    # When this code is done, the following applies:
    # %ebp => byte offset into the page table for the entry (index * sizeof(entry))
    # %eax => page table entry that needs to be written into %ecx
    # %ecx => pointer to the entry of the page table where the entry is written to;
    #         already contains the index from %ebp

    # calculate index into the phys_src_table
    M_CALC_PAGE_TABLE_ENTRY_INDEX    \level,  \vaddr_reg,    %ebp

    # Create 32-bit page table entry in %eax.
    mov     $\entry_flags,  %ecx
    and     $0xfff,         %ecx   # ensure we only use the lowest 12 bits from the provided flags
    M_RESOLVE_RUNTIME_ADDR  \phys_dest, %eax # put load address of page table into %eax
    add     %ecx,           %eax   # add flags to entry

    # Write the entry to the table
    M_RESOLVE_RUNTIME_ADDR  \phys_src_table,    %ecx    # dest phys address in %ecx
    add     %ebp,           %ecx   # add index to table => %ecx is now pointer to table entry
    mov     %eax,           (%ecx) # write entry to the correct index of the table
.endm

# Wrapper around debugcon_print_string_until_null_32bit that prints a null-terminated string
# that ensures that no registers are clobbered.
#
# Input registers:
#  - None
# Parameters:
#  - string: Symbol name of null-terminated string.
# Outputs:
#   - None.
# Clobbers: %eax, %ecx
.macro M_DEBUGCON_PRINT_32 string
    push %eax
    push %ecx
    push %edx
    M_RESOLVE_RUNTIME_ADDR \string,    %eax
    call debugcon_print_string_until_null_32bit
    pop %edx
    pop %ecx
    pop %eax
.endm
