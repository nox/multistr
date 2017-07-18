use std::borrow::{Borrow, BorrowMut};
use std::ffi::{CStr, FromBytesWithNulError};
use std::fmt;
use std::mem;
use std::ops::{Index, RangeFull};
use std::str::{Utf8Error, from_utf8, from_utf8_unchecked};

use bow::ToBox;
use extra_default::DefaultRef;
use len_trait::{CapacityMut, Len, LenMut, SplitAt, SplitAtMut};
use push_trait::PushBack;
use void::Void;

/// Required for `StrLike::Data`.
pub trait StrData: ToBox + Len + SplitAt<usize> + Index<RangeFull, Output = Self> + DefaultRef {}
impl<T: ?Sized + ToBox + Len + SplitAt<usize> + Index<RangeFull, Output = Self> + DefaultRef> StrData for T {}


/// Required for `StrLike::OwnedData`.
pub trait OwnsStrData<D: ?Sized>: LenMut + CapacityMut + for<'a> PushBack<&'a D> + Into<Box<D>> {}
impl<D: ?Sized, T: ?Sized + LenMut + CapacityMut + for<'a> PushBack<&'a D> + Into<Box<D>>> OwnsStrData<D> for T {}


/// String-like container.
pub trait StrLike: Len + ToOwned + DefaultRef + 'static {
    /// Data backing this string.
    type Data: ?Sized + StrData + ToOwned<Owned = Self::OwnedData> + 'static;

    /// Owned data.
    type OwnedData: OwnsStrData<Self::Data> + Borrow<Self::Data>;

    /// Error that comes up when converting data back into a string.
    type ConvError: fmt::Display;

    /// Converts this string into its data backend.
    fn to_data(&self) -> &Self::Data;

    /// Coerces a string from its backend data, performing validation if necessary.
    fn from_data(data: &Self::Data) -> Result<&Self, Self::ConvError>;

    /// Similar to `from_data`, ignoring validity checking.
    unsafe fn from_data_unchecked(data: &Self::Data) -> &Self;
}

/// Extension to `StrLike`: types where concatenating data is equivalent to concatenating strings.
pub unsafe trait DataConcat: StrLike {}

/// Extension to `StrLike`: has mutable version.
pub trait StrLikeMut: StrLike
    where Self::Data: SplitAtMut<usize>,
          Self::OwnedData: BorrowMut<Self::Data>
{
    /// Mutable version of `to_data`.
    unsafe fn to_data_mut(&mut self) -> &mut Self::Data;

    /// Mutable version of `from_data`.
    fn from_data_mut(data: &mut Self::Data) -> Result<&mut Self, Self::ConvError>;

    /// Mutable version of `from_data_unchecked`.
    unsafe fn from_data_mut_unchecked(data: &mut Self::Data) -> &mut Self;
}

impl<T: 'static + Copy> StrLike for [T] {
    type Data = [T];
    type OwnedData = Vec<T>;

    type ConvError = Void;

    fn to_data(&self) -> &[T] {
        self
    }
    fn from_data(data: &[T]) -> Result<&[T], Void> {
        Ok(data)
    }
    unsafe fn from_data_unchecked(data: &[T]) -> &[T] {
        data
    }
}

impl StrLike for str {
    type Data = [u8];
    type OwnedData = Vec<u8>;

    type ConvError = Utf8Error;

    fn to_data(&self) -> &[u8] {
        self.as_bytes()
    }
    fn from_data(data: &[u8]) -> Result<&str, Utf8Error> {
        from_utf8(data)
    }
    unsafe fn from_data_unchecked(data: &[u8]) -> &str {
        from_utf8_unchecked(data)
    }
}

impl StrLike for CStr {
    type Data = [u8];
    type OwnedData = Vec<u8>;

    type ConvError = FromBytesWithNulError;

    fn to_data(&self) -> &[u8] {
        self.to_bytes_with_nul()
    }
    fn from_data(data: &[u8]) -> Result<&CStr, FromBytesWithNulError> {
        CStr::from_bytes_with_nul(data)
    }
    unsafe fn from_data_unchecked(data: &[u8]) -> &CStr {
        CStr::from_bytes_with_nul_unchecked(data)
    }
}

unsafe impl DataConcat for str {}
unsafe impl<T: 'static + Copy> DataConcat for [T] {}

impl<T: 'static + Copy> StrLikeMut for [T] {
    unsafe fn to_data_mut(&mut self) -> &mut [T] {
        self
    }
    fn from_data_mut(data: &mut [T]) -> Result<&mut [T], Void> {
        Ok(data)
    }
    unsafe fn from_data_mut_unchecked(data: &mut [T]) -> &mut [T] {
        data
    }
}

impl StrLikeMut for str {
    unsafe fn to_data_mut(&mut self) -> &mut [u8] {
        mem::transmute(self)
    }
    fn from_data_mut(data: &mut [u8]) -> Result<&mut str, Utf8Error> {
        from_utf8(data)?;
        Ok(unsafe { mem::transmute(data) })
    }
    unsafe fn from_data_mut_unchecked(data: &mut [u8]) -> &mut str {
        mem::transmute(data)
    }
}
