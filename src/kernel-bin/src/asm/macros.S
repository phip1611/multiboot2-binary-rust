.macro M_SETUP_STACK stack_begin
    lea     \stack_begin,   %eax
    mov     %eax,           %ebp
    mov     %eax,           %esp
.endmacro
