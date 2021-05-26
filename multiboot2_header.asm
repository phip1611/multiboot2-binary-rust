; This file uses "Netwide Assembler Syntax" and can be compiled by running
; `nasm -f elf64 multiboot2_header.asm -o multiboot2_header.o`
;
; More general info here: https://intermezzos.github.io/book/first-edition/multiboot-headers.html
;
; TODO: Open question if this can combined by Rust itself, using linker magic properties
;       in `x86-none-bare_metal-multiboot2.json`
section .multiboot_header
header_start:
    dd 0xe85250d6                ; magic number (multiboot 2)
    ; todo check if I have to change this to x86
    dd 0                         ; architecture 0 (protected mode i386)
    dd header_end - header_start ; header length
    ; checksum
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

    ; insert optional multiboot tags here

    ; required end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end:
