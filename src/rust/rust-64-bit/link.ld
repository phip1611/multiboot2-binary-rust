/*
 * This linker script is used to assemble the ELF-file from cargos build with the ELF-object-file
 * from the compilation of `multiboot2_header.asm` into one single multiboot2-compatible ELF-file.
 * This "final ELF-file" can later be our Rust kernel, an OS-specific bootloader or another bootmanager.
 */

/** The "start"-symbol from start.asm. */
ENTRY(start)

SECTIONS {

    /* Not relevant anymore, because our binary gets relocated by GRUB
       and has PIC. */
    . = 0M;

    /* Multiboot2-Header must be 64-bit (8 byte) aligned according to spec. */
    . = ALIGN(8);

    .text :
    {
        *(.multiboot2_header)
        *(.text .text.*)
    }

    /* Julian Stecklina (@blitzclone) told me that the GNU linker also adds a lot of other
       sections automatically when it thinks they are useful, e.g. debug sections and .bss */

    .rodata :
    {
      *(.rodata .rodata.*)
    }

    .data :
    {
      *(.data .data.*)
    }

    .bss :
    {
      *(COMMON)
      *(.bss .bss.*)
    }


}
