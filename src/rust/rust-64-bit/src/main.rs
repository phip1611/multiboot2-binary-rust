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
use core::ops::Add;

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

const HELLO_WORLD: &[u8; 11] = b"Hallo Welt\n";
#[no_mangle]
static HELLO_WORLD2: &str = "Hallo Welt\n";
const HELLO_WORLD3: [char; 11] = ['H', 'a', 'l', 'l', 'o', ' ', 'W', 'e', 'l', 't', '\n'];
const HELLO_WORLD4: *const u8 = b"Hallo Welt\n" as *const u8;

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
fn entry_64_bit() -> ! {
    // we save ebx at the beginning,
    // otherwise it is overwritten, when we call another function
    let ebx: u32;
    // todo what is the :e?
    unsafe { asm!("mov {:e}, ebx", out(reg) ebx, options(pure, nomem, nostack)) };
    unsafe { asm!("mov r12, 0xffffeeeeddddcccc") };
    unsafe { asm!("mov r13, 0x1111222233334444") };
    unsafe { asm!("mov r14, {0}", "hlt", in(reg) HELLO_WORLD4) };


    unsafe { x86::io::outb(0xe9, 'h' as u8) };
    unsafe { x86::io::outb(0xe9, 'a' as u8) };
    unsafe { x86::io::outb(0xe9, 'l' as u8) };
    unsafe { x86::io::outb(0xe9, 'l' as u8) };
    unsafe { x86::io::outb(0xe9, 'o' as u8) };
    unsafe { x86::io::outb(0xe9, 'o' as u8) };
    unsafe { x86::io::outb(0xe9, '\n' as u8) };

    // results in INVALID OPCODE?! Compiler Bug?
    /*HELLO_WORLD2.chars().for_each(|c| {
        unsafe { x86::io::outb(0xe9, c as u8) };
    });*/

    // results in weird stuff written to the port
    /*for i in 0..HELLO_WORLD.len() {
        // wtf is going on here? why this fails? this only writes "��..." into the file
        unsafe { x86::io::outb(0xe9, HELLO_WORLD[i]) };

    }*/

    for i in 0..11 {
        unsafe { x86::io::outb(0xe9, *HELLO_WORLD4.add(i)) };
    }

    let char_ptr = HELLO_WORLD2.as_ptr();
    let len = HELLO_WORLD2.len() as u64;
    // TODO WTF is going on here, why zero?
    // assert_ne!(len, 0);
    unsafe {
        asm!("mov r11, {0}", "hlt", in(reg) len);
    }
    /*for i in 0..HELLO_WORLD2.len() {
        unsafe {
            let char = *(char_ptr.add(i));
            x86::io::outb(0xe9, char);
        }
    }*/
    unsafe { x86::io::outb(0xe9, '\n' as u8) };
    unsafe { x86::io::outb(0xe9, 'm' as u8) };
    unsafe { x86::io::outb(0xe9, 'o' as u8) };
    unsafe { x86::io::outb(0xe9, 'i' as u8) };
    unsafe { x86::io::outb(0xe9, 'n' as u8) };
    unsafe { asm!("cli", "hlt") };

    // unsafe { asm!("cli", "hlt") };
    /*let boot_info = unsafe { multiboot2::load(ebx as usize) };
    let efi_img_handle = boot_info.efi_sdt_64_tag();
    if (efi_img_handle.is_some()) {
        unsafe { asm!("mov r13, 0x5555555555555") };
    } else {
        unsafe { asm!("mov r13, 0x77777777777777") };
    }
    unsafe { asm!("cli", "hlt") };



    let a = 5;
    let b = 4;
    let c = add_numbers(a, b);
    // unsafe { asm!("mov r14, {0}", in(reg) ebx)  };
    // unsafe { asm!("cli", "hlt") };



    for i in 0..HELLO_WORLD.len() {
        unsafe { x86::io::outb(0xe9, HELLO_WORLD[i]) };
    }*/

    // check what was saved in ebx register
    // unsafe { asm!("mov esi, {0}", in(reg) ebx) };

    loop {}
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // error code in all of this registers mean => Rust panic
    unsafe { asm!("mov r8, 0xDEADC0DE") };
    unsafe { asm!("mov r9, 0xDEADC0DE") };
    unsafe { asm!("mov r10, 0xDEADC0DE") };
    unsafe { asm!("mov r11, 0xDEADC0DE") };
    unsafe { asm!("mov r12, 0xDEADC0DE") };
    unsafe { asm!("mov eax, 0xDEADC0DE") };
    unsafe { asm!("mov ebx, 0xDEADC0DE") };
    unsafe { asm!("mov ecx, 0xDEADC0DE") };
    unsafe { asm!("mov edx, 0xDEADC0DE") };
    unsafe { asm!("mov edi, 0xDEADC0DE") };
    unsafe { asm!("mov esi, 0xDEADC0DE") };
    loop {
        atomic::compiler_fence(Ordering::SeqCst);
    }
}

#[no_mangle]
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
