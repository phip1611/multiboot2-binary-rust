.section .init_asm.stack, "a"
.align 8
boot_stack_begin:
# 1 KiB stack is more then enough for the boot code
.fill 1024, 1, 0
boot_stack_end:
