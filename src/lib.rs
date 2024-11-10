#![forbid(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

mod tailvec;
mod retain;
mod drain;
mod utils;
pub use tailvec::*;

#[cfg(test)]
#[cfg(feature = "std")]
mod tests;
