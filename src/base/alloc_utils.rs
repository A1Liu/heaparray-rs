//! Contains pointer math and allocation utilities.
use const_utils::cond;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::mem::{align_of, size_of};

/// Allocate a block of memory, and then coerce it to type `T`
pub unsafe fn allocate<T>(a: impl GlobalAlloc, layout: Layout) -> *mut T {
    &mut *(a.alloc(layout) as *mut T)
}

/// Deallocate a block of memory using the given size and alignment information.
///
/// Completely ignores the type of the input pointer, so the layout
/// needs to be correct.
pub unsafe fn deallocate<T>(a: impl GlobalAlloc, ptr: *mut T, layout: Layout) {
    a.dealloc(ptr as *mut u8, layout);
}

/// Get the size and alignment, in bytes, of a type repeated `repeat` many times.
pub const fn size_align<T>(repeat: usize) -> (usize, usize) {
    let align = align_of::<T>();
    let size = size_of::<T>();
    (size * repeat, align)
}

/// Gets the aligned size of a type given a specific alignment
pub const fn aligned_size<T>(align: usize) -> usize {
    let size = size_of::<T>();
    let off_by = size % align;
    let adjusted_size = size + align - off_by;
    cond(off_by == 0, size, adjusted_size)
}
