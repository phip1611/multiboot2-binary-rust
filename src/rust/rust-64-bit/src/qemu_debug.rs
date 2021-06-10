use crate::error::BootError;
use multiboot2::{BootInformation, EFIImageHandle32, MemoryArea, EFIMemoryDesc, EFIMemoryAreaType, ElfSection};
use utils::convert::bytes_to_hex_ascii;
use uefi::{CStr16, Char16};
use uefi::prelude::{SystemTable, Boot};

pub fn qemu_debug_stdout_str(msg: &str) {
    qemu_debug_stdout_u8_arr(msg.as_bytes());
}

pub fn qemu_debug_stdout_c16str(msg: &CStr16) {
    msg.iter()
        .for_each(|c: &Char16| {
            let val: u16 = (*c).into();
            qemu_debug_stdout_u8_arr(&val.to_be_bytes());
    });
}

/// Assumes that the output is valid ASCII.
/// Data is not transformed to ASCII.
pub fn qemu_debug_stdout_u8_arr(bytes: &[u8]) {
    for byte in bytes {
        unsafe { x86::io::outb(0xe9, *byte) };
    }
}

pub fn qemu_debug_stdout_char_arr(chars: &[char]) {
    for char in chars {
        unsafe { x86::io::outb(0xe9, *char as u8) };
    }
}

pub fn qemu_debug_stdout_bootinfo_to_string(info: &BootInformation) {
    qemu_debug_stdout_str("Multiboot2BootInformation {\n");

    {
        qemu_debug_stdout_str("  boot_loader_name_tag: ");
        if let Some(s) = info.boot_loader_name_tag() {
            qemu_debug_stdout_str(s.name());
        } else {
            qemu_debug_stdout_str("<none>");
        }
        qemu_debug_stdout_str("\n");
    }

    {
        qemu_debug_stdout_str("  efi_64_ih: ");
        if let Some(s) = info.efi_64_ih() {
            qemu_debug_stdout_str("<present>");
            /*let multiboot2::EFIImageHandle64 {pointer, ..} = s;
            qemu_debug_stdout_str("0x");
            qemu_debug_stdout_char_arr(
                &bytes_to_hex_ascii::<16>(&pointer.to_be_bytes())
            )*/
        } else {
            qemu_debug_stdout_str("<none>");
        }
        qemu_debug_stdout_str("\n");
    }

    {
        qemu_debug_stdout_str("  efi_std_64_ih: ");
        if let Some(s) = info.efi_sdt_64_tag() {
            let chars = bytes_to_hex_ascii::<8>((s.sdt_address() as u32).to_be_bytes().as_ref());
            qemu_debug_stdout_str("0x");
            qemu_debug_stdout_char_arr(&chars);
        } else {
            qemu_debug_stdout_str("<none>");
        }
        qemu_debug_stdout_str("\n");
    }

    {
        qemu_debug_stdout_str("  command_line_tag: ");
        if let Some(s) = info.command_line_tag() {
            qemu_debug_stdout_str("'");
            qemu_debug_stdout_str(s.command_line());
            qemu_debug_stdout_str("'");
        } else {
            qemu_debug_stdout_str("<none>");
        }
        qemu_debug_stdout_str("\n");
    }

    {
        qemu_debug_stdout_str("  efi_memory_map_tag: ");
        if let Some(s) = info.efi_memory_map_tag() {
            qemu_debug_stdout_str("\n    memory areas:");
            s.memory_areas().for_each(|ma: &EFIMemoryDesc| {
                let phys_addr_chars = bytes_to_hex_ascii::<16>(&ma.physical_address().to_be_bytes());
                let virt_addr_chars = bytes_to_hex_ascii::<16>(&ma.virtual_address().to_be_bytes());
                let size_chars = bytes_to_hex_ascii::<16>(&ma.size().to_be_bytes());

                qemu_debug_stdout_str("      physical: 0x");
                qemu_debug_stdout_char_arr(&phys_addr_chars);
                qemu_debug_stdout_str("\n      virtual: 0x");
                qemu_debug_stdout_char_arr(&virt_addr_chars);
                qemu_debug_stdout_str("\n      size: 0x");
                qemu_debug_stdout_char_arr(&size_chars);
                qemu_debug_stdout_str("\n      typ: ");
                let type_str = match ma.typ() {
                    EFIMemoryAreaType::EfiReservedMemoryType => { "EfiReservedMemoryType" }
                    EFIMemoryAreaType::EfiLoaderCode => { "EfiLoaderCode" }
                    EFIMemoryAreaType::EfiLoaderData => { "EfiLoaderData" }
                    EFIMemoryAreaType::EfiBootServicesCode => { "EfiBootServicesCode" }
                    EFIMemoryAreaType::EfiBootServicesData => { "EfiBootServicesData" }
                    EFIMemoryAreaType::EfiRuntimeServicesCode => { "EfiRuntimeServicesCode" }
                    EFIMemoryAreaType::EfiRuntimeServicesData => { "EfiRuntimeServicesData" }
                    EFIMemoryAreaType::EfiConventionalMemory => { "EfiConventionalMemory" }
                    EFIMemoryAreaType::EfiUnusableMemory => { "EfiUnusableMemory" }
                    EFIMemoryAreaType::EfiACPIReclaimMemory => { "EfiACPIReclaimMemory" }
                    EFIMemoryAreaType::EfiACPIMemoryNVS => { "EfiACPIMemoryNVS" }
                    EFIMemoryAreaType::EfiMemoryMappedIO => { "EfiMemoryMappedIO" }
                    EFIMemoryAreaType::EfiMemoryMappedIOPortSpace => { "EfiMemoryMappedIOPortSpace" }
                    EFIMemoryAreaType::EfiPalCode => { "EfiPalCode" }
                    EFIMemoryAreaType::EfiPersistentMemory => { "EfiPersistentMemory" }
                    EFIMemoryAreaType::EfiUnknown => { "EfiUnknown" }
                };
                qemu_debug_stdout_str(type_str);
                qemu_debug_stdout_str("\n");
            });
        } else {
            qemu_debug_stdout_str("<none>");
        }
        qemu_debug_stdout_str("\n");
    }

    {
        qemu_debug_stdout_str("  elf_sections_tag: ");
        if let Some(s) = info.elf_sections_tag() {
            qemu_debug_stdout_str("<present>"); // (hundreds of lines, not relevant)
            /*s.sections().for_each(|s: ElfSection| {
                qemu_debug_stdout_str("    name: ");
                qemu_debug_stdout_str(s.name());
                qemu_debug_stdout_str("\n");
            });*/
        } else {
            qemu_debug_stdout_str("<none>");
        }
        qemu_debug_stdout_str("\n");
    }

    {
        qemu_debug_stdout_str("  memory_map_tag: ");
        if let Some(s) = info.memory_map_tag() {
            qemu_debug_stdout_str("<present>"); // (hundreds of lines, not relevant)
            /*s.sections().for_each(|s: ElfSection| {
                qemu_debug_stdout_str("    name: ");
                qemu_debug_stdout_str(s.name());
                qemu_debug_stdout_str("\n");
            });*/
        } else {
            qemu_debug_stdout_str("<none>");
        }
        qemu_debug_stdout_str("\n");
    }

    {
        qemu_debug_stdout_str("  framebuffer: ");
        if let Some(s) = info.framebuffer_tag() {
            qemu_debug_stdout_str("<present>");
        } else {
            qemu_debug_stdout_str("<none>");
        }
        qemu_debug_stdout_str("\n");
    }

    {
        qemu_debug_stdout_str("  rdsp_v1: \n");
        if let Some(s) = info.rsdp_v1_tag() {
            let addr_hex = bytes_to_hex_ascii::<8>(&(s.rsdt_address() as u32).to_be_bytes());
            qemu_debug_stdout_str("    rsdt addr (phys): 0x");
            qemu_debug_stdout_char_arr(&addr_hex);
            qemu_debug_stdout_str("\n    signature: ");
            qemu_debug_stdout_str(s.signature().unwrap_or("<none>"));
        } else {
            qemu_debug_stdout_str("<none>");
        }
        qemu_debug_stdout_str("\n");
    }

    {
        qemu_debug_stdout_str("  rdsp_v2: \n");
        if let Some(s) = info.rsdp_v2_tag() {
            let addr_hex = bytes_to_hex_ascii::<8>(&(s.xsdt_address() as u32).to_be_bytes());
            qemu_debug_stdout_str("    xsdt addr (phys): 0x");
            qemu_debug_stdout_char_arr(&addr_hex);
            qemu_debug_stdout_str("\n    signature: ");
            qemu_debug_stdout_str(s.signature().unwrap_or("<none>"));
        } else {
            qemu_debug_stdout_str("<none>");
        }
        qemu_debug_stdout_str("\n");
    }

    qemu_debug_stdout_str("}\n");
}

pub fn qemu_debug_stdout_uefi_info(st: &SystemTable<Boot>) {
    qemu_debug_stdout_str("UEFI Firmware Vendor: ");
    qemu_debug_stdout_c16str(&st.firmware_vendor());
    qemu_debug_stdout_str("\nUEFI Firmware Version: ");
    qemu_debug_stdout_char_arr(
        bytes_to_hex_ascii::<4>(
            &st.firmware_revision().major().to_be_bytes()
        ).as_ref()
    );
    qemu_debug_stdout_str(".");
    qemu_debug_stdout_char_arr(
        bytes_to_hex_ascii::<4>(
            &st.firmware_revision().minor().to_be_bytes()
        ).as_ref()
    );
    qemu_debug_stdout_str("\n");
}

// tests don't work so far
/*#[cfg(test)]
mod tests {
    use crate::BootError;
    use super::*;
    use alloc::string::String;
    use core::iter::FromIterator;

    #[test]
    fn test_u64_to_hex_ascii() {
        assert_eq!("123456789abcdef0", String::from_iter(u64_to_hex_ascii(0x123456789abcdef0).iter()));
        assert_eq!("0fedcba987654321", String::from_iter(u64_to_hex_ascii(0x0fedcba987654321).iter()));
    }
}*/