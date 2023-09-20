// Copyright (c) 2021-2023 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! OSAL APIs.

use cfs_sys::*;
use core::ffi::c_ulong;

use crate::utils::NegativeI32;

pub(crate) mod error;
pub mod file;
pub mod socket;
pub mod sync;
pub mod task;

// NOTE: much of the following will probably get moved to submodules as `osal` gets flushed out.

/// The maximum length of strings for names of many OSAL objects.
///
/// The length includes the null terminator.
pub const MAX_NAME_LEN: usize = OS_MAX_API_NAME as usize;

const I_OS_SUCCESS: i32 = OS_SUCCESS as i32;

/// An error code, as returned by many OSAL API functions.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OsalError {
    /// Numeric error code from OSAL.
    pub code: NegativeI32,
}

/// An instant in time.
///
/// Many of the time-related functions in OSAL apply equally to
/// _instants_ in time and to _intervals_ over time. In these bindings,
/// we separate the two for greater clarity as to which is meant.
///
/// Wraps `OS_time_t`.
#[doc(alias = "OS_time_t")]
#[derive(Clone, Copy, Debug)]
pub struct OSTime {
    pub(crate) tm: OS_time_t,
}

/// A time interval.
///
/// Many of the time-related functions in OSAL apply equally to
/// _instants_ in time and to _intervals_ over time. In these bindings,
/// we separate the two for greater clarity as to which is meant.
///
/// Wraps `OS_time_t`.
#[doc(alias = "OS_time_t")]
#[derive(Clone, Copy, Debug)]
pub struct OSTimeInterval {
    pub(crate) int: OS_time_t,
}

/// Methods in common between [`OSTime`] and [`OSTimeInterval`].
macro_rules! time_methods {
    (@
        $fraction_lower:ident, $field:ident, $term:literal,
        $name_from:ident, $wrapped_function_from:ident, $c_from:literal,
        $name_part:ident, $wrapped_function_part:ident, $c_part:literal
        $(, $name_total:ident, $wrapped_function_total:ident, $c_total:literal)?
    ) => {
        #[doc = concat!(
            "Creates the ", $term, " with the specified (seconds, ", stringify!($fraction_lower), ").\n",
            "\n",
            "Wraps `", $c_from, "`.\n",
        )]
        #[doc(alias = $c_from)]
        #[inline]
        pub fn $name_from(seconds: i64, $fraction_lower: u32) -> Self {
            let tm = unsafe { $wrapped_function_from(seconds, $fraction_lower) };
            Self { $field: tm }
        }

        $(
        #[doc = concat!(
            "Converts the ", $term, " into a count of ", stringify!($fraction_lower), ".\n",
            "\n",
            "Wraps `", $c_total, "`.\n",
        )]
        #[doc(alias = $c_total)]
        #[inline]
        pub fn $name_total(&self) -> i64 {
            unsafe { $wrapped_function_total(self.$field) }
        }
        )?

        #[doc = concat!(
            "Returns the fractional-seconds part of the ", $term, " in ", stringify!($fraction_lower), ".\n",
            "\n",
            "Wraps `", $c_part, "`.\n",
        )]
        #[doc(alias = $c_part)]
        #[inline]
        pub fn $name_part(&self) -> u32 {
            unsafe { $wrapped_function_part(self.$field) }
        }
    };
    ($t:ident, $field:ident, $term:literal) => {
        impl $t {
            time_methods!(@
                nanoseconds, $field, $term,
                from_nanoseconds, SHIM_OS_TimeAssembleFromNanoseconds, "OS_TimeAssembleFromNanoseconds",
                nanoseconds_part, SHIM_OS_TimeGetNanosecondsPart, "OS_TimeGetNanosecondsPart",
                total_nanoseconds, SHIM_OS_TimeGetTotalNanoseconds, "OS_TimeGetTotalNanoseconds"
            );

            time_methods!(@
                microseconds, $field, $term,
                from_microseconds, SHIM_OS_TimeAssembleFromMicroseconds, "OS_TimeAssembleFromMicroseconds",
                microseconds_part, SHIM_OS_TimeGetMicrosecondsPart, "OS_TimeGetMicrosecondsPart",
                total_microseconds, SHIM_OS_TimeGetTotalMicroseconds, "OS_TimeGetTotalMicroseconds"
            );

            time_methods!(@
                milliseconds, $field, $term,
                from_milliseconds, SHIM_OS_TimeAssembleFromMilliseconds, "OS_TimeAssembleFromMilliseconds",
                milliseconds_part, SHIM_OS_TimeGetMillisecondsPart, "OS_TimeGetMillisecondsPart",
                total_milliseconds, SHIM_OS_TimeGetTotalMilliseconds, "OS_TimeGetTotalMilliseconds"
            );

            time_methods!(@
                subseconds, $field, $term,
                from_subseconds, SHIM_OS_TimeAssembleFromSubseconds, "OS_TimeAssembleFromSubseconds",
                subseconds_part, SHIM_OS_TimeGetSubsecondsPart, "OS_TimeGetSubsecondsPart"
            );

            #[doc = concat!(
                "Converts the ", $term, " into a count of seconds.\n",
                "\n",
                "Wraps `OS_TimeGetTotalSeconds`.\n"
            )]
            #[doc(alias = "OS_TimeGetTotalSeconds")]
            #[inline]
            pub fn total_seconds(&self) -> i64 {
                unsafe { SHIM_OS_TimeGetTotalSeconds(self.$field) }
            }

            #[doc = concat!(
                "Returns the fractional-seconds part of the ", $term, " in (non-standardized) ticks.\n",
                "\n",
                "Wraps `OS_TimeGetFractionalPart`.\n"
            )]
            #[doc(alias = "OS_TimeGetFractionalPart")]
            #[inline]
            pub fn fractional_part(&self) -> i64 {
                unsafe { SHIM_OS_TimeGetFractionalPart(self.$field) }
            }

            #[inline]
            const fn as_os_time(&self) -> OS_time_t {
                self.$field
            }

            #[inline]
            const fn from_os_time(tm: OS_time_t) -> Self {
                Self { $field: tm }
            }
        }

        impl core::cmp::PartialOrd for $t {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl core::cmp::Ord for $t {
            #[inline]
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.total_nanoseconds().cmp(&other.total_nanoseconds())
            }
        }

        impl core::cmp::PartialEq for $t {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.total_nanoseconds() == other.total_nanoseconds()
            }
        }

        impl core::cmp::Eq for $t {}
    };
}

time_methods!(OSTime, tm, "time");
time_methods!(OSTimeInterval, int, "interval");

/// Quick generation of an implementation of arithmetic for times, time intervals.
macro_rules! arith_impl {
    ($trait:ident, $lhs:ident, $rhs:ident, $method:ident, $result:ident, $func:ident, $func_cname:literal) => {
        #[doc = concat!("Wraps `", $func_cname, "`.")]
        impl $trait<$rhs> for $lhs {
            type Output = $result;

            #[doc(alias = $func_cname)]
            #[inline]
            fn $method(self, other: $rhs) -> $result {
                let s = self.as_os_time();
                let o = other.as_os_time();
                let res = unsafe { $func(s, o) };
                $result::from_os_time(res)
            }
        }
    };
}

#[rustfmt::skip]
mod time_arith_impls {
    use cfs_sys::*;
    use core::ops::{Add, Sub};
    use super::*;

    arith_impl!(Add, OSTime,         OSTimeInterval, add, OSTime,         SHIM_OS_TimeAdd, "OS_TimeAdd");
    arith_impl!(Add, OSTimeInterval, OSTime,         add, OSTime,         SHIM_OS_TimeAdd, "OS_TimeAdd");
    arith_impl!(Add, OSTimeInterval, OSTimeInterval, add, OSTimeInterval, SHIM_OS_TimeAdd, "OS_TimeAdd");

    arith_impl!(Sub, OSTime,         OSTime,         sub, OSTimeInterval, SHIM_OS_TimeSubtract, "OS_TimeSubtract");
    arith_impl!(Sub, OSTime,         OSTimeInterval, sub, OSTime,         SHIM_OS_TimeSubtract, "OS_TimeSubtract");
    arith_impl!(Sub, OSTimeInterval, OSTimeInterval, sub, OSTimeInterval, SHIM_OS_TimeSubtract, "OS_TimeSubtract");
}

/// An identifier for an object managed by OSAL.
///
/// Wraps `osal_id_t`.
#[doc(alias = "osal_id_t")]
#[derive(Clone, Copy, Debug)]
pub struct ObjectId {
    id: osal_id_t,
}

impl ObjectId {
    /// An object ID guaranteed never to refer to a valid resource.
    ///
    /// Wraps `OS_OBJECT_ID_UNDEFINED`.
    #[doc(alias = "OS_OBJECT_ID_UNDEFINED")]
    pub const UNDEFINED: ObjectId = ObjectId { id: X_OS_OBJECT_ID_UNDEFINED };

    /// Returns whether `self` refers to a defined resource.
    ///
    /// Wraps `OS_ObjectIdDefined`.
    #[doc(alias = "OS_ObjectIdDefined")]
    #[inline]
    pub fn is_defined(&self) -> bool {
        unsafe { SHIM_OS_ObjectIdDefined(self.id) }
    }

    /// Returns the object type of `self` as a raw
    /// (non-Rustic) value.
    ///
    /// Wraps `OS_IdentifyObject`.
    #[doc(alias = "OS_IdentifyObject")]
    #[inline]
    pub(crate) fn obj_type(&self) -> osal_objtype_t {
        unsafe { OS_IdentifyObject(self.id) }
    }
}

/// Wraps `OS_ObjectIdFromInteger`.
impl From<c_ulong> for ObjectId {
    #[doc(alias = "OS_ObjectIdFromInteger")]
    #[inline]
    fn from(val: c_ulong) -> ObjectId {
        ObjectId {
            id: unsafe { SHIM_OS_ObjectIdFromInteger(val) },
        }
    }
}

/// Wraps `OS_ObjectIdToInteger`.
impl From<ObjectId> for c_ulong {
    #[doc(alias = "OS_ObjectIdToInteger")]
    #[inline]
    fn from(oid: ObjectId) -> c_ulong {
        unsafe { SHIM_OS_ObjectIdToInteger(oid.id) }
    }
}

/// Wraps `OS_ObjectIdEqual`.
impl PartialEq<Self> for ObjectId {
    #[doc(alias = "OS_ObjectIdEqual")]
    #[inline]
    fn eq(&self, other_id: &Self) -> bool {
        unsafe { SHIM_OS_ObjectIdEqual(self.id, other_id.id) }
    }
}

impl Eq for ObjectId {}

/// Error when trying to convert an `ObjectId` to a
/// more-specialized type.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ObjectTypeConvertError {}

/// Utility function to convert a "timeout or `None`" option into an `i32`,
/// as used by multiple OSAL functions as a timeout value
/// (where negative values mean "wait indefinitely").
#[inline]
pub(crate) fn as_timeout(timeout: Option<u32>) -> i32 {
    timeout.map(|t| t.min(i32::MAX as u32) as i32).unwrap_or(-1)
}
