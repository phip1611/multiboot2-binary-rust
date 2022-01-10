use core::fmt::{Debug, Formatter, Write};
use log::Record;
use uart_16550::SerialPort;

/// Implementation of a logger for the [`log`] crate, that writes everything to
/// the serial device of the platform. This device is also called "COM1" and is a
/// 16550 UART device behind I/O port 0x38f under the hood on almost any mainboard
/// by convention.
pub struct SerialLogger {
    port: SerialPort,
}

impl SerialLogger {
    const IO_PORT: u16 = 0x3f8;

    pub fn new() -> Self {
        let mut port = unsafe { SerialPort::new(Self::IO_PORT) };
        port.init();
        Self { port }
    }

    /// Similar to [`log::Log::log`] except that it takes `&mut self`.
    /// Formats the message and writes it to the serial device.
    pub fn log(&mut self, record: &Record) {
        let _ = writeln!(
            &mut self.port,
            "[{:>5}] {:>15}@{}: {}",
            record.level(),
            record.file().unwrap_or("<unknown file>"),
            record.line().unwrap_or(0),
            record.args()
        );
    }
}

impl Debug for SerialLogger {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SerialLogger")
            .field("port", &(Self::IO_PORT))
            .finish()
    }
}
