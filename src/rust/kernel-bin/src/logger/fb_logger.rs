use crate::UefiGopFramebuffer;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter, Write};
use kernel_lib::fakelock::FakeLock;
use log::Record;

/// Uses the framebuffer retrieved by UEFI GOP (Graphics Output Protocol) to draw
/// log messages to the screen.
pub struct FramebufferLogger<'a> {
    // framebuffer: Arc<SimpleMutex<UefiGopFramebuffer<'a>>>,
    framebuffer: Arc<FakeLock<UefiGopFramebuffer<'a>>>,
}

impl<'a> FramebufferLogger<'a> {
    pub fn new(framebuffer: Arc<FakeLock<UefiGopFramebuffer<'a>>>) -> Self {
        Self { framebuffer }
    }

    /// Similar to [`log::Log::log`] except that it takes `&mut self`.
    /// Formats the message and writes it to the serial device.
    pub fn log(&mut self, record: &Record) {
        // let mut framebuffer = self.framebuffer.lock();
        let mut framebuffer = self.framebuffer.get_mut();
        let _ = writeln!(
            framebuffer,
            "[{:>5}] {:>15}@{}: {}",
            record.level(),
            record.file().unwrap_or("<unknown file>"),
            record.line().unwrap_or(0),
            record.args()
        );
    }
}

impl<'a> Debug for FramebufferLogger<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FramebufferLogger")
            .field("framebuffer", &self.framebuffer)
            .finish()
    }
}
