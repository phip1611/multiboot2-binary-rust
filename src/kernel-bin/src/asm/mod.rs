//! This module includes all my global assembly files. Rust uses a GAS-like (GNU Assembly) syntax
//! for global and inline assembler. This is documented here:
//! https://doc.rust-lang.org/nightly/reference/inline-assembly.html
//!
//! I prefer att_syntax in prefix mode as it is clearer. It doesn't look so nice but it's always
//! clear whether something is an immediate or an address. Additionally, there is more existing
//! assembly code available in AT&T syntax for comparison.

core::arch::global_asm!(include_str!("macros.S"), options(att_syntax));
core::arch::global_asm!(include_str!("boot_stack.S"), options(att_syntax));
#[rustfmt::skip]
core::arch::global_asm!(include_str!("mb2_start.S"), options(att_syntax));
core::arch::global_asm!(include_str!("mb2_header.S"));
core::arch::global_asm!(include_str!("strings.S"), options(att_syntax));
core::arch::global_asm!(include_str!("qemu_debugcon.S"), options(att_syntax));
