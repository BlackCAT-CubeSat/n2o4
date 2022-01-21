// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

//! OSAL APIs

use cfs_sys::*;
use libc::c_ulong;

use core::ops::{Add,Sub};

// NOTE: the following will probably get moved to submodules as `osal` gets flushed out.

/// An instant in time.
///
/// Many of the time-related functions in OSAL apply equally to
/// _instants_ in time and to _intervals_ over time. In these bindings,
/// we separate the two for greater clarity as to which is meant.
#[derive(Clone,Copy,Debug)]
pub struct OSTime {
    pub(crate) tm: OS_time_t
}

/// A time interval.
///
/// Many of the time-related functions in OSAL apply equally to
/// _instants_ in time and to _intervals_ over time. In these bindings,
/// we separate the two for greater clarity as to which is meant.
#[derive(Clone,Copy,Debug)]
pub struct OSTimeInterval {
    pub(crate) int: OS_time_t
}

macro_rules! time_methods {
    (@
        $fraction_lower:ident, $fraction_upper:ident,
        $field:ident, $term:literal,
        $name_from:ident, $wrapped_function_from:ident,
        $name_part:ident, $wrapped_function_part:ident
        $(, $name_total:ident, $wrapped_function_total:ident)?
    ) => {
        #[doc = concat!(
            "Creates the ", $term, " with the specified (seconds, ", stringify!($fraction_lower), ").\n",
            "\n",
            "Wraps OS_TimeAssembleFrom", stringify!($fraction_upper), ".\n",
        )]
        #[inline]
        pub fn $name_from(seconds: i64, $fraction_lower: u32) -> Self {
            let tm = unsafe { $wrapped_function_from(seconds, $fraction_lower) };
            Self { $field: tm }
        }

        $(
        #[doc = concat!(
            "Converts the ", $term, " into a count of ", stringify!($fraction_lower), ".\n",
            "\n",
            "Wraps OS_TimeGetTotal", stringify!($fraction_upper), ".\n",
        )]
        #[inline]
        pub fn $name_total(&self) -> i64 {
            unsafe { $wrapped_function_total(self.$field) }
        }
        )?

        #[doc = concat!(
            "Returns the fractional-seconds part of the ", $term, " in ", stringify!($fraction_lower), ".\n",
            "\n",
            "Wraps OS_TimeGet", stringify!($fraction_upper), "Part.\n",
        )]
        #[inline]
        pub fn $name_part(&self) -> u32 {
            unsafe { $wrapped_function_part(self.$field) }
        }
    };
    ($t:ident, $field:ident, $term:literal) => {
        impl $t {
            time_methods!(@
                nanoseconds, Nanoseconds, $field, $term,
                from_nanoseconds, SHIM_OS_TimeAssembleFromNanoseconds,
                nanoseconds_part, SHIM_OS_TimeGetNanosecondsPart,
                total_nanoseconds, SHIM_OS_TimeGetTotalNanoseconds
            );

            time_methods!(@
                microseconds, Microseconds, $field, $term,
                from_microseconds, SHIM_OS_TimeAssembleFromMicroseconds,
                microseconds_part, SHIM_OS_TimeGetMicrosecondsPart,
                total_microseconds, SHIM_OS_TimeGetTotalMicroseconds
            );

            time_methods!(@
                milliseconds, Milliseconds, $field, $term,
                from_milliseconds, SHIM_OS_TimeAssembleFromMilliseconds,
                milliseconds_part, SHIM_OS_TimeGetMillisecondsPart,
                total_milliseconds, SHIM_OS_TimeGetTotalMilliseconds
            );

            time_methods!(@
                subseconds, Subseconds, $field, $term,
                from_subseconds, SHIM_OS_TimeAssembleFromSubseconds,
                subseconds_part, SHIM_OS_TimeGetSubsecondsPart
            );

            #[doc = concat!(
                "Converts the ", $term, " into a count of seconds.\n",
                "\n",
                "Wraps OS_TimeGetTotalSeconds.\n"
            )]
            #[inline]
            pub fn total_seconds(&self) -> i64 {
                unsafe { SHIM_OS_TimeGetTotalSeconds(self.$field) }
            }

            #[doc = concat!(
                "Returns the fractional-seconds part of the ", $term, " in (non-standardized) ticks.\n",
                "\n",
                "Wraps OS_TimeGetFractionalPart.\n"
            )]
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
    };
}

time_methods!(OSTime, tm, "time");
time_methods!(OSTimeInterval, int, "interval");

macro_rules! arith_impl {
    ($trait:ident, $lhs:ident, $rhs:ident, $method:ident, $result:ident, $func:ident, $func_suffix:ident) => {
        #[doc = concat!("Wraps OS_Time", stringify!($func_suffix), ".")]
        impl $trait<$rhs> for $lhs {
            type Output = $result;

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

arith_impl!(Add, OSTime,         OSTimeInterval, add, OSTime,         SHIM_OS_TimeAdd, Add);
arith_impl!(Add, OSTimeInterval, OSTime,         add, OSTime,         SHIM_OS_TimeAdd, Add);
arith_impl!(Add, OSTimeInterval, OSTimeInterval, add, OSTimeInterval, SHIM_OS_TimeAdd, Add);

arith_impl!(Sub, OSTime,         OSTime,         sub, OSTimeInterval, SHIM_OS_TimeSubtract, Subtract);
arith_impl!(Sub, OSTime,         OSTimeInterval, sub, OSTime,         SHIM_OS_TimeSubtract, Subtract);
arith_impl!(Sub, OSTimeInterval, OSTimeInterval, sub, OSTimeInterval, SHIM_OS_TimeSubtract, Subtract);

#[derive(Clone,Copy,Debug)]
pub struct ObjectId {
    id: osal_id_t
}

impl ObjectId {
    /// Returns whether `self` refers to a defined resource.
    ///
    /// Wraps OS_ObjectIdDefined.
    #[inline]
    pub fn is_defined(&self) -> bool {
        unsafe { SHIM_OS_ObjectIdDefined(self.id) }
    }
}

/// Wraps OS_ObjectIdFromInteger.
impl From<c_ulong> for ObjectId {
    #[inline]
    fn from(val: c_ulong) -> ObjectId {
        ObjectId { id: unsafe { SHIM_OS_ObjectIdFromInteger(val) } }
    }
}

/// Wraps OS_ObjectIdToInteger.
impl From<ObjectId> for c_ulong {
    #[inline]
    fn from(oid: ObjectId) -> c_ulong {
        unsafe { SHIM_OS_ObjectIdToInteger(oid.id) }
    }
}

/// Wraps OS_ObjectIdEqual.
impl PartialEq<Self> for ObjectId {
    #[inline]
    fn eq(&self, other_id: &Self) -> bool {
        unsafe { SHIM_OS_ObjectIdEqual(self.id, other_id.id) }
    }
}

impl Eq for ObjectId { }
