/*!
Defines the `BaseArray` struct.
*/

mod alloc_utils;
mod base;
mod mem_block;
mod traits;

pub use base::{BaseArray, BaseArrayIter};
pub use mem_block::MemBlock;
pub use traits::*;
