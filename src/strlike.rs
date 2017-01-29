use std::borrow::Borrow;
use std::ffi::{CStr, FromBytesWithNulError};
use std::fmt;
use std::str::{Utf8Error, from_utf8, from_utf8_unchecked};

use bow::ToBox;
use extra_default::DefaultRef;
use len_trait::{CapacityMut, Len, SplitAt, SplitOff};
use push_trait::PushRefBack;

/// Required for `StrLike::Data`.
pub trait StrData: ToBox + SplitAt + DefaultRef {}
impl<T: ?Sized + ToBox + SplitAt + DefaultRef> StrData for T {}


/// Required for `StrLike::OwnedData`.
pub trait OwnsStrData<D: ?Sized>: SplitOff + CapacityMut + PushRefBack<D> + Into<Box<D>> {}
impl<D: ?Sized, T: ?Sized + SplitOff + CapacityMut + PushRefBack<D> + Into<Box<D>>> OwnsStrData<D> for T {}


/// String-like container.
pub trait StrLike: Len + ToOwned + DefaultRef {
    /// Data backing this string.
    type Data: ?Sized + StrData + ToOwned<Owned = Self::OwnedData>;

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

impl<T: 'static + Copy> StrLike for [T] {
    type Data = [T];
    type OwnedData = Vec<T>;

    type ConvError = !;

    fn to_data(&self) -> &[T] {
        self
    }
    fn from_data(data: &[T]) -> Result<&[T], !> {
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
