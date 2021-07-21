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
        mov     edi, eax
        mov     esi, ebx



        # Loads the link time address into rcx
        lea     rcx, [start]
        # Loads the real address of start into rdx
        lea     rdx, [rip - start]
        # the relative runtime offset
        sub     rdx, rcx



        # Set stack top (stack grows downwards, from high to low address).
        # GRUB already used the stack provided by the UEFI firmware and
        # Multiboot2 spec also says, application needs to set it's own stack.
        mov     rax, _initial_stack_top
        # add offset
        add     rax, rcx
        mov     rsp, rax
        mov     rbp, rax



        # rcx contains the runtime address of the entry into Rust after this lne
        lea     rcx, [entry_64_bit + rdx]
        jmp     rcx



        # here we should only land if some error occurs
        cli     # clear interrupts, otherwise the hlt will not work
        hlt

# -----------------------------------------------------------------
.section .bss

    # reserve 128 KiB as stack (no stack overflow protection so far!)
    # when we use "resb" (reserve bytes), the memory grows upwards
    #
    # We later drop this memory area as stack if we fully manage the memory in our kernel
    _initial_stack_bottom:
        .fill 0x20000
    _initial_stack_top:

