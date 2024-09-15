//! The implementation comes from std

use std::{
    fmt::Debug,
    iter::FusedIterator,
    mem,
    ops::{Range, RangeBounds},
    ptr::{self, NonNull},
    slice,
};

use crate::{utils, TailVec, VecLike};

impl<'a, T, V: VecLike<T = T>> TailVec<'a, T, V> {
    /// Removes the specified range from the vector in bulk, returning all
    /// removed elements as an iterator. If the iterator is dropped before
    /// being fully consumed, it drops the remaining removed elements.
    ///
    /// The returned iterator keeps a mutable borrow on the vector to optimize
    /// its implementation.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// # Leaking
    ///
    /// If the returned iterator goes out of scope without being dropped (due to
    /// [`mem::forget`], for example), the vector may have lost and leaked
    /// elements arbitrarily, including elements outside the range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tailvec::*;
    /// let mut vec = vec![0, 1, 2, 3];
    /// let (_, mut v) = vec.split_tail(1);
    /// let u: Vec<_> = v.drain(1..).collect();
    /// assert_eq!(v, &[1]);
    /// assert_eq!(u, &[2, 3]);
    ///
    /// // A full range clears the vector, like `clear()` does
    /// v.drain(..);
    /// assert_eq!({v}, &[]);
    /// assert_eq!(vec, &[0]);
    /// ```
    ///
    /// *Copy and edited from [`Vec::drain`]*
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, V>
    where R: RangeBounds<usize>,
    {
        let len = self.len();
        let Range { start, end } = utils::range(range, ..len);

        unsafe {
            self.set_len(start);

            let ptr = self.as_ptr().add(start);
            let slice = slice::from_raw_parts(ptr, end - start);

            Drain {
                tail_start: end,
                tail_len: len - end,
                iter: slice.iter(),
                vec: NonNull::from(self),
            }
        }
    }
}

struct DropGuard<'r, 'a, V: VecLike>(&'r mut Drain<'a, V>);
impl<'r, 'a, V: VecLike> Drop for DropGuard<'r, 'a, V> {
    fn drop(&mut self) {
        // a a a a a d d d i i r r r r r
        //           ^     ^   ^
        // src_vec.len() iter tail_start
        // start              tail
        unsafe {
            let src_vec = self.0.vec.as_mut();
            let start = src_vec.len();
            let tail = self.0.tail_start;
            let count = self.0.tail_len;

            if tail != start {
                let src = src_vec.as_ptr().add(tail);
                let dst = src_vec.as_mut_ptr().add(start);
                ptr::copy(src, dst, count)
            }

            src_vec.set_len(start + count);
        }
    }
}

/// A draining iterator for [`TailVec`]
///
/// This struct is created by [`TailVec::drain`].
///
/// See its documentation for more.
///
/// # Examples
///
/// ```
/// # use tailvec::*;
/// let mut vec = vec!['a', 'b', 'c'];
/// let (_, mut rvec) = vec.split_tail(0);
/// let iter = rvec.drain(..);
pub struct Drain<'a, V: VecLike> where V::T: 'a {
    tail_start: usize,
    tail_len: usize,
    iter: slice::Iter<'a, V::T>,
    vec: NonNull<TailVec<'a, V::T, V>>,
}
impl<'a, V: VecLike> Iterator for Drain<'a, V> {
    type Item = V::T;

    fn next(&mut self) -> Option<Self::Item> {
        let ele = self.iter.next()?;
        Some(unsafe { ptr::read(ele) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
impl<'a, V: VecLike> DoubleEndedIterator for Drain<'a, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let ele = self.iter.next_back()?;
        Some(unsafe { ptr::read(ele) })
    }
}
impl<'a, V: VecLike> ExactSizeIterator for Drain<'a, V> {
}
impl<'a, V: VecLike> FusedIterator for Drain<'a, V> {
}
unsafe impl<'a, V: VecLike> Send for Drain<'a, V> where V::T: Send {
}
unsafe impl<'a, V: VecLike> Sync for Drain<'a, V> where V::T: Sync {
}
impl<'a, V: VecLike> Drop for Drain<'a, V> {
    fn drop(&mut self) {
        let is_zst = mem::size_of::<V::T>() == 0;

        let iter = mem::take(&mut self.iter);
        let drop_len = iter.len();
        let mut vec = self.vec;

        if is_zst {
            unsafe {
                let vec = self.vec.as_mut();
                let old_len = vec.len();
                vec.set_len(old_len + drop_len + self.tail_len);
                vec.truncate(old_len + self.tail_len);
            }
            return;
        }

        let _guard = DropGuard(self);

        if drop_len == 0 {
            return;
        }

        let drop_ptr = iter.as_slice().as_ptr();

        unsafe {
            let vec_ptr = vec.as_mut().as_mut_ptr();
            let drop_offset = drop_ptr.offset_from(vec_ptr) as usize;
            let to_drop = ptr::slice_from_raw_parts_mut(vec_ptr.add(drop_offset), drop_len);
            ptr::drop_in_place(to_drop);
        }
    }
}
impl<'a, V: VecLike> Drain<'a, V> {
    /// Get slice of rest elements
    ///
    /// # Examples
    ///
    /// ```
    /// # use tailvec::*;
    /// let mut vec = vec!['a', 'b', 'c'];
    /// let (_, mut vec) = vec.split_tail(0);
    /// let mut drain = vec.drain(..);
    /// assert_eq!(drain.as_slice(), &['a', 'b', 'c']);
    /// let _ = drain.next().unwrap();
    /// assert_eq!(drain.as_slice(), &['b', 'c']);
    /// ```
    pub fn as_slice(&self) -> &[V::T] {
        self.iter.as_slice()
    }
}
impl<'a, V: VecLike> Debug for Drain<'a, V>
where V::T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Drain").field(&self.iter.as_slice()).finish()
    }
}
