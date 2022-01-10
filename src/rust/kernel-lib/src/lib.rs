#![feature(const_mut_refs)]
#![feature(const_fn_trait_bound)]
#![cfg_attr(not(test), no_std)]

pub mod fakelock;
pub mod kernelheap;
pub mod mem;
pub mod mutex;
pub mod rwlock;
