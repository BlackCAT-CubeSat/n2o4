// Copyright (c) 2021-2023 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Time Services system.

use crate::sys::*;
use core::cmp::Ordering;
use core::ops::{Add, Sub};

macro_rules! cfe_time_type {
    ($name:ident : $type_docstring:literal, $accessor_docstring:literal, $osal:ty) => {
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
            /// (in type-native units of 2<sup>&#8722;32</sup>&nbsp;seconds).
            #[inline]
            pub const fn subseconds(self) -> u32 {
                self.tm.Subseconds
            }

            #[doc = concat!("Returns the fractional number of seconds ", $accessor_docstring)]
            /// (in units of microseconds).
            ///
            /// Wraps `CFE_TIME_Sub2MicroSecs`.
            #[doc(alias = "CFE_TIME_Sub2MicroSecs")]
            #[inline]
            pub fn microseconds(self) -> u32 {
                unsafe { CFE_TIME_Sub2MicroSecs(self.tm.Subseconds) }
            }
        }

        /// Wraps `CFE_TIME_Compare`.
        impl PartialEq for $name {
            #[doc(alias = "CFE_TIME_Compare")]
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                crate::sys::CFE_TIME_Compare_CFE_TIME_EQUAL == unsafe { CFE_TIME_Compare(self.tm, other.tm) }
            }
        }

        impl Eq for $name {}

        /// Wraps `CFE_TIME_Compare`.
        impl Ord for $name {
            #[doc(alias = "CFE_TIME_Compare")]
            #[inline]
            fn cmp(&self, other: &Self) -> Ordering {
                match unsafe { CFE_TIME_Compare(self.tm, other.tm) } {
                    crate::sys::CFE_TIME_Compare_CFE_TIME_A_LT_B => Ordering::Less,
                    crate::sys::CFE_TIME_Compare_CFE_TIME_EQUAL => Ordering::Equal,
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

        /// Wraps `OS_TimeGetTotalSeconds`, `OS_TimeGetMicrosecondsPart`, and `CFE_TIME_Micro2SubSecs`.
        impl TryFrom<$osal> for $name {
            type Error = core::num::TryFromIntError;

            #[inline]
            fn try_from(value: $osal) -> Result<Self, Self::Error> {
                let seconds: u32 = value.total_seconds().try_into()?;
                let subseconds = unsafe { CFE_TIME_Micro2SubSecs(value.microseconds_part()) };
                Ok(Self::new(seconds, subseconds))
            }
        }

        /// Wraps `OS_TimeAssembleFromMicroseconds` and `CFE_TIME_Sub2MicroSecs`.
        impl From<$name> for $osal {
            #[inline]
            fn from(value: $name) -> Self {
                let microseconds = unsafe { CFE_TIME_Sub2MicroSecs(value.subseconds()) };
                <$osal>::from_microseconds(value.seconds() as i64, microseconds)
            }
        }
    };
}

cfe_time_type!(SysTime:
    "A system time value, represented as seconds/subseconds since some epoch.",
    "since the relevant epoch",
    crate::osal::OSTime
);
cfe_time_type!(DeltaTime:
    "An increment in time, represented in seconds/subseconds.",
    "in the time delta",
    crate::osal::OSTimeInterval
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

/// Converts `microseconds` &mu;s to units of cFE sub-seconds (2<sup>&#8722;32</sup>&nbsp;seconds),
/// or returns `!0` if `microseconds` is over `999_999`.
///
/// Wraps `CFE_TIME_Micro2SubSecs`.
#[doc(alias = "CFE_TIME_Micro2SubSecs")]
#[inline]
pub fn micro_to_subsecs(microseconds: u32) -> u32 {
    unsafe { CFE_TIME_Micro2SubSecs(microseconds) }
}

/// Converts `subseconds` cFE sub-seconds (2<sup>&#8722;32</sup>&nbsp;seconds) to microseconds.
///
/// Wraps `CFE_TIME_Sub2MicroSecs`.
#[doc(alias = "CFE_TIME_Sub2MicroSecs")]
#[inline]
pub fn sub_to_microsecs(subseconds: u32) -> u32 {
    unsafe { CFE_TIME_Sub2MicroSecs(subseconds) }
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
