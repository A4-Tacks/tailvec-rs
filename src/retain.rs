use std::ptr;

use super::*;

impl<'a, T, V: VecLike<T = T>> TailVec<'a, T, V> {
    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tailvec::*;
    /// let mut vec = vec![0, 1, 2, 3, 4];
    /// let (_, mut rest) = vec.split_tail(1);
    /// assert_eq!(rest, &mut [1, 2, 3, 4]);
    ///
    /// rest.retain(|n| *n % 2 == 0);
    /// assert_eq!({rest}, [2, 4]);
    /// assert_eq!(vec, [0, 2, 4]);
    /// ```
    ///
    /// Because the elements are visited exactly once in the original order,
    /// external state may be used to decide which elements to keep.
    ///
    /// ```
    /// # use tailvec::*;
    /// let mut vec = vec![0, 1, 2, 3, 4, 5];
    /// let (_, mut rest) = vec.split_tail(1);
    /// assert_eq!(rest, &mut [1, 2, 3, 4, 5]);
    ///
    /// let keep = [false, true, true, false, true];
    /// let mut iter = keep.iter();
    /// rest.retain(|_| *iter.next().unwrap());
    /// assert_eq!({rest}, [2, 3, 5]);
    /// assert_eq!(vec, [0, 2, 3, 5]);
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where F: FnMut(&T) -> bool,
    {
        self.retain_mut(|ele| {
            f(ele)
        })
    }

    /// Retains only the elements specified by the predicate, passing a mutable reference to it.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tailvec::*;
    /// let mut vec = vec![0, 1, 2, 3, 4];
    /// let (_, mut rest) = vec.split_tail(1);
    /// assert_eq!(rest, &mut [1, 2, 3, 4]);
    ///
    /// rest.retain_mut(|x| if *x <= 3 {
    ///     *x += 1;
    ///     true
    /// } else {
    ///     false
    /// });
    /// assert_eq!({rest}, [2, 3, 4]);
    /// assert_eq!(vec, [0, 2, 3, 4]);
    /// ```
    pub fn retain_mut<F>(&mut self, mut f: F)
    where F: FnMut(&mut T) -> bool,
    {
        struct Guard<'a, 'b, V: VecLike> {
            this: &'b mut TailVec<'a, V::T, V>,
            orig_len: usize,
            proced_len: usize,
            deleted_cnt: usize,
        }

        impl<'a, V: VecLike> Drop for Guard<'a, '_, V> {
            fn drop(&mut self) {
                let Self {
                    ref mut this,
                    orig_len,
                    proced_len,
                    deleted_cnt,
                } = *self;
                unsafe {
                    if deleted_cnt > 0 {
                        let ptr = this.parts()
                            .as_mut_ptr();

                        let src = ptr.add(proced_len);
                        let dst = src.sub(deleted_cnt);
                        let count = orig_len - proced_len;

                        ptr::copy(src, dst, count);
                    }

                    this.set_len(orig_len - deleted_cnt);
                }
            }
        }

        impl<'a, V: VecLike> Guard<'a, '_, V> {
            fn run<F, const DELETED: bool>(&mut self, f: &mut F)
            where F: FnMut(&mut V::T) -> bool,
            {
                let parts = unsafe {
                    self.this.parts().as_mut_ptr()
                };
                // SAFETY: 代码逻辑基本来自`alloc::Vec::retain_mut`
                while self.proced_len != self.orig_len {
                    let cur = unsafe {
                        &mut *parts.add(self.proced_len)
                    };
                    if !f(unsafe { cur.assume_init_mut() }) {
                        self.proced_len += 1;
                        self.deleted_cnt += 1;

                        unsafe { ptr::drop_in_place(cur) }

                        if DELETED {
                            continue;
                        } else {
                            break;
                        }
                    }
                    if DELETED {
                        unsafe {
                            let hole_slot = parts
                                .add(self.proced_len - self.deleted_cnt);
                            ptr::copy_nonoverlapping(cur, hole_slot, 1);
                        }
                    }
                    self.proced_len += 1;
                }
            }
        }

        let orig_len = self.len();
        unsafe { self.set_len(0) }

        let mut g = Guard {
            this: self,
            orig_len,
            proced_len: 0,
            deleted_cnt: 0,
        };

        g.run::<F, false>(&mut f);
        g.run::<F, true>(&mut f);

        drop(g)
    }
}
