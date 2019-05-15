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
pub trait MakeArray<'a, E>: containers::Array<'a, E>
where
    E: 'a,
{
    /// Create a new array, with values initialized using a provided function.
    fn new<F>(len: usize, func: F) -> Self
    where
        F: FnMut(usize) -> E;
}

/// Array with an optional label struct stored next to the data.
pub trait LabelledArray<'a, E, L>: containers::Array<'a, E>
where
    E: 'a,
{
    /// Create a new array, with values initialized using a provided function, and label
    /// initialized to a provided value.
    fn with_label<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E;
    /// Create a new array, without initializing the values in it.
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self;

    /// Get immutable access to the label.
    fn get_label(&self) -> &L;

    /// Get mutable reference to the label.
    fn get_label_mut(&mut self) -> &mut L;

    /// Get a mutable reference to the label. Implementations of this
    /// method shouldn't do any safety checks.
    unsafe fn get_label_unsafe(&self) -> &mut L;

    /// Get a mutable reference to the element at a specified index.
    /// Implementations of this method shouldn't do any safety checks.
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E;
}

/// Trait for a labelled array with a default value.
pub trait DefaultLabelledArray<'a, E, L>: LabelledArray<'a, E, L>
where
    E: 'a + Default,
{
    /// Create a new array, initialized to default values.
    fn with_len(label: L, len: usize) -> Self;
}
