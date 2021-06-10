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
        ; TODO maybe this is not necessary: according to 2.3.4 x64 state (UEFI spec), a 128KiB stack is already available
        mov     rsp, _initial_stack_top
        mov     rbp, _initial_stack_top

        ; Check for multiboot2 magic value in eax register.
        ; If grub loads our multiboot2 binary properly, the value is stored there.
        ; Otherwise, we go to label "bad_boot_exit" and stop/halt.
        ; Note, that register ebx contains a pointer to the multiboot2 data structure.
        cmp     eax, 0x36d76289
        jne     bad_boot_exit

        jmp     entry_64_bit
        hlt

        ; "start" is defined by the Rust binary.
        ; When the linker builts everything into a single binary,
        ; this is available.
        ; jmp     entry_32_bit

    ; write "bad boot" in hexspeak into some registers and stop/halt
    bad_boot_exit:
        mov     rbx, 0x0badb001
        mov     rcx, 0x0badb001
        mov     rdx, 0x0badb001
        mov     rdi, 0x0badb001
        mov     rsi, 0x0badb001
        cli
        .hlt:
        hlt
        jmp     .hlt ; make sure our program definitely halts, even if HLT is cancelled

; -----------------------------------------------------------------
SECTION .bss

    ; reserve 128 KiB as stack (no stack overflow protection so far!)
    ; when we use "resb" (reserve bytes), the memory grows upwards
    ;
    ; We later drop this memory area as stack if we fully manage the memory in our kernel
    _initial_stack_bottom:
        resb 0x20000
    _initial_stack_top:

