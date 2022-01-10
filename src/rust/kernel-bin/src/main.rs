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
mod kernelheap;
mod logger;
mod sysinfo;
mod uefi_gop_fb;

use crate::error::BootError;
use crate::logger::LOGGER;
use crate::sysinfo::SysInfo;
use crate::uefi_gop_fb::UefiGopFramebuffer;
use core::{mem, slice};
use log::LevelFilter;
use multiboot2::{BootInformation as Multiboot2Info, MbiLoadError};
use uefi::prelude::Boot;
use uefi::table::boot::{MemoryDescriptor, MemoryType};
use uefi::table::{Runtime, SystemTable};
use uefi::Handle;
// use uefi::proto::console::text::Color;

/// This symbol is referenced in "start.S". It doesn't need the "pub"-keyword,
/// because visibility is a Rust feature and not important for the object file.
#[no_mangle]
fn entry_rust(multiboot2_magic: u32, multiboot2_info_ptr: u32) -> ! {
    // Error, Warn, Info, Debug -> Log to screen
    // everything + Trace -> Log only to file
    LOGGER.init(LevelFilter::Debug);
    kernelheap::init();

    let multiboot2_info = get_multiboot2_info(multiboot2_magic, multiboot2_info_ptr)
        .expect("Multiboot2 information structure pointer must be valid!");
    log::info!("Valid Multiboot2 boot.");

    let (uefi_boot_system_table, uefi_image_handle) = get_uefi_info(&multiboot2_info)
        .expect("Can't fetch UEFI system table and UEFI image handle.");
    log::info!("UEFI system table and UEFI image handle valid.");

    let uefi_fb =
        UefiGopFramebuffer::new(&uefi_boot_system_table).expect("No Framebuffer available!");
    LOGGER.init_framebuffer_logger(uefi_fb.clone());

    let (uefi_rt_system_table, _uefi_memory_map) =
        exit_uefi_boot_services(uefi_boot_system_table, uefi_image_handle)
            .expect("Exit UEFI boot services failed.");

    log::info!("UEFI boot services exited");

    if runs_inside_qemu::runs_inside_qemu().is_very_likely() {
        log::info!("We run inside QEMU :)");
    } else {
        log::info!("We don't run in QEMU :O");
    }

    let sysinfo = SysInfo::new(&uefi_rt_system_table, &x86::cpuid::CpuId::new());
    log::debug!("CPU: {:#?}", sysinfo.cpu_info().extended_brand_string());
    log::debug!(
        "Caches: {:#?}",
        sysinfo
            .cpu_info()
            .cache_descriptions()
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<alloc::vec::Vec<_>>()
    );

    loop {}
}

/// Returns [`Multiboot2Info`] or dies/panics.
fn get_multiboot2_info(
    multiboot2_magic: u32,
    multiboot2_info_ptr: u32,
) -> Result<Multiboot2Info, MbiLoadError> {
    const MULTIBOOT2_MAGIC: u32 = 0x36d76289;
    if multiboot2_magic != MULTIBOOT2_MAGIC {
        boot_error!(
            BootError::Multiboot2MagicWrong,
            "multiboot2 magic invalid, abort boot!"
        );
    }
    unsafe { multiboot2::load(multiboot2_info_ptr as usize) }
}

/// Returns a pair of the UEFI system table with boot services enabled and the UEFI
/// image handle.
fn get_uefi_info(info: &Multiboot2Info) -> Result<(SystemTable<Boot>, Handle), ()> {
    let handle = info.efi_64_ih().ok_or(())?.image_handle() as *mut _;
    let handle = unsafe { Handle::from_ptr(handle) }.ok_or(())?;

    let table = info.efi_sdt_64_tag().ok_or(())?.sdt_address() as *mut _;
    let table = unsafe { SystemTable::<Boot>::from_ptr(table) }.ok_or(())?;

    Ok((table, handle))
}

fn exit_uefi_boot_services<'a>(
    table: SystemTable<Boot>,
    handle: Handle,
) -> Result<(SystemTable<Runtime>, &'a mut [u8]), ()> {
    let mmap_storage = {
        let max_mmap_size = table.boot_services().memory_map_size().map_size
            + 8 * mem::size_of::<MemoryDescriptor>();
        let ptr = table
            .boot_services()
            .allocate_pool(MemoryType::LOADER_DATA, max_mmap_size)
            .map_err(|_| ())?
            .log();
        unsafe { slice::from_raw_parts_mut(ptr, max_mmap_size) }
    };

    let uefi_rt_system_table = table
        .exit_boot_services(handle, mmap_storage)
        .unwrap()
        .unwrap()
        .0;

    Ok((uefi_rt_system_table, mmap_storage))
}

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[no_mangle]
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
