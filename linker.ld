/*
 * This linker script is used to assemble the ELF-file from cargos build with the ELF-object-file
 * from the compilation of `multiboot2_header.asm` into one single multiboot2-compatible ELF-file.
 * This "final ELF-file" can later be our Rust kernel, an OS-specific bootloader or another bootmanager.
 */

/** The "_start"-symbol */
ENTRY(start)

SECTIONS {
    /* According to https://intermezzos.github.io/book/first-edition/hello-world.html
     * there is lots of memory mapped I/O and 1M is a conventional padding (legacy?!)
     * TODO find out if this only is valid for legacy BIOS boot or also for UEFI!
     */
    . = 1M;

    .boot :
    {
        /* ensure that the multiboot header is at the beginning */
        *(.multiboot_header)
    }

    .text :
    {
        *(.text)
    }

    /* Julian Stecklina (@blitzclone) told me that the GNU linker also adds a lot of other
       sections automatically when it thinks they are useful, e.g. debug sections. */
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
