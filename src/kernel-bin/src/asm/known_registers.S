# A few well-known registers worth to have an type alias.

// The register that contains the Multiboot2 magic after handoff.
.set REG_MB2_MAGIC, %eax
// The register that contains the runtime load address offset. Required to find symbols by their
// absolute link address if we are relocated.
.set REG_LOAD_OFFSET, %edx
// Pointer to the Multiboot2 Information Structure (MBI).
.set REG_MB2_INFO, %ebx
// Register usable for general functionality depending on the context.
// Due to limited registers in 32-bit mode, I reuse %eax as soon as the MB2 magic was checked.
.set REG_IN_OUT_1, %eax
// Register usable for general functionality depending on the context.
.set REG_IN_OUT_2, %ecx
