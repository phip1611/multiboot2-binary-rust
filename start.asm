; This file uses "Netwide Assembler Syntax" and can be compiled by running
; `nasm -f elf64 start.asm -o start.o`
;
; This file is used as entry point by the linker script. It sets up the stack inside
; the "start" symbol/function. It calls the "_start" symbol afterwards,
; which is the entry point into the Rust binary.

; external symbol, that will come from Rust multiboot2 binary
EXTERN _start

; start symbol must be globally available (linker must find it, don't discard it)
GLOBAL start

SECTION .text

    start:
        ; set stack top (stack grows downwards, from high to low address)
        mov     esp, _stack_top
        mov     ebp, _stack_top

        ; Check for multiboot2 magic value in eax register.
        ; If grub loads our multiboot2 binary properly, the value is stored there.
        ; Otherwise, we go to label "bad_boot_exit" and stop/halt.
        ; Note, that register ebx contains a pointer to the multiboot2 data structure.
        cmp     eax, 0x36d76289
        jne     bad_boot_exit

        ; "start" is defined by the Rust binary.
        ; When the linker builts everything into a single binary,
        ; this is available.
        jmp     _start

    ; write "bad boot" in hexspeak int some registers and stop/halt
    bad_boot_exit:
        mov ebx, 0x0badb001
        mov ecx, 0x0badb001
        mov edx, 0x0badb001
        mov edi, 0x0badb001
        mov esi, 0x0badb001
        hlt

; -----------------------------------------------------------------
SECTION .bss

    ; reserve 128 KiB as stack (no stack overflow protection so far!)
    ; when we use "resb" (reserve bytes), the memory grows upwards
    _stack_bottom:
        resb 0x20000
    _stack_top:

