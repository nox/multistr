use std::error::Error;
use std::ops::{RangeFrom, Index, IndexMut, Range};
use std::fmt;

/// A split of indices.
#[derive(Clone, Copy, Debug)]
pub struct Split<'a> {
    inner: &'a [usize],
}
impl<'a> Split<'a> {
    unsafe fn get_idx(self, idx: usize) -> usize {
        *self.inner.get_unchecked(idx)
    }

    /// Creates a new `Split`.
    pub fn new(inner: &'a [usize]) -> Split<'a> {
        Split { inner: inner }
    }

    /// Gets the position of the `idx`th item.
    pub fn get(self, idx: usize) -> SplitRange {
        let n = self.inner.len();
        unsafe {
            if idx == 0 {
                SplitRange {
                    start: 0,
                    end: Some(self.get_idx(0)),
                }
            } else if idx < n {
                SplitRange {
                    start: self.get_idx(idx - 1),
                    end: Some(self.get_idx(idx)),
                }
            } else if idx == n {
                SplitRange {
                    start: self.get_idx(idx - 1),
                    end: None,
                }
            } else {
                panic!("index {} was out of bounds", idx)
            }
        }
    }

    /// Gets the range of positions for the given range of items.
    pub fn get_slice(self, range: SplitRange) -> SplitRange {
        let n = self.inner.len();
        unsafe {
            let start = if range.start == 0 {
                0
            } else if range.start <= n {
                self.get_idx(range.start - 1)
            } else {
                panic!("start index {} was out of bounds", range.start)
            };
            let end = range.end.and_then(|end| if end < n {
                Some(self.get_idx(end))
            } else if end == n {
                None
            } else {
                panic!("end index {} was out of bounds", end)
            });
            SplitRange {
                start: start,
                end: end,
            }
        }
    }

    /// Splits the first `SplitRange` off and returns the remaining split.
    pub fn split_first(self) -> Option<(SplitRange, Split<'a>)> {
        if let Some((&start, rest)) = self.inner.split_first() {
            if let Some(&end) = rest.first() {
                Some((SplitRange {
                          start: start,
                          end: Some(end),
                      },
                      Split::new(rest)))
            } else {
                Some((SplitRange {
                          start: start,
                          end: None,
                      },
                      Split::new(rest)))
            }
        } else {
            None
        }
    }

    /// Checks the validity of the split.
    pub fn check_valid(self, buf_len: usize) -> Result<(), SplitError> {
        for win in self.inner.windows(2) {
            if win[0] > win[1] {
                return Err(SplitError::NotMonotonic(win[0], win[1]));
            }
        }
        if let Some(&idx) = self.inner.last() {
            if idx > buf_len {
                return Err(SplitError::OutOfBounds(idx));
            }
        }
        Ok(())
    }
}

/// Error when checking validity of split.
#[derive(Copy, Clone, Debug)]
pub enum SplitError {
    NotMonotonic(usize, usize),
    OutOfBounds(usize),
}
impl fmt::Display for SplitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SplitError::NotMonotonic(lhs, rhs) => {
                write!(f,
                       "split indices are supposed to increase, but {} came before {}",
                       lhs,
                       rhs)
            }
            SplitError::OutOfBounds(idx) => write!(f, "split index {} was out of bounds", idx),
        }
    }
}
impl Error for SplitError {
    fn description(&self) -> &str {
        match *self {
            SplitError::NotMonotonic(..) => "split indices were not monotonically increasing",
            SplitError::OutOfBounds(..) => "split index was out of bounds",
        }
    }
}

/// The range returned from a split index.
#[derive(Copy, Clone, Debug)]
pub struct SplitRange {
    start: usize,
    end: Option<usize>,
}
impl SplitRange {
    /// Index into a buffer with this range.
    pub fn index_into<I: ?Sized + Index<RangeFrom<usize>, Output=I> + Index<Range<usize>, Output=I>>(self, buffer: &I) -> &I {
        if let Some(end) = self.end {
            &buffer[self.start..end]
        } else {
            &buffer[self.start..]
        }
    }

    /// Mutably index into a buffer with this range.
    pub fn index_into_mut<I: ?Sized + IndexMut<RangeFrom<usize>, Output=I> + IndexMut<Range<usize>, Output=I>>(self, buffer: &mut I) -> &mut I {
        if let Some(end) = self.end {
            &mut buffer[self.start..end]
        } else {
            &mut buffer[self.start..]
        }
    }
}
