// disable rust standard library (which could lead to illegal syscalls)
#![no_std]
// disables Rust runtime init,
#![no_main]
// enable inline assembly (new, modern asm, not legacy llvm_asm)
#![feature(asm)]
// needed temporarily
#![feature(llvm_asm)]

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]

use core::panic::PanicInfo;
use core::sync::atomic;
use core::sync::atomic::Ordering;
use multiboot2::{EFIMemoryAreaType, load};

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

const HELLO_WORLD: &[u8; 10] = b"Hallo Welt";
const COUNTER_TEST: usize = 0;

/// This symbol is referenced in "start.asm". It doesn't need the "pub"-keyword,
/// because visibility is a Rust feature and not important for the object file.
///
/// When this gets executed, we are definitely in a multiboot2 environment.
/// This is checked in the assambly. After many attempts, I decided against doing
/// this in Rust, because `eax` was always overwritten, before the initial read.
///
/// I don't understand why but this symbol has to be exactly named `_start`,
/// otherwise the output is empty (discarded by rustc). I don't know why this is so.
/// TODO, check where this comes from!
#[no_mangle]
fn _start() -> ! {
    let ebx: u32;
    unsafe { asm!("", out("ebx") ebx) };

    let a = 5;
    let b = 4;
    let c = add_numbers(a, b);


    /*unsafe {
        x86::io::outb(0xe9, b'h');
        x86::io::outb(0xe9, b'e');
        x86::io::outb(0xe9, b'l');
        x86::io::outb(0xe9, b'l');
        x86::io::outb(0xe9, b'o');
        x86::io::outb(0xe9, b' ');
        x86::io::outb(0xe9, b'f');
        x86::io::outb(0xe9, b'r');
        x86::io::outb(0xe9, b'o');
        x86::io::outb(0xe9, b'm');
        x86::io::outb(0xe9, b' ');
        x86::io::outb(0xe9, b'b');
        x86::io::outb(0xe9, b'i');
        x86::io::outb(0xe9, b'n');
        x86::io::outb(0xe9, b'a');
        x86::io::outb(0xe9, b'r');
        x86::io::outb(0xe9, b'y');
        x86::io::outb(0xe9, b'\n');
        x86::io::outb(0xe9, b'\0');
    }*/

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
