/*
 * This linker script is used to assemble the ELF-file from cargos build with the ELF-object-file
 * from the compilation of `multiboot2_header.asm` into one single multiboot2-compatible ELF-file.
 * This "final ELF-file" can later be our Rust kernel, an OS-specific bootloader or another bootmanager.
 */

/** The "start"-symbol from start.asm. */
ENTRY(start)

/* I HAVE NO IDEA WHAT THIS IS. TAKEN FROM https://sourceware.org/binutils/docs/ld/PHDRS.html */
/* Because of: https://stackoverflow.com/questions/63420037/error-phdr-segment-not-covered-by-load-segment1 */
PHDRS
{
  headers PT_PHDR PHDRS ;
  interp PT_INTERP ;
  text PT_LOAD FILEHDR PHDRS ;
  data PT_LOAD ;
  dynamic PT_DYNAMIC ;
}

SECTIONS {

    /* Not relevant anymore, because our binary gets relocated by GRUB. */
    /* . = 8M; */

    /* Multiboot2-Header must be 64-bit (8 byte) aligned according to spec. */
    . = ALIGN(8);
    .multiboot_header :
    {
        /* ensure that the multiboot header is at the beginning */
        *(.multiboot_header)
    }

    .text :
    {
        *(.text)
    }

    /* Julian Stecklina (@blitzclone) told me that the GNU linker also adds a lot of other
       sections automatically when it thinks they are useful, e.g. debug sections and .bss */
    /*
        .rodata :
        {
          *(.rodata)
        }

        .data :
        {
          *(.data)
        }

        .bss :
        {
          *(COMMON)
          *(.bss)
        }
    */

}
