# The ability of Rust to include global assembly is poorly documented.
# It uses GAS (GNU Assembly) syntax

.intel_syntax noprefix
.code64
# entry point into start routine (start.S)
.EXTERN start

.ALIGN 8 # according to spec, the header must be 64-bit (8 byte) aligned
.section .multiboot2_header

    mb2_header_start:
        #   dd => int 32, see https:#www.cs.uaf.edu/2017/fall/cs301/reference/x86_64.html
        .LONG  0xe85250d6                # magic number (multiboot2 spec)
        .LONG  0                         # architecture 0 (protected mode i386; spec doesn't specify many options)
        .LONG  mb2_header_end - mb2_header_start # header length
        #   checksum
        .LONG  0x100000000 - (0xe85250d6 + 0 + (mb2_header_end - mb2_header_start))

        # OPTIONAL MULTIBOOT2 TAGS (additional to required END TAG)
        # In order to boot into "EFI amd64 machine state with boot services enabled" (3.5 in Spec, 2021-06)
        # machine state, we must specify a few additional tags:
        #
        # ------------------------------------------------------------------------------------
        # "EFI boot services"-tag: leaves UEFI boot services enabled: its our task to exit them
        .ALIGN 8 # alignment in bits: according to multiboot2 spec all tags are 8-byte (64-bit) aligned
        .Lmb2_header_tag_ebs_start:
            .WORD  7       # type  (16bit)
            .WORD  0       # flags (16bit) (0 means required, 1 optional)
            .LONG  .Lmb2_header_tag_ebs_end - .Lmb2_header_tag_ebs_start       # size  (32bit)
        .Lmb2_header_tag_ebs_end:
        # ------------------------------------------------------------------------------------
        # "EFI amd64 entry address tag of Multiboot2 header"-tag
        .ALIGN 8
        .Lmb2_header_tag_efiamd64_start:
            .WORD  9       # type  (16bit)
            .WORD  0       # flags (16bit) (0 means required, 1 optional)
            .LONG  .Lmb2_header_tag_efiamd64_end - .Lmb2_header_tag_efiamd64_start     # size  (32bit)
            # Address to jump to.
            #  GRUB source code: https://github.com/rhboot/grub2/blob/a53e530f8ad3770c3b03c208c08ae4162f68e3b1/grub-core/loader/multiboot_mbi2.c#L212
            # According to MB2 spec, this has a higher precedence, than the regular start-symbol from the ELF.
            .LONG  start   # entry_addr (32bit)
        .Lmb2_header_tag_efiamd64_end:
        # ------------------------------------------------------------------------------------
        # "Relocatable"-tag
        #.ALIGN 8
        #.Lmb2_header_tag_relocatable_start:
        #    .WORD  10      # type  (16bit)
        #    .WORD  0       # flags (16bit) (0 means required, 1 optional)
        #    .LONG  .Lmb2_header_tag_relocatable_end - .Lmb2_header_tag_relocatable_start   # size  (32bit)
        #    # According to spec, this has a higher precedence, than the regular start-symbol from the ELF.
        #    .LONG  0x100000    # lowest possible address (8MiB)
        #    .LONG  0xffffffff  # highest possible address (4GiB)
        #    .LONG  4096        # alignment
        #    .LONG  0           # preference: 0 (none), 1 (lowest possible), 2 (highest possible)
        #.Lmb2_header_tag_relocatable_end:
        # ------------------------------------------------------------------------------------
        # REQUIRED END TAG
        .ALIGN 8
        .Lmb2_header_tag_end_start:
            .WORD  0       # type  (16bit)
            .WORD  0       # flags (16bit)
            .LONG  .Lmb2_header_tag_end_end - .Lmb2_header_tag_end_start # size  (32bit)
        .Lmb2_header_tag_end_end:
    mb2_header_end:
