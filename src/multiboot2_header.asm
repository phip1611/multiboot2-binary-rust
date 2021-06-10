; This file uses "Netwide Assembler Syntax" and can be compiled by running
; `nasm -f elf64 multiboot2_header.asm -o multiboot2_header.o`
;
; To verify/debug this file, you can use:
;   - `$ grub-file --is-x86-multiboot2 <compiled-file-with-multiboot2-header>`
;   - `$ bootinfo <compiled-file-with-multiboot2-header>` (https://crates.io/crates/bootinfo)
;
; Links:
; - nasm cheatsheets: https://gist.github.com/justinian/385c70347db8aca7ba93e87db90fc9a6
; - spec https://www.gnu.org/software/grub/manual/multiboot2/multiboot.pdf
; - https://intermezzos.github.io/book/first-edition/multiboot-headers.html
;
; External symbol, that comes "start.asm"
EXTERN start

ALIGN 8 ; according to spec, the header must be 64-bit (8 byte) aligned
section .multiboot_header

    mb2_header_start:
        ;   dd => int 32, see https://www.cs.uaf.edu/2017/fall/cs301/reference/x86_64.html
        dd  0xe85250d6                ; magic number (multiboot2 spec)
        dd  0                         ; architecture 0 (protected mode i386; spec doesn't specify many options)
        dd  mb2_header_end - mb2_header_start ; header length
        ;   checksum
        dd  0x100000000 - (0xe85250d6 + 0 + (mb2_header_end - mb2_header_start))

        ; OPTIONAL MULTIBOOT2 TAGS (additional to required END TAG)
        ; In order to boot into "EFI amd64 machine state with boot services enabled" (3.5 in Spec, 2021-06)
        ; machine state, we must specify a few additional tags:
        ;


        ; IT SEEMS LIKE THIS DOESN'T HAS ANY EFFECT
        ;
        ; ------------------------------------------------------------------------------------
        ; "Information Request"-tag: advise bootloader to give us the requested information in the mb2 info structure
        ;ALIGN 8 ; alignment in bits: according to multiboot2 spec all tags are 8-byte (64-bit) aligned
        ;mb2_header_tag_ir_start:
        ;    dw  1       ; type  (16bit)
        ;    dw  0       ; flags (16bit) (0 means required, 1 optional)
        ;    dd  mb2_header_tag_ir_end - mb2_header_tag_ir_start       ; size  (32bit)
        ;    ; values you can put here are in multiboot2 spec example impl: the constants with
        ;    ; "MULTIBOOT_TAG_TYPE_" as prefix
        ;    dd  6   ; MULTIBOOT_TAG_TYPE_MMAP
        ;    dd 17   ; MULTIBOOT_TAG_TYPE_EFI_MMAP
        ;    dd 18   ; MULTIBOOT_TAG_TYPE_EFI_BS
        ;mb2_header_tag_ir_end:

        ; ------------------------------------------------------------------------------------
        ; "EFI boot services"-tag: leaves UEFI boot services enabled: its our task to exit them
        ALIGN 8 ; alignment in bits: according to multiboot2 spec all tags are 8-byte (64-bit) aligned
        mb2_header_tag_ebs_start:
            dw  7       ; type  (16bit)
            dw  0       ; flags (16bit) (0 means required, 1 optional)
            dd  mb2_header_tag_ebs_end - mb2_header_tag_ebs_start       ; size  (32bit)
        mb2_header_tag_ebs_end:
        ; ------------------------------------------------------------------------------------
        ; "EFI amd64 entry address tag of Multiboot2 header"-tag
        ALIGN 8
        mb2_header_tag_efiamd64_start:
            dw  9       ; type  (16bit)
            dw  0       ; flags (16bit) (0 means required, 1 optional)
            dd  mb2_header_tag_efiamd64_end - mb2_header_tag_efiamd64_start      ; size  (32bit)
            ; According to spec, this has a higher precedence, than the regular start-symbol from the ELF.
            dd  start   ; entry_addr (32bit)
        mb2_header_tag_efiamd64_end:
        ; ------------------------------------------------------------------------------------
        ; REQUIRED END TAG
        ALIGN 8
        mb2_header_tag_end_start:
            dw  0       ; type  (16bit)
            dw  0       ; flags (16bit)
            dd  mb2_header_tag_end_end -  mb2_header_tag_end_start ; size  (32bit)
        mb2_header_tag_end_end:
    mb2_header_end:
