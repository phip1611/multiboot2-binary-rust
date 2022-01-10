use crate::logger::qemu_debugcon::QemuDebugconLogger;
use crate::logger::serial::SerialLogger;
use core::sync::atomic::{AtomicBool, Ordering};
use kernel_lib::mutex::SimpleMutex;
use log::{LevelFilter, Log, Metadata, Record};
use runs_inside_qemu::runs_inside_qemu;

mod qemu_debugcon;
mod serial;

/// Public logger that gets used by [`log`].
static LOGGER: LoggerFacade = LoggerFacade::new();

pub fn init(level: LevelFilter) {
    LOGGER.init(level);
    log::set_logger(&LOGGER).expect("logger init must happen only once");
    log::set_max_level(LevelFilter::max());
    log::info!("KernelLogger init done");
}

/// Logger facade that glues the log log level together with
/// all possible logging implementations. Uses the [`log`]-crate
/// under the hood.
#[derive(Debug)]
pub struct LoggerFacade {
    init_done: AtomicBool,
    level: SimpleMutex<LevelFilter>,
    inner: SimpleMutex<Loggers>,
}

impl LoggerFacade {
    const fn new() -> Self {
        Self {
            init_done: AtomicBool::new(false),
            level: SimpleMutex::new(LevelFilter::Trace),
            inner: SimpleMutex::new(Loggers::new()),
        }
    }

    fn init(&self, level: LevelFilter) {
        assert!(
            !self.init_done.load(Ordering::SeqCst),
            "logger may only be initialized once!"
        );

        let mut inner = self.inner.lock();
        inner.init();

        *self.level.lock() = level;

        self.init_done.store(true, Ordering::SeqCst);
    }
}

/// Helper struct for [`LoggerFacade`] that contains references to all
/// (possibly) existing loggers.
#[derive(Debug)]
struct Loggers {
    qemu_debugcon: Option<QemuDebugconLogger>,
    serial: Option<SerialLogger>,
}

impl Loggers {
    const fn new() -> Self {
        Self {
            qemu_debugcon: None,
            serial: None,
        }
    }

    fn init(&mut self) {
        self.serial.replace(SerialLogger::new());
        if runs_inside_qemu().is_very_likely() {
            self.qemu_debugcon.replace(QemuDebugconLogger::new());
        }
    }
}

impl Log for LoggerFacade {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level().to_level_filter() <= *self.level.lock()
    }

    fn log(&self, record: &Record) {
        let mut inner = self.inner.lock();

        if let Some(logger) = inner.qemu_debugcon.as_mut() {
            logger.log(record);
        }
        if let Some(logger) = inner.serial.as_mut() {
            logger.log(record);
        }
    }

    fn flush(&self) {
        // no flushing yet
    }
}
