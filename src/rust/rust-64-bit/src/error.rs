use core::sync::atomic;
use core::sync::atomic::Ordering;
use utils::convert::bytes_to_hex_ascii;
use crate::qemu_debug::{qemu_debug_stdout_char_arr, qemu_debug_stdout_str};

/// If these errors occur very soon in the boot process,
/// these errors are stored in multiple registers.
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum BootError {
    /// Multiboot2 information structure (passed via `ebx` register) doesn't contain UEFI system table.
    MBISMissingSystemTable = 0x00000001,
    /// A Rust panic
    Panic = 0xefffffff,
    Other = 0xffffffff,
}

impl BootError {
    pub fn code(self) -> u64 {
        0x0bad_b001 << 32 | (self as u64)
    }
    pub fn qemu_print(self) {
        let code = self.code();
        let msg = "BOOT ERROR: 0x";
        let chars = bytes_to_hex_ascii::<16>(&code.to_be_bytes());
        qemu_debug_stdout_str(msg);
        qemu_debug_stdout_char_arr(&chars);
        qemu_debug_stdout_char_arr(&['\n']);
    }
}


pub fn bad_boot_exit(err: BootError) -> ! {
    // error code in all of this registers mean => Rust panic
    unsafe { asm!("mov r8, {0}", in(reg) err.code()) };
    unsafe { asm!("mov r9, {0}", in(reg) err.code()) };
    unsafe { asm!("mov r10, {0}", in(reg) err.code()) };
    unsafe { asm!("mov r11, {0}", in(reg) err.code()) };
    unsafe { asm!("mov r12, {0}", in(reg) err.code()) };
    unsafe { asm!("mov rax, {0}", in(reg) err.code()) };
    unsafe { asm!("mov rbx, {0}", in(reg) err.code()) };
    unsafe { asm!("mov rcx, {0}", in(reg) err.code()) };
    unsafe { asm!("mov rdx, {0}", in(reg) err.code()) };
    unsafe { asm!("mov rdi, {0}", in(reg) err.code()) };
    unsafe { asm!("mov rsi, {0}", in(reg) err.code()) };

    loop {
        atomic::compiler_fence(Ordering::SeqCst);
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