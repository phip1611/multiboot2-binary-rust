# The ability of Rust to include global assembly is poorly documented.
# It uses GAS (GNU Assembly) syntax

# is already the default
# . intel_syntax noprefix

.code64
# entry point into Rust
.EXTERN entry_rust

# start symbol must be globally available (linker must find it, don't discard it)
.GLOBAL start

.section .text

# always produce x-bit x86 code (even if this would be compiled to an ELF-32 file)
.code64

    start:
        # Save values in non-volatile registers. With these, we can call the entry function
        # in the Rust binary with two parameters accordingly.
        #   eax: Multiboot2 Magic Value
        #   ebx: pointer to Multiboot2 information structure
        #
        # first argument is edi, second is esi => SYSTEM V x86_64 calling convention
        mov         rdi,    0
        mov         rsi,    0
        mov         edi,    eax
        mov         esi,    ebx

        # Set stack top (stack grows downwards, from high to low address).
        # GRUB already used the stack provided by the UEFI firmware and
        # Multiboot2 spec also says, application needs to set it's own stack.

        # OFFSET in intel syntax is similar to $ in AT&T syntax
        # This way we get the symbol address as immediate in assembly output
        # https://stackoverflow.com/questions/39355188/

        # This is different to nasm, where the symbol address would be used
        # by default as immediate.
        movabs      rax,    OFFSET _initial_stack_top
        # _initial_stack_top is 16 byte aligned but exclusive -> get next lower aligned address
        sub         rax,    16
        # x86-thingy: the stack pointer minus eight must be correctly aligned
        # -> this comes from how stack frames are created at function entry
        # -> first argument is actually stored in `rsp - 8`
        add         rax,    8
        mov         rsp,    rax
        mov         rbp,    rax

        jmp         entry_rust
        ud2

# -----------------------------------------------------------------
.section .bss

    # Reserve 128 KiB as stack (no stack overflow protection so far!).
    # Note: _initial_stack_top is exclusive, so the most top valid address
    # is `_initial_stack_top - 1`. Needs further decrement for correct alignment.
    .ALIGN 16
    _initial_stack_bottom:
        # implicitly fills zeroes
        # https://ftp.gnu.org/old-gnu/Manuals/gas-2.9.1/html_chapter/as_7.html#SEC91
        .FILL 0x20000
    _initial_stack_top:

