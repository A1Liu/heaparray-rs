/// Array that returns slices into its contents
pub trait SliceArray<E> {
    /// Returns a reference to a slice into this array.
    fn as_slice(&self) -> &[E];
    /// Returns a mutable reference to a slice into this array.
    fn as_slice_mut(&mut self) -> &mut [E];
}

/// Array reference that can return a slice into its contents.
pub trait SliceArrayRef<E> {
    /// Returns a reference to a slice into this array.
    fn as_slice(&self) -> &[E];
    /// Returns a mutable reference to a slice into this array.
    fn as_slice_mut(&mut self) -> Option<&mut [E]>;
}