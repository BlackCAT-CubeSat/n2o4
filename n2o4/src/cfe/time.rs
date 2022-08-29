// Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Time Services system.

use cfs_sys::*;
use core::cmp::Ordering;
use core::ops::{Add, Sub};

macro_rules! cfe_time_type {
    ($name:ident : $type_docstring:literal, $accessor_docstring:literal) => {
        #[doc = $type_docstring]
        ///
        /// Wraps `CFE_TIME_SysTime_t`.
        #[doc(alias = "CFE_TIME_SysTime_t")]
        #[derive(Clone, Copy, Debug)]
        pub struct $name {
            pub(crate) tm: CFE_TIME_SysTime_t,
        }

        impl $name {
            #[doc = concat!("Creates a new `", stringify!($name), "` with the specified seconds/subseconds count.")]
            #[inline]
            pub const fn new(seconds: u32, subseconds: u32) -> Self {
                Self { tm: CFE_TIME_SysTime_t { Seconds: seconds, Subseconds: subseconds } }
            }

            #[doc = concat!("Returns the number of whole seconds ", $accessor_docstring, ".")]
            #[inline]
            pub const fn seconds(self) -> u32 {
                self.tm.Seconds
            }

            #[doc = concat!("Returns the fractional number of seconds ", $accessor_docstring)]
            /// (in units of 2<sup>&#8722;32</sup>&nbsp;seconds).
            #[inline]
            pub const fn subseconds(self) -> u32 {
                self.tm.Subseconds
            }
        }

        /// Wraps `CFE_TIME_Compare`.
        impl PartialEq for $name {
            #[doc(alias = "CFE_TIME_Compare")]
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                cfs_sys::CFE_TIME_Compare_CFE_TIME_EQUAL == unsafe { CFE_TIME_Compare(self.tm, other.tm) }
            }
        }

        impl Eq for $name {}

        /// Wraps `CFE_TIME_Compare`.
        impl Ord for $name {
            #[doc(alias = "CFE_TIME_Compare")]
            #[inline]
            fn cmp(&self, other: &Self) -> Ordering {
                match unsafe { CFE_TIME_Compare(self.tm, other.tm) } {
                    cfs_sys::CFE_TIME_Compare_CFE_TIME_A_LT_B => Ordering::Less,
                    cfs_sys::CFE_TIME_Compare_CFE_TIME_EQUAL => Ordering::Equal,
                    _ => Ordering::Greater,
                }
            }
        }

        /// Wraps `CFE_TIME_Compare`.
        impl PartialOrd for $name {
            #[doc(alias = "CFE_TIME_Compare")]
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
    };
}

cfe_time_type!(SysTime:
    "A system time value, represented as seconds/subseconds since some epoch.",
    "since the relevant epoch"
);
cfe_time_type!(DeltaTime:
    "An increment in time, represented in seconds/subseconds.",
    "in the time delta"
);

macro_rules! cfe_time_op {
    ($trait:ident $method:ident $wrapped:ident $wrapped_str:literal : $($lhs:ty , $rhs:ty => $output:ty),*) => {
        $(
            #[doc = concat!("Wraps `", stringify!($wrapped), "`.")]
            impl $trait<$rhs> for $lhs {
                type Output = $output;

                #[doc(alias = $wrapped_str)]
                #[inline]
                fn $method(self, rhs: $rhs) -> $output {
                    Self::Output { tm: unsafe { $wrapped(self.tm, rhs.tm) } }
                }
            }
        )*
    };
}

#[rustfmt::skip]
cfe_time_op! {
    Add add CFE_TIME_Add "CFE_TIME_Add" :
    SysTime   , DeltaTime => SysTime,
    DeltaTime , SysTime   => SysTime,
    DeltaTime , DeltaTime => DeltaTime
}

#[rustfmt::skip]
cfe_time_op! {
    Sub sub CFE_TIME_Subtract "CFE_TIME_Subtract" :
    SysTime   , SysTime   => DeltaTime,
    SysTime   , DeltaTime => SysTime,
    DeltaTime , DeltaTime => DeltaTime
}

/// Returns the current spacecraft time,
/// using the epoch specified in the mission configuration.
///
/// The time is returned either as seconds TAI or seconds UTC,
/// depending on mission configuration.
///
/// Wraps `CFE_TIME_GetTime`.
#[doc(alias = "CFE_TIME_GetTime")]
#[inline]
pub fn get_time() -> SysTime {
    let tm = unsafe { CFE_TIME_GetTime() };
    SysTime { tm }
}
