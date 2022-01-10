use core::alloc::{GlobalAlloc, Layout};

#[global_allocator]
static ALLOCATOR: KernelAlloc = KernelAlloc;

struct KernelAlloc;

unsafe impl GlobalAlloc for KernelAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        panic!("alloc() unsupported yet! layout = {:?}", layout);
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, layout: Layout) {
        panic!("dealloc() unsupported yet! layout = {:?}", layout);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(l: core::alloc::Layout) -> ! {
    panic!("alloc error: {:#?}", l);
}
