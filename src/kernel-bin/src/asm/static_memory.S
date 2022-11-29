# Static Memory


# Static backing storage for a minimal 512 byte stack.
# Enough to call multiple levels of code.
.align 4
mem_boot_stack_begin:
.fill 512, 1, 0
mem_boot_stack_end:

# Backing memory for the PML4 base page table (with 4-level paging)
.align 4096
mem_lvl4:
.fill 4096, 1, 0

# Backing memory for the Page Directory Pointer Table (PDPT)
# for the high memory mapping (link address).
.align 4096
mem_lvl3h:
.fill 4096, 1, 0

# Backing memory for the Page Directory Table that has a 2MiB
# for the high memory mapping (link address).
.align 4096
mem_lvl2h:
.fill 4096, 1, 0

# Backing memory for the Page Directory Pointer Table (PDPT)
# for the low memory mapping (load address).
.align 4096
mem_lvl3l:
.fill 4096, 1, 0

# Backing memory for the Page Directory Table that has a 2MiB
# for the low memory mapping (load address).
.align 4096
mem_lvl2l:
.fill 4096, 1, 0

