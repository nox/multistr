use std::borrow::{Borrow, BorrowMut};
use std::cmp::Ordering;
use std::ffi::CStr;
use std::borrow::Cow;
use std::ops::{Index, IndexMut, Range, RangeTo, RangeFrom, RangeFull};
use std::fmt;
use std::iter::FromIterator;

use extra_default::DefaultRef;
use len_trait::{Capacity, CapacityMut, WithCapacity, Len, LenMut, Clear, SplitAtMut};
use push_trait::PushCopyBack;

use super::{Split, StrLike, Iter, DataConcat, StrLikeMut};

/// Vec of immutable strings stored on the heap in the same buffer.
///
/// Slicing ranges of the vector yields the strings in the range concatenated together.
pub struct Dynamic<T: StrLike + ?Sized> {
    buffer: Cow<'static, T::Data>,
    split: Vec<usize>,
}

unsafe impl<T: StrLike + ?Sized> Send for Dynamic<T>
    where &'static T::Data: Send,
          T::OwnedData: Send,
{}

unsafe impl<T: StrLike + ?Sized> Sync for Dynamic<T>
    where &'static T::Data: Sync,
          T::OwnedData: Sync,
{}

impl<T: StrLike + ?Sized> Default for Dynamic<T> {
    fn default() -> Dynamic<T> {
        Dynamic::new()
    }
}

impl<'a, T: StrLike + ?Sized> FromIterator<&'a T> for Dynamic<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Dynamic<T> {
        let mut v = Self::new();
        for item in iter {
            v.push(item);
        }
        v
    }
}
impl<'a, T: StrLike + ?Sized> FromIterator<&'a &'a T> for Dynamic<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a &'a T>>(iter: I) -> Dynamic<T> {
        let mut v = Self::new();
        for &item in iter {
            v.push(item);
        }
        v
    }
}
impl<'a, T: StrLike + ?Sized> Extend<&'a &'a T> for Dynamic<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a &'a T>>(&mut self, iter: I) {
        for &item in iter {
            self.push(item);
        }
    }
}
impl<'a, T: StrLike + ?Sized> Extend<&'a T> for Dynamic<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}
impl<'a, T: StrLike + ?Sized> IntoIterator for &'a Dynamic<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<T: StrLike + ?Sized> Dynamic<T> {
    /// Creates an empty `Dynamic`.
    #[inline]
    pub fn new() -> Dynamic<T> {
        Dynamic {
            buffer: Cow::Borrowed(DefaultRef::default_ref()),
            split: Vec::new(),
        }
    }

    /// Creates an empty `Dynamic` with the given capacities.
    ///
    /// The `Dynamic` will be able to hold exactly `num` strings totallying up to `data` in
    /// length without reallocating. If `num` and `data` are zero, the vector will not
    /// allocate.
    #[inline]
    pub fn with_capacities(num: usize, data: usize) -> Dynamic<T> {
        Dynamic {
            buffer: Cow::Owned(WithCapacity::with_capacity(data)),
            split: Vec::with_capacity(num),
        }
    }

    /// Returns the number of strings this vector can hold without reallocating.
    #[inline]
    pub fn num_capacity(&self) -> usize {
        self.split.capacity()
    }

    /// Returns the total length of strings this vector can hold without reallocating.
    #[inline]
    pub fn data_capacity(&self) -> usize {
        match self.buffer {
            Cow::Borrowed(slice) => slice.len(),
            Cow::Owned(ref buf) => buf.capacity(),
        }
    }

    /// Reserves capacity for at least `additional` more strings totalling to `bytes` more
    /// bytes.
    #[inline]
    pub fn reserve(&mut self, additional: usize, bytes: usize) {
        self.buffer.to_mut().reserve(bytes);
        self.split.reserve(additional);
    }

    /// Similar to `reserve`, calling `reserve_exact` on the inner `String` and `Vec`.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize, bytes: usize) {
        self.buffer.to_mut().reserve_exact(bytes);
        self.split.reserve_exact(additional);
    }

    /// See: `Vec::shrink_to_fit`.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.buffer.to_mut().shrink_to_fit();
        self.split.shrink_to_fit();
    }

    /// Shortens the buffer, keeping the first `len` slices and dropping the rest.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.buffer.to_mut().truncate(self.split[len]);
        self.split.truncate(len);
    }

    /// Moves all of the elements of `other` into `self`, leaving `other` empty.
    pub fn append(&mut self, other: &mut Dynamic<T>) {
        if let Some(&idx) = self.split.last() {
            for other_idx in &mut other.split {
                *other_idx += idx;
            }
        }

        self.buffer.to_mut().push_copy_back(other.buffer.borrow());
        other.buffer.to_mut().clear();

        self.split.append(&mut other.split);
    }

    /// Returns the number of strings in the vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.split.len()
    }

    /// Returns `true` iff the vector contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.split.is_empty()
    }

    /// Splits the collection into two at the given index.
    pub fn split_off(&mut self, at: usize) -> Dynamic<T> {
        let mut new_split = self.split.split_off(at);
        if let Some(&split_idx) = self.split.last() {
            for idx in &mut new_split {
                *idx -= split_idx;
            }
        }

        let new_buffer = self.buffer.to_mut().split_off(at);

        Dynamic {
            buffer: Cow::Owned(new_buffer),
            split: new_split,
        }
    }

    /// Clears the vector, removing all strings.
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.to_mut().clear();
        self.split.clear();
    }

    /// Adds a string to the end of the vec.
    pub fn push(&mut self, t: &T) {
        let t = t.to_data();
        let split = self.split.last().cloned().unwrap_or(0) + t.len();
        self.buffer.to_mut().push_copy_back(t);
        self.split.push(split);
    }

    /// Removes a string from the end of the vec and discards it.
    pub fn pop(&mut self) -> bool {
        match self.split.pop() {
            None => false,
            Some(idx) => {
                self.buffer.to_mut().truncate(idx);
                true
            }
        }
    }

    /// Removes a string from the end of the vec and allocates it onto a new buffer.
    pub fn pop_off(&mut self) -> Option<<T as ToOwned>::Owned> {
        /// TODO: why do I need this?
        #[inline]
        fn hack<T: ?Sized + ::len_trait::IndexRange<usize> + Index<RangeFull, Output = T>>(val: &T, idx: usize) -> &T {
            &val[idx..]
        }

        if self.split.pop().is_none() {
            return None;
        }

        let idx = self.split.last().cloned().unwrap_or(0);

        let ret = unsafe { T::from_data_unchecked(hack(&self.buffer, idx)).to_owned() };
        self.buffer.to_mut().truncate(idx);
        Some(ret)
    }

    /// Returns an iterator over the strings in the vector.
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        Iter::new(&*self.buffer, &*self.split)
    }
}

impl<T: ?Sized + StrLike> Index<usize> for Dynamic<T> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &T {
        assert_ne!(index, self.len());
        unsafe {
            let split = Split::new(&*self.split);
            T::from_data_unchecked(split.get(index).index_into(&*self.buffer))
        }
    }
}

impl<T: ?Sized + StrLike + StrLikeMut> IndexMut<usize> for Dynamic<T>
    where T::Data: SplitAtMut<usize>,
          T::OwnedData: BorrowMut<T::Data>
{
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        assert_ne!(index, self.len());
        unsafe {
            let idx = Split::new(&*self.split).get(index);
            T::from_data_mut_unchecked(idx.index_into_mut(self.buffer.to_mut().borrow_mut()))
        }
    }
}

impl<T: ?Sized + DataConcat> Index<Range<usize>> for Dynamic<T> {
    type Output = T;
    #[inline]
    fn index(&self, range: Range<usize>) -> &T {
        unsafe {
            let split = Split::new(&*self.split);
            T::from_data_unchecked(split.get_slice(range.into()).index_into(&*self.buffer))
        }
    }
}

impl<T: ?Sized + DataConcat> Index<RangeFrom<usize>> for Dynamic<T> {
    type Output = T;
    #[inline]
    fn index(&self, range: RangeFrom<usize>) -> &T {
        unsafe {
            let split = Split::new(&*self.split);
            T::from_data_unchecked(split.get_slice(range.into()).index_into(&*self.buffer))
        }
    }
}

impl<T: ?Sized + DataConcat> Index<RangeTo<usize>> for Dynamic<T> {
    type Output = T;
    #[inline]
    fn index(&self, range: RangeTo<usize>) -> &T {
        unsafe {
            let split = Split::new(&*self.split);
            T::from_data_unchecked(split.get_slice(range.into()).index_into(&*self.buffer))
        }
    }
}

impl<T: ?Sized + DataConcat> Index<RangeFull> for Dynamic<T> {
    type Output = T;
    #[inline]
    fn index(&self, _: RangeFull) -> &T {
        unsafe {
            T::from_data_unchecked(&*self.buffer)
        }
    }
}

impl<T: ?Sized + StrLike> Clone for Dynamic<T>
    where Cow<'static, T::Data>: Clone
{
    fn clone(&self) -> Dynamic<T> {
        Dynamic {
            buffer: self.buffer.clone(),
            split: self.split.clone(),
        }
    }
    fn clone_from(&mut self, source: &Dynamic<T>) {
        self.buffer.clone_from(&source.buffer);
        self.split.clone_from(&source.split);
    }
}

impl<T: ?Sized + StrLike> ::std::hash::Hash for Dynamic<T>
    where T::Data: ::std::hash::Hash
{
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        self.buffer.hash(state);
        self.split.hash(state);
    }
}

impl<T: ?Sized + StrLike + PartialEq> PartialEq for Dynamic<T> {
    fn eq(&self, rhs: &Dynamic<T>) -> bool {
        self.iter().eq(rhs.iter())
    }
}

impl<'a, T: ?Sized + StrLike + PartialEq> PartialEq<&'a [&'a T]> for Dynamic<T> {
    fn eq(&self, rhs: &&'a [&'a T]) -> bool {
        self.iter().eq(rhs.iter().cloned())
    }
}

impl<'a, T: ?Sized + StrLike + PartialEq> PartialEq<Vec<&'a T>> for Dynamic<T> {
    fn eq(&self, rhs: &Vec<&'a T>) -> bool {
        self.iter().eq(rhs.iter().cloned())
    }
}

/*
impl<T: ?Sized + StrLike + PartialEq> PartialEq<Vec<T::Owned>> for Dynamic<T> {
    fn eq(&self, rhs: &Vec<T::Owned>) -> bool {
        self.iter().eq(rhs.iter().map(|s| &*s))
    }
}
*/

impl<T: ?Sized + StrLike + Eq> Eq for Dynamic<T> {}

impl<T: ?Sized + StrLike + PartialOrd> PartialOrd for Dynamic<T> {
    fn partial_cmp(&self, rhs: &Dynamic<T>) -> Option<Ordering> {
        self.iter().partial_cmp(rhs.iter())
    }
}

impl<'a, T: ?Sized + StrLike + PartialOrd> PartialOrd<&'a [&'a T]> for Dynamic<T> {
    fn partial_cmp(&self, rhs: &&'a [&'a T]) -> Option<Ordering> {
        self.iter().partial_cmp(rhs.iter().cloned())
    }
}

impl<'a, T: ?Sized + StrLike + PartialOrd> PartialOrd<Vec<&'a T>> for Dynamic<T> {
    fn partial_cmp(&self, rhs: &Vec<&'a T>) -> Option<Ordering> {
        self.iter().partial_cmp(rhs.iter().cloned())
    }
}

/*
impl<T: ?Sized + StrLike + PartialOrd> PartialOrd<Vec<T::Owned>> for Dynamic<T> {
    fn partial_cmp(&self, rhs: &Vec<T::Owned>) -> Option<Ordering> {
        self.iter().partial_cmp(rhs.iter().map(|s| &*s))
    }
}
*/

impl<T: ?Sized + StrLike + Ord> Ord for Dynamic<T> {
    fn cmp(&self, rhs: &Dynamic<T>) -> Ordering {
        self.iter().cmp(rhs.iter())
    }
}

impl<T: ?Sized + StrLike + fmt::Debug> fmt::Debug for Dynamic<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.iter())
            .finish()
    }
}

#[cfg(feature = "quickcheck")]
impl<T: ?Sized + StrLike> quickcheck::Arbitrary for Dynamic<T>
    where T::Owned: quickcheck::Arbitrary,
          Dynamic<T>: Send + Sync
{
    fn arbitrary<G: ::quickcheck::Gen>(g: &mut G) -> Dynamic<T> {
        let mut vec = Dynamic::new();

        let size = g.size();
        let size = g.gen_range(0, size);
        for _ in 0..size {
            let s: <T as ToOwned>::Owned = quickcheck::Arbitrary::arbitrary(g);
            vec.push(s.borrow());
        }

        vec
    }

    fn shrink(&self) -> Box<Iterator<Item=Dynamic<T>>> {
        let new_self: Vec<<T as ToOwned>::Owned> = self.iter().map(ToOwned::to_owned).collect();
        Box::new(new_self.shrink().map(|v| v.iter().map(|s| s.borrow()).collect()))
    }
}

/// Ve of immutable slices stored on the heap in the same buffer.
pub type SliceVec<T: 'static + Copy> = Dynamic<[T]>;

/// Vec of immutable `str`s stored on the heap in the same buffer.
pub type StringVec = Dynamic<str>;

/// Vec of immutable `CStr`s stored on the heap in the same buffer.
pub type CStringVec = Dynamic<CStr>;

///// Vec of immutable `OsStr`s stored on the heap in the same buffer.
//pub type OsStringVec = Dynamic<OsStr>;

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::super::StrLike;
    use super::Dynamic;

    fn test_cmp<T: ?Sized + StrLike + PartialOrd + ::std::fmt::Debug>(test_slice: &[&T]) {
        let test_vec = test_slice.to_owned();

        let vec = test_slice.iter().collect::<Dynamic<T>>();
        let collect = vec.iter().collect::<Vec<_>>();

        assert_eq!(vec, test_slice);
        assert_eq!(vec, test_vec);
        assert_eq!(collect, test_vec);
    }

    #[test]
    fn slice() {
        test_cmp::<[u8]>(&[&b"hello"[..], &b"world"[..], &b"123"[..]]);
    }

    #[test]
    fn str() {
        test_cmp::<str>(&["what", "a", "wonderful", "day"]);
    }

    #[test]
    fn c_str() {
        test_cmp::<CStr>(&[CStr::from_bytes_with_nul(&b"just\0"[..]).unwrap(),
                           CStr::from_bytes_with_nul(&b"testing\0"[..]).unwrap()]);
    }

    #[test]
    fn debug() {
        let vec = ["English", "Français", "中文"].iter().collect::<Dynamic<str>>();
        assert_eq!(format!("{:?}", vec), r#"["English", "Français", "中文"]"# )
    }

    #[test]
    #[should_panic]
    fn panic_oob() {
        let vec = <Dynamic<[u8]>>::new();
        let _ = &vec[0];
    }

    #[test]
    #[should_panic]
    fn panic_oob_str() {
        let vec = <Dynamic<str>>::new();
        let _ = &vec[0];
    }

    #[test]
    #[should_panic]
    fn panic_oob_c_str() {
        let vec = <Dynamic<CStr>>::new();
        let _ = &vec[0];
    }

    #[test]
    fn index() {
        let vec = ["English", "Français", "中文"].iter().collect::<Dynamic<str>>();
        assert_eq!(&vec[0], "English");
        assert_eq!(&vec[1], "Français");
        assert_eq!(&vec[2], "中文");
        assert_eq!(&vec[0..0], "");
        assert_eq!(&vec[0..1], "English");
        assert_eq!(&vec[0..2], "EnglishFrançais");
        assert_eq!(&vec[0..3], "EnglishFrançais中文");
        assert_eq!(&vec[1..1], "");
        assert_eq!(&vec[1..2], "Français");
        assert_eq!(&vec[1..3], "Français中文");
        assert_eq!(&vec[2..2], "");
        assert_eq!(&vec[2..3], "中文");
        assert_eq!(&vec[3..3], "");
        assert_eq!(&vec[0..], "EnglishFrançais中文");
        assert_eq!(&vec[1..], "Français中文");
        assert_eq!(&vec[2..], "中文");
        assert_eq!(&vec[3..], "");
        assert_eq!(&vec[..0], "");
        assert_eq!(&vec[..1], "English");
        assert_eq!(&vec[..2], "EnglishFrançais");
        assert_eq!(&vec[..3], "EnglishFrançais中文");
        assert_eq!(&vec[..], "EnglishFrançais中文");
    }

    #[test]
    #[should_panic]
    fn panic_oob_nonempty() {
        let vec = ["English", "Français", "中文"].iter().collect::<Dynamic<str>>();
        let _ = &vec[3];
    }

    #[test]
    #[should_panic]
    fn panic_left_oob() {
        let vec = ["English", "Français", "中文"].iter().collect::<Dynamic<str>>();
        let _ = &vec[4..];
    }

    #[test]
    #[should_panic]
    fn panic_right_oob() {
        let vec = ["English", "Français", "中文"].iter().collect::<Dynamic<str>>();
        let _ = &vec[..4];
    }

    #[test]
    fn ord() {
        let fst = ["aha"].iter().collect::<Dynamic<str>>();
        let snd = ["ah", "a"].iter().collect::<Dynamic<str>>();
        let thd = ["a", "ha"].iter().collect::<Dynamic<str>>();
        let fth = ["a", "a"].iter().collect::<Dynamic<str>>();
        let slc = &mut [&fst, &snd, &thd, &fth];
        slc.sort();
        assert_eq!(slc, &[&fth, &thd, &snd, &fst]);
    }

    quickcheck! {
        fn pop_off(vec: Dynamic<str>) -> bool {
            let mut vec = vec;

            let cloned = vec.clone();

            let mut owned = Vec::new();
            while let Some(item) = vec.pop_off() {
                owned.push(item);
            }
            owned.iter().rev().eq(cloned.iter())
        }

        fn extend(vec: Vec<String>) -> bool {
            let mut extend = <Dynamic<str>>::new();
            extend.extend(vec.iter().map(String::as_str));
            let collect = vec.iter().map(String::as_str).collect::<Dynamic<str>>();
            extend == collect
        }
    }

    #[test]
    fn pop() {
        let mut vec = ["English", "Français", "中文"].iter().collect::<Dynamic<str>>();
        assert_eq!(vec.pop(), true);
        assert_eq!(vec.pop(), true);
        assert_eq!(vec.pop(), true);
        assert_eq!(vec.pop(), false);
    }
}
