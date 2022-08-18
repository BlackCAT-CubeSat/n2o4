// Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Time Services system.

use cfs_sys::*;

/// A system time value, represented as seconds since some epoch.
///
/// Wraps `CFE_TIME_SysTime_t`.
#[doc(alias = "CFE_TIME_SysTime_t")]
#[derive(Clone, Copy, Debug)]
pub struct SysTime {
    pub(crate) tm: CFE_TIME_SysTime_t,
}

impl SysTime {
    /// Returns the number of whole seconds since the relevant epoch.
    pub const fn seconds(self) -> u32 {
        self.tm.Seconds
    }

    /// Returns the fractional number of seconds since the relevant epoch
    /// (in units of 2<sup>&#8722;32</sup>&nbsp;seconds).
    pub const fn subseconds(self) -> u32 {
        self.tm.Subseconds
    }
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
