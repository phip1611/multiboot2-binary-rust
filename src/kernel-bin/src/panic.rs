use crate::error::BootError;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::sync::atomic::Ordering;
use kernel_lib::mutex::SimpleMutex;

/// Global single instance of [`BootStageAwarePanicHandler`].
pub static PANIC_HANDLER: PanicHandler = PanicHandler::new();

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    PANIC_HANDLER.handle_panic(info)
}

/// Replacement for [`panic!`] which takes a [`crate::error::BootError`] as first parameter.
/// This helps to get more contextual information, especially when a panic can only be reported
/// via registers (in early boot stage).
#[macro_export]
macro_rules! boot_error {
    ($error_enum_variant:path, $($arg:tt)*) => {
        {
            let mut lock = $crate::panic::PANIC_HANDLER.panic_error_code().lock();
            lock.replace($error_enum_variant);
        }
        panic!($($arg)*)
    };
    ($error_enum_variant:path) => {
        panic_error!($error_enum_variant, "")
    }
}

pub struct PanicHandler {
    /// Tells if there is contextual information for the panic. This is set
    /// when the [`panic_error!`]-macro is used instead of [`panic!`].
    panic_error_code: SimpleMutex<Option<BootError>>,
}

impl PanicHandler {
    const fn new() -> Self {
        Self {
            panic_error_code: SimpleMutex::new(None),
        }
    }

    /// Creates a stack allocated formatted error message from a panic info.
    fn generate_panic_msg(&self, info: &PanicInfo) -> arrayvec::ArrayString<1024> {
        let mut buf = arrayvec::ArrayString::<1024>::new();
        // if this is an error, we ignore it. It would most probably mean, that the message
        // was only created partly.
        let _ = writeln!(
            &mut buf,
            "PANIC in {}@{}:{}: {:#?}",
            info.location()
                .map(|l| l.file())
                .unwrap_or("<Unknown File>"),
            info.location().map(|l| l.line()).unwrap_or(0),
            info.location().map(|l| l.column()).unwrap_or(0),
            info.message().unwrap_or(&format_args!("")),
            // info.payload(),
        )
        .unwrap();
        buf
    }

    /// Panic handler that fulfills the contract of [`BootStageAware`]. We have a small
    /// "chicken egg" problem here, because we need "cool" logic to generate error messages
    /// etc. We just trust in case of a panic, that this code is correct. Otherwise,
    /// an error can only be found in the register specified in the function.
    ///
    /// **Rust panics are not recoverable!** The kernel halts afterwards.
    /// TODO: so far unclear how to handle this when eventually multiple cores are available
    pub fn handle_panic(&self, info: &PanicInfo) -> ! {
        // In case anything goes wrong in the panic handler (e.g. nested panic), we have
        // at least a hint in the register, that a panic occurred.
        {
            let lock = self.panic_error_code.lock();
            let error_code = lock.unwrap_or(BootError::PanicGeneric);

            // Uses the `r15` register of the boot processor (BP) to signal the specific [`BootError`].
            // This is useful, if our panic handler itself fails for example with printing an error.
            unsafe { core::arch::asm!("mov r15, {0}", in(reg) error_code.code()) };
        }

        // Make sure we print a nice error; the Logger will take care of this
        let msg = self.generate_panic_msg(info);
        // the logger implementation will log this to an appropriate place
        log::error!("{}", msg);

        // After a panic in the Rust kernel, we do not recover in any way
        // Game Over :)
        loop {
            // clear interrupts to prevent any more damage
            unsafe { core::arch::asm!("cli") };
            core::sync::atomic::compiler_fence(Ordering::SeqCst);
        }
    }

    pub fn panic_error_code(&self) -> &SimpleMutex<Option<BootError>> {
        &self.panic_error_code
    }
}
