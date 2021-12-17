// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

//! OSAL APIs

use cfs_sys::*;
use libc::c_ulong;

use core::ops::{Add,Sub};

// NOTE: the following will probably get moved to submodules as `osal` gets flushed out.

#[derive(Clone,Copy,Debug)]
pub struct OSTime {
    tm: OS_time_t
}

#[derive(Clone,Copy,Debug)]
pub struct OSTimeInterval {
    int: OS_time_t
}

macro_rules! time_methods {
    ($t:ident, $field:ident) => {
        impl $t {
            #[inline]
            pub fn from_nanoseconds(seconds: i64, nanoseconds: u32) -> Self {
                let tm = unsafe { SHIM_OS_TimeAssembleFromNanoseconds(seconds, nanoseconds) };
                $t { $field: tm }
            }

            #[inline]
            pub fn from_microseconds(seconds: i64, microseconds: u32) -> Self {
                let tm = unsafe { SHIM_OS_TimeAssembleFromMicroseconds(seconds, microseconds) };
                $t { $field: tm }
            }

            #[inline]
            pub fn from_milliseconds(seconds: i64, milliseconds: u32) -> Self {
                let tm = unsafe { SHIM_OS_TimeAssembleFromMilliseconds(seconds, milliseconds) };
                $t { $field: tm }
            }

            #[inline]
            pub fn from_subseconds(seconds: i64, subseconds: u32) -> Self {
                let tm = unsafe { SHIM_OS_TimeAssembleFromSubseconds(seconds, subseconds) };
                $t { $field: tm }
            }

            #[inline]
            pub fn total_seconds(&self) -> i64 {
                unsafe { SHIM_OS_TimeGetTotalSeconds(self.$field) }
            }

            #[inline]
            pub fn total_milliseconds(&self) -> i64 {
                unsafe { SHIM_OS_TimeGetTotalMilliseconds(self.$field) }
            }

            #[inline]
            pub fn total_microseconds(&self) -> i64 {
                unsafe { SHIM_OS_TimeGetTotalMicroseconds(self.$field) }
            }

            #[inline]
            pub fn total_nanoseconds(&self) -> i64 {
                unsafe { SHIM_OS_TimeGetTotalNanoseconds(self.$field) }
            }

            #[inline]
            pub fn fractional_part(&self) -> i64 {
                unsafe { SHIM_OS_TimeGetFractionalPart(self.$field) }
            }

            #[inline]
            pub fn subseconds_part(&self) -> u32 {
                unsafe { SHIM_OS_TimeGetSubsecondsPart(self.$field) }
            }

            #[inline]
            pub fn milliseconds_part(&self) -> u32 {
                unsafe { SHIM_OS_TimeGetMillisecondsPart(self.$field) }
            }

            #[inline]
            pub fn microseconds_part(&self) -> u32 {
                unsafe { SHIM_OS_TimeGetMicrosecondsPart(self.$field) }
            }

            #[inline]
            pub fn nanoseconds_part(&self) -> u32 {
                unsafe { SHIM_OS_TimeGetNanosecondsPart(self.$field) }
            }
        }

        impl From<OS_time_t> for $t {
            #[inline]
            fn from(tm: OS_time_t) -> $t {
                $t { $field: tm }
            }
        }

        impl From<$t> for OS_time_t {
            #[inline]
            fn from(tm: $t) -> OS_time_t {
                tm.$field
            }
        }
    };
}

time_methods!(OSTime, tm);
time_methods!(OSTimeInterval, int);

macro_rules! arith_impl {
    ($trait:ident, $lhs:ident, $rhs:ident, $method:ident, $result:ident, $func:ident) => {
        impl $trait<$rhs> for $lhs {
            type Output = $result;

            #[inline]
            fn $method(self, other: $rhs) -> $result {
                let s = self.into();
                let o = other.into();
                unsafe { $func(s, o) }.into()
            }
        }
    };
}

arith_impl!(Add, OSTime,         OSTimeInterval, add, OSTime,         SHIM_OS_TimeAdd);
arith_impl!(Add, OSTimeInterval, OSTime,         add, OSTime,         SHIM_OS_TimeAdd);
arith_impl!(Add, OSTimeInterval, OSTimeInterval, add, OSTimeInterval, SHIM_OS_TimeAdd);

arith_impl!(Sub, OSTime,         OSTime,         sub, OSTimeInterval, SHIM_OS_TimeSubtract);
arith_impl!(Sub, OSTime,         OSTimeInterval, sub, OSTime,         SHIM_OS_TimeSubtract);
arith_impl!(Sub, OSTimeInterval, OSTimeInterval, sub, OSTimeInterval, SHIM_OS_TimeSubtract);

#[derive(Clone,Copy,Debug)]
pub struct ObjectId {
    id: osal_id_t
}

impl ObjectId {
    #[inline]
    pub fn is_defined(&self) -> bool {
        unsafe { SHIM_OS_ObjectIdDefined(self.id) }
    }
}

impl From<c_ulong> for ObjectId {
    #[inline]
    fn from(val: c_ulong) -> ObjectId {
        ObjectId { id: unsafe { SHIM_OS_ObjectIdFromInteger(val) } }
    }
}

impl From<ObjectId> for c_ulong {
    #[inline]
    fn from(oid: ObjectId) -> c_ulong {
        unsafe { SHIM_OS_ObjectIdToInteger(oid.id) }
    }
}

impl PartialEq<Self> for ObjectId {
    #[inline]
    fn eq(&self, other_id: &Self) -> bool {
        unsafe { SHIM_OS_ObjectIdEqual(self.id, other_id.id) }
    }
}

impl Eq for ObjectId { }
