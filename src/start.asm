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
        ; set stack top (stack grows downwards, from high to low address)
        ; TODO maybe this is not necessary: according to 2.3.4 x64 state (UEFI spec),
        ;  a 128KiB stack is already available (but probably used by GRUB..)
        mov     rsp, _initial_stack_top
        mov     rbp, _initial_stack_top

        ; Call Rust binary with two parameters
        ; eax: Multiboot2 Magic Value
        ; ebx: pointer to Multiboot2 information structure
        ;
        ; first argument is edi, second is esi => SYSTEM V x86_64 calling convention
        mov     edi, eax
        mov     esi, ebx
        jmp     entry_64_bit
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

