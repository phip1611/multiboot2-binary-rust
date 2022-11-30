# Helper functions to access QEMU's debugcon device.
# See https://phip1611.de/blog/how-to-use-qemus-debugcon-feature-and-write-to-a-file/

.section .init_asm, "ax"

.set DEBUGCON_PORT, 0xe9

.code32

# Prints a C-style string to the QEMU debugcon port. Only callable from 32-bit code.
#
# Input registers:
#  - %eax - Pointer to the begin of the string.
#
# Clobbers: %eax, %ecx, %edx/%dx
debugcon_print_string_until_null_32bit:
    # load byte behind string pointer
    movb    (%eax),     %cl
    # null byte check
    cmp     $0,         %cl
    # jump zero is always relative
    jz      0f
    # prepare function arguments; %cl => %al
    xchg    %eax,       %ecx
    # (Compiler produces a relative call -> it works with relocations)
    call    debugcon_print_byte_32bit
    # restore string pointer in %eax
    xchg    %eax,       %ecx
    # increase string pointer
    inc     %eax
    # do again for next byte of string
    # (Compiler produces a relative jump -> it works with relocations)
    jmp     debugcon_print_string_until_null_32bit
    0: ret



# Prints a character to the QEMU debugcon port.Only callable from 32-bit code.
#
# Parameters:
#   1: %al - Character to print.
#
# Clobbers: %edx/%dx
#
# Example:
# ```asm
# mov       $'o',   %al
# call      debugcon_print_byte
# ```
debugcon_print_byte_32bit:
    # 0xe9 => I/O port of debugcon device
    # movw: 16 bit
    movw    $DEBUGCON_PORT,  %dx
    out     %al,    %dx
    ret
