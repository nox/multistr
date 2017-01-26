use std::cmp::Ordering;
use std::fmt;

/// Immutable triple of strings stored on the heap in the same buffer.
#[derive(Eq, PartialEq, Clone, Default, Hash)]
pub struct StringTriple {
    buffer: String,
    split: (usize, usize),
}

impl<S1: Into<String>, S2: AsRef<str>, S3: AsRef<str>> From<(S1, S2, S3)> for StringTriple {
    #[inline]
    fn from((s1, s2, s3): (S1, S2, S3)) -> StringTriple {
        StringTriple::new(s1, s2, s3)
    }
}

impl StringTriple {
    /// Creates a `StringTriple` from a triple of strings.
    pub fn new<S1: Into<String>, S2: AsRef<str>, S3: AsRef<str>>(s1: S1,
                                                                 s2: S2,
                                                                 s3: S3)
                                                                 -> StringTriple {
        let s1 = s1.into();
        let s2 = s2.as_ref();
        let s3 = s3.as_ref();
        let split = (s1.len(), s1.len() + s2.len());

        let mut buffer = s1;
        buffer.push_str(s2.as_ref());
        buffer.push_str(s3.as_ref());
        StringTriple {
            buffer: buffer,
            split: split,
        }
    }

    /// Creates a `StringTriple` from a string and split indices.
    #[inline]
    pub fn from_raw<S: Into<String>>(string: S, lsplit: usize, rsplit: usize) -> StringTriple {
        let buffer = string.into();
        let split = (lsplit, rsplit);

        assert!(lsplit <= buffer.len(),
                "StrTriple: left split index was out of bounds");
        assert!(rsplit <= buffer.len(),
                "StrTriple: right split index was out of bounds");
        assert!(lsplit <= rsplit,
                "StrTriple: split indices were in wrong order");
        StringTriple {
            buffer: buffer,
            split: split,
        }
    }

    /// Creates a `StringTriple` from a string and split indices without checking the indices.
    #[inline]
    pub unsafe fn from_raw_unchecked<S: Into<String>>(string: S,
                                                      lsplit: usize,
                                                      rsplit: usize)
                                                      -> StringTriple {
        let buffer = string.into();
        let split = (lsplit, rsplit);
        StringTriple {
            buffer: buffer,
            split: split,
        }
    }

    /// Gets the first string in the triple.
    #[inline]
    pub fn left(&self) -> &str {
        unsafe { self.buffer.slice_unchecked(0, self.split.0) }
    }

    /// Gets the second string in the triple.
    #[inline]
    pub fn middle(&self) -> &str {
        unsafe { self.buffer.slice_unchecked(self.split.0, self.split.1) }
    }

    /// Gets the third string in the triple.
    #[inline]
    pub fn right(&self) -> &str {
        unsafe { self.buffer.slice_unchecked(self.split.1, self.buffer.len()) }
    }
}

impl PartialOrd for StringTriple {
    fn partial_cmp(&self, rhs: &StringTriple) -> Option<Ordering> {
        Some(self.left()
            .cmp(rhs.left())
            .then_with(|| self.middle().cmp(rhs.middle()))
            .then_with(|| self.right().cmp(rhs.right())))
    }
}
impl Ord for StringTriple {
    fn cmp(&self, rhs: &StringTriple) -> Ordering {
        self.left()
            .cmp(rhs.left())
            .then_with(|| self.middle().cmp(rhs.middle()))
            .then_with(|| self.right().cmp(rhs.right()))
    }
}
impl fmt::Debug for StringTriple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("StringTriple::new")
            .field(&self.left().to_owned())
            .field(&self.middle().to_owned())
            .field(&self.right().to_owned())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_left() {
        assert_eq!(StringTriple::from_raw("helloworldtoday", 5, 10).left(),
                   "hello");
    }

    #[test]
    fn raw_middle() {
        assert_eq!(StringTriple::from_raw("helloworldtoday", 5, 10).middle(),
                   "world");
    }

    #[test]
    fn raw_right() {
        assert_eq!(StringTriple::from_raw("helloworldtoday", 5, 10).right(),
                   "today");
    }

    quickcheck! {
        fn partial_ord(s1: String, s2: String, s3: String, s4: String, s5: String, s6: String) -> bool {
            let lhs = StringTriple::new(&*s1, &s2, &s3);
            let rhs = StringTriple::new(&*s4, &s5, &s6);
            lhs.partial_cmp(&rhs) == (&s1, &s2, &s3).partial_cmp(&(&s4, &s5, &s6))
        }
        fn ord(s1: String, s2: String, s3: String, s4: String, s5: String, s6: String) -> bool {
            let lhs = StringTriple::new(&*s1, &s2, &s3);
            let rhs = StringTriple::new(&*s4, &s5, &s6);
            lhs.cmp(&rhs) == (&s1, &s2, &s3).cmp(&(&s4, &s5, &s6))
        }
        fn left(s1: String, s2: String, s3: String) -> bool {
            let pair = StringTriple::new(&*s1, &s2, &s3);
            pair.left() == s1
        }
        fn middle(s1: String, s2: String, s3: String) -> bool {
            let pair = StringTriple::new(&*s1, &s2, &s3);
            pair.middle() == s2
        }
        fn right(s1: String, s2: String, s3: String) -> bool {
            let pair = StringTriple::new(&*s1, &s2, &s3);
            pair.right() == s3
        }
    }
}
