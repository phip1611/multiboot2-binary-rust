// Disable rust standard library: will not work for several reasons:
//   1) the minimal Rust runtime is not there (similar to crt0 for C programs)
//   2) we write Kernel code, but standard lib makes syscalls and is meant for userland programs
#![no_std]
#![no_main]
// enable inline assembly (new, modern asm, not legacy llvm_asm)
#![feature(asm)]

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#![feature(lang_items)]

// to use custom allocator
// #![feature(default_alloc_error_handler)]
// default_alloc_error_handler makes links errors ("rust_oom not found")
// We just use our own/custom error handler.
#![feature(alloc_error_handler)]

// required to access ".message()" on PanicInfo
#![feature(panic_info_message)]

// required to include "global assembler", i.e. include
// "object files by assembly source"
// it will compile these as GAS (GNU Assembly)
#![feature(global_asm)]

global_asm!(include_str!("start.S"));
global_asm!(include_str!("multiboot2_header.S"));

// ONLY USE ALLOCATIONS WHEN AN ALLOCATOR WAS SET UP!
#[macro_use]
extern crate alloc;

// macro use must be above other module, otherwise the macro is not available in these modules
#[macro_use]
mod panic;

mod error;
mod qemu_debug;
mod boot_stage;
mod logger;
mod xuefi;
mod kernelalloc;
mod mb2;
mod sysinfo;

use uefi::prelude::{SystemTable, Boot};
use crate::boot_stage::BootStage;
use crate::xuefi::UEFI_ST_BS;
use crate::error::BootError;
use multiboot2::BootInformation as Multiboot2Info;
use crate::mb2::MULTIBOOT2_INFO_STRUCTURE;
use uefi::{Event, Status};
use uefi::table::runtime::ResetType;
use crate::sysinfo::SysInfo;
// use uefi::proto::console::text::Color;

/// This symbol is referenced in "start.S". It doesn't need the "pub"-keyword,
/// because visibility is a Rust feature and not important for the object file.
#[no_mangle]
fn entry_64_bit(eax: u32, ebx: u32) -> ! {
    // Make sure all "BootStageAware"-compatible structs get their right initial state
    BootStage::S0_Initial.enter(&|| {
        // MUST BE EMPTY, because everything must get in init state first (e.g. QEMU logger)
    });
    // ############################################################################################
    BootStage::S1_MB2Handoff.enter(&|| {
        const MULTIBOOT2_MAGIC: u32 = 0x36d76289;
        if eax != MULTIBOOT2_MAGIC {
            panic_error!(BootError::Multiboot2MagicWrong, "multiboot2 magic invalid, abort boot!");
        }

        let mb2_boot_info: Multiboot2Info = unsafe { multiboot2::load(ebx as usize) }.expect("Couldn't load MBI");

        let lock = MULTIBOOT2_INFO_STRUCTURE.get_mut();
        lock.replace(mb2_boot_info)
    });
    // ############################################################################################
    BootStage::S2_UEFIBootServices.enter(& || {
        let mb2_boot_info = MULTIBOOT2_INFO_STRUCTURE.get().as_ref().unwrap();
        let uefi_system_table = mb2_boot_info.efi_sdt_64_tag();
        if uefi_system_table.is_none() {
            panic_error!(BootError::PanicMBISUefiSystemTableMissing, "UEFI System table is not present!");
        }
        let uefi_system_table = uefi_system_table.unwrap();

        // It's important that we "own" the system table here and not have a pointer to it.
        // Otherwise there would be a double-dereference which causes errors.
        let mut uefi_system_table: SystemTable<Boot> = unsafe { core::mem::transmute( uefi_system_table.sdt_address()) };

        // prepare screen
        uefi_system_table.stdout().clear().unwrap();
        /*uefi_system_table.stdout().set_color(
            Color::Yellow,
            Color::Black,
        );*/

        // set the global instance
        UEFI_ST_BS.get_mut().replace(uefi_system_table);

        log::info!("Entering UEFI boot service stage now");
    });
    // ############################################################################################
    BootStage::S3_UEFIRuntimeServices.enter(&|| {
        log::info!("Entered UEFI boot service stage");
        let uefi_st_bs = UEFI_ST_BS.get().as_ref().unwrap();
        // log::info!("UEFI System Table (Boot Services enabled):\n{:#?}", uefi_st_bs);

        if runs_inside_qemu::runs_inside_qemu() {
            //log::info!("We run inside QEMU :)");
        } else {
            //log::info!("We don't run in QEMU :O");
        }

        // test heap allocation works
        /*let mut vec = vec![1, 2, 3];
        vec.push(4);
        log::debug!("{:#?}", vec);*/

        // log::debug!("{:#?}", SysInfo::new(uefi_st_bs));
        // log::debug!("{:#?}", raw_cpuid::CpuId::new());
        //log::debug!("Time {:#?}", uefi_st_bs.runtime_services().get_time());
        // let x = raw_cpuid::CpuId::new();
        // log::debug!("{:#?}", x.get_hypervisor_info().unwrap().identify());

        // panic_error!(BootError::PanicAlloc, "foobar");
        /*x.get_cache_parameters().unwrap().for_each(|c| {
            log::debug!("{:#?}", c);
        }*/
        // log::info!("UEFI System Table: \n{:#?}", uefi_st_bs);
        // uefi_st_bs.boot_services().stall(1_000_000);
        // uefi_st_bs.runtime_services().reset(ResetType::Shutdown, Status::SUCCESS, None);
        // panic_error!(BootError::PanicAlloc, "houston, we have a (test) problem");
    });

    // test that stack and function calling works
    let a = 5;
    let b = 4;
    let _c = add_numbers(a, b);

    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(_l: core::alloc::Layout) -> ! {
    loop {}
}

// see https://docs.rust-embedded.org/embedonomicon/smallest-no-std.html
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[no_mangle]
fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
