use core::fmt::Write;
use log::Record;

/// Implementation of a logger for the [`log`] crate, that writes everything to
/// QEMUs "debugcon" feature, i.e. x86 i/o-port 0xe9.
#[derive(Debug)]
pub struct QemuDebugconLogger {}

impl QemuDebugconLogger {
    const IO_PORT: u16 = 0xe9;

    pub fn new() -> Self {
        Self {}
    }
}

impl QemuDebugconLogger {
    /// Similar to [`log::Log::log`] except that it takes `&mut self`.
    /// Formats the message and writes it to the debugcon I/O port.
    pub fn log(&mut self, record: &Record) {
        let _ = writeln!(
            self,
            "[{:>5}] {:>15}@{}: {}",
            record.level(),
            record.file().unwrap_or("<unknown file>"),
            record.line().unwrap_or(0),
            record.args()
        );
    }
}

impl Write for QemuDebugconLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.as_bytes() {
            unsafe { x86::io::outb(Self::IO_PORT, *byte) };
        }
        Ok(())
    }
}
