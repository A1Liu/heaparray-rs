//! Contains the struct `MemBlock`, which handles pointer math and very low-level
//! interactions with memory.

use super::alloc_utils::*;
use const_utils::{cond, max, safe_div};
use core::alloc::Layout;
use core::marker::PhantomData;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr;
use core::ptr::NonNull;

/// An array block that can hold arbitrary information, and cannot be
/// constructed on the stack.
///
/// The label type, `L`, and element type, `E`, are both held in the same block;
/// i.e. this block holds exactly one instance of `L`, and some arbitrary number
/// of instances of `E`.
///
/// It's not recommended to use this type directly; instead, use the safe pointer
/// types that refer to these, namely `HeapArray`, `FatPtrArray`, and
/// `ThinPtrArray`. If you need more low level control of how to initialize your
/// data, try using the `BaseArray` class first.
///
/// # Invariants
/// These conditions will hold as long as you hold a reference to an instance of
/// `MemBlock` that you haven't deallocated yet.
///
/// 1. The memory block allocated will always have a size (in bytes) less than
///    or equal to `core::isize::MAX`
/// 2. Pointers to valid memory blocks cannot be null
///
/// Additional guarrantees are provided by the instantiation functions, `new`
/// and `new_init`.
///
/// ### Invariant Invalidation
/// Some crate features invalidate the invariants above. Namely:
/// - **`mem-block-skip-size-check`** prevents `MemBlock::new`, `MemBlock::new_init`,
///   `MemBlock::dealloc`, `MemBlock::get_ptr`, and `MemBlock::get_ptr_mut`
///   from checking the size of the array being created or accessed. This can
///   cause undefined behavior with pointer arithmetic when accessing elements
///   with `MemBlock::get_ptr`; note that `MemBlock::dealloc` and `MemBlock::new_init`
///   internally use `MemBlock::get_ptr` to do element construction and destruction
/// - **`mem-block-skip-layout-check`** prevents `MemBlock::new` and
///   `MemBlock::new_init` from checking whether or not the size of the block you
///   try to allocate is valid on the platform you're allocating it on
/// - **`mem-block-skip-ptr-check`** prevents `MemBlock::new` and `MemBlock::new_init`
///   from checking the pointer they return; **this invalidates invariant 2**,
///   and causes undefined behavior.
/// - **`mem-block-skill-all`** enables `mem-block-skip-layout-check`,
///   `mem-block-skip-ptr-check`, and `mem-block-skip-size-check`
///
/// Use all of the above with caution, as their behavior is inherently undefined.
///
/// # Safety of Deallocating References
/// Deallocation methods on `MemBlock` take a `len` argument as a parameter
/// describing the number of instances of `E` that the block stores. In general,
/// deallocation methods on some reference `let r: &mut MemBlock<E,L>` are safe
/// if the following conditions hold, in addition to the invariants discussed above:
///
/// 1. The memory pointed to by `r` has not already been deallocated
/// 2. `r` was allocated with a size, large enough to hold `len` many
///    elements; this means that its size is at least the size of `L` aligned
///    to the alignment of `E`, plus the size of `E` times `len`, i.e.
///    `size_of(L).aligned_to(E) + size_of(E) * len`
/// 3. The elements of `r` have all been initialized; i.e. the element pointed to
///    `r.get_ptr(i)` for all `i < len` is initialized to a valid instance of `E`
///
/// The above are sufficient for a memory block to be safely deallocated; depending
/// on the invariants your codebase holds, they may not be necessary.
#[repr(align(1))]
pub struct MemBlock<E, L = ()> {
    label: ManuallyDrop<L>,
    phantom: PhantomData<(E, L)>,
}

impl<E, L> MemBlock<E, L> {
    /// Get the maximum length of a `MemBlock`, based on the types that it contains.
    ///
    /// This function is used to maintain the invariant that all `MemBlock` instances
    /// are of size (in bytes) less than or equal to `core::isize::MAX`.
    pub const fn max_len() -> usize {
        let max_len = core::isize::MAX as usize;
        let max_len_calc = {
            let (esize, ealign) = size_align::<E>();
            let lsize = aligned_size::<L>(ealign);
            safe_div(max_len - lsize, esize)
        };
        cond(mem::size_of::<E>() == 0, max_len, max_len_calc)
    }

    /// Get size and alignment of the memory that a block of length `len` would need.
    ///
    /// Returns a tuple in the form `(size, align)`
    pub const fn memory_layout(len: usize) -> (usize, usize) {
        let (l_size, l_align) = size_align::<L>();
        let (calc_size, calc_align) = {
            let (dsize, dalign) = size_align_array::<E>(len);
            let l_size = aligned_size::<L>(dalign);
            (l_size + dsize, max(l_align, dalign))
        };
        (
            cond(len == 0, l_size, calc_size),
            cond(len == 0, l_align, calc_align),
        )
    }

    /// Returns a `*const` pointer to an object at index `idx`.
    ///
    /// # Panics
    /// This method panics if `idx` is greater than or equal to the largest value
    /// that this `MemBlock`'s length could be (as defined by
    /// `MemBlock::max_len()`), unless the feature `mem-block-skip-size-check`
    /// is enabled.
    ///
    /// # Safety
    /// The following must hold to safely dereference the pointer `r.get_ptr(idx)`
    /// for some `let r: &MemBlock<E,L>`:
    ///
    /// 1. The memory pointed to by `r` has not already been deallocated
    /// 2. `r` was allocated with a size, large enough to hold at least
    ///    `idx + 1` many elements; this means that its size is at least the
    ///    size of `L` aligned to the alignment of `E`, plus the size of `E`
    ///    times `idx + 1`, i.e. `size_of(L).aligned_to(E) + size_of(E) * (idx + 1)`
    /// 3. The element pointed to by `r.get_ptr(idx)` has been properly initialized.
    ///
    /// The above is sufficient to ensure safe behavior using the default feature
    /// set of this crate. See below for exceptions.
    ///
    /// ### Safety with `mem-block-skip-size-check` Enabled
    /// In addition to the above conditions, `idx` must also be less than
    /// `MemBlock::max_len()`. This is checked at runtime with an
    /// assertion, unless the feature `mem-block-skip-size-check` is enabled,
    /// and causes undefined behavior with pointer math.
    pub fn get_ptr(&self, idx: usize) -> *const E {
        #[cfg(not(feature = "mem-block-skip-size-check"))]
        assert!(
            idx < Self::max_len(),
            "Index {} is invalid: Block cannot be bigger than\
             core::isize::MAX bytes ({} elements)",
            idx,
            Self::max_len()
        );

        let e_align = mem::align_of::<E>();
        let lsize = aligned_size::<L>(e_align);
        let element = unsafe { (self as *const _ as *const u8).add(lsize) as *const E };
        unsafe { element.add(idx) }
    }

    /// Returns a `*mut` pointer to an object at index `idx`.
    ///
    /// # Panics
    /// This method panics if `idx` is greater than or equal to the largest value
    /// that this `MemBlock`'s length could be (as defined by
    /// `MemBlock::max_len()`), unless the feature `mem-block-skip-size-check`
    /// is enabled.
    ///
    /// # Safety
    /// The following must hold to safely dereference the pointer `r.get_ptr(idx)`
    /// for some `let r: &MemBlock<E,L>`:
    ///
    /// 1. The memory pointed to by `r` has not already been deallocated
    /// 2. `r` was allocated with a size, large enough to hold at least
    ///    `idx + 1` many elements; this means that its size is at least the
    ///    size of `L` aligned to the alignment of `E`, plus the size of `E`
    ///    times `idx + 1`, i.e. `size_of(L).aligned_to(E) + size_of(E) * (idx + 1)`
    /// 3. The element pointed to by `r.get_ptr_mut(idx)` has been properly
    ///    initialized.
    ///
    /// The above is sufficient to ensure safe behavior using the default feature
    /// set of this crate. See below for exceptions.
    ///
    /// ### Safety with `mem-block-skip-size-check` Enabled
    /// In addition to the above conditions, `idx` must also be less than
    /// `MemBlock::max_len()`. This is checked at runtime with an
    /// assertion, unless the feature `mem-block-skip-size-check` is enabled,
    /// and causes undefined behavior with pointer math.
    pub fn get_ptr_mut(&mut self, idx: usize) -> *mut E {
        self.get_ptr(idx) as *mut E
    }

    /// Deallocates a reference to this struct, calling the destructor of its
    /// label as well as all contained elements in the process.
    ///
    /// # Panics
    /// This method panics if `len` is larger than the maximum size for a `MemBlock`,
    /// as defined by `MemBlock::max_len()`, unless the feature
    /// `mem-block-skip-size-check` is enabled. It also panics if `len` is too
    /// large for the target platform or the alignment of the block is incorrect,
    /// unless the feature `mem-block-skip-layout-check` is enabled.
    ///
    /// # Safety
    /// The following must hold to safely use `r.dealloc(len)` to deallocate a
    /// `MemBlock` for some `let r: &mut MemBlock<E,L>`, in addition to all
    /// the invariants discussed in the `MemBlock` documentation:
    ///
    /// 1. The memory pointed to by `r` has not already been deallocated
    /// 2. `r` was allocated with a size, large enough to hold at least
    ///    `len` many elements; this means that its size is at least the
    ///    size of `L` aligned to the alignment of `E`, plus the size of `E`
    ///    times `len`, i.e. `size_of(L).aligned_to(E) + size_of(E) * len`
    /// 3. The element pointed to by `r.get_ptr(i)` has been properly initialized,
    ///    for all `let i: usize` such that `i < len`
    ///
    /// The above is sufficient to ensure safe behavior using the default feature
    /// set of this crate. See below for exceptions.
    ///
    /// ### Safety with `mem-block-skip-size-check` Enabled
    /// In addition to the above conditions, `len` must also be less than or equal to
    /// `MemBlock::<E,L>::max_len()`. This is checked at runtime with an
    /// assertion, unless the feature `mem-block-skip-size-check` is enabled, and
    /// causes undefined behavior with pointer math.
    pub unsafe fn dealloc(&mut self, len: usize) {
        #[cfg(not(feature = "mem-block-skip-size-check"))]
        assert!(
            len <= Self::max_len(),
            "Deallocating array of length {} is invalid:\
             Blocks cannot be larger than core::isize::MAX bytes ({} elements)",
            len,
            Self::max_len()
        );

        ptr::drop_in_place(self.get_label_mut());
        for i in 0..len {
            ptr::drop_in_place(self.get_ptr_mut(i));
        }
        self.dealloc_lazy(len);
    }

    /// Deallocates a reference to this struct, without destructing the associated
    /// label or the elements contained inside.
    ///
    /// # Panics
    /// This method panics if `len` is too large for the target platform or the
    /// alignment of the block is incorrect, unless the feature
    /// `mem-block-skip-layout-check` is enabled.
    ///
    /// # Safety
    /// The following must hold to safely use `r.dealloc(len)` to deallocate a
    /// `MemBlock` for some `let r: &mut MemBlock<E,L>`, in addition to all
    /// the invariants discussed in the `MemBlock` documentation:
    ///
    /// 1. The memory pointed to by `r` has not already been deallocated
    /// 2. `r` was allocated with a size, large enough to hold at least
    ///    `len` many elements; this means that its size is at least the
    ///    size of `L` aligned to the alignment of `E`, plus the size of `E`
    ///    times `len`, i.e. `size_of(L).aligned_to(E) + size_of(E) * len`
    ///
    /// The above is sufficient to ensure safe behavior using the default feature
    /// set of this crate. See below for exceptions.
    ///
    /// ### Safety with `mem-block-skip-layout-check` Enabled
    /// In addition to the above conditions, `len` must also be less than or equal to
    /// `MemBlock::<E,L>::max_len()`. This is checked at runtime with an
    /// assertion, unless the feature `mem-block-skip-layout-check` is enabled, and
    /// causes undefined behavior with pointer math.
    pub unsafe fn dealloc_lazy(&mut self, len: usize) {
        let (size, align) = Self::memory_layout(len);
        let layout = if cfg!(feature = "mem-block-skip-layout-check") {
            Layout::from_size_align_unchecked(size, align)
        } else {
            match Layout::from_size_align(size, align) {
                Ok(layout) => layout,
                Err(err) => {
                    panic!(
                        "MemBlock of length {} is invalid for this platform;\n\
                         it has (size, align) = ({}, {}), causing error\n{:#?}",
                        len, size, align, err
                    );
                }
            }
        };

        deallocate(self, layout);
    }

    /// Returns a pointer to a new `MemBlock` without initializing the elements
    /// in the block.
    ///
    /// If you use this function, and don't initialize all the elements in the array
    /// you need to remember to deallocate using `dealloc_lazy`, and optionally
    /// run the destructor for the `label` field as well (as `dealloc_lazy` doesn't
    /// run *any* destructors).
    ///
    /// ## Initialization of Fields
    /// You will need to initialize the elements of the block yourself:
    ///
    /// ```rust
    /// use heaparray::base::MemBlock;
    /// use core::ptr;
    /// let len = 100;
    /// let initialize = |i| { i * i };
    /// let mut block = unsafe { MemBlock::<usize, ()>::new((), len) };
    /// for i in 0..len {
    ///     unsafe {
    ///         ptr::write(block.as_mut().get_ptr_mut(i), initialize(i));
    ///     }
    /// }
    /// ```
    ///
    /// Note that the above is almost the exact same thing that `MemBlock::new_init`
    /// does under the hood.
    pub unsafe fn new<'a>(label: L, len: usize) -> NonNull<Self> {
        let mut block = Self::alloc(len);
        if mem::size_of::<L>() != 0 {
            ptr::write(&mut block.as_mut().label, ManuallyDrop::new(label));
        }
        block
    }

    /// Returns a pointer to a new `MemBlock` without initializing the elements
    /// or label in the block.
    ///
    /// If you use this function, and don't initialize all the elements in the array
    /// you need to remember to deallocate using `dealloc_lazy`, as it skips
    /// destructors alltogether.
    ///
    /// ## Initialization of Fields
    /// You will need to initialize the label yourself to use it:
    ///
    /// ```rust
    /// use heaparray::base::MemBlock;
    /// use core::ptr;
    /// let len = 100;
    /// let initial_value = 12;
    /// let mut block = unsafe { MemBlock::<usize, usize>::alloc(len) };
    /// unsafe {
    ///     ptr::write(block.as_mut().get_label_mut(), initial_value);
    /// }
    /// ```
    ///
    /// ... and also initialize the elements of the block yourself:
    ///
    /// ```rust
    /// use heaparray::base::MemBlock;
    /// use core::ptr;
    /// let len = 100;
    /// let initialize = |i| { i * i };
    /// let mut block = unsafe { MemBlock::<usize, ()>::new((), len) };
    /// for i in 0..len {
    ///     unsafe {
    ///         ptr::write(block.as_mut().get_ptr_mut(i), initialize(i));
    ///     }
    /// }
    /// ```
    ///
    /// Note that the above is almost the exact same thing that `MemBlock::new_init`
    /// does under the hood.
    pub unsafe fn alloc(len: usize) -> NonNull<Self> {
        #[cfg(not(feature = "mem-block-skip-size-check"))]
        assert!(
            len <= Self::max_len(),
            "New array of length {} is invalid: Cannot allocate a block\
             larger than core::isize::MAX bytes ({} elements)",
            len,
            Self::max_len()
        );

        let (size, align) = Self::memory_layout(len);

        let layout = if cfg!(feature = "mem-block-skip-layout-check") {
            Layout::from_size_align_unchecked(size, align)
        } else {
            match Layout::from_size_align(size, align) {
                Ok(layout) => layout,
                Err(err) => {
                    panic!(
                        "MemBlock of length {} is invalid for this platform;\n\
                         it has (size, align) = ({}, {}), causing error\n{:#?}",
                        len, size, align, err
                    );
                }
            }
        };

        if cfg!(feature = "mem-block-skip-ptr-check") {
            NonNull::new_unchecked(allocate::<Self>(layout))
        } else {
            NonNull::new(allocate::<Self>(layout))
                .expect("Allocated a null pointer. You may be out of memory.")
        }
    }

    /// Returns a pointer to a labelled memory block, with elements initialized
    /// using the provided function.
    ///
    /// Function is safe, because the following invariants will always hold:
    ///
    /// - A pointer returned by `block.get_ptr(i)` where `i < len` will always
    ///   point to a valid, aligned instance of `E`
    /// - A memory access `block.label` will always be valid
    /// - Dropping the value doesn't run any destructors; thus the worst that can
    ///   happen is leaking memory
    pub fn new_init<F>(label: L, len: usize, mut func: F) -> NonNull<Self>
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let mut block = unsafe { Self::new(label, len) };
        let block_ref = unsafe { block.as_mut() };
        for i in 0..len {
            let item = func(&mut block_ref.label, i);
            unsafe { ptr::write(block_ref.get_ptr_mut(i), item) }
        }
        block
    }

    /// Returns an immutable reference to the label of this array.
    pub fn get_label(&self) -> &L {
        &self.label
    }

    /// Returns a mutable reference to the label of this array.
    pub fn get_label_mut(&mut self) -> &mut L {
        &mut self.label
    }
}
