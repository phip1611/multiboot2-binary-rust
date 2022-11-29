//! This module includes all my global assembly files. Rust uses a GAS-like (GNU Assembly) syntax
//! for global and inline assembler. This is documented here:
//! https://doc.rust-lang.org/nightly/reference/inline-assembly.html
//!
//! I prefer att_syntax in prefix mode as it is clearer. It doesn't look so nice but it's always
//! clear whether something is an immediate or an address. Additionally, there is more existing
//! assembly code available in AT&T syntax for comparison.

// To the compiler, this will look like a big single assembly file. Hence, if multiple files add
// code to ".section .foo", the symbols will be compiled and linked in the same order into the
// binary.

core::arch::global_asm!(include_str!("known_registers.S"), options(att_syntax));
// macros for start.S and other files
core::arch::global_asm!(include_str!("macros.S"), options(att_syntax));
// main file
core::arch::global_asm!(include_str!("start.S"), options(att_syntax));
// Helper functions used in start.S. Useful for debugging.
core::arch::global_asm!(include_str!("debugcon.S"), options(att_syntax));
// Strings for printing. Useful for debugging.
core::arch::global_asm!(include_str!("strings.S"), options(att_syntax));
// Static backing memory.
core::arch::global_asm!(include_str!("static_memory.S"), options(att_syntax));

// Multiboot2 header
core::arch::global_asm!(include_str!("mb2_header.S"));
