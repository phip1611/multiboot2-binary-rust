use multiboot2::{BootInformation, EFIMemoryDesc, EFIMemoryAreaType};
use utils::convert::bytes_to_hex_ascii;
use uefi::{CStr16, Char16};
use uefi::prelude::{SystemTable, Boot};
use log::{Metadata, Record};
use core::fmt::Write;
use crate::error::BootError;

/// Implementation of a logger for the [`log`] crate, that writes everything to
/// QEMUs "debugcon" feature, i.e. x86 i/o-port 0xe9.
pub struct QemuDebugLogger {}

impl QemuDebugLogger {
    pub const fn new() -> Self {
        Self {}
    }
}

impl log::Log for QemuDebugLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let mut buf = arrayvec::ArrayString::<16384>::new();

        let res = writeln!(
            &mut buf,
            "[{:>5}] {:>15}@{}: {}",
            record.level(),
            record.file().unwrap_or("<unknown file>"),
            record.line().unwrap_or(0),
            record.args()
        );
        if let Err(e) = res {
            let mut buf = arrayvec::ArrayString::<256>::new();
            let _ = write!(buf, "QemuDebugLoggerError({}): {}", BootError::StackArrayTooSmall, e);
            qemu_debug_stdout_str("QemuDebugLogger: ");
            qemu_debug_stdout_str(buf.as_str());
            qemu_debug_stdout_str("\n");
            // panic_error!(BootError::PanicStackArrayTooSmall, "");
        }

        // in any way, write the string as far as it was formatted (even if it failed in the middle)
        qemu_debug_stdout_str(buf.as_str());

    }

    fn flush(&self) {
    }
}

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