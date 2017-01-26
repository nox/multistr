use std::cmp::Ordering;
use std::fmt;

/// Immutable pair of strings stored on the heap in the same buffer.
#[derive(Eq, PartialEq, Clone, Default, Hash)]
pub struct StringPair {
    buffer: String,
    split: usize,
}

impl<S1: Into<String>, S2: AsRef<str>> From<(S1, S2)> for StringPair {
    #[inline]
    fn from((s1, s2): (S1, S2)) -> StringPair {
        StringPair::new(s1, s2)
    }
}

impl StringPair {
    /// Creates a `StringPair` from a pair of strings.
    pub fn new<S1: Into<String>, S2: AsRef<str>>(s1: S1, s2: S2) -> StringPair {
        let s1 = s1.into();
        let s2 = s2.as_ref();
        let split = s1.len();

        let mut buffer = s1;
        buffer.push_str(s2.as_ref());

        StringPair {
            buffer: buffer,
            split: split,
        }
    }

    /// Creates a `StringPair` from a string and split index.
    #[inline]
    pub fn from_raw<S: Into<String>>(string: S, split: usize) -> StringPair {
        let buffer = string.into();

        assert!(split <= buffer.len(),
                "StrPair: split index was out of bounds");
        StringPair {
            buffer: buffer,
            split: split,
        }
    }

    /// Creates a `StringPair` from a string and split index without checking the index.
    #[inline]
    pub unsafe fn from_raw_unchecked<S: Into<String>>(string: S, split: usize) -> StringPair {
        let buffer = string.into();
        StringPair {
            buffer: buffer,
            split: split,
        }
    }

    /// Gets the first string in the pair.
    #[inline]
    pub fn left(&self) -> &str {
        unsafe { self.buffer.slice_unchecked(0, self.split) }
    }

    /// Gets the second string in the pair.
    #[inline]
    pub fn right(&self) -> &str {
        unsafe { self.buffer.slice_unchecked(self.split, self.buffer.len()) }
    }
}

impl PartialOrd for StringPair {
    fn partial_cmp(&self, rhs: &StringPair) -> Option<Ordering> {
        Some(self.left().cmp(rhs.left()).then_with(|| self.right().cmp(rhs.right())))
    }
}
impl Ord for StringPair {
    fn cmp(&self, rhs: &StringPair) -> Ordering {
        self.left().cmp(rhs.left()).then_with(|| self.right().cmp(rhs.right()))
    }
}
impl fmt::Debug for StringPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("StringPair::new")
            .field(&self.left().to_owned())
            .field(&self.right().to_owned())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_left() {
        assert_eq!(StringPair::from_raw("helloworld", 5).left(), "hello");
    }

    #[test]
    fn raw_right() {
        assert_eq!(StringPair::from_raw("helloworld", 5).right(), "world");
    }

    quickcheck! {
        fn partial_ord(s1: String, s2: String, s3: String, s4: String) -> bool {
            let lhs = StringPair::new(&*s1, &s2);
            let rhs = StringPair::new(&*s3, &s4);
            lhs.partial_cmp(&rhs) == (&s1, &s2).partial_cmp(&(&s3, &s4))
        }
        fn ord(s1: String, s2: String, s3: String, s4: String) -> bool {
            let lhs = StringPair::new(&*s1, &s2);
            let rhs = StringPair::new(&*s3, &s4);
            lhs.cmp(&rhs) == (&s1, &s2).cmp(&(&s3, &s4))
        }
        fn left(s1: String, s2: String) -> bool {
            let pair = StringPair::new(&*s1, &s2);
            pair.left() == s1
        }
        fn right(s1: String, s2: String) -> bool {
            let pair = StringPair::new(&*s1, &s2);
            pair.right() == s2
        }
    }
}
