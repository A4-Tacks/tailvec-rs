#![forbid(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]

mod tailvec;
pub use tailvec::*;

#[cfg(test)]
mod tests;
