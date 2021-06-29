use derive_more::Display;

/// If these errors occur very soon in the boot process,
/// these errors are stored in multiple registers.
#[derive(Debug, Copy, Clone, Display)]
#[allow(dead_code)]
#[repr(u32)]
pub enum BootError {
    Multiboot2MagicWrong = 0x00000000,
    /// Happens when a stack array allocated is not large enough. We rely on a big enough buffer
    /// in several panic and logging utilities.
    StackArrayTooSmall = 0x00000001,

    ///
    PanicGeneric = 0xe0000000,
    PanicAlloc = 0xe0000001,
    PanicDealloc = 0xe0000002,
    /// Multiboot2 information structure (passed via `ebx` register) doesn't contain UEFI system table.
    PanicMBISUefiSystemTableMissing = 0xe0000003,

    Other = 0xffffffff,
}

impl BootError {
    pub fn code(self) -> u64 {
        0x0bad_b001 << 32 | (self as u64)
    }
}

// tests don't work so far
/*#[cfg(test)]
mod tests {
    use crate::BootError;

    #[test]
    fn test() {
        assert_eq!(0x0bad_b001_00000001, BootError::MBISMissingSystemTable.code());
        assert_eq!(0x0bad_b001_ffffffff, BootError::Other.code());
    }
}*/