# "ax": section flags. Required so that the linker actually puts this into a LOAD segment. Linker
#                      script magic alone is not enough.
#                      See: https://phip1611.de/blog/gnu-ld-discards-section-containing-code/
.section .init_asm, "ax"

.code64

    # Prints s C-style string to the QEMU debugcon port.
    # Parameters:
    #   1: eax - Pointer to the begin of the string.
    # Clobbers: eax, ecx
    print_debugcon_string_until_nul:
        # load byte behind string pointer
        movb    (%eax),    %cl
        # null byte check
        cmp     $0,         %cl
        jz      out
        # prepare function arguments; %cl => %al
        xchg    %eax,   %ecx
        call    print_debugcon_single_char
        # switch back
        xchg    %eax,   %ecx
        # increase file pointer
        add     $1,         %eax
        jmp     print_debugcon_string_until_nul
        out: ret


    # Prints a character to the QEMU debugcon port.
    # Parameters:
    #   1: al - Character to print.
    # Clobbers: edx/dx
    # Example:
    # ```asm
    # call print_debugcon_single_char
    # mov     $'o', %al
    # ```
    print_debugcon_single_char:
        # movw: 16 bit
        movw    $0xe9,  %dx
        out     %al,    %dx
        ret
