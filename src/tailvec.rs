#![allow(clippy::partialeq_ne_impl)]

use std::{
    borrow::{Borrow, BorrowMut}, cmp::Ordering, fmt::Debug, hash::Hash, marker::PhantomData, mem::{transmute, MaybeUninit}, ops::{Deref, DerefMut, Index, IndexMut}, ptr::NonNull
};

unsafe fn slice_assume_init<T>(
    slice: &[MaybeUninit<T>],
) -> &[T] {
    unsafe { transmute(slice) }
}

unsafe fn slice_assume_init_mut<T>(
    slice: &mut [MaybeUninit<T>],
) -> &mut [T] {
    unsafe { transmute(slice) }
}


/// Like vec struct trait
///
/// # Safety
/// - [`..self.len()`] of [`self.spare_capacity_mut()`] must be inited
///
/// [`..self.len()`]: VecLike::len
/// [`self.spare_capacity_mut()`]: VecLike::spare_capacity_mut
pub unsafe trait VecLike {
    type T;

    /// [`Vec`] valided elements length
    fn len(&self) -> usize;

    /// [`Vec`] capacity
    fn capacity(&self) -> usize;

    /// [`Vec`] uninit parts
    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<Self::T>];

    /// This is lower operation
    ///
    /// # Safety
    /// - `new_len` must be less than or equal [`capacity`]
    /// - `old_len..new_len` must be inited
    ///
    /// [`capacity`]: Self::capacity
    unsafe fn set_len(&mut self, new_len: usize);

    /// Return collection is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
unsafe impl<T> VecLike for Vec<T> {
    type T = T;

    fn len(&self) -> usize {
        self.len()
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<Self::T>] {
        self.spare_capacity_mut()
    }

    unsafe fn set_len(&mut self, new_len: usize) {
        unsafe {
            self.set_len(new_len)
        }
    }
}
unsafe impl<T, V: VecLike<T = T>> VecLike for TailVec<'_, T, V> {
    type T = T;

    fn len(&self) -> usize {
        self.len
    }

    fn capacity(&self) -> usize {
        self.parts.len()
    }

    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<Self::T>] {
        let range = self.len()..;
        unsafe {
            &mut self.parts.as_mut()[range]
        }
    }

    unsafe fn set_len(&mut self, new_len: usize) {
        self.len = new_len;
    }
}


trait Sealed { }
pub trait SplitTail: Sealed + VecLike + Sized {
    #![allow(private_bounds)]

    /// Split at index, tail part is [`TailVec`]
    ///
    /// It can call [`push`] and [`pop`] operations.
    ///
    /// # Panics
    /// - `mid` greater than [`len`]
    ///
    /// [`push`]: TailVec::push
    /// [`pop`]: TailVec::pop
    /// [`len`]: VecLike::len
    fn split_tail(&mut self, mid: usize) -> (
        &mut [Self::T],
        TailVec<'_, Self::T, Self>,
    );
}
impl<T> Sealed for Vec<T> { }
impl<T, V: VecLike<T = T>> Sealed for TailVec<'_, T, V> { }
impl<T: Sealed + VecLike> SplitTail for T {
    #[track_caller]
    fn split_tail(&mut self, mid: usize) -> (
        &mut [Self::T],
        TailVec<'_, Self::T, Self>,
    ) {
        let len = self.len();
        let mut vec = NonNull::from(self);

        let datas = unsafe {
            let vec = vec.as_mut();
            vec.set_len(0);
            vec.spare_capacity_mut()
        };

        let (left, rest)
            = datas.split_at_mut(mid);
        let tailvec = TailVec {
            parts: rest.into(),
            len: len - mid,
            vec: Some(vec),
            _phantom: PhantomData,
            _phantom_vec: PhantomData,
        };
        (unsafe { slice_assume_init_mut(left) }, tailvec)
    }
}


/// [`Vec`] splitted tail part, create from [`split_tail`]
///
/// [`split_tail`]: SplitTail::split_tail
pub struct TailVec<'a, T, V: VecLike<T = T> = Vec<T>> {
    parts: NonNull<[MaybeUninit<T>]>,
    len: usize,
    vec: Option<NonNull<V>>,
    _phantom: PhantomData<&'a mut T>,
    _phantom_vec: PhantomData<&'a mut V>,
}
impl<'a, T, V: VecLike<T = T>> Drop for TailVec<'a, T, V> {
    #[track_caller]
    fn drop(&mut self) {
        let tail_cap = self.capacity();
        let tail_len = self.len();
        let inner_capacity = self.vec_capacity();
        let mid = inner_capacity - tail_cap;

        if let Some(vec) = &mut self.vec {
            unsafe {
                vec.as_mut().set_len(mid + tail_len)
            }
        } else {
            debug_assert_eq!(tail_len, 0, "len bugs, please report issue");
            debug_assert_eq!(tail_cap, 0, "cap bugs, please report issue");
        }
    }
}
impl<'a, T, V: VecLike<T = T>> Default for TailVec<'a, T, V> {
    fn default() -> Self {
        let parts = <&[_]>::default();
        Self {
            parts: parts.into(),
            len: 0,
            vec: None,
            _phantom: PhantomData,
            _phantom_vec: PhantomData,
        }
    }
}
impl<'a, T, V: VecLike<T = T>> TailVec<'a, T, V> {
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            let slice = self.parts.as_ref();
            slice_assume_init(&slice[..self.len()])
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe {
            let slice = self.parts.as_mut();
            slice_assume_init_mut(&mut slice[..self.len()])
        }
    }

    pub fn into_slice(self) -> &'a mut [T] {
        let rng = ..self.len();
        let mut parts = self.parts;
        drop(self);
        unsafe {
            let slice = &mut parts.as_mut()[rng];
            slice_assume_init_mut(slice)
        }
    }

    pub fn vec_capacity(&self) -> usize {
        self.vec.as_ref()
            .map(|ptr| unsafe { ptr.as_ref() })
            .map(V::capacity)
            .unwrap_or_default()
    }

    pub fn split_point(&self) -> usize {
        self.vec_capacity() - self.capacity()
    }

    pub fn vec_len(&self) -> usize {
        self.split_point() + self.len()
    }

    /// Change `len`, and return `old_len`
    ///
    /// - `offset` by zero is no op.
    /// - `new_len` greater than [`capacity`] return `Err(())`
    /// - `new_len` less than zero return `Err(())`
    ///
    /// [`capacity`]: Self::capacity
    fn try_len(&mut self, offset: isize) -> Result<usize, ()> {
        let old_len = self.len();
        let cap = self.capacity();
        match offset {
            0 => (),
            ..=-1 => {
                let new_len = old_len
                    .checked_sub(-offset as usize).ok_or(())?;
                unsafe { self.set_len(new_len) }
            },
            1.. => {
                let new_len = old_len + offset as usize;
                if new_len > cap { return Err(()); }
                unsafe { self.set_len(new_len) }
            },
        }
        Ok(old_len)
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        let Ok(old_len) = self.try_len(1) else {
            return Err(value);
        };
        let parts = unsafe { self.parts.as_mut() };
        parts[old_len].write(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        self.try_len(-1).ok()?;
        let last_idx = self.len();
        let value = unsafe {
            let parts = self.parts.as_mut();
            parts[last_idx].assume_init_read()
        };
        Some(value)
    }

    #[track_caller]
    fn _remove(&mut self, _index: usize) -> Result<T, ()> {
        unimplemented!("记得给越界函数打上cold")
    }
}
impl<'a, T: Debug, V: VecLike<T = T>> Debug for TailVec<'a, T, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_slice().fmt(f)
    }
}
impl<'a, T: Eq, V: VecLike<T = T>> Eq for TailVec<'a, T, V> {
}
impl<'a, T: PartialEq, V: VecLike<T = T>> PartialEq for TailVec<'a, T, V> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }

    fn ne(&self, other: &Self) -> bool {
        self.as_slice() != other.as_slice()
    }
}
impl<'a, T, U, V> PartialEq<[U]> for TailVec<'a, T, V>
where T: PartialEq<U>,
      V: VecLike<T = T>,
{
    fn eq(&self, other: &[U]) -> bool {
        self.as_slice() == other
    }

    fn ne(&self, other: &[U]) -> bool {
        self.as_slice() != other
    }
}
impl<'a, T, U, V> PartialEq<&'_ [U]> for TailVec<'a, T, V>
where T: PartialEq<U>,
      V: VecLike<T = T>,
{
    fn eq(&self, other: &&[U]) -> bool {
        self.as_slice() == *other
    }

    fn ne(&self, other: &&[U]) -> bool {
        self.as_slice() != *other
    }
}
impl<'a, T, U, V> PartialEq<&'_ mut [U]> for TailVec<'a, T, V>
where T: PartialEq<U>,
      V: VecLike<T = T>,
{
    fn eq(&self, other: &&mut [U]) -> bool {
        self.as_slice() == *other
    }

    fn ne(&self, other: &&mut [U]) -> bool {
        self.as_slice() != *other
    }
}
impl<'a, T, U, V, const N: usize> PartialEq<[U; N]> for TailVec<'a, T, V>
where T: PartialEq<U>,
      V: VecLike<T = T>,
{
    fn eq(&self, other: &[U; N]) -> bool {
        self.as_slice() == other
    }

    fn ne(&self, other: &[U; N]) -> bool {
        self.as_slice() != other
    }
}
impl<'a, T, U, V, const N: usize> PartialEq<&'_ [U; N]> for TailVec<'a, T, V>
where T: PartialEq<U>,
      V: VecLike<T = T>,
{
    fn eq(&self, other: &&[U; N]) -> bool {
        self.as_slice() == *other
    }

    fn ne(&self, other: &&[U; N]) -> bool {
        self.as_slice() != *other
    }
}
impl<'a, T, U, V, const N: usize> PartialEq<&'_ mut [U; N]> for TailVec<'a, T, V>
where T: PartialEq<U>,
      V: VecLike<T = T>,
{
    fn eq(&self, other: &&mut [U; N]) -> bool {
        self.as_slice() == *other
    }

    fn ne(&self, other: &&mut [U; N]) -> bool {
        self.as_slice() != *other
    }
}
impl<'a, T: PartialOrd, V: VecLike<T = T>> PartialOrd for TailVec<'a, T, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}
impl<'a, T: Ord, V: VecLike<T = T>> Ord for TailVec<'a, T, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}
impl<'a, T, V: VecLike<T = T>> Deref for TailVec<'a, T, V> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<'a, T, V: VecLike<T = T>> DerefMut for TailVec<'a, T, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}
impl<'a, T, V, I> Index<I> for TailVec<'a, T, V>
where V: VecLike<T = T>,
      [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.as_slice().index(index)
    }
}
impl<'a, T, V, I> IndexMut<I> for TailVec<'a, T, V>
where V: VecLike<T = T>,
      [T]: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.as_slice_mut().index_mut(index)
    }
}
impl<'a, T, V: VecLike<T = T>> AsRef<[T]> for TailVec<'a, T, V> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<'a, T, V: VecLike<T = T>> AsMut<[T]> for TailVec<'a, T, V> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_slice_mut()
    }
}
impl<'a, T, V: VecLike<T = T>> Borrow<[T]> for TailVec<'a, T, V> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}
impl<'a, T, V: VecLike<T = T>> BorrowMut<[T]> for TailVec<'a, T, V> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_slice_mut()
    }
}
impl<'a, T: Hash, V: VecLike<T = T>> Hash for TailVec<'a, T, V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}
impl<'a, T, V: VecLike<T = T>> IntoIterator for TailVec<'a, T, V> {
    type Item = &'a mut T;
    type IntoIter = <&'a mut [T] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into_slice().iter_mut()
    }
}
impl<'a, T, V: VecLike<T = T>> IntoIterator for &'a TailVec<'_, T, V> {
    type Item = &'a T;
    type IntoIter = <&'a [T] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
impl<'a, T, V: VecLike<T = T>> IntoIterator for &'a mut TailVec<'_, T, V> {
    type Item = &'a mut T;
    type IntoIter = <&'a mut [T] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().iter_mut()
    }
}
impl<'a, T, V: VecLike<T = T>> Extend<T> for &'a mut TailVec<'_, T, V> {
    /// Extends a collection with the contents of an iterator.
    ///
    /// # Panics
    /// [`iter.count()`] greater than `capacity() - len()`
    ///
    /// [`iter.count()`]: Iterator::count
    #[track_caller]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter()
            .for_each(|ele| {
                if self.push(ele).is_err() {
                    panic!("overflow of capacity when extend elements")
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_len_test() {
        let mut vec = Vec::with_capacity(5);
        vec.extend(0..5);
        let (_, mut rest) = vec.split_tail(0);
        unsafe { rest.set_len(2) };
        assert_eq!(rest.len(), 2);
        assert_eq!(rest.try_len(0), Ok(2));
        assert_eq!(rest.len(), 2);
        assert_eq!(rest.try_len(-3), Err(()));
        assert_eq!(rest.len(), 2);
        assert_eq!(rest.try_len(-2), Ok(2));
        assert_eq!(rest.len(), 0);
        assert_eq!(rest.try_len(-2), Err(()));
        assert_eq!(rest.len(), 0);
        assert_eq!(rest.try_len(-1), Err(()));
        assert_eq!(rest.len(), 0);
        assert_eq!(rest.try_len(0), Ok(0));
        assert_eq!(rest.len(), 0);
        assert_eq!(rest.try_len(1), Ok(0));
        assert_eq!(rest.len(), 1);
        assert_eq!(rest.try_len(1), Ok(1));
        assert_eq!(rest.len(), 2);
        assert_eq!(rest.try_len(-1), Ok(2));
        assert_eq!(rest.len(), 1);
        assert_eq!(rest.try_len(-1), Ok(1));
        assert_eq!(rest.len(), 0);
        assert_eq!(rest.try_len(5), Ok(0));
        assert_eq!(rest.len(), 5);
        assert_eq!(rest.try_len(1), Err(()));
        assert_eq!(rest.len(), 5);
        assert_eq!(rest.try_len(-1), Ok(5));
        assert_eq!(rest.len(), 4);
        assert_eq!(rest.try_len(1), Ok(4));
        assert_eq!(rest.len(), 5);
        assert_eq!(rest.try_len(-1), Ok(5));
        assert_eq!(rest.len(), 4);
        assert_eq!(rest.try_len(2), Err(()));
        assert_eq!(rest.len(), 4);
    }
}
