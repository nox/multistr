#![feature(field_init_shorthand, ordering_chaining)]

#![cfg_attr(test, feature(inclusive_range, plugin))]
#![cfg_attr(test, plugin(clippy))]

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

pub mod pair;
pub mod triple;
pub mod vec;

pub use pair::StringPair;
pub use triple::StringTriple;
pub use vec::StringVec;
