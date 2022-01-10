use crate::UefiGopFramebuffer;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter, Write};
use kernel_lib::mutex::SimpleMutex;
use log::Record;

/// Additional vertical space between separate log messages
const LOG_SPACING: usize = 2;

/// Uses the framebuffer retrieved by UEFI GOP (Graphics Output Protocol) to draw
/// log messages to the screen.
pub struct FramebufferLogger<'a> {
    framebuffer: Arc<SimpleMutex<UefiGopFramebuffer<'a>>>,
}

impl<'a> FramebufferLogger<'a> {
    pub fn new(framebuffer: Arc<SimpleMutex<UefiGopFramebuffer<'a>>>) -> Self {
        Self { framebuffer }
    }

    /// Similar to [`log::Log::log`] except that it takes `&mut self`.
    /// Formats the message and writes it to the serial device.
    pub fn log(&mut self, record: &Record) {
        let mut framebuffer = self.framebuffer.lock();
        let _ = writeln!(
            framebuffer,
            "[{:>5}] {:>15}@{}: {}",
            record.level(),
            record.file().unwrap_or("<unknown file>"),
            record.line().unwrap_or(0),
            record.args()
        );
        framebuffer.add_vspace(LOG_SPACING);
    }
}

impl<'a> Debug for FramebufferLogger<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FramebufferLogger")
            .field("framebuffer", &self.framebuffer)
            .finish()
    }
}
