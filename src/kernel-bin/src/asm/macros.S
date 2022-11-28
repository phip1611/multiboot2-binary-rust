
.set MULTIBOOT2_MAGIC, 0x36d76289

# Verifies that there was a valid handoff from a multiboot2 bootloader by checking the
# magic value in %ecx.
#
# Inputs:
# - %ecx
# Outputs:
# - none
# Clobbers:
# - %edx, rflags
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
