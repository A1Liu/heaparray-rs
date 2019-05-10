//! This crate holds a struct, HeapArray, that internally points to a
//! contiguous block of memory. It also supports storing arbitrary data
//! adjacent to the block of memory.
//!
//! ## Examples
//!
//! Creating an array:
//! ```rust
//! use heaparray::*;
//! let len = 10;
//! let array = HeapArray::new(len, |idx| idx + 3);
//! ```
//!
//! Indexing works as you would expect:
//! ```rust
//! # use heaparray::*;
//! # let mut array = HeapArray::new(10, |idx| idx + 3);
//! array[3] = 2;
//! assert!(array[3] == 2);
//! ```
//!
//! Notably, you can take ownership of objects back from the container:
//!
//! ```rust
//! # use heaparray::*;
//! let mut array = HeapArray::new(10, |_| Vec::<u8>::new());
//! let replacement_object = Vec::new();
//! let owned_object = array.insert(0, replacement_object);
//! ```
//!
//! but you need to give the array a replacement object to fill its slot with.
//!
//! Additionally, you can customize what information should be stored alongside the elements in
//! the array using the HeapArray::new_labelled function:
//!
//! ```rust
//! # use heaparray::*;
//! struct MyLabel {
//!     pub even: usize,
//!     pub odd: usize,
//! }
//!
//! let mut array = HeapArray::new_labelled(
//!     MyLabel { even: 0, odd: 0 },
//!     100,
//!     |label, index| {
//!         if index % 2 == 0 {
//!             label.even += 1;
//!             index
//!         } else {
//!             label.odd += 1;
//!             index
//!         }
//!     });
//! ```

extern crate containers_rs as containers;

/// Array with an optional label struct stored next to the data.
pub trait LabelledArray<E, L>: containers::Array<E> {
    /// Get immutable access to the label.
    fn get_label(&self) -> &L;
    /// Get mutable reference to the label.
    fn get_label_mut(&mut self) -> &mut L;
}

mod alloc;
mod fat_array_ptr;
mod memory_block;
mod thin_array_ptr;

mod prelude {
    pub(crate) use super::memory_block::*;
    pub(crate) use super::LabelledArray;
    pub use containers::{Array, Container, CopyMap};
    pub(crate) use core::mem::ManuallyDrop;
    pub(crate) use core::ops::{Index, IndexMut};
}

pub use fat_array_ptr::FatPtrArray as HeapArray;

pub use fat_array_ptr::*;
pub use prelude::*;
pub use thin_array_ptr::*;

#[cfg(test)]
pub mod tests;
