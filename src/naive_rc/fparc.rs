pub use super::prelude::*;

type RC<L> = ArcStruct<L>;
type ArrPtr<'a, E, L> = FpArr<'a, E, RC<L>>;
type Inner<'a, E, L> = RcArray<'a, ArrPtr<'a, E, L>, RC<L>, E, L>;

/// Fat-pointer implementation of `generic::RcArray` with atomic reference counting.
#[repr(C)]
pub struct FpArcArray<'a, E, L = ()>(Inner<'a, E, L>);

impl<'a, E, L> BaseArrayRef for FpArcArray<'a, E, L> {
    #[inline]
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
impl<'a, E, L> Clone for FpArcArray<'a, E, L> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<'a, E, L> ArrayRef for FpArcArray<'a, E, L> {
    fn to_null(&mut self) {
        self.0.to_null()
    }
    fn null_ref() -> Self {
        Self(Inner::null_ref())
    }
}
impl<'a, E, L> Index<usize> for FpArcArray<'a, E, L> {
    type Output = E;
    fn index(&self, idx: usize) -> &E {
        self.0.index(idx)
    }
}
impl<'a, E, L> Container for FpArcArray<'a, E, L> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, E, L> CopyMap<usize, E> for FpArcArray<'a, E, L> {
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

impl<'a, E, L> LabelledArray<E, L> for FpArcArray<'a, E, L> {
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
    unsafe fn get_label_unsafe(&self) -> &mut L {
        self.0.get_label_unsafe()
    }
    unsafe fn get_unsafe(&self, idx: usize) -> &mut E {
        self.0.get_unsafe(idx)
    }
}

impl<'a, E, L> LabelledArrayRefMut<E, L> for FpArcArray<'a, E, L> {
    fn get_label_mut(&mut self) -> Option<&mut L> {
        self.0.get_label_mut()
    }
}

impl<'a, E> MakeArray<E> for FpArcArray<'a, E, ()>
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

impl<'a, E, L> DefaultLabelledArray<E, L> for FpArcArray<'a, E, L>
where
    E: Default,
{
    fn with_len(label: L, len: usize) -> Self {
        Self(Inner::with_len(label, len))
    }
}

unsafe impl<'a, E, L> Send for FpArcArray<'a, E, L> where Inner<'a, E, L>: Send {}
unsafe impl<'a, E, L> Sync for FpArcArray<'a, E, L> where Inner<'a, E, L>: Sync {}
