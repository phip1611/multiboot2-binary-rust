//! Module for the kernel heap. So far, my solution is simple. I have all the heap
//! already inside the binary as static array. Therefore, I don't have to work with
//! page tables, find free frames from the memory map etc.
//!
//! My chunk allocator is the heart of the functionality. It gets a slice of memory,
//! a second slice as management storage, and then can manage the memory. It manages
//! the memory in chunks of 256 bytes.

use kernel_lib::kernelheap::global_static_allocator::GlobalStaticChunkAllocator;
use kernel_lib::mem::PageAlignedByteBuf;

/// Chunk size must be a multiple of 8, so that the bitmap can cover all fields properly.
const MULTIPLE_OF: usize = 8;
/// 32768 chunks -> 8 MiB Heap. Must be be a multiple of 8.
pub const HEAP_SIZE: usize = GlobalStaticChunkAllocator::CHUNK_SIZE * MULTIPLE_OF * 4096;
static mut HEAP: PageAlignedByteBuf<HEAP_SIZE> = PageAlignedByteBuf::new_zeroed();
// always make sure, that the division is "clean", i.e. no remainder
const BITMAP_SIZE: usize = HEAP_SIZE / GlobalStaticChunkAllocator::CHUNK_SIZE / 8;
static mut BITMAP: PageAlignedByteBuf<BITMAP_SIZE> = PageAlignedByteBuf::new_zeroed();

#[global_allocator]
static KERNEL_HEAP: GlobalStaticChunkAllocator = GlobalStaticChunkAllocator::new();

/// Initializes the global static rust allocator. It uses static memory already available
/// inside the address space.
pub fn init() {
    unsafe { KERNEL_HEAP.init(HEAP.get_mut(), BITMAP.get_mut()).unwrap() }
    log::debug!("initialized allocator");
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("alloc error: {:#?}", layout);
}
