# Specifies the Multiboot2 header.
#
# Instructs the bootloader to brings the machine into the "i386 machine state" according to the
# Multiboot2 spec. We can't use the "EFI I386/amd64 machine state with boot services enabled" states
# as they only work with activated UEFI boot services. I can't set up page tables from assembly
# with activated boot services, as UEFI expects to own the firmware. For example, interrupts would
# be handled by UEFI. Once I change the page table, certain UEFI data structures will usually
# not be found anymore.
#
# Because of that, this kernel uses the "i386 machine state" as unified entry point on legacy
# BIOS and UEFI systems.
#
# The file uses GAS-like syntax in intel noprefix mode.

.align 8 # according to spec, the header must be 64-bit (8 byte) aligned
.section .multiboot2_header

    mb2_header_start:
        #   dd => int 32, see https://www.cs.uaf.edu/2017/fall/cs301/reference/x86_64.html
        .LONG  0xe85250d6                # magic number (multiboot2 spec)
        .LONG  0                         # architecture 0 (protected mode i386; spec doesn't specify many options)
        .LONG  mb2_header_end - mb2_header_start # header length
        #   checksum
        .LONG  0x100000000 - (0xe85250d6 + 0 + (mb2_header_end - mb2_header_start))

        # OPTIONAL MULTIBOOT2 TAGS
        # ------------------------------------------------------------------------------------
        # "Relocatable"-tag
        .ALIGN 8
        .Lmb2_header_tag_relocatable_start:
            .WORD  10      # type  (16bit)
            .WORD  0       # flags (16bit) (0 means required, 1 optional)
            .LONG  .Lmb2_header_tag_relocatable_end - .Lmb2_header_tag_relocatable_start   # size  (32bit)
            # According to spec, this has a higher precedence, than the regular start-symbol from the ELF.
            .LONG  0x00800000  # lowest possible address where binary can begin (8 MiB)
            .LONG  0xffffffff  # highest possible address where binary must end (4 GiB)
            .LONG  0x00400000  # alignment (4 MiB to enable super pages)
            .LONG  0           # preference: 0 (none), 1 (lowest possible), 2 (highest possible)
        .Lmb2_header_tag_relocatable_end:
        # ------------------------------------------------------------------------------------
        # REQUIRED END TAG
        .ALIGN 8
        .Lmb2_header_tag_end_start:
            .WORD  0       # type  (16bit)
            .WORD  0       # flags (16bit)
            .LONG  .Lmb2_header_tag_end_end - .Lmb2_header_tag_end_start # size  (32bit)
        .Lmb2_header_tag_end_end:
    mb2_header_end:
