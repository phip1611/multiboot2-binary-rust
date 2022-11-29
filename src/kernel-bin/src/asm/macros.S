# Assembly macro helpers. Compatible with GNU As (GAS) and LLVM's assembler
# - https://llvm.org/doxygen/AsmParser_8cpp_source.html

.set MULTIBOOT2_MAGIC, 0x36d76289

# Verifies that there was a valid handoff from a multiboot2 bootloader by checking the
# magic value in %ecx.
#
# Inputs:
# - %ecx - Multiboot magic
# Outputs:
# - none
# Clobbers: %edx, rflags
.macro M_ASSERT_MB2_HANDOFF
    mov     $MULTIBOOT2_MAGIC,  %edx
    # %ecx should contain the magic value
    cmp     %edx,               %eax
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

# Calculates the offset between link address and load address during runtime and put it into
# register %edx. This offset can be added to known link addresses of certain symbols to find them
# during runtime.
#
# Useful in 32-bit code. In 64-bit x86, an approach using the lea instruction should be used
# instead.
#
# Inputs:
# - reg_tmp_addr: Register (distinct from the clobbered ones down below) with the begin address
#                 of a 4-byte long memory area that can be used as temporal storage. Must be
#                 valid and in physical ram. The original content will be restored after this
#                 code is done. Valid options depend on the platform. One valid option is the
#                 pointer to the Multiboot Information Structure (MBI) in Multiboot payloads.
# Outputs:
# - %edx
# Clobbers: %eax, %edx, %esp
.macro M_LOAD_ADDR_OFFSET_IN_EDX_32BIT reg_tmp_addr
        # prepare minimal stack for the next call
        mov     (\reg_tmp_addr),   %eax    # save original memory content
        mov     \reg_tmp_addr,     %esp

        call 1f
    1:  # After the call, the stack contains the old eip (which points to runtime address of local
        # symbol 1:). We pop the eip from the stack intp register %edx.
        pop     %edx
        # Substracting the link address of label "1" from the instruction pointer in %edx.
        # %eax now contains the offset between link address and load address (=runtime address).
        sub     $1b,            %edx

        # restore memory
        mov     %eax,           (\reg_tmp_addr)
.endm

# Calculates the real load address from the link address of a known symbol and stores
# it in register %eax. Assumes that %edx contains the offset.
#
# Operates on 32 bit registers.
#
# Inputs:
# - %edx: The runtime offset from the link address
# - link_addr: Link address as immediate or by its symbol name
# Outputs:
# - %eax - The real load address of the symbol
# Clobbers: %eax
.macro M_LINK_TO_LOAD_ADDR_IN_EAX_32 link_addr
    mov     %edx,           %eax
    add     \link_addr,     %eax
.endm

# Calculates the offset between link address and load address during runtime and put it into
# register %rax. This offset can be added to known link addresses of certain symbols to find them
# during runtime.
#
# Operates on 64 bit registers and needs 64-bit mode opcodes.
#
# Inputs:
# - None
# Outputs:
# - %rax - The real load address of the symbol
# Clobbers: %rax
.macro M_LOAD_ADDR_OFFSET_IN_RDX_64BIT
        # load address of label "1" plus value in %rip into %rax
        lea        1f(%rip),    %rax
    1:
        # substract value from label "1" from %rax => actual runtime address of symbol "1"
        # => rax now contains the runtime load offset
        sub        $1b,         %rax
.endm

# Calculates the real load address from the link address of a known symbol and stores
# it in register %eax. Assumes that %edx contains the offset.
#
# Operates on 64 bit registers and needs 64-bit mode opcodes.
#
# Inputs:
# - link_addr: Link address by its symbol name
# Outputs:
# - %rax - The real load address of the symbol
# Clobbers: %rax
.macro M_LINK_TO_LOAD_ADDR_IN_RAX_64 link_addr
    M_LOAD_ADDR_OFFSET_IN_RDX_64BIT
    add     $\link_addr,     %rax
.endm
