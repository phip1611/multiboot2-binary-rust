; This file uses "Netwide Assembler Syntax" and can be compiled by running
; `nasm -f elf64 start.asm -o start.o`
;
; This file is used as entry point by the linker script. It sets up the stack inside
; the "start" symbol/function. It calls the "entry_32_bit" symbol afterwards,
; which is the entry point into the Rust binary.

; external symbol, that will come from Rust multiboot2 binary
EXTERN entry_64_bit

; start symbol must be globally available (linker must find it, don't discard it)
GLOBAL start

SECTION .text

; always produce x-bit x86 code (even if this would be compiled to an ELF-32 file)
[BITS 64]

    start:
        ; Save values in non-volatile registers. With these, we can call the entry function
        ; in the Rust binary with two parameters accordingly.
        ;   eax: Multiboot2 Magic Value
        ;   ebx: pointer to Multiboot2 information structure
        ;
        ; first argument is edi, second is esi => SYSTEM V x86_64 calling convention
        mov     edi, eax
        mov     esi, ebx

        ; Set stack top (stack grows downwards, from high to low address).
        ; GRUB already used the stack provided by the UEFI firmware and
        ; Multiboot2 spec also says, application needs to set it's own stack.
        mov     rsp, _initial_stack_top
        mov     rbp, _initial_stack_top


        ; NASM is really restricted, when it comes to access the `rip` register.
        ; The only working way I found is using the `rel` keyword along with a symbol.
        ; To use it with registers or immediates doesn't work.
        ; Doc:
        ;  - https://www.nasm.us/doc/nasmdoc7.html#section-7.2.1
        ;  - https://www.tortall.net/projects/yasm/manual/html/nasm-effaddr.html

        ; rbx: static link address
        mov     rbx, .eff_addr_magic_end
        ; rax: runtime address (relative to instruction pointer)
        lea     rax, [rel + .eff_addr_magic_end]

    .eff_addr_magic_end:
        ; subtract address difference => offset
        sub     rax, rbx
        ; rax: address of Rust entry point (static link address + runtime offset
        add     rax, entry_64_bit

        jmp     rax
        ; here we should only land if some error occurs
        cli     ; clear interrupts, otherwise the hlt will not work
        hlt

; -----------------------------------------------------------------
SECTION .bss

    ; reserve 128 KiB as stack (no stack overflow protection so far!)
    ; when we use "resb" (reserve bytes), the memory grows upwards
    ;
    ; We later drop this memory area as stack if we fully manage the memory in our kernel
    _initial_stack_bottom:
        resb 0x20000
    _initial_stack_top:

