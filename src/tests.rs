use std::{mem::forget, panic::{catch_unwind, AssertUnwindSafe}};

use super::*;

#[derive(Debug, PartialEq, Eq)]
struct PanicDrop;

impl Drop for PanicDrop {
    fn drop(&mut self) {
        panic!("panic in droping")
    }
}

#[derive(Debug, PartialEq, Eq)]
enum IfPanic<T> {
    Data(T),
    Panic(PanicDrop),
}
impl<T: PartialEq> PartialEq<T> for IfPanic<T> {
    fn eq(&self, other: &T) -> bool {
        match self {
            Data(data) => data == other,
            Panic(_) => true,
        }
    }
}
use IfPanic::*;

#[test]
fn new() {
    let mut vec = vec!["a", "b", "c", "d"];
    vec.reserve_exact(4);
    let (left, tail) = vec.split_tail(3);
    assert_eq!(left, ["a", "b", "c"]);
    assert_eq!(tail.as_slice().len(), 1);
    assert_eq!(tail.as_slice(), &["d"]);
    drop(tail);
    assert_eq!(left, ["a", "b", "c"]);
    assert_eq!(vec.len(), 4);
    assert_eq!(vec, ["a", "b", "c", "d"]);
}

#[test]
fn split_of_end() {
    let mut vec = vec![1, 2, 3];
    vec.reserve_exact(2);
    let (left, rest) = vec.split_tail(vec.len());
    assert_eq!(left.len(), 3);
    assert_eq!(rest.len(), 0);
    assert_eq!(rest.capacity(), 2);
}

#[test]
fn len_and_cap_test() {
    let mut vec = Vec::with_capacity(17);
    vec.extend(0..=6);
    let (left, rest) = vec.split_tail(4);
    assert_eq!(left.len(), 4);
    assert_eq!(rest.len(), 3);
    assert_eq!(rest.capacity(), 13);
    assert_eq!(rest.split_point(), 4);
    assert_eq!(rest.vec_len(), 7);
    assert_eq!(rest.vec_capacity(), 17);
    assert_eq!(left, [0, 1, 2, 3]);
    assert_eq!(rest, [4, 5, 6]);
}

#[test]
fn inner_tailvec_test() {
    let mut vec = vec!["a", "b", "c", "d"];
    vec.reserve_exact(4);
    let (left, mut tail) = vec.split_tail(1);
    assert_eq!(left, ["a"]);
    assert_eq!(tail.as_slice().len(), 3);
    assert_eq!(tail.as_slice(), &["b", "c", "d"]);
    assert_eq!(tail.capacity(), 7);
    let (mid, mut ttail) = tail.split_tail(1);
    assert_eq!(left, ["a"]);
    assert_eq!(mid, ["b"]);
    assert_eq!(ttail.as_slice().len(), 2);
    assert_eq!(ttail.as_slice(), &["c", "d"]);
    assert_eq!(ttail.capacity(), 6);
    ttail.push("e").unwrap();
    assert_eq!(ttail, ["c", "d", "e"]);
    drop(ttail);
    assert_eq!(tail, ["b", "c", "d", "e"]);
    tail.push("f").unwrap();
    assert_eq!(tail, ["b", "c", "d", "e", "f"]);
    drop(tail);
    assert_eq!(left, ["a"]);
    assert_eq!(vec.len(), 6);
    assert_eq!(vec, ["a", "b", "c", "d", "e", "f"]);
}

#[test]
fn push_test() {
    let mut vec = Vec::with_capacity(5);
    vec.extend([Box::new("a")]);
    let (left, mut rest) = vec.split_tail(1);
    assert_eq!(left.len(), 1);
    assert_eq!(rest.len(), 0);
    assert_eq!(rest.capacity(), 4);
    assert_eq!(rest.split_point(), 1);
    assert_eq!(rest.vec_len(), 1);
    assert_eq!(rest.vec_capacity(), 5);
    assert_eq!(left, [Box::new("a")]);
    assert_eq!(rest, []);
    assert_eq!(rest.push(Box::new("b")), Ok(()));
    assert_eq!(rest.vec_len(), 2);
    assert_eq!(rest.vec_capacity(), 5);
    assert_eq!(rest.push(Box::new("c")), Ok(()));
    assert_eq!(rest.vec_len(), 3);
    assert_eq!(rest.vec_capacity(), 5);
    assert_eq!(rest, [Box::new("b"), Box::new("c")]);
    assert_eq!(rest.push(Box::new("d")), Ok(()));
    assert_eq!(rest.vec_len(), 4);
    assert_eq!(rest.vec_capacity(), 5);
    assert_eq!(rest.push(Box::new("e")), Ok(()));
    assert_eq!(rest.vec_len(), 5);
    assert_eq!(rest.vec_capacity(), 5);
    assert_eq!(rest, [Box::new("b"), Box::new("c"), Box::new("d"), Box::new("e")]);
    assert_eq!(rest.push(Box::new("f")), Err(Box::new("f")));
    assert_eq!(rest.vec_len(), 5);
    assert_eq!(rest.vec_capacity(), 5);
    assert_eq!(rest, [Box::new("b"), Box::new("c"), Box::new("d"), Box::new("e")]);
    assert_eq!(rest.push(Box::new("f")), Err(Box::new("f")));
    assert_eq!(rest.vec_len(), 5);
    assert_eq!(rest.vec_capacity(), 5);
    assert_eq!(rest, [Box::new("b"), Box::new("c"), Box::new("d"), Box::new("e")]);
}

#[test]
fn push_zst_test() {
    let mut vec = Vec::new(); // zst inf capacity!
    vec.extend([()]);
    let (left, mut rest) = vec.split_tail(1);
    assert_eq!(left.len(), 1);
    assert_eq!(rest.len(), 0);
    assert_eq!(rest.capacity(), usize::MAX-1);
    assert_eq!(rest.split_point(), 1);
    assert_eq!(rest.vec_len(), 1);
    assert_eq!(rest.vec_capacity(), usize::MAX);
    assert_eq!(left, [()]);
    assert_eq!(rest, []);
    assert_eq!(rest.push(()), Ok(()));
    assert_eq!(rest.vec_len(), 2);
    assert_eq!(rest.vec_capacity(), usize::MAX);
    assert_eq!(rest.push(()), Ok(()));
    assert_eq!(rest.vec_len(), 3);
    assert_eq!(rest.vec_capacity(), usize::MAX);
    assert_eq!(rest, [(), ()]);
    assert_eq!(rest.push(()), Ok(()));
    assert_eq!(rest.vec_len(), 4);
    assert_eq!(rest.vec_capacity(), usize::MAX);
    assert_eq!(rest.push(()), Ok(()));
    assert_eq!(rest.vec_len(), 5);
    assert_eq!(rest.vec_capacity(), usize::MAX);
    assert_eq!(rest, [(), (), (), ()]);
    assert_eq!(rest.push(()), Ok(()));
    assert_eq!(rest.vec_len(), 6);
    assert_eq!(rest.vec_capacity(), usize::MAX);
    assert_eq!(rest, [(), (), (), (), ()]);
    assert_eq!(rest.push(()), Ok(()));
    assert_eq!(rest.vec_len(), 7);
    assert_eq!(rest.vec_capacity(), usize::MAX);
    assert_eq!(rest, [(), (), (), (), (), ()]);
}

#[test]
fn pop_test() {
    let mut vec = Vec::with_capacity(5);
    vec.extend([Box::new("a"), Box::new("b")]);
    let (left, mut rest) = vec.split_tail(1);
    assert_eq!(left.len(), 1);
    assert_eq!(rest.len(), 1);
    assert_eq!(rest.capacity(), 4);
    assert_eq!(rest.split_point(), 1);
    assert_eq!(rest.vec_len(), 2);
    assert_eq!(rest.vec_capacity(), 5);
    assert_eq!(left, [Box::new("a")]);
    assert_eq!(rest, [Box::new("b")]);
    assert_eq!(rest.pop(), Some(Box::new("b")));
    assert_eq!(rest, []);
    assert_eq!(left.len(), 1);
    assert_eq!(rest.len(), 0);
    assert_eq!(rest.capacity(), 4);
    assert_eq!(rest.split_point(), 1);
    assert_eq!(rest.vec_len(), 1);
    assert_eq!(rest.vec_capacity(), 5);
    assert_eq!(left, [Box::new("a")]);
    assert_eq!(rest, []);
    assert_eq!(rest.pop(), None);
    assert_eq!(rest, []);
    assert_eq!(left.len(), 1);
    assert_eq!(rest.len(), 0);
    assert_eq!(rest.capacity(), 4);
    assert_eq!(rest.split_point(), 1);
    assert_eq!(rest.vec_len(), 1);
    assert_eq!(rest.vec_capacity(), 5);
    assert_eq!(rest.pop(), None);
    assert_eq!(rest.pop(), None);
    assert_eq!(left, [Box::new("a")]);

    for _ in 0..5 {
        assert_eq!(rest.pop(), None);
        assert_eq!(rest, []);
        assert_eq!(left.len(), 1);
        assert_eq!(rest.len(), 0);
        assert_eq!(rest.capacity(), 4);
        assert_eq!(rest.split_point(), 1);
        assert_eq!(rest.vec_len(), 1);
        assert_eq!(rest.vec_capacity(), 5);
    }
    drop(rest);
    assert_eq!(left, [Box::new("a")]);
}

#[test]
fn pop_zst_test() {
    let mut vec = Vec::with_capacity(5);
    vec.extend([(), ()]);
    let (left, mut rest) = vec.split_tail(1);
    assert_eq!(left.len(), 1);
    assert_eq!(rest.len(), 1);
    assert_eq!(rest.capacity(), usize::MAX-1);
    assert_eq!(rest.split_point(), 1);
    assert_eq!(rest.vec_len(), 2);
    assert_eq!(rest.vec_capacity(), usize::MAX);
    assert_eq!(left, [()]);
    assert_eq!(rest, [()]);
    assert_eq!(rest.pop(), Some(()));
    assert_eq!(rest, []);
    assert_eq!(left.len(), 1);
    assert_eq!(rest.len(), 0);
    assert_eq!(rest.capacity(), usize::MAX-1);
    assert_eq!(rest.split_point(), 1);
    assert_eq!(rest.vec_len(), 1);
    assert_eq!(rest.vec_capacity(), usize::MAX);
    assert_eq!(left, [()]);
    assert_eq!(rest, []);
    assert_eq!(rest.pop(), None);
    assert_eq!(rest, []);
    assert_eq!(left.len(), 1);
    assert_eq!(rest.len(), 0);
    assert_eq!(rest.capacity(), usize::MAX-1);
    assert_eq!(rest.split_point(), 1);
    assert_eq!(rest.vec_len(), 1);
    assert_eq!(rest.vec_capacity(), usize::MAX);
    assert_eq!(rest.pop(), None);
    assert_eq!(rest.pop(), None);
    assert_eq!(left, [()]);

    for _ in 0..5 {
        assert_eq!(rest.pop(), None);
        assert_eq!(rest, []);
        assert_eq!(left.len(), 1);
        assert_eq!(rest.len(), 0);
        assert_eq!(rest.capacity(), usize::MAX-1);
        assert_eq!(rest.split_point(), 1);
        assert_eq!(rest.vec_len(), 1);
        assert_eq!(rest.vec_capacity(), usize::MAX);
    }
    drop(rest);
    assert_eq!(left, [()]);
}

#[test]
fn start_test() {
    let mut vec: Vec<i32> = vec![2];
    let (left, rest) = vec.split_tail(0);
    assert_eq!(left.len(), 0);
    assert_eq!(rest.len(), 1);
    assert_eq!(rest.capacity(), 1);
    assert_eq!(rest.vec_len(), 1);
    assert_eq!(rest.vec_capacity(), 1);
    assert_eq!(rest.split_point(), 0);
}

#[test]
fn end_test() {
    let mut vec: Vec<i32> = vec![2];
    let (left, rest) = vec.split_tail(1);
    assert_eq!(left.len(), 1);
    assert_eq!(rest.len(), 0);
    assert_eq!(rest.capacity(), 0);
    assert_eq!(rest.vec_len(), 1);
    assert_eq!(rest.vec_capacity(), 1);
    assert_eq!(rest.split_point(), 1);
}

#[test]
fn default_test() {
    {
        let val: TailVec<i32> = TailVec::default();
        assert_eq!(val.len(), 0);
        assert_eq!(val.capacity(), 0);
        assert_eq!(val.split_point(), 0);
        assert_eq!(val.vec_len(), 0);
        assert_eq!(val.vec_capacity(), 0);
        assert!(val.is_empty());
    }
    {
        let val: TailVec<i32, TailVec<i32>> = TailVec::default();
        assert_eq!(val.len(), 0);
        assert_eq!(val.capacity(), 0);
        assert_eq!(val.split_point(), 0);
        assert_eq!(val.vec_len(), 0);
        assert_eq!(val.vec_capacity(), 0);
        assert!(val.is_empty());
    }
    {
        let val: TailVec<i32, TailVec<i32, TailVec<i32>>> = TailVec::default();
        assert_eq!(val.len(), 0);
        assert_eq!(val.capacity(), 0);
        assert_eq!(val.split_point(), 0);
        assert_eq!(val.vec_len(), 0);
        assert_eq!(val.vec_capacity(), 0);
        assert!(val.is_empty());
    }
}

#[test]
fn into_slice() {
    {
        let val: TailVec<i32> = TailVec::default();
        assert_eq!(val.len(), 0);
        assert_eq!(val.capacity(), 0);
        assert_eq!(val.split_point(), 0);
        assert_eq!(val.vec_len(), 0);
        assert_eq!(val.vec_capacity(), 0);
        assert!(val.is_empty());
        let slice = val.into_slice();
        assert_eq!(slice.len(), 0);
        assert_eq!(slice, &mut []);
    }
    {
        let val: TailVec<i32, TailVec<i32>> = TailVec::default();
        assert_eq!(val.len(), 0);
        assert_eq!(val.capacity(), 0);
        assert_eq!(val.split_point(), 0);
        assert_eq!(val.vec_len(), 0);
        assert_eq!(val.vec_capacity(), 0);
        assert!(val.is_empty());
        let slice = val.into_slice();
        assert_eq!(slice.len(), 0);
        assert_eq!(slice, &mut []);
    }
    {
        let val: TailVec<i32, TailVec<i32, TailVec<i32>>> = TailVec::default();
        assert_eq!(val.len(), 0);
        assert_eq!(val.capacity(), 0);
        assert_eq!(val.split_point(), 0);
        assert_eq!(val.vec_len(), 0);
        assert_eq!(val.vec_capacity(), 0);
        assert!(val.is_empty());
        let slice = val.into_slice();
        assert_eq!(slice.len(), 0);
        assert_eq!(slice, &mut []);
    }

    let mut vec = vec!["a", "b", "c", "d"];
    vec.reserve_exact(4);
    let (left, tail) = vec.split_tail(3);
    assert_eq!(left, ["a", "b", "c"]);
    assert_eq!(tail.as_slice().len(), 1);
    assert_eq!(tail.as_slice(), &["d"]);
    let tail_slice = tail.into_slice();
    assert_eq!(tail_slice, &mut ["d"]);
    assert_eq!(left, ["a", "b", "c"]);
    assert_eq!(vec.len(), 4);
    assert_eq!(vec, ["a", "b", "c", "d"]);
}

#[test]
fn empty_test() {
    let mut vec: Vec<i32> = vec![];
    let (left, mut rest) = vec.split_tail(0);
    assert_eq!(left.len(), 0);
    assert_eq!(rest.len(), 0);
    assert_eq!(rest.capacity(), 0);
    assert_eq!(rest.vec_len(), 0);
    assert_eq!(rest.vec_capacity(), 0);
    assert_eq!(rest.split_point(), 0);
    assert_eq!(rest.as_slice(), &[]);
    assert_eq!(rest.as_slice_mut(), &mut []);
    assert_eq!(rest.into_slice(), &mut []);
}

#[test]
fn forget_test() {
    let mut vec = vec![1, 2, 3, 4];
    vec.reserve_exact(3);
    assert_eq!(vec.capacity(), 7);

    let (left, mut rest) = vec.split_tail(2);
    assert_eq!(left, &mut [1, 2]);
    assert_eq!(rest, &mut [3, 4]);
    assert_eq!(rest.pop(), Some(4));
    assert_eq!(left, &mut [1, 2]);
    assert_eq!(rest, &mut [3]);
    forget(rest);
    assert_eq!(left, &mut [1, 2]);
    assert_eq!(vec, []);
}

#[test]
fn remove_last() {
    let mut vec: Vec<i32> = vec![1, 2, 3, 4];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [2, 3, 4]);
    assert_eq!(rest.remove(2), 4);
    assert_eq!(rest.remove(1), 3);
    assert_eq!(rest.as_slice_mut(), &mut [2]);
    assert_eq!(rest.remove(0), 2);
    assert_eq!(rest.as_slice_mut(), &mut []);
}

#[test]
fn swap_remove_last() {
    let mut vec: Vec<i32> = vec![1, 2, 3, 4];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [2, 3, 4]);
    assert_eq!(rest.swap_remove(2), 4);
    assert_eq!(rest.as_slice_mut(), &mut [2, 3]);
    assert_eq!(rest.swap_remove(1), 3);
    assert_eq!(rest.as_slice_mut(), &mut [2]);
    assert_eq!(rest.swap_remove(0), 2);
    assert_eq!(rest.as_slice_mut(), &mut []);
}

#[test]
fn remove_first() {
    let mut vec: Vec<i32> = vec![1, 2, 3, 4];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [2, 3, 4]);
    assert_eq!(rest.remove(0), 2);
    assert_eq!(rest.as_slice_mut(), &mut [3, 4]);
    assert_eq!(rest.remove(0), 3);
    assert_eq!(rest.as_slice_mut(), &mut [4]);
    assert_eq!(rest.remove(0), 4);
    assert_eq!(rest.as_slice_mut(), &mut []);
}

#[test]
fn swap_remove_first() {
    let mut vec: Vec<i32> = vec![1, 2, 3, 4];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [2, 3, 4]);
    assert_eq!(rest.swap_remove(0), 2);
    assert_eq!(rest.as_slice_mut(), &mut [4, 3]);
    assert_eq!(rest.swap_remove(0), 4);
    assert_eq!(rest.as_slice_mut(), &mut [3]);
    assert_eq!(rest.swap_remove(0), 3);
    assert_eq!(rest.as_slice_mut(), &mut []);
}

#[test]
fn retain_noop_pred_test() {
    let mut vec: Vec<i32> = vec![1, 2, 3, 4];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [2, 3, 4]);
    rest.retain(|_| true);
    assert_eq!(rest.as_slice_mut(), &mut [2, 3, 4]);
}

#[test]
fn retain_normal_test() {
    let mut vec: Vec<i32> = vec![1, 2, 3, 4, 5, 7, 6];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [2, 3, 4, 5, 7, 6]);
    rest.retain(|n| *n % 2 == 0);
    assert_eq!(rest.as_slice_mut(), &mut [2, 4, 6]);
}

#[test]
fn retain_all_false_test() {
    let mut vec: Vec<i32> = vec![1, 2, 3, 4];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [2, 3, 4]);
    rest.retain(|_| false);
    assert_eq!(rest.as_slice_mut(), &mut []);
}

#[test]
fn retain_once_and_false_test() {
    let mut vec: Vec<i32> = vec![1, 2];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [2]);
    rest.retain(|_| false);
    assert_eq!(rest.as_slice_mut(), &mut []);
}

#[test]
fn retain_once_and_true_test() {
    let mut vec: Vec<i32> = vec![1, 2];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [2]);
    rest.retain(|_| true);
    assert_eq!(rest.as_slice_mut(), &mut [2]);
}

#[test]
fn retain_empty_and_false_test() {
    let mut vec: Vec<i32> = vec![1];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut []);
    rest.retain(|_| false);
    assert_eq!(rest.as_slice_mut(), &mut []);
}

#[test]
fn retain_empty_and_true_test() {
    let mut vec: Vec<i32> = vec![1];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut []);
    rest.retain(|_| true);
    assert_eq!(rest.as_slice_mut(), &mut []);
}

#[test]
fn retain_unwind_test() {
    {
        let mut vec: Vec<i32> = vec![1, 2, 3, 4, 5, 8, 7, 6];
        let (_, mut rest) = vec.split_tail(1);
        assert_eq!(rest.as_slice_mut(), &mut [2, 3, 4, 5, 8, 7, 6]);
        let arest = AssertUnwindSafe(&mut rest);
        catch_unwind(|| {
            {arest}.0.retain(|&n| {
                assert_ne!(n, 8);
                n % 2 == 0
            });
        }).unwrap_err();
        assert_eq!(rest.as_slice_mut(), &mut [2, 4, /*panic point*/8, 7, 6]);
    }
    {
        let mut vec: Vec<i32> = vec![1, 2];
        let (_, mut rest) = vec.split_tail(1);
        assert_eq!(rest.as_slice_mut(), &mut [2]);
        let arest = AssertUnwindSafe(&mut rest);
        catch_unwind(|| {
            {arest}.0.retain(|&_| {
                panic!()
            });
        }).unwrap_err();
        assert_eq!(rest.as_slice_mut(), &mut [/*panic point*/2]);
    }
    {
        let mut vec: Vec<i32> = vec![1, 2, 3];
        let (_, mut rest) = vec.split_tail(1);
        assert_eq!(rest.as_slice_mut(), &mut [2, 3]);
        let arest = AssertUnwindSafe(&mut rest);
        catch_unwind(|| {
            {arest}.0.retain(|&_| {
                panic!()
            });
        }).unwrap_err();
        assert_eq!(rest.as_slice_mut(), &mut [/*panic point*/2, 3]);
    }
    {
        let mut vec: Vec<i32> = vec![1, 2, 3];
        let (_, mut rest) = vec.split_tail(1);
        assert_eq!(rest.as_slice_mut(), &mut [2, 3]);
        let arest = AssertUnwindSafe(&mut rest);
        catch_unwind(|| {
            {arest}.0.retain(|&n| {
                assert_eq!(n, 2);
                true
            });
        }).unwrap_err();
        assert_eq!(rest.as_slice_mut(), &mut [2, /*panic point*/3]);
    }
    {
        let mut vec: Vec<i32> = vec![1, 2, 3];
        let (_, mut rest) = vec.split_tail(1);
        assert_eq!(rest.as_slice_mut(), &mut [2, 3]);
        let arest = AssertUnwindSafe(&mut rest);
        catch_unwind(|| {
            {arest}.0.retain(|&n| {
                assert_eq!(n, 2);
                false
            });
        }).unwrap_err();
        assert_eq!(rest.as_slice_mut(), &mut [/*panic point*/3]);
    }
    {
        let mut vec: Vec<i32> = vec![1, 2, 3, 4, 5];
        let (_, mut rest) = vec.split_tail(1);
        let (_, mut rest) = rest.split_tail(1);
        assert_eq!(rest.as_slice_mut(), &mut [3, 4, 5]);
        let arest = AssertUnwindSafe(&mut rest);
        catch_unwind(|| {
            {arest}.0.retain(|&n| {
                assert_eq!(n, 3);
                true
            });
        }).unwrap_err();
        assert_eq!(rest.as_slice_mut(), &mut [3, /*panic point*/4, 5]);
    }
    {
        let mut vec: Vec<i32> = vec![1, 2, 3, 4, 5];
        let (_, mut rest) = vec.split_tail(1);
        let (_, mut rest) = rest.split_tail(1);
        assert_eq!(rest.as_slice_mut(), &mut [3, 4, 5]);
        let arest = AssertUnwindSafe(&mut rest);
        catch_unwind(|| {
            {arest}.0.retain(|&n| {
                assert_eq!(n, 3);
                false
            });
        }).unwrap_err();
        assert_eq!(rest.as_slice_mut(), &mut [/*panic point*/4, 5]);
    }
    {
        let mut vec: Vec<i32> = vec![1, 2, 3, 4, 5];
        let (_, mut rest) = vec.split_tail(1);
        let (_, mut rest) = rest.split_tail(1);
        assert_eq!(rest.as_slice_mut(), &mut [3, 4, 5]);
        let arest = AssertUnwindSafe(&mut rest);
        catch_unwind(|| {
            {arest}.0.retain(|&n| {
                assert!(n == 3 || n == 4, "{n}");
                true
            });
        }).unwrap_err();
        assert_eq!(rest.as_slice_mut(), &mut [3, 4, /*panic point*/5]);
    }
    {
        let mut vec: Vec<i32> = vec![1, 2, 3, 4, 5];
        let (_, mut rest) = vec.split_tail(1);
        let (_, mut rest) = rest.split_tail(1);
        assert_eq!(rest.as_slice_mut(), &mut [3, 4, 5]);
        let arest = AssertUnwindSafe(&mut rest);
        catch_unwind(|| {
            {arest}.0.retain(|&n| {
                assert!(n == 3 || n == 4, "{n}");
                false
            });
        }).unwrap_err();
        assert_eq!(rest.as_slice_mut(), &mut [/*panic point*/5]);
    }
    {
        let mut vec: Vec<i32> = vec![1, 2, 3, 4, 5];
        let (_, mut rest) = vec.split_tail(1);
        let (_, mut rest) = rest.split_tail(1);
        assert_eq!(rest.as_slice_mut(), &mut [3, 4, 5]);
        let arest = AssertUnwindSafe(&mut rest);
        catch_unwind(|| {
            {arest}.0.retain(|&n| {
                assert!(n == 3 || n == 4, "{n}");
                n == 3
            });
        }).unwrap_err();
        assert_eq!(rest.as_slice_mut(), &mut [3, /*panic point*/5]);
    }
}

#[test]
fn retain_zst_test() {
    let mut vec = vec![(), (), (), (), (), ()];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [(), (), (), (), ()]);
    let mut i = 0;
    rest.retain(|()| {
        i += 1;
        i % 2 == 1
    });
    assert_eq!(rest.as_slice_mut(), &mut [(), (), ()]);
}

#[test]
fn drain_zst_test() {
    let mut vec = vec![(), (), (), (), (), ()];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [(), (), (), (), ()]);
    rest.drain(1..=3);
    assert_eq!(rest.as_slice_mut(), &mut [(), ()]);
}

#[test]
fn drain_zst_readed_test() {
    let mut vec = vec![(), (), (), (), (), ()];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [(), (), (), (), ()]);
    assert_eq!(rest.drain(1..=3).next(), Some(()));
    assert_eq!(rest.as_slice_mut(), &mut [(), ()]);
}

#[test]
fn drain_readed_test() {
    let mut vec = vec![0, 1, 2, 3, 4, 5, 6];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [1, 2, 3, 4, 5, 6]);
    assert_eq!(rest.drain(1..=3).next(), Some(2));
    assert_eq!(rest.as_slice_mut(), &mut [1, 5, 6]);
}

#[test]
fn drain_all_readed_test() {
    let mut vec = vec![0, 1, 2, 3, 4, 5, 6];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [1, 2, 3, 4, 5, 6]);
    assert_eq!(rest.drain(1..=3).collect::<Vec<_>>(), vec![2, 3, 4]);
    assert_eq!(rest.as_slice_mut(), &mut [1, 5, 6]);
}

#[test]
fn drain_back_read_test() {
    let mut vec = vec![0, 1, 2, 3, 4, 5, 6];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [1, 2, 3, 4, 5, 6]);
    assert_eq!(rest.drain(1..=3).next_back(), Some(4));
    assert_eq!(rest.as_slice_mut(), &mut [1, 5, 6]);
}

#[test]
fn drain_back_read_all_test() {
    let mut vec = vec![0, 1, 2, 3, 4, 5, 6];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [1, 2, 3, 4, 5, 6]);
    assert_eq!(rest.drain(1..=3).rev().collect::<Vec<_>>(),
               vec![4, 3, 2]);
    assert_eq!(rest.as_slice_mut(), &mut [1, 5, 6]);
}

#[test]
#[should_panic]
fn drain_out_of_range_test() {
    let mut vec = vec![0, 1, 2, 3, 4, 5, 6];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [1, 2, 3, 4, 5, 6]);
    rest.drain(6..7);
}

#[test]
#[should_panic]
fn drain_range_ord_fail_test() {
    let mut vec = vec![0, 1, 2, 3, 4, 5, 6];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [1, 2, 3, 4, 5, 6]);
    #[allow(clippy::reversed_empty_ranges)]
    rest.drain(4..2);
}

#[test]
#[should_panic]
fn drain_range_greater_end_test() {
    let mut vec = vec![0, 1, 2, 3, 4, 5, 6];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [1, 2, 3, 4, 5, 6]);
    rest.drain(3..7);
}

#[test]
fn drain_panic_test() {
    let mut vec = vec![
        Data(0),
        Data(1),
        Data(2),
        Panic(PanicDrop),
        Data(4),
        Data(5),
        Data(6),
    ];
    let (_, mut rest) = vec.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [1, 2, 3, 4, 5, 6]);
    let drain = AssertUnwindSafe(rest.drain(1..=3));
    catch_unwind(|| {
        {drain}.0.for_each(|_| ());
    }).unwrap_err();
    assert_eq!(rest.as_slice_mut(), &mut [Data(1), Data(5), Data(6)]);
}

#[test]
fn send_test() {
    let mut vec: Vec<i32> = vec![1, 2, 3, 4, 5];
    let (_, mut rest) = vec.split_tail(1);
    let (_, mut rest) = rest.split_tail(1);
    assert_eq!(rest.as_slice_mut(), &mut [3, 4, 5]);
    assert_eq!(rest.pop(), Some(5));
    let mut rest = std::thread::scope(|scope| {
        scope.spawn(|| {
            assert_eq!(rest.pop(), Some(4));
            rest
        }).join().unwrap()
    });
    assert_eq!(rest.pop(), Some(3));
    assert_eq!(rest.pop(), None);
}

fn _borrow_sign_test<'a, T>(x: &'a mut TailVec<'a, T>) -> &'a mut [T] {
    x.as_slice_mut()
}

fn _borrow_sign_test2<'a, 'b: 'a, T>(x: &'a mut TailVec<'b, T>) -> &'a mut [T] {
    x.as_slice_mut()
}
