// Disable rust standard library: will not work for several reasons:
//   1) the minimal Rust runtime is not there (similar to crt0 for C programs)
//   2) we write Kernel code, but standard lib makes syscalls and is meant for userland programs
#![no_std]
// disables Rust runtime init,
#![no_main]
// enable inline assembly (new, modern asm, not legacy llvm_asm)
#![feature(asm)]

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]

use core::panic::PanicInfo;
use core::sync::atomic;
use core::sync::atomic::Ordering;

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

const HELLO_WORLD: &[u8; 12] = b"Hallo Welt\n\0";

/// This symbol is referenced in "start.asm". It doesn't need the "pub"-keyword,
/// because visibility is a Rust feature and not important for the object file.
///
/// This symbol is also specified as the entry symbol in `pre-link-args`
/// in `x86_64-none-bare_metal.json`. This is necessary, because otherwise
/// the linker throws away everything because it assumes all code is dead.
///
/// When this gets executed, we are definitely in a multiboot2 environment.
/// This is checked in the assambly. After many attempts, I decided against doing
/// this in Rust, because `eax` was always overwritten, before the initial read.
///
#[no_mangle]
fn entry_32_bit() -> ! {
    // we save ebx at the beginning,
    // otherwise it is overwritten, when we call another function
    let ebx: u32;
    unsafe { asm!("", out("ebx") ebx) };

    let a = 5;
    let b = 4;
    let _c = add_numbers(a, b);

    /*for i in 0..HELLO_WORLD.len() {
        unsafe { x86::io::outb(0xe9, HELLO_WORLD[i]) };
    }*/

    // check what was saved in ebx register
    // unsafe { asm!("mov esi, {0}", in(reg) ebx) };

    loop {}
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

#[no_mangle]
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
