// Disable rust standard library: will not work for several reasons:
//   1) the minimal Rust runtime is not there (similar to crt0 for C programs)
//   2) we write Kernel code, but standard lib makes syscalls and is meant for userland programs
#![no_std]
#![no_main]
// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]
// to use custom allocator
// #![feature(default_alloc_error_handler)]
// default_alloc_error_handler makes links errors ("rust_oom not found")
// We just use our own/custom error handler.
#![feature(alloc_error_handler)]
// required to access ".message()" on PanicInfo
#![feature(panic_info_message)]
#![deny(missing_debug_implementations)]

core::arch::global_asm!(include_str!("start.S"));
core::arch::global_asm!(include_str!("multiboot2_header.S"));

// ONLY USE ALLOCATIONS WHEN AN ALLOCATOR WAS SET UP!
#[allow(unused)]
#[macro_use]
extern crate alloc;

// macro use must be above other module, otherwise the macro is not available in these modules
#[macro_use]
mod panic;
mod error;
mod kernelalloc;
mod logger;
mod sysinfo;

use crate::error::BootError;
use log::LevelFilter;
use multiboot2::BootInformation as Multiboot2Info;
// use uefi::proto::console::text::Color;

/// This symbol is referenced in "start.S". It doesn't need the "pub"-keyword,
/// because visibility is a Rust feature and not important for the object file.
#[no_mangle]
fn entry_rust(multiboot2_magic: u32, multiboot2_info_ptr: u32) -> ! {
    logger::init(LevelFilter::Trace);

    const MULTIBOOT2_MAGIC: u32 = 0x36d76289;
    if multiboot2_magic != MULTIBOOT2_MAGIC {
        boot_error!(
            BootError::Multiboot2MagicWrong,
            "multiboot2 magic invalid, abort boot!"
        );
    }
    let _mb2_boot_info: Multiboot2Info = unsafe { multiboot2::load(multiboot2_info_ptr as usize) }
        .expect("MBI pointer must be valid!");

    // Make s

    // panic_error!(BootError::PanicAlloc, "foobar");
    /*x.get_cache_parameters().unwrap().for_each(|c| {
        log::debug!("{:#?}", c);
    }*/
    // log::info!("UEFI System Table: \n{:#?}", uefi_st_bs);
    // uefi_st_bs.boot_services().stall(1_000_000);
    // uefi_st_bs.runtime_services().reset(ResetType::Shutdown, Status::SUCCESS, None);
    // panic_error!(BootError::PanicAlloc, "houston, we have a (test) problem");
    //});

    // test that stack and function calling works
    let a = 5;
    let b = 4;
    let _c = add_numbers(a, b);

    loop {}
}

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[no_mangle]
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
