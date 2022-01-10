//! Module for the kernel heap. So far, my solution is simple. I have all the heap
//! already inside the binary as static array. Therefore, I don't have to work with
//! page tables, find free frames from the memory map etc.
//!
//! My chunk allocator is the heart of the functionality. It gets a slice of memory,
//! a second slice as management storage, and then can manage the memory. It manages
//! the memory in chunks of 256 bytes.

pub mod chunk_allocator;
pub mod global_static_allocator;
