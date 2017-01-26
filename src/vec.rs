use std::cmp::Ordering;
use std::ops::{Index, Range, RangeFrom, RangeFull, RangeTo};
use std::fmt;
use std::iter::FromIterator;

#[cfg(any(inclusive_range, test))]
use std::ops::{RangeInclusive, RangeToInclusive};

/// Vec of immutable strings stored on the heap in the same buffer.
///
/// Slicing ranges of the vector yields the strings in the range concatenated together.
#[derive(Eq, PartialEq, Clone, Default, Hash)]
pub struct StringVec {
    buffer: String,
    split: Vec<usize>,
}

impl<S: AsRef<str>> FromIterator<S> for StringVec {
    #[inline]
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> StringVec {
        let mut v = StringVec::new();
        for item in iter {
            v.push(item.as_ref());
        }
        v
    }
}
impl<S: AsRef<str>> Extend<S> for StringVec {
    #[inline]
    fn extend<T: IntoIterator<Item = S>>(&mut self, iter: T) {
        for item in iter {
            self.push(item.as_ref());
        }
    }
}

impl StringVec {
    /// Creates an empty `StringVec`.
    #[inline]
    pub fn new() -> StringVec {
        StringVec {
            buffer: String::new(),
            split: Vec::new(),
        }
    }

    /// Creates an empty `StringVec` with the given capacities.
    ///
    /// The `StringVec` will be able to hold exactly `count` strings totallying up to `bytes` in
    /// length without reallocating. If `count` and `bytes` are zero, the vector will not
    /// allocate.
    #[inline]
    pub fn with_capacities(num: usize, bytes: usize) -> StringVec {
        StringVec {
            buffer: String::with_capacity(bytes),
            split: Vec::with_capacity(num),
        }
    }

    /// Returns the number of strings this vector can hold without reallocating.
    #[inline]
    pub fn num_capacity(&self) -> usize {
        self.split.capacity()
    }

    /// Returns the number of bytes this vector can hold without reallocating.
    #[inline]
    pub fn byte_capacity(&self) -> usize {
        self.buffer.capacity()
    }

    /// Reserves capacity for at least `additional` more strings totalling to `bytes` more
    /// bytes.
    #[inline]
    pub fn reserve(&mut self, additional: usize, bytes: usize) {
        self.buffer.reserve(bytes);
        self.split.reserve(additional);
    }

    /// Similar to `reserve`, calling `reserve_exact` on the inner `String` and `Vec`.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize, bytes: usize) {
        self.buffer.reserve_exact(bytes);
        self.split.reserve_exact(additional);
    }

    /// See: `Vec::shrink_to_fit`.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.buffer.shrink_to_fit();
        self.split.shrink_to_fit();
    }

    /// Shortens the vector, keeping the first `len` strings and dropping the rest.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len < self.split.len() {
            let idx = self.split[len];
            self.buffer.truncate(idx);
            self.split.truncate(len);
        }
    }

    /// Moves all of the elements of `other` into `self`, leaving `other` empty.
    pub fn append(&mut self, other: &mut StringVec) {
        if let Some(&idx) = self.split.last() {
            for other_idx in &mut other.split {
                *other_idx += idx;
            }
        }
        self.buffer.push_str(&*other.buffer);
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
    pub fn split_off(&mut self, at: usize) -> StringVec {
        let mut new_split = self.split.split_off(at);
        let split_idx = self.split.last().cloned().unwrap_or(0);
        for idx in &mut new_split {
            *idx -= split_idx;
        }

        let new_buffer = self.buffer.split_off(at);
        StringVec {
            buffer: new_buffer,
            split: new_split,
        }
    }

    /// Clears the vector, removing all strings.
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.split.clear();
    }

    /// Adds a string to the end of the vec.
    pub fn push<S: AsRef<str>>(&mut self, s: S) {
        let s = s.as_ref();
        self.buffer.push_str(s);

        let idx = self.split.last().cloned().unwrap_or(0) + s.len();
        self.split.push(idx);
    }

    /// Removes a string from the end of the vec and discards it.
    pub fn pop(&mut self) -> bool {
        match self.split.pop() {
            None => false,
            Some(idx) => {
                self.buffer.truncate(idx);
                true
            }
        }
    }

    /// Removes a string from the end of the vec and allocates it onto a new buffer.
    pub fn pop_off(&mut self) -> Option<String> {
        self.split.pop().map(|idx| {
            let ret = String::from(&self.buffer[idx..]);
            self.buffer.truncate(idx);
            ret
        })
    }

    /// Returns an iterator over the strings in the vector.
    #[inline]
    pub fn iter(&self) -> Iter {
        Iter {
            inner: self,
            idx: 0,
        }
    }

    #[inline]
    fn get_start_idx(&self, index: usize) -> usize {
        if index == 0 { 0 } else { self.split[index - 1] }
    }

    #[inline]
    fn get_end_idx(&self, index: usize) -> usize {
        self.split[index]
    }
}

impl Index<usize> for StringVec {
    type Output = str;
    #[inline]
    fn index(&self, index: usize) -> &str {
        &self.buffer[self.get_start_idx(index)..self.get_end_idx(index)]
    }
}

impl Index<Range<usize>> for StringVec {
    type Output = str;
    #[inline]
    fn index(&self, index: Range<usize>) -> &str {
        assert!(index.start <= index.end);
        &self.buffer[self.get_start_idx(index.start)..self.get_start_idx(index.end)]
    }
}

impl Index<RangeTo<usize>> for StringVec {
    type Output = str;
    #[inline]
    fn index(&self, index: RangeTo<usize>) -> &str {
        &self.buffer[..self.get_start_idx(index.end)]
    }
}

impl Index<RangeFrom<usize>> for StringVec {
    type Output = str;
    #[inline]
    fn index(&self, index: RangeFrom<usize>) -> &str {
        &self.buffer[self.get_start_idx(index.start)..]
    }
}

impl Index<RangeFull> for StringVec {
    type Output = str;
    #[inline]
    fn index(&self, _: RangeFull) -> &str {
        &self.buffer[..]
    }
}

#[cfg(any(inclusive_range, test))]
impl Index<RangeInclusive<usize>> for StringVec {
    type Output = str;
    #[inline]
    fn index(&self, index: RangeInclusive<usize>) -> &str {
        match index {
            RangeInclusive::Empty { .. } => &self.buffer[..0],
            RangeInclusive::NonEmpty { start, end } => {
                &self.buffer[self.get_start_idx(start)..self.get_end_idx(end)]
            }
        }
    }
}

#[cfg(any(inclusive_range, test))]
impl Index<RangeToInclusive<usize>> for StringVec {
    type Output = str;
    #[inline]
    fn index(&self, index: RangeToInclusive<usize>) -> &str {
        &self.buffer[..self.get_end_idx(index.end)]
    }
}

#[derive(Clone, Debug)]
pub struct Iter<'a> {
    inner: &'a StringVec,
    idx: usize,
}
impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        if self.idx < self.inner.len() {
            let ret = Some(&self.inner[self.idx]);
            self.idx += 1;
            ret
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct IntoIter {
    inner: StringVec,
    idx: usize,
}
impl Iterator for IntoIter {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        if self.idx < self.inner.len() {
            let ret = Some(String::from(&self.inner[self.idx]));
            self.idx += 1;
            ret
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a StringVec {
    type IntoIter = Iter<'a>;
    type Item = &'a str;
    #[inline]
    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl IntoIterator for StringVec {
    type IntoIter = IntoIter;
    type Item = String;
    #[inline]
    fn into_iter(self) -> IntoIter {
        IntoIter {
            inner: self,
            idx: 0,
        }
    }
}

impl PartialOrd for StringVec {
    #[inline]
    fn partial_cmp(&self, rhs: &StringVec) -> Option<Ordering> {
        self.iter().partial_cmp(rhs.iter())
    }
}
impl Ord for StringVec {
    #[inline]
    fn cmp(&self, rhs: &StringVec) -> Ordering {
        self.iter().cmp(rhs.iter())
    }
}
impl fmt::Debug for StringVec {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.iter())
            .finish()
    }
}

#[macro_export]
macro_rules! string_vec {
    ( $($x:expr),* ) => {
        {
            let mut temp = StringVec::new();
            $(
                temp.push($x);
            )*
            temp
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice() {
        let v = string_vec!["hello", "world", "!"];
        assert_eq!(&v[2], "!");
        assert_eq!(&v[..2], "helloworld");
        assert_eq!(&v[1..], "world!");
        assert_eq!(&v[1..2], "world");
        assert_eq!(&v[..], "helloworld!");
        assert_eq!(&v[...1], "helloworld");
        assert_eq!(&v[1...2], "world!");
    }

    quickcheck! {
        fn partial_ord(v1: Vec<String>, v2: Vec<String>) -> bool {
            let lhs: StringVec = v1.iter().collect();
            let rhs: StringVec = v2.iter().collect();
            lhs.partial_cmp(&rhs) == v1.partial_cmp(&v2)
        }
        fn ord(v1: Vec<String>, v2: Vec<String>) -> bool {
            let lhs: StringVec = v1.iter().collect();
            let rhs: StringVec = v2.iter().collect();
            lhs.cmp(&rhs) == v1.cmp(&v2)
        }
        fn iter(v1: Vec<String>) -> bool {
            let v2: StringVec = v1.iter().collect();
            let v3: Vec<String> = v2.iter().map(ToOwned::to_owned).collect();
            v1 == v3
        }
        fn into_iter(v1: Vec<String>) -> bool {
            let v2: StringVec = v1.iter().collect();
            let v3: Vec<String> = v2.into_iter().collect();
            v1 == v3
        }
    }
}
