use crate::boot_stage::{BootStage, BootStageAware};
use crate::qemu_debug::QemuDebugLogger;
use crate::xuefi::UEFI_ST_BS;
use log::{LevelFilter, Log, Metadata, Record};
use runs_inside_qemu::runs_inside_qemu;
use uefi::logger::Logger as UefiLogger;
use utils::fakelock::FakeLock;
use utils::mutex::SimpleMutex;

/// Public single static logger instance. This must be global+static, because
/// that's the only way we can attach this logger to [`log::set_logger`] in `no_std` environments.
pub static LOGGER: BootStageAwareLogger = BootStageAwareLogger::new();

/// An implementation of [`log::Log`] that is aware of [`crate::boot_stage::BootStage`].
/// Dependent on the stage, it outputs the log to more/different convenient places. At the
/// very first beginning we only have registers and QEMU debug connection (I/O port 0xe9).
/// Afterwards we have the UEFI console. Eventually, we will have a fully, standalone system
/// with a display driver, which we can use.
pub struct BootStageAwareLogger {
    /// QEMU Debug port is a SimpleMutex, because we can also use this later,
    /// when we have multiple cores.
    qemu_debug_port: SimpleMutex<Option<QemuDebugLogger>>,
    /// The UEFI console will only be available to us, before we exited UEFI boot services.
    /// Therefore we only have a single core and the FakeLock is fine.
    uefi_console: FakeLock<Option<UefiLogger>>,
}

impl BootStageAwareLogger {
    const fn new() -> Self {
        Self {
            qemu_debug_port: SimpleMutex::new(None),
            uefi_console: FakeLock::new(None),
        }
    }

    /// Applies an action to all enabled loggers.
    fn apply_to_each(&self, action: &dyn Fn(&dyn Log)) {
        if let Some(l) = self.qemu_debug_port.lock().as_ref() {
            action(l);
        }
        if let Some(l) = self.uefi_console.get().as_ref() {
            action(l);
        }
    }

    pub fn enable_qemu_debug_port_logger(&self) {
        let mut lock = self.qemu_debug_port.lock();
        let logger = QemuDebugLogger::new();
        lock.replace(logger);
    }

    #[allow(unused)]
    pub fn disable_qemu_debug_port_logger(&self) {
        let mut lock = self.qemu_debug_port.lock();
        lock.take()
            .expect("You can only disable the logger if it is enabled!");
    }

    pub fn enable_uefi_console_logger(&self) {
        let uefi_st = UEFI_ST_BS.get_mut().as_mut().unwrap();
        let lock = self.uefi_console.get_mut();
        // let logger = unsafe { UefiLogger::new(uefi_st.stderr()) };
        let logger = unsafe { UefiLogger::new(uefi_st.stdout()) };
        lock.replace(logger);
    }

    pub fn disable_uefi_console_logger(&self) {
        let lock = self.uefi_console.get_mut();
        lock.take()
            .expect("You can only disable the logger if it is enabled!");
    }
}

impl Log for BootStageAwareLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // TODO decide later when we don't want to log everything
        //  maybe at some point in boot we only want to log critical errors by default
        true
        // log::max_level() >= metadata.level()
    }

    fn log(&self, record: &Record) {
        self.apply_to_each(&|l| l.log(record));
    }

    fn flush(&self) {
        self.apply_to_each(&|l| l.flush());
    }
}

impl BootStageAware for BootStageAwareLogger {
    fn next_boot_stage(&self, boot_stage: BootStage) {
        match boot_stage {
            // Logging to QEMU debug I/O port only
            BootStage::S0_Initial => {
                if runs_inside_qemu().is_very_likely() {
                    self.enable_qemu_debug_port_logger();
                }
                // enable all log levels, otherwise things may not printed
                // logger implementations still can restrict what they print
                log::set_max_level(LevelFilter::Trace);

                // enable this once if in QEMU and leave it on the whole time
                log::set_logger(&LOGGER).expect("Should be called once!");
            }
            BootStage::S1_MB2Handoff => {}
            // Also log to UEFI console.
            BootStage::S2_UEFIBootServices => {
                self.enable_uefi_console_logger();
            }
            // Stop logging to UEFI console, since it isn't available anymore.
            BootStage::S3_UEFIRuntimeServices => {
                self.disable_uefi_console_logger();
            }
        }
    }
}
