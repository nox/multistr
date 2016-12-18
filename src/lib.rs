#![feature(field_init_shorthand, ordering_chaining, string_split_off)]

#![cfg_attr(inclusive_range, feature(inclusive_range, inclusive_range_syntax))]
#![cfg_attr(test, feature(inclusive_range, inclusive_range_syntax, plugin))]
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
