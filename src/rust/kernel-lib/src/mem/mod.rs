//! Utilities for memory. Mainly page alignment stuff.

use core::ops::{Deref, DerefMut};

pub const PAGE_SIZE: usize = 4096;

#[repr(align(4096))]
#[derive(Clone, Debug)]
pub struct PageAligned<T>(T);

impl<T> PageAligned<T> {
    /// Constructor that takes ownership of the data. The data is guaranteed to be aligned.
    pub const fn new(t: T) -> Self {
        Self(t)
    }

    #[cfg(test)]
    const fn self_ptr(&self) -> *const Self {
        self as *const _
    }

    /// Returns the pointer to the data. The pointer is the address of a page, because
    /// the data is page-aligned.
    pub const fn data_ptr(&self) -> *const T {
        (&self.0) as *const _
    }

    /// Returns the number of the page inside the address space.
    pub fn page_num(&self) -> usize {
        self.data_ptr() as usize / PAGE_SIZE
    }

    /// Returns the address of this struct. Because this struct is page-aligned,
    /// the address is the address of a page.
    pub fn page_addr(&self) -> usize {
        self.data_ptr() as usize /*& !0xfff not relevant because aligned*/
    }

    /// Returns a reference to the underlying data.
    pub const fn get(&self) -> &T {
        &self.0
    }

    /// Returns a mutable reference to the underlying data.
    pub const fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Consumes the struct and returns the owned, inner data.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Copy> Copy for PageAligned<T> {}

impl<T> From<T> for PageAligned<T> {
    fn from(data: T) -> Self {
        PageAligned::new(data)
    }
}

impl<T> Deref for PageAligned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for PageAligned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

/// Convenient wrapper around [`PageAligned`] for aligned stack-buffers, with exactly
/// the same restrictions and properties.
#[repr(align(4096))]
#[derive(Clone, Debug)]
pub struct PageAlignedBuf<T, const N: usize>(PageAligned<[T; N]>);

impl<T: Copy, const N: usize> PageAlignedBuf<T, N> {
    /// Constructor that fills the default element into each index of the slice.
    /// Uses this approach in favor of `Default`, because this works in a const context.
    pub const fn new(default: T) -> Self {
        Self(PageAligned::new([default; N]))
    }
}

impl<T, const N: usize> PageAlignedBuf<T, N> {
    /// Return a pointer to self.
    pub const fn self_ptr(&self) -> *const Self {
        self.0.data_ptr() as *const _
    }

    /// Returns the number of the page inside the address space.
    pub fn page_num(&self) -> usize {
        self.0.page_num()
    }

    /// Returns the page base address of this struct.
    pub fn page_base_addr(&self) -> usize {
        self.0.page_addr()
    }

    /// Returns a reference to the underlying data.
    pub const fn get(&self) -> &[T; N] {
        self.0.get()
    }

    /// Returns a reference to the underlying data.
    pub const fn get_mut(&mut self) -> &mut [T; N] {
        self.0.get_mut()
    }
}

impl<T: Copy, const N: usize> Copy for PageAlignedBuf<T, N> {}

impl<const N: usize> PageAlignedBuf<u8, N> {
    /// New `u8` buffer that is initialized with zeroes.
    pub const fn new_zeroed() -> Self {
        Self::new(0)
    }
}

impl<T, const N: usize> Deref for PageAlignedBuf<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T, const N: usize> DerefMut for PageAlignedBuf<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

/// Convenient alias for [`PageAlignedBuf`].
pub type PageAlignedByteBuf<const N: usize> = PageAlignedBuf<u8, N>;
