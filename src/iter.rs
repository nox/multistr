use super::{Split, StrLike};

/// Iterator over `Dynamic` and `Static` types.
pub struct Iter<'a, T: 'a + StrLike + ?Sized> {
    buffer: &'a T::Data,
    split: Split<'a>,
    idx: usize,
}
impl<'a, T: 'a + StrLike + ?Sized> Iter<'a, T> {
    pub(crate) fn new(buffer: &'a T::Data, split: &'a [usize]) -> Iter<'a, T> {
        Iter {
            buffer: buffer,
            split: Split::new(split),
            idx: 0,
        }
    }
}

impl<'a, T: 'a + StrLike + ?Sized> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Iter {
            buffer: self.buffer,
            split: self.split,
            idx: self.idx,
        }
    }
}

impl<'a, T: 'a + StrLike + ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.idx < self.split.len() {
            let ret = unsafe {
                T::from_data_unchecked(self.split.get(self.idx).index_into(self.buffer))
            };
            self.idx += 1;
            Some(ret)
        } else {
            None
        }
    }
}
