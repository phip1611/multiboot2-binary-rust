use crate::boot_stage::{BootStage, BootStageAware};
use crate::error::BootError;
use crate::xuefi::UefiBsAllocator;
use core::alloc::{GlobalAlloc, Layout};
use utils::fakelock::FakeLock;

#[global_allocator]
pub static ALLOCATOR: BootStageAwareAllocator = BootStageAwareAllocator::new();

/// Global Allocator for the Rustkernel, that is compliant to [`crate::boot_stage::BootStageAware`].
/// Currently this panics when no memory is available. Maybe we should look at how it is solved
/// in the "Rust for Linux" project: https://github.com/Rust-for-Linux/linux/
pub struct BootStageAwareAllocator {
    uefi_bs_allocator: FakeLock<Option<UefiBsAllocator>>,
}

impl BootStageAwareAllocator {
    const fn new() -> Self {
        Self {
            uefi_bs_allocator: FakeLock::new(None),
        }
    }
}

impl BootStageAware for BootStageAwareAllocator {
    fn next_boot_stage(&self, boot_stage: BootStage) {
        match boot_stage {
            BootStage::S0_Initial => {}
            BootStage::S1_MB2Handoff => {}
            BootStage::S2_UEFIBootServices => {
                self.uefi_bs_allocator.get_mut().replace(UefiBsAllocator);
            }
            BootStage::S3_UEFIRuntimeServices => {
                // boot services are not longer available
                self.uefi_bs_allocator.get_mut().take();
            }
        }
    }
}

unsafe impl GlobalAlloc for BootStageAwareAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(a) = self.uefi_bs_allocator.get() {
            a.alloc(layout)
        } else {
            panic_error!(
                BootError::PanicAlloc,
                "No Allocator for alloc; layout={:#?}",
                layout
            );
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(a) = self.uefi_bs_allocator.get() {
            a.dealloc(ptr, layout)
        } else {
            panic_error!(
                BootError::PanicDealloc,
                "No Allocator for dealloc; layout={:#?}",
                layout
            );
        }
    }
}
