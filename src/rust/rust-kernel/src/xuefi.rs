//! This is called "xuefi" in order to have no name conflict with "uefi" crate.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use uefi::table::boot::{BootServices, MemoryType};
use uefi::table::{Boot, Runtime, SystemTable};
use uefi::ResultExt;
use utils::fakelock::FakeLock;
use utils::mutex::SimpleMutex;

/// Stores the reference to the UEFI system table (with boot services enabled).
/// This uses a [`FakeLock`] because when we have boot services available, we one
/// single core available only anyway.
/// Technically, this is just a convenient Rust language feature. The pointer will be
/// the same as for [`UEFI_ST_RS`].
pub static UEFI_ST_BS: FakeLock<Option<SystemTable<Boot>>> = FakeLock::new(None);
/// Stores the reference to the UEFI system table (with boot services exited).
/// This uses a [`SimpleMutex`] because eventually we will have multiple cores.
/// Technically, this is just a convenient Rust language feature. The pointer will be
/// the same as for [`UEFI_ST_BS`].
#[allow(unused)]
pub static UEFI_ST_RS: SimpleMutex<Option<SystemTable<Runtime>>> = SimpleMutex::new(None);

/// A shameless copy of the allocator in [`uefi::alloc`]. Allocator while the
/// UEFI boot services are enabled.
pub struct UefiBsAllocator;

impl UefiBsAllocator {
    fn boot_services(&self) -> &BootServices {
        UEFI_ST_BS.get().as_ref().unwrap().boot_services()
    }
}

unsafe impl GlobalAlloc for UefiBsAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mem_ty = MemoryType::LOADER_DATA;
        let size = layout.size();
        let align = layout.align();

        if align > 8 {
            // allocate more space for alignment
            let ptr = if let Ok(ptr) = self
                .boot_services()
                .allocate_pool(mem_ty, size + align)
                .warning_as_error()
            {
                ptr
            } else {
                return ptr::null_mut();
            };
            // calculate align offset
            let mut offset = ptr.align_offset(align);
            if offset == 0 {
                offset = align;
            }
            let return_ptr = ptr.add(offset);
            // store allocated pointer before the struct
            (return_ptr as *mut *mut u8).sub(1).write(ptr);
            return_ptr
        } else {
            self.boot_services()
                .allocate_pool(mem_ty, size)
                .warning_as_error()
                .unwrap_or(ptr::null_mut())
        }
    }

    unsafe fn dealloc(&self, mut ptr: *mut u8, layout: Layout) {
        if layout.align() > 8 {
            ptr = (ptr as *const *mut u8).sub(1).read();
        }
        self.boot_services()
            .free_pool(ptr)
            .warning_as_error()
            .unwrap();
    }
}
