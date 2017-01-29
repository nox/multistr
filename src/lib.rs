#![feature(exclusive_range_pattern, never_type, pub_restricted)]

//#![cfg_attr(inclusive_range, feature(inclusive_range, inclusive_range_syntax))]
//#![cfg_attr(test, feature(inclusive_range, inclusive_range_syntax))]
#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(clippy))]

extern crate bow;
extern crate extra_default;
extern crate len_trait;
extern crate push_trait;

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
