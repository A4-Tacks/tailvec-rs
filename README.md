This crate can split Vec in half, with the front part being `&mut [T]`,
and the back part being able to perform [`push`], [`pop`] etc

[`push`]: TailVec::push
[`pop`]: TailVec::pop

# Examples

```rust
use tailvec::{TailVec, SplitTail};

let mut vec = vec![1, 2, 3, 4];
vec.reserve_exact(3);
assert_eq!(vec.capacity(), 7);

let (left, mut rest) = vec.split_tail(2);
assert_eq!(left, &mut [1, 2]);
assert_eq!(rest, &mut [3, 4]);

assert_eq!(rest.push(5), Ok(()));
assert_eq!(rest, &mut [3, 4, 5]);

assert_eq!(rest.push(6), Ok(()));
assert_eq!(rest, &mut [3, 4, 5, 6]);

assert_eq!(rest.push(7), Ok(()));
assert_eq!(rest, &mut [3, 4, 5, 6, 7]);

assert_eq!(rest.push(8), Err(8)); // overflow of capacity
assert_eq!(rest, &mut [3, 4, 5, 6, 7]);

assert_eq!(rest.pop(), Some(7));
assert_eq!(rest, &mut [3, 4, 5, 6]);

drop(rest); // drop guard
assert_eq!(vec, [1, 2, 3, 4, 5, 6]);
```

By using this crate,
you can avoid the alloc costs caused by code similar to the following

```rust
let mut vec = vec![1, 2, 3, 4];
vec.reserve_exact(3);
assert_eq!(vec.capacity(), 7);
let a = vec.as_ptr();

let mut rest = vec.split_off(2);
assert_eq!(vec, &[1, 2]);
assert_eq!(rest, &[3, 4]);

rest.push(5);
rest.push(6);
rest.push(7);
rest.pop();

assert_eq!(rest, [3, 4, 5, 6]);
assert!(rest.as_ptr() >= a.wrapping_add(vec.capacity()) ||
        rest.as_ptr() < a); // allocated a new vec
assert_eq!(vec, [1, 2]);

vec.extend(rest);
assert_eq!(vec, [1, 2, 3, 4, 5, 6]);

assert!(core::ptr::eq(vec.as_ptr(), a));
```

# Safety
- miri has passed
