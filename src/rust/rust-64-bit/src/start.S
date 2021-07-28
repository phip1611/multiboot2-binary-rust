# The ability of Rust to include global assembly is poorly documented.
# It uses GAS (GNU Assembly) syntax

.intel_syntax noprefix
.code64
# entry point into Rust
.EXTERN entry_64_bit

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
        mov     rdi, 0
        mov     rsi, 0
        mov     edi, eax
        mov     esi, ebx


        # Loads the link time address into rcx
            # OFFSET in intel syntax is similar to $ in AT&T syntax
            # This way we get the symbol address as immediate in assembly output
            # https://stackoverflow.com/questions/39355188/
            # This is different to nasm, where the symbol address would be used
            # by default as immediate.
        mov     rcx, OFFSET start
        # Loads the real address of start into rdx
        #  results in "-0x20(%rip)"
        lea     rdx, [rip - start]
        # rdx, the relative runtime offset
        # example: link addr = 8MiB
        #          real addr = 1MiB
        #          offset    = 1MiB - 8MiB = -7MiB
        # => we need to add "-7MiB" (the offset in rdx) to every static/old/invalid address next
        sub     rdx, rcx

        # Set stack top (stack grows downwards, from high to low address).
        # GRUB already used the stack provided by the UEFI firmware and
        # Multiboot2 spec also says, application needs to set it's own stack.

            # relocate stack top
        mov     rax, OFFSET _initial_stack_top
        add     rax, rdx
        mov     rsp, rax
        mov     rbp, rax
            # relocate rust entry
        mov     rax, OFFSET entry_64_bit
        add     rax, rdx

        jmp     rax

        # here we should only land if some error occurs
        cli     # clear interrupts, otherwise the hlt will not work
        hlt

# -----------------------------------------------------------------
.section .bss

    # reserve 128 KiB as stack (no stack overflow protection so far!)
    .ALIGN 16
    _initial_stack_bottom:
        # implicitly fills zeroes
        # https://ftp.gnu.org/old-gnu/Manuals/gas-2.9.1/html_chapter/as_7.html#SEC91
        .FILL 0x20000
    _initial_stack_top:

