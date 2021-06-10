// Disable rust standard library: will not work for several reasons:
//   1) the minimal Rust runtime is not there (similar to crt0 for C programs)
//   2) we write Kernel code, but standard lib makes syscalls and is meant for userland programs
#![no_std]
// enable inline assembly (new, modern asm, not legacy llvm_asm)
#![feature(asm)]

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]

mod error;
mod qemu_debug;

use core::panic::PanicInfo;
use crate::error::{bad_boot_exit, BootError};
use uefi::prelude::{SystemTable, Boot};
use core::fmt::Write;
use crate::qemu_debug::{qemu_debug_stdout_str, qemu_debug_stdout_bootinfo_to_string, qemu_debug_stdout_u8_arr, qemu_debug_stdout_c16str, qemu_debug_stdout_char_arr, qemu_debug_stdout_uefi_info};
use utils::convert::bytes_to_hex_ascii;

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

static HELLO_WORLD: &str = "Hallo Welt\n";

/// This symbol is referenced in "start.asm". It doesn't need the "pub"-keyword,
/// because visibility is a Rust feature and not important for the object file.
///
///
/// When this gets executed, we are definitely in a multiboot2 environment.
/// This is checked in the assembly. After many attempts, I decided against doing
/// this in Rust, because `eax` was always overwritten, before the initial read.
///
#[no_mangle]
fn entry_64_bit() -> ! {
    // we save ebx at the beginning,
    // otherwise it is overwritten, when we call another function
    let ebx: u32;
    // todo what is the :e?
    unsafe { asm!("mov {:e}, ebx", out(reg) ebx, options(pure, nomem, nostack)) };
    let boot_info = unsafe { multiboot2::load(ebx as usize) };
    qemu_debug_stdout_bootinfo_to_string(&boot_info);

    let uefi_system_table = boot_info.efi_sdt_64_tag();
    if uefi_system_table.is_none() {
        bad_boot_exit(BootError::MBISMissingSystemTable)
    }

    let uefi_system_table = uefi_system_table.unwrap();
    unsafe { asm!("mov r12, {0}", in(reg) uefi_system_table.sdt_address()) };
    let uefi_system_table = uefi_system_table.sdt_address() as *mut SystemTable<Boot>;
    let uefi_system_table = unsafe { uefi_system_table.as_mut().unwrap() };

    // TODO also thros general protection fault..
    uefi_system_table.runtime_services().get_time().unwrap();

    // TODO throws general protection fault!? as soon as I access any field of the uefi system table
    // qemu_debug_stdout_uefi_info(&uefi_system_table);


    let a = 5;
    let b = 4;
    let c = add_numbers(a, b);


    loop {}
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    bad_boot_exit(BootError::Panic)
}

#[no_mangle]
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
