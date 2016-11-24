#![feature(field_init_shorthand, ordering_chaining)]

#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(clippy))]

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod pair;
mod triple;

pub use pair::*;
pub use triple::*;
