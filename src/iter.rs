use super::{Split, StrLike};

/// Iterator over `Dynamic` and `Static` types.
pub struct Iter<'a, T: 'a + StrLike + ?Sized> {
    buffer: &'a T::Data,
    split: Split<'a>,
}
impl<'a, T: 'a + StrLike + ?Sized> Iter<'a, T> {
    pub(crate) fn new(buffer: &'a T::Data, split: &'a [usize]) -> Iter<'a, T> {
        Iter {
            buffer: buffer,
            split: Split::new(split),
        }
    }
}

impl<'a, T: 'a + StrLike + ?Sized> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Iter {
            buffer: self.buffer,
            split: self.split.clone(),
        }
    }
}

impl<'a, T: 'a + StrLike + ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        let (range, rest) = if let Some(x) = self.split.split_first() {
            x
        } else {
            return None;
        };
        self.split = rest;
        unsafe {
            Some(T::from_data_unchecked(range.index_into(self.buffer)))
        }
    }
}
