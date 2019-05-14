//! Contains definition for `RcArray`, which is a fat pointer to an atomically reference-counted
//! array, and the implementation for `heaparray::naive_rc::ArcArray`.
pub use crate::naive_rc::prelude::*;
use core::marker::PhantomData;

pub(crate) struct RcArray<'a, A, R, B, E, L = ()>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    data: ManuallyDrop<A>,
    phantom: PhantomData<(&'a E, R, L, B)>,
}

impl<'a, A, R, B, E, L> RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    fn check_null(&self) {
        assert!(
            !self.is_null(),
            "Null dereference of naively reference-counted array"
        );
    }
    fn from_ref(ptr: A) -> Self {
        Self {
            data: ManuallyDrop::new(ptr),
            phantom: PhantomData,
        }
    }
    // This would be unsafe if the `A` were returned to caller, but since it
    // isn't, everything should be fiiiiiiiine
    fn to_ref(self) -> A {
        let ptr = &*self.data;
        let ret = unsafe { mem::transmute_copy(ptr) };
        mem::forget(ptr);
        mem::forget(self);
        ret
    }
}

impl<'a, A, R, B, E, L> BaseArrayRef for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    fn is_null(&self) -> bool {
        (*self.data).is_null()
    }
}

impl<'a, A, R, B, E, L> Clone for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    fn clone(&self) -> Self {
        if self.is_null() {
            Self::null_ref()
        } else {
            (*self.data).get_label().increment();
            unsafe { mem::transmute_copy(self) }
        }
    }
}

impl<'a, A, R, B, E, L> ArrayRef for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    fn to_null(&mut self) {
        if self.is_null() {
            return;
        }
        let ref_count = self.data.get_label_mut().decrement();
        if ref_count == 0 {
            let to_drop = mem::replace(&mut *self.data, unsafe { A::null_ref() });
            mem::drop(to_drop);
        }
    }

    fn null_ref() -> Self {
        Self::from_ref(unsafe { A::null_ref() })
    }
}

impl<'a, A, R, B, E, L> Index<usize> for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.check_null();
        &self.data[idx]
    }
}

impl<'a, A, R, B, E, L> IndexMut<usize> for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    fn index_mut(&mut self, idx: usize) -> &mut E {
        self.check_null();
        &mut self.data[idx]
    }
}

impl<'a, A, R, B, E, L> Drop for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    fn drop(&mut self) {
        self.to_null();
    }
}

impl<'a, A, R, B, E, L> Container<(usize, E)> for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    fn add(&mut self, elem: (usize, E)) {
        self.check_null();
        self[elem.0] = elem.1;
    }
    fn len(&self) -> usize {
        self.check_null();
        self.data.len()
    }
}

impl<'a, A, R, B, E, L> CopyMap<'a, usize, E> for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    fn get(&'a self, key: usize) -> Option<&'a E> {
        self.check_null();
        if key > self.len() {
            None
        } else {
            Some(&self[key])
        }
    }
    #[inline]
    fn get_mut(&'a mut self, key: usize) -> Option<&'a mut E> {
        self.check_null();
        if key > self.len() {
            None
        } else {
            Some(&mut self[key])
        }
    }
    #[inline]
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        self.check_null();
        if key > self.len() {
            None
        } else {
            Some(mem::replace(&mut self[key], value))
        }
    }
}

impl<'a, A, R, B, E, L> Array<'a, E> for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
}

impl<'a, A, R, B, E, L> LabelledArray<'a, E, L> for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    fn with_label<F>(label: L, len: usize, mut func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        let new_ptr = A::with_label(R::new(label), len, |rc_struct, idx| {
            func(&mut rc_struct.get_data_mut(), idx)
        });
        Self::from_ref(new_ptr)
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        Self::from_ref(A::with_label_unsafe(R::new(label), len))
    }
    fn get_label(&self) -> &L {
        self.check_null();
        self.data.get_label().get_data()
    }
    fn get_label_mut(&mut self) -> &mut L {
        self.check_null();
        self.data.get_label_mut().get_data_mut()
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        self.data.get_label_unsafe().get_data_mut()
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.data.get_unsafe(idx)
    }
}

impl<'a, A, R, B, E> MakeArray<'a, E> for RcArray<'a, A, R, B, E, ()>
where
    A: 'a + LabelledArray<'a, E, R> + MakeArray<'a, E> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<()>,
    E: 'a,
    B: 'a + ?Sized,
{
    fn new<F>(len: usize, mut func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self::with_label((), len, |_, idx| func(idx))
    }
}

impl<'a, A, R, B, E, L> DefaultLabelledArray<'a, E, L> for RcArray<'a, A, R, B, E, L>
where
    A: 'a
        + DefaultLabelledArray<'a, E, L>
        + LabelledArray<'a, E, R>
        + BaseArrayRef
        + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a + Default,
    B: 'a + ?Sized,
{
    fn with_len(label: L, len: usize) -> Self {
        Self::from_ref(A::with_len(label, len))
    }
}

unsafe impl<'a, A, R, B, E, L> Send for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L> + Send + Sync,
    L: 'a + Send + Sync,
    E: 'a + Send + Sync,
    B: 'a + ?Sized,
{
}

unsafe impl<'a, A, R, B, E, L> Sync for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B>,
    R: 'a + RefCounter<L> + Send + Sync,
    L: 'a + Send + Sync,
    E: 'a + Send + Sync,
    B: 'a + ?Sized,
{
}

impl<'a, A, R, B, E, L> AtomicArrayRef for RcArray<'a, A, R, B, E, L>
where
    A: 'a + LabelledArray<'a, E, R> + BaseArrayRef + UnsafeArrayRef<'a, B> + AtomicArrayRef,
    R: 'a + RefCounter<L>,
    L: 'a,
    E: 'a,
    B: 'a + ?Sized,
{
    fn compare_and_swap(&self, current: Self, new: Self, order: Ordering) -> Self {
        Self::from_ref((*self.data).compare_and_swap(current.to_ref(), new.to_ref(), order))
    }
    fn compare_exchange(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        match (*self.data).compare_exchange(current.to_ref(), new.to_ref(), success, failure) {
            Ok(data) => Ok(Self::from_ref(data)),
            Err(data) => Err(Self::from_ref(data)),
        }
    }
    fn compare_exchange_weak(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        match (*self.data).compare_exchange_weak(current.to_ref(), new.to_ref(), success, failure) {
            Ok(data) => Ok(Self::from_ref(data)),
            Err(data) => Err(Self::from_ref(data)),
        }
    }
    fn load(&self, order: Ordering) -> Self {
        Self::from_ref((*self.data).load(order))
    }
    fn store(&self, ptr: Self, order: Ordering) {
        (*self.data).store(ptr.to_ref(), order);
    }
    fn swap(&self, ptr: Self, order: Ordering) -> Self {
        Self::from_ref((*self.data).swap(ptr.to_ref(), order))
    }
}
