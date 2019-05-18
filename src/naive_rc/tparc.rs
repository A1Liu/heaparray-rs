pub use super::prelude::*;
use core::sync::atomic::Ordering;

type RC<L> = ArcStruct<L>;
type ArrPtr<'a, E, L> = TpArr<'a, E, RC<L>>;
type Inner<'a, E, L> = RcArray<'a, ArrPtr<'a, E, L>, RC<L>, E, L>;

/// Thin-pointer implementation of `generic::RcArray` with atomic reference counting.
#[repr(C)]
pub struct TpArcArray<'a, E, L = ()>(Inner<'a, E, L>);

impl<'a, E, L> BaseArrayRef for TpArcArray<'a, E, L> {
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
impl<'a, E, L> Clone for TpArcArray<'a, E, L> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<'a, E, L> ArrayRef for TpArcArray<'a, E, L> {
    fn to_null(&mut self) {
        self.0.to_null()
    }
    fn null_ref() -> Self {
        Self(Inner::null_ref())
    }
}
impl<'a, E, L> Index<usize> for TpArcArray<'a, E, L> {
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.0.index(idx)
    }
}
impl<'a, E, L> IndexMut<usize> for TpArcArray<'a, E, L> {
    fn index_mut(&mut self, idx: usize) -> &mut E {
        self.0.index_mut(idx)
    }
}

impl<'a, E, L> Container<(usize, E)> for TpArcArray<'a, E, L> {
    fn add(&mut self, elem: (usize, E)) {
        self.0.add(elem)
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, E, L> CopyMap<usize, E> for TpArcArray<'a, E, L> {
    fn get(&self, key: usize) -> Option<&E> {
        self.0.get(key)
    }
    fn get_mut(&mut self, key: usize) -> Option<&mut E> {
        self.0.get_mut(key)
    }
    fn insert(&mut self, key: usize, value: E) -> Option<E> {
        self.0.insert(key, value)
    }
}

impl<'a, E, L> Array<E> for TpArcArray<'a, E, L> {}

impl<'a, E, L> LabelledArray<E, L> for TpArcArray<'a, E, L> {
    fn with_label<F>(label: L, len: usize, func: F) -> Self
    where
        F: FnMut(&mut L, usize) -> E,
    {
        Self(Inner::with_label(label, len, func))
    }
    unsafe fn with_label_unsafe(label: L, len: usize) -> Self {
        Self(Inner::with_label_unsafe(label, len))
    }
    fn get_label(&self) -> &L {
        self.0.get_label()
    }
    fn get_label_mut(&mut self) -> &mut L {
        self.0.get_label_mut()
    }
    unsafe fn get_label_unsafe(&self) -> &mut L {
        self.0.get_label_unsafe()
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.0.get_unsafe(idx)
    }
}

impl<'a, E> MakeArray<E> for TpArcArray<'a, E, ()>
where
    E: 'a,
{
    fn new<F>(len: usize, func: F) -> Self
    where
        F: FnMut(usize) -> E,
    {
        Self(Inner::new(len, func))
    }
}

impl<'a, E, L> DefaultLabelledArray<E, L> for TpArcArray<'a, E, L>
where
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self(Inner::with_len(label, len))
    }
}

unsafe impl<'a, E, L> Send for TpArcArray<'a, E, L> where Inner<'a, E, L>: Send {}
unsafe impl<'a, E, L> Sync for TpArcArray<'a, E, L> where Inner<'a, E, L>: Sync {}

impl<'a, E, L> AtomicArrayRef for TpArcArray<'a, E, L> {
    fn compare_and_swap(&self, current: Self, new: Self, order: Ordering) -> Self {
        let Self(current) = current;
        let Self(new) = new;
        Self(self.0.compare_and_swap(current, new, order))
    }
    fn compare_exchange(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        let Self(current) = current;
        let Self(new) = new;
        match self.0.compare_exchange(current, new, success, failure) {
            Ok(r) => Ok(Self(r)),
            Err(r) => Err(Self(r)),
        }
    }
    fn compare_exchange_weak(
        &self,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        let Self(current) = current;
        let Self(new) = new;
        match self.0.compare_exchange_weak(current, new, success, failure) {
            Ok(r) => Ok(Self(r)),
            Err(r) => Err(Self(r)),
        }
    }
    fn load(&self, order: Ordering) -> Self {
        Self(self.0.load(order))
    }
    fn store(&self, ptr: Self, order: Ordering) {
        let Self(ptr) = ptr;
        self.0.store(ptr, order)
    }
    fn swap(&self, ptr: Self, order: Ordering) -> Self {
        let Self(ptr) = ptr;
        Self(self.0.swap(ptr, order))
    }
}
