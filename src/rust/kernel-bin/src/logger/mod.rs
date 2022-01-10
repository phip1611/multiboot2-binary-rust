use crate::logger::fb_logger::FramebufferLogger;
use crate::logger::qemu_debugcon::QemuDebugconLogger;
use crate::logger::serial::SerialLogger;
use crate::UefiGopFramebuffer;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use kernel_lib::mutex::SimpleMutex;
use log::{LevelFilter, Log, Metadata, Record};
use runs_inside_qemu::runs_inside_qemu;

mod fb_logger;
mod qemu_debugcon;
mod serial;

/// Public logger that gets used by [`log`].
pub static LOGGER: LoggerFacade = LoggerFacade::new();

/// Logger facade that glues the log log level together with
/// all possible logging implementations. Uses the [`log`]-crate
/// under the hood.
#[derive(Debug)]
pub struct LoggerFacade<'a> {
    init_done: AtomicBool,
    /// Level for log messages that get logged to screen instead of a file.
    /// Usually, we don't want to pollute the screen but keep all log messages
    /// in a file.
    screen_level: SimpleMutex<LevelFilter>,
    inner: SimpleMutex<Loggers<'a>>,
}

impl<'a> LoggerFacade<'a> {
    const fn new() -> Self {
        Self {
            init_done: AtomicBool::new(false),
            screen_level: SimpleMutex::new(LevelFilter::Trace),
            inner: SimpleMutex::new(Loggers::new()),
        }
    }

    pub fn init(&self, screen_level: LevelFilter) {
        assert!(
            !self.init_done.load(Ordering::SeqCst),
            "logger may only be initialized once!"
        );

        self.init_self(screen_level);
        self.init_generic();

        log::info!("KernelLogger init done");
    }

    pub fn init_framebuffer_logger(&self, framebuffer: Arc<SimpleMutex<UefiGopFramebuffer<'a>>>) {
        let mut inner = self.inner.lock();
        inner.init_framebuffer(FramebufferLogger::new(framebuffer))
    }

    /// Sets the level of messages that should be logged to the screen.
    pub fn set_screen_level(&self, level: LevelFilter) {
        *self.screen_level.lock() = level;
    }

    fn init_self(&self, screen_level: LevelFilter) {
        let mut inner = self.inner.lock();
        inner.init();

        self.set_screen_level(screen_level);

        self.init_done.store(true, Ordering::SeqCst);
    }

    fn init_generic(&self) {
        log::set_logger(&LOGGER).expect("logger init must happen only once");
        // by default: enable ALL levels
        // --> un-enable some fields on the logger implementation, i.e. drop several messages
        //     if the level is too unimportant for a certain logger
        log::set_max_level(LevelFilter::max());
    }
}

/// Helper struct for [`LoggerFacade`] that contains references to all
/// (possibly) existing loggers.
#[derive(Debug)]
struct Loggers<'a> {
    qemu_debugcon: Option<QemuDebugconLogger>,
    serial: Option<SerialLogger>,
    framebuffer: Option<FramebufferLogger<'a>>,
}

impl<'a> Loggers<'a> {
    const fn new() -> Self {
        Self {
            qemu_debugcon: None,
            serial: None,
            framebuffer: None,
        }
    }

    fn init(&mut self) {
        self.serial.replace(SerialLogger::new());
        if runs_inside_qemu().is_very_likely() {
            self.qemu_debugcon.replace(QemuDebugconLogger::new());
        }
    }

    fn init_framebuffer(&mut self, framebuffer: FramebufferLogger<'a>) {
        self.framebuffer.replace(framebuffer);
    }
}

impl<'a> Log for LoggerFacade<'a> {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // TODO: When is this getting called?!
        metadata.level().to_level_filter() >= *self.screen_level.lock()
    }

    fn log(&self, record: &Record) {
        let mut inner = self.inner.lock();

        // QEMU_DEBUGCON: log everything @ trace level, because I log this
        // into a file instead of polluting the screen or the framebuffer.
        if let Some(logger) = inner.qemu_debugcon.as_mut() {
            logger.log(record);
        }

        // I'm not sure about the interplay between .enabled() and log::set_max_level.
        // Actually, I want something like this: Allow all levels but I can remove some
        // levels for certain loggers.
        if record.level().to_level_filter() == LevelFilter::Trace {
            // todo use screen level property at all?!
            return;
        }

        // now only log stuff, that should not pollute the screen too much.

        if let Some(logger) = inner.serial.as_mut() {
            logger.log(record);
        }

        if let Some(logger) = inner.framebuffer.as_mut() {
            logger.log(record);
        }
    }

    fn flush(&self) {
        // no flushing yet
    }
}
