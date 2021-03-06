/// An array of arbitrary (sized) values that can be safely initialized.
///
/// # Example
///
/// ```rust
/// # use heaparray::*;
/// let array = HeapArray::<usize,()>::new(100, |i| i * i);
/// for i in 0..array.len() {
///     assert!(array[i] == i * i);
/// }
/// ```
pub trait MakeArray<E>: containers::CopyMap<usize, E> {
    /// Create a new array, with values initialized using a provided function.
    fn new<F>(len: usize, func: F) -> Self
    where
        F: FnMut(usize) -> E;
}
