// Copyright (c) 2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Utility types, functions, etc.
//!
//! This module is for types that aren't cFS-specific,
//! but which turn out to be useful in APIs and not big
//! enough to spin out into their own crates.

use core::ffi::{c_char, CStr};
use core::ops::Deref;

/// A wrapper for [`i32`] that guarantees its value is always negative.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(transparent)]
pub struct NegativeI32 {
    n: i32,
}

impl NegativeI32 {
    /// The minimum possible [`NegativeI32`].
    pub const MIN: Self = Self::new_or_panic(i32::MIN);

    /// The maximum possible [`NegativeI32`].
    pub const MAX: Self = Self::new_or_panic(-1);

    /// If `n` is negative, returns a [`NegativeI32`] with value `n`.
    ///
    /// Otherwise, returns [`None`].
    #[inline]
    pub const fn new(n: i32) -> Option<Self> {
        if n < 0 {
            Some(Self { n })
        } else {
            None
        }
    }

    /// If `n` is negative, returns a [`NegativeI32`] with value `n`.
    ///
    /// This variant of [`new()`](Self::new) is especially useful
    /// for creating compile-time constants.
    ///
    /// # Panics
    ///
    /// If `n` is non-negative, this function will panic.
    #[inline]
    pub const fn new_or_panic(n: i32) -> Self {
        match Self::new(n) {
            Some(ni32) => ni32,
            None => {
                panic!("Tried to create a NegativeI32 using a non-negative i32!");
            }
        }
    }

    /// Returns the value of `self` as an [`i32`].
    #[inline]
    pub const fn as_i32(self) -> i32 {
        self.n
    }
}

impl From<NegativeI32> for i32 {
    #[inline]
    fn from(val: NegativeI32) -> Self {
        val.n
    }
}

/// Error: an attempt was made to convert a non-negative value to a [`NegativeI32`].
#[derive(Clone, Copy, Debug)]
pub struct NotNegativeError {}

impl TryFrom<i32> for NegativeI32 {
    type Error = NotNegativeError;

    #[inline]
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(NotNegativeError {})
    }
}

/// An owned null-terminated C-compatible string of at most `SIZE` bytes
/// (including null terminator).
#[derive(Clone, Copy, Debug)]
pub struct CStrBuf<const SIZE: usize> {
    buf: [c_char; SIZE],
}

const fn min(a: usize, b: usize) -> usize {
    if a <= b {
        a
    } else {
        b
    }
}

impl<const SIZE: usize> CStrBuf<SIZE> {
    /// Creates a new `CStrBuf<SIZE>` from `src`;
    /// if `src` is longer than `SIZE - 1` bytes,
    /// only the first `SIZE - 1` bytes of `src`
    /// are copied over.
    ///
    /// # Panics
    ///
    /// Panics if and only if `SIZE` is `0`.
    #[inline]
    pub const fn new(src: &[c_char]) -> Self {
        if SIZE == 0 {
            panic!("CStrBuf instances of length 0 not allowed");
        }

        let mut buf = [b'\0' as c_char; SIZE];

        let copy_len = min(src.len(), SIZE - 1);

        let mut i = 0usize;
        while i < copy_len {
            buf[i] = src[i];
            i += 1;
        }

        Self { buf }
    }

    /// Creates a new `CStrBuf<SIZE>` from `src`, handling conversion from [`u8`] to [`c_char`];
    /// if `src` is longer than `SIZE - 1` bytes,
    /// only the first `SIZE - 1` bytes of `src`
    /// are copied over.
    ///
    /// # Panics
    ///
    /// Panics if and only if `SIZE` is `0`.
    #[inline]
    pub const fn new_u8(src: &[u8]) -> Self {
        if SIZE == 0 {
            panic!("CStrBuf instances of length 0 not allowed");
        }

        let mut buf = [b'\0' as c_char; SIZE];

        let copy_len = min(src.len(), SIZE - 1);

        let mut i = 0usize;
        while i < copy_len {
            buf[i] = src[i] as c_char;
            i += 1;
        }

        Self { buf }
    }

    /// Creates a new `CStrBuf<SIZE>` using `src`.
    ///
    /// `src` is modified to ensure null-termination.
    ///
    /// # Panics
    ///
    /// Panics if and only if `SIZE` is `0`.
    #[inline]
    pub const fn new_into(src: [c_char; SIZE]) -> Self {
        if SIZE == 0 {
            panic!("CStrBuf instances of length 0 not allowed");
        }

        let mut src = src;
        src[SIZE - 1] = b'\0' as c_char;
        Self { buf: src }
    }

    /// Creates a new `CStrBuf<SIZE>` from the string `src`.
    ///
    /// If `src` is longer than `SIZE` (including null-terminator),
    /// only the first `SIZE-1` characters will be copied over.
    ///
    /// # Panics
    ///
    /// Panics if and only if `SIZE` is `0`.
    #[inline]
    pub const fn from_cstr(src: &CStr) -> Self {
        if SIZE == 0 {
            panic!("CStrBuf instances of length 0 not allowed");
        }

        let mut buf: [c_char; SIZE] = [b'\0' as c_char; SIZE];
        let ptr = src.as_ptr();

        let mut i = 0;
        while i < SIZE - 1 {
            let next_char = unsafe { *(ptr.add(i)) };
            buf[i] = next_char;
            if next_char == (b'\0' as c_char) {
                break;
            }
            i += 1;
        }

        Self { buf }
    }

    /// Returns a pointer to the start of the string.
    #[inline]
    pub const fn as_ptr(&self) -> *const c_char {
        self.buf.as_ptr()
    }

    /// Returns a reference to the underlying array.
    #[inline]
    pub const fn as_array(&self) -> &[c_char; SIZE] {
        &self.buf
    }
}

impl<const SIZE: usize> Deref for CStrBuf<SIZE> {
    type Target = [c_char; SIZE];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl<const SIZE: usize> AsRef<CStr> for CStrBuf<SIZE> {
    #[inline]
    fn as_ref(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.buf.as_ptr()) }
    }
}

impl<const SIZE: usize, const OTHER: usize> PartialEq<CStrBuf<OTHER>> for CStrBuf<SIZE> {
    #[inline]
    fn eq(&self, other: &CStrBuf<OTHER>) -> bool {
        let s: &CStr = self.as_ref();
        let o: &CStr = other.as_ref();

        *s == *o
    }
}

impl<const SIZE: usize> Eq for CStrBuf<SIZE> {}

/// A way to get the `Atomic*` type associated with a given integer type.
pub(crate) trait AtomicVersion {
    /// The atomic type of the same size and signedness as `Self`.
    type Atomic;
}

mod atomic_version_impls {
    macro_rules! atom {
        ($base:ty, $atomic:ident) => {
            impl super::AtomicVersion for $base {
                type Atomic = core::sync::atomic::$atomic;
            }
        };
    }

    atom!(u8, AtomicU8);
    atom!(u16, AtomicU16);
    atom!(u32, AtomicU32);
    atom!(u64, AtomicU64);
    atom!(usize, AtomicUsize);

    atom!(i8, AtomicI8);
    atom!(i16, AtomicI16);
    atom!(i32, AtomicI32);
    atom!(i64, AtomicI64);
    atom!(isize, AtomicIsize);
}

/// A type for which no values can possibly exist.
pub enum Unconstructable {}
