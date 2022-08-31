/*
 * Custom linker script for the `cargo` build.
 *
 */

/** Symbol comes from mb2_start_32bit_legacy.S */
ENTRY(mb2_32bit_default_entry)

/* Program headers. Also called segments. */
PHDRS
{
    /* PT_LOAD FLAGS(x): The flags of an ELF program header/segment. Always 32 bit long, also for
                         64-bit ELFs. Also called "Segment Permissions" in ELF specification or
                         "p_flags". Helps loaders to set the right page table bits. */

    /* As GRUB performs the hand-off in 64-bit mode on UEFI platforms (thus, with enabled paging),
       I also need the right permission bits here. */
    init_asm     PT_LOAD FLAGS(5); /* 0b101 */
    kernel       PT_LOAD FLAGS(5); /* 0b101 */
    kernel_ro    PT_LOAD FLAGS(4); /* 0b100 */
    kernel_rw    PT_LOAD FLAGS(6); /* 0b110 */

    /* TODO find out how I can include debug symbols */
    /* debug        PT_NOTE; */
}

/* ######### VARIABLE DECLARATIONS ######## */
/* Variables for the ELF file. They also end up in the global symbol table (.symtab) in the ELF
   file. They can be referenced from Assembly code, for example. */

/* Base load address for the ELF
   The value 8M is chosen at will. It is not important as the bootloader relocates us
   anyway. Must be within a range where GRUB can load us with Multiboot2, thus, in the first
   4 GB. */
LOAD_ADDR = 8M;

/* Virtual address where we want our kernel in the end. Only high level code from Rust lives
   here.  */
LINK_ADDR = 0xffffffff88000000;

SECTIONS {

    /* Syntax is described here: https://sourceware.org/binutils/docs/ld/Output-Section-Attributes.html

       ".init_asm"       :    The name of the (output) section. This overrides the name of the
                              input sections. Name can be arbitrary but should follow conventions.
       "8M"              :    Virtual address where the code is linked.
       ": AT(LOAD_ADDR)" :    Set the physical load address to this address. GRUB uses this for
                              example during boot.
          ALIGN(4K)      :    Ensure a 4K-aligned offset within the ELF-file.
     */
    .init_asm 8M : AT(LOAD_ADDR) ALIGN(4K)
    {
        /* According to spec, this must be in the first 32K of the binary. Because this section
           is in the the first load segment, this will also be at the beginning of the ELF. */
        *(.multiboot2_header)
        *(.init_asm)
        *(.init_asm.*)
    } : init_asm

    /* Push one page further so that the offset between load address and link address is the same.
       This enables us to use super pages for the kernel. */
    TEXT_LINK_ADDRESS = ALIGN(LINK_ADDR + SIZEOF(.init_asm), 4K);
    TEXT_LOAD_ADDRESS = ALIGN(LOADADDR(.init_asm) + SIZEOF(.init_asm), 4K);

    .text TEXT_LINK_ADDRESS : AT(TEXT_LOAD_ADDRESS) ALIGN(4K)
    {
        *(.text .text.*)
    } : kernel


    RODATA_LOAD_ADDRESS = ALIGN(LOADADDR(.text) + SIZEOF(.text), 4K);

    /* ALIGN(4K): Linker parses this file from top to bottom. It increases the virtual/link
       address automatically, but it doesn't do this for the physical/link address.
     */
    .rodata ALIGN(4K) : AT(RODATA_LOAD_ADDRESS) ALIGN(4K)
    {
      *(.rodata .rodata.*)
    } : kernel_ro


    DATA_LOAD_ADDRESS = ALIGN(LOADADDR(.rodata) + SIZEOF(.rodata), 4K);

    .data ALIGN(4K) : AT(DATA_LOAD_ADDRESS) ALIGN(4K)
    {
      *(.data .data.*)
      /* Putting .bss into the data segment simplifies loading an ELF file (if I ever use sth.
         different than GRUB) as FileSize==MemSize for all LOAD segments. */
      *(.bss .bss.*)
    } : kernel_rw


    /* TODO find out how I can include debug symbols */
    /*.debug . : ALIGN(4K)
    {
      *(.debug .debug*)
    } : debug*/


    /DISCARD/ :
    {
        *(.note.*)
        *(.eh_frame*)
        *(.got*)
    }

}
