use multiboot2::BootInformation;
use utils::fakelock::FakeLock;

/// Global available pointer to the multiboot2 information structure.
pub static MULTIBOOT2_INFO_STRUCTURE: FakeLock<Option<BootInformation>> = FakeLock::new(None);
