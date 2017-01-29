use std::borrow::Borrow;
use std::cmp::Ordering;
use std::ffi::CStr;
use std::borrow::Cow;
use std::ops::Index;
use std::fmt;
use std::iter::FromIterator;

use extra_default::DefaultRef;
use len_trait::{Capacity, CapacityMut, DefaultCapacity, Len, LenMut, LenZero, SplitOff};
use push_trait::PushRefBack;

use super::{Split, StrLike, Iter};

/// Vec of immutable strings stored on the heap in the same buffer.
///
/// Slicing ranges of the vector yields the strings in the range concatenated together.
pub struct Dynamic<T: StrLike + ?Sized> {
    buffer: Cow<'static, T::Data>,
    split: Vec<usize>,
}

impl<T: StrLike + ?Sized> Default for Dynamic<T> {
    fn default() -> Dynamic<T> {
        Dynamic::new()
    }
}

impl<'a, T: StrLike + ?Sized, S: 'a + AsRef<T>> FromIterator<&'a S> for Dynamic<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a S>>(iter: I) -> Dynamic<T> {
        let mut v = Self::new();
        for item in iter {
            v.push(item);
        }
        v
    }
}
impl<'a, T: StrLike + ?Sized, S: 'a + AsRef<T>> Extend<&'a S> for Dynamic<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = &'a S>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
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
            buffer: Cow::Owned(DefaultCapacity::default_capacity(data)),
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

        self.buffer.to_mut().push_ref_back(other.buffer.borrow());
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
    pub fn push<S: AsRef<T>>(&mut self, s: &S) {
        let s = s.as_ref().to_data();
        let split = self.split.last().cloned().unwrap_or(0) + s.len();
        self.buffer.to_mut().push_ref_back(s);
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
        #[inline(always)]
        fn hack<T: ?Sized + ::len_trait::IndexRanges>(val: &T, idx: usize) -> &T {
            &val[idx..]
        }
        self.split.pop().map(|idx| unsafe {
            let ret = T::from_data_unchecked(hack(&self.buffer, idx)).to_owned();
            self.buffer.to_mut().truncate(idx);
            ret
        })
    }

    /// Returns an iterator over the strings in the vector.
    #[inline]
    pub fn iter(&self) -> Iter<T> {
        Iter::new(&*self.buffer, &*self.split)
    }
}

impl<T: StrLike> Index<usize> for Dynamic<T> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &T {
        unsafe {
            let split = Split::new(&*self.split);
            T::from_data_unchecked(split.get(index).index_into(&*self.buffer))
        }
    }
}

impl<T: StrLike + ?Sized> Clone for Dynamic<T>
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

impl<T: StrLike + ?Sized> ::std::hash::Hash for Dynamic<T>
    where T::Data: ::std::hash::Hash
{
    fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
        self.buffer.hash(state);
        self.split.hash(state);
    }
}

impl<T: StrLike + PartialEq> PartialEq for Dynamic<T> {
    fn eq(&self, rhs: &Dynamic<T>) -> bool {
        self.iter().eq(rhs.iter())
    }
}

impl<T: StrLike + Eq> Eq for Dynamic<T> {}

impl<T: StrLike + PartialOrd> PartialOrd for Dynamic<T> {
    fn partial_cmp(&self, rhs: &Dynamic<T>) -> Option<Ordering> {
        self.iter().partial_cmp(rhs.iter())
    }
    fn lt(&self, rhs: &Dynamic<T>) -> bool {
        self.iter().lt(rhs.iter())
    }
    fn le(&self, rhs: &Dynamic<T>) -> bool {
        self.iter().le(rhs.iter())
    }
    fn gt(&self, rhs: &Dynamic<T>) -> bool {
        self.iter().gt(rhs.iter())
    }
    fn ge(&self, rhs: &Dynamic<T>) -> bool {
        self.iter().ge(rhs.iter())
    }
}

impl<T: StrLike + Ord> Ord for Dynamic<T> {
    fn cmp(&self, rhs: &Dynamic<T>) -> Ordering {
        self.iter().cmp(rhs.iter())
    }
}

impl<T: StrLike + fmt::Debug + ?Sized> fmt::Debug for Dynamic<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.iter())
            .finish()
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
