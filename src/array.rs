use std::cmp::Ordering;
use std::fmt;
use std::ops::{Index};

use bow::Bow;
use len_trait::Len;
use push_trait::PushRefBack;

macro_rules! gen_impl {
    ($($name:ident, $slice_name:ident, $str_name:ident, $c_str_name:ident, $os_str_name:ident, $n:expr,)*) => {
        $(
            /// Array of immutable strings stored on the heap in the same buffer.
            pub struct $name<T: $crate::StrLike + ?Sized> {
                buffer: ::bow::Bow<'static, T::Data>,
                split: [usize; $n - 1],
            }

            impl<'a, T: $crate::StrLike + ?Sized, S: 'a + AsRef<T>> From<[&'a S; $n]> for $name<T> {
                fn from(inner: [&'a S; $n]) -> $name<T> {
                    $name::new(inner)
                }
            }

            impl<T: $crate::StrLike + ?Sized> Default for $name<T> {
                fn default() -> $name<T> {
                    let def: &'static T = ::extra_default::DefaultRef::default_ref();
                    let data = def.to_data();
                    let len = data.len();
                    let mut buffer = data.to_owned();

                    let mut split = [len; $n - 1];
                    let mut acc = 0;
                    for s in split.iter_mut().take($n - 1) {
                        *s = acc;
                        acc += len;
                        buffer.push_ref_back(data);
                    }
                    $name { buffer: Bow::Boxed(buffer.into()), split }
                }
            }

            impl<T: $crate::StrLike + ?Sized> $name<T> {
                /// Creates a new `Static` from the given array of values.
                pub fn new<S: AsRef<T>>(inner: [&S; $n]) -> $name<T> {
                    let inner: &[&S] = &inner;

                    let mut buffer: T::OwnedData = Default::default();
                    for item in inner.iter().map(AsRef::as_ref) {
                        buffer.push_ref_back(item.to_data());
                    }
                    let buffer: Box<T::Data> = buffer.into();
                    let buffer: Bow<'static, T::Data> = buffer.into();
                    let mut split = [0; $n - 1];
                    inner.iter().map(AsRef::as_ref).map(::len_trait::Len::len).enumerate().fold(0, |mut curr, (i, len)| {
                        split[i] = curr;
                        curr += len;
                        curr
                    });

                    $name { buffer, split }
                }

                /// Creates a `Static` from its raw parts: a buffer and a list of split indices.
                #[inline]
                pub fn from_raw<D: Into<Bow<'static, T::Data>>>(buffer: D, split: [usize; $n - 1]) -> $name<T> {
                    let buffer = buffer.into();
                    let check = $crate::Split::new(&split);
                    check.check_valid(buffer.len())
                        .unwrap_or_else(|e| panic!("split indices were invalid: {}", e));
                    for idx in 0..$n {
                        T::from_data(check.get(idx).index_into(&*buffer))
                            .unwrap_or_else(|e| panic!("string {} was not valid: {}", idx, e));
                    }
                    $name { buffer, split }
                }

                /// Creates a `Static` from its raw parts (unsafe version).
                #[inline]
                pub unsafe fn from_raw_unchecked<D: Into<Bow<'static, T::Data>>>(buffer: D, split: [usize; $n - 1]) -> $name<T> {
                    let buffer = buffer.into();
                    $name { buffer, split }
                }

                /// Returns an iterator over the elements in this `Static`.
                #[inline]
                pub fn iter(&self) -> $crate::Iter<T> {
                    $crate::Iter::new(&*self.buffer, &self.split)
                }
            }

            impl<T: $crate::StrLike> Index<usize> for $name<T> {
                type Output = T;
                fn index(&self, idx: usize) -> &T {
                    unsafe {
                        let split = $crate::Split::new(&self.split);
                        T::from_data_unchecked(split.get(idx).index_into(&self.buffer))
                    }
                }
            }

            impl<T: $crate::StrLike + ?Sized> Clone for $name<T>
                where Box<T::Data>: Clone
            {
                fn clone(&self) -> $name<T> {
                    $name {
                        buffer: self.buffer.clone(),
                        split: self.split.clone(),
                    }
                }
                fn clone_from(&mut self, source: &$name<T>) {
                    self.buffer.clone_from(&source.buffer);
                    self.split.clone_from(&source.split);
                }
            }

            impl<T: $crate::StrLike + ?Sized> ::std::hash::Hash for $name<T>
                where T::Data: ::std::hash::Hash
            {
                fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                    self.buffer.hash(state);
                    self.split.hash(state);
                }
            }

            impl<T: $crate::StrLike + PartialEq + ?Sized> PartialEq for $name<T> {
                fn eq(&self, rhs: &$name<T>) -> bool {
                    self.iter().eq(rhs.iter())
                }
            }

            impl<T: $crate::StrLike + Eq + ?Sized> Eq for $name<T> {}

            impl<T: $crate::StrLike + PartialOrd + ?Sized> PartialOrd for $name<T> {
                fn partial_cmp(&self, rhs: &$name<T>) -> Option<Ordering> {
                    self.iter().partial_cmp(rhs.iter())
                }
                fn lt(&self, rhs: &$name<T>) -> bool {
                    self.iter().lt(rhs.iter())
                }
                fn le(&self, rhs: &$name<T>) -> bool {
                    self.iter().le(rhs.iter())
                }
                fn gt(&self, rhs: &$name<T>) -> bool {
                    self.iter().gt(rhs.iter())
                }
                fn ge(&self, rhs: &$name<T>) -> bool {
                    self.iter().ge(rhs.iter())
                }
            }

            impl<T: $crate::StrLike + Ord + ?Sized> Ord for $name<T> {
                fn cmp(&self, rhs: &$name<T>) -> Ordering {
                    self.iter().cmp(rhs.iter())
                }
            }

            impl<T: $crate::StrLike + fmt::Debug + ?Sized> fmt::Debug for $name<T> {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.debug_list()
                        .entries(self.iter())
                        .finish()
                }
            }

            /// Array of immutable slices stored on the heap in the same buffer.
            pub type $slice_name<T: 'static + Copy> = $name<[T]>;

            /// Array of immutable `str`s stored on the heap in the same buffer.
            pub type $str_name = $name<str>;

            /// Array of immutable `CStr`s stored on the heap in the same buffer.
            pub type $c_str_name = $name<::std::ffi::CStr>;

            ///// Array of immutable `OsStr`s stored on the heap in the same buffer.
            //pub type $os_str_name = $name<::std::ffi::OsStr>;
        )*
    }
}

gen_impl! {
    Static2, SliceArray2, StringArray2, CStringArray2, OsStringArray2, 2,
    Static3, SliceArray3, StringArray3, CStringArray3, OsStringArray3, 3,
    Static4, SliceArray4, StringArray4, CStringArray4, OsStringArray4, 4,
    Static5, SliceArray5, StringArray5, CStringArray5, OsStringArray5, 5,
    Static6, SliceArray6, StringArray6, CStringArray6, OsStringArray6, 6,
    Static7, SliceArray7, StringArray7, CStringArray7, OsStringArray7, 7,
    Static8, SliceArray8, StringArray8, CStringArray8, OsStringArray8, 8,
    Static9, SliceArray9, StringArray9, CStringArray9, OsStringArray9, 9,
    Static10, SliceArray10, StringArray10, CStringArray10, OsStringArray10, 10,
    Static11, SliceArray11, StringArray11, CStringArray11, OsStringArray11, 11,
    Static12, SliceArray12, StringArray12, CStringArray12, OsStringArray12, 12,
    Static13, SliceArray13, StringArray13, CStringArray13, OsStringArray13, 13,
    Static14, SliceArray14, StringArray14, CStringArray14, OsStringArray14, 14,
    Static15, SliceArray15, StringArray15, CStringArray15, OsStringArray15, 15,
    Static16, SliceArray16, StringArray16, CStringArray16, OsStringArray16, 16,
}
