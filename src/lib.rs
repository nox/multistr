#![cfg_attr(test, deny(warnings))]

//#![cfg_attr(inclusive_range, feature(inclusive_range, inclusive_range_syntax))]
//#![cfg_attr(test, feature(inclusive_range, inclusive_range_syntax))]

extern crate bow;
extern crate extra_default;
extern crate len_trait;
extern crate push_trait;
extern crate void;

#[cfg_attr(test, macro_use)]
extern crate quickcheck;

mod array;
mod iter;
mod split;
mod strlike;
mod vec;

pub use array::*;
pub use iter::Iter;
pub use strlike::*;
pub use vec::*;
use split::*;
