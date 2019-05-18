/// Array with an optional label struct stored next to the data.
pub trait LabelledArray<E, L>: containers::Array<E> {
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
pub trait DefaultLabelledArray<E, L>: LabelledArray<E, L>
where
    E: Default,
{
    /// Create a new array, initialized to default values.
    fn with_len(label: L, len: usize) -> Self;
}
