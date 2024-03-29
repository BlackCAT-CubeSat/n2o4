// Copyright (c) 2021-2023 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! cFE APIs.

use crate::sys::*;
use core::ffi::c_ulong;

pub mod es;
pub mod evs;
pub mod fs;
pub mod msg;
pub mod sb;
pub mod tbl;
pub mod time;

mod status_consts;

use printf_wrap::{null_str, NullString};

/// An ID to identify cFE-managed resources.
///
/// Wraps `CFE_ResourceId_t`.
#[doc(alias = "CFE_ResourceId_t")]
#[derive(Clone, Copy, Debug)]
pub struct ResourceId {
    id: CFE_ResourceId_t,
}

impl ResourceId {
    /// Checks if a resource ID value is defined.
    ///
    /// Wraps `CFE_ResourceId_IsDefined`.
    #[doc(alias = "CFE_ResourceId_IsDefined")]
    #[inline]
    pub fn is_defined(&self) -> bool {
        unsafe { SHIM_CFE_ResourceId_IsDefined(self.id) }
    }

    /// A value that represents an undefined/unused resource.
    ///
    /// Wraps `CFE_RESOURCEID_UNDEFINED`.
    #[doc(alias = "CFE_RESOURCEID_UNDEFINED")]
    pub const UNDEFINED: Self = ResourceId { id: X_CFE_RESOURCEID_UNDEFINED };

    /// A value that represents a reserved entry.
    ///
    /// Wraps `CFE_RESOURCEID_RESERVED`.
    #[doc(alias = "CFE_RESOURCEID_RESERVED")]
    pub const RESERVED: Self = ResourceId { id: X_CFE_RESOURCEID_RESERVED };
}

/// Wraps `CFE_ResourceId_Equal`.
impl PartialEq<ResourceId> for ResourceId {
    #[doc(alias = "CFE_ResourceId_Equal")]
    #[inline]
    fn eq(&self, other: &ResourceId) -> bool {
        unsafe { SHIM_CFE_ResourceId_Equal(self.id, other.id) }
    }
}

impl Eq for ResourceId {}

/// Wraps `CFE_ResourceId_FromInteger`.
impl From<c_ulong> for ResourceId {
    #[doc(alias = "CFE_ResourceId_FromInteger")]
    #[inline]
    fn from(val: c_ulong) -> ResourceId {
        let rid = unsafe { SHIM_CFE_ResourceId_FromInteger(val) };
        ResourceId { id: rid }
    }
}

/// Wraps `CFE_ResourceId_ToInteger`.
impl From<ResourceId> for c_ulong {
    #[doc(alias = "CFE_ResourceId_ToInteger")]
    #[inline]
    fn from(id: ResourceId) -> c_ulong {
        unsafe { SHIM_CFE_ResourceId_ToInteger(id.id) }
    }
}

/// The numeric type corresponding to a [`Status`].
///
/// This is the same as `CFE_Status_t`.
#[doc(alias = "CFE_Status_t")]
#[doc(inline)]
pub use crate::sys::CFE_Status_t as CFE_Status;

/// A status-code type often used as a return type in this crate.
///
/// Wraps `CFE_Status_t`.
#[doc(alias = "CFE_Status_t")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Status {
    pub(crate) status: CFE_Status_t,
}

impl From<CFE_Status> for Status {
    #[inline]
    fn from(status: CFE_Status) -> Status {
        Status { status: status }
    }
}

impl From<Status> for CFE_Status {
    #[inline]
    fn from(status: Status) -> CFE_Status {
        status.status
    }
}

/// The severity part of a [`Status`].
#[doc(alias = "CFE_SEVERITY_BITMASK")]
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StatusSeverity {
    /// Operation completed successfully.
    #[doc(alias = "CFE_SEVERITY_SUCCESS")]
    Success = 0b00,

    /// Operation not completed for non-error reasons, or completed with caveats.
    #[doc(alias = "CFE_SEVERITY_INFO")]
    Informational = 0b01,

    /// Operation failed.
    #[doc(alias = "CFE_SEVERITY_ERROR")]
    Error   = 0b11,
}

/// The cFE service that generated a [`Status`].
#[doc(alias = "CFE_SERVICE_BITMASK")]
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StatusServiceId {
    /// Not actually a cFE service;
    /// use this value for application-defined statuses.
    NotCfe  = 0b000,

    /// Event service.
    #[doc(alias = "CFE_EVENTS_SERVICE")]
    EVS     = 0b001,

    /// Executive service.
    #[doc(alias = "CFE_EXECUTIVE_SERVICE")]
    ES      = 0b010,

    /// File service.
    #[doc(alias = "CFE_FILE_SERVICE")]
    FS      = 0b011,

    /// Generic service.
    #[doc(alias = "CFE_GENERIC_SERVICE")]
    Generic = 0b100,

    /// Software Bus service.
    #[doc(alias = "CFE_SOFTWARE_BUS_SERVICE")]
    SB      = 0b101,

    /// Table service.
    #[doc(alias = "CFE_TABLE_SERVICE")]
    TBL     = 0b110,

    /// Time service.
    #[doc(alias = "CFE_TIME_SERVICE")]
    TIME    = 0b111,
}

impl Status {
    /// Constructs a `Status` from its component parts.
    ///
    /// ## Notes
    ///
    /// * Only the lower 9 bits of `mission_defined` get used.
    /// * All 16 bits of `code` get used.
    #[inline]
    pub const fn new(
        severity: StatusSeverity,
        service_id: StatusServiceId,
        mission_defined: u16,
        code: u16,
    ) -> Status {
        let n = (severity as u32) << 30
            | (service_id as u32) << 25
            | ((mission_defined as u32) & 0x01ff) << 16
            | (code as u32);
        Status { status: n as CFE_Status_t }
    }

    /// Returns the status's severity.
    #[doc(alias = "CFE_SEVERITY_BITMASK")]
    #[inline]
    pub const fn severity(&self) -> StatusSeverity {
        use StatusSeverity::*;

        match (self.status >> 30) & 0b0011 {
            0b00 => Success,
            0b01 => Informational,
            _ => Error,
        }
    }

    /// Returns the cFE service that generated this status.
    #[doc(alias = "CFE_SERVICE_BITMASK")]
    #[inline]
    pub const fn service(&self) -> StatusServiceId {
        use StatusServiceId::*;

        match (self.status >> 25) & 0b0111 {
            0b000 => NotCfe,
            0b001 => EVS,
            0b010 => ES,
            0b011 => FS,
            0b100 => Generic,
            0b101 => SB,
            0b110 => TBL,
            _ => TIME,
        }
    }

    /// Returns the mission-defined portion of the status.
    #[inline]
    pub const fn mission_defined(&self) -> u16 {
        ((self.status >> 16) & 0x01ff) as u16
    }

    /// Returns the status code.
    #[inline]
    pub const fn code(&self) -> u16 {
        self.status as u16
    }

    /// If `self` has a severity of [`Success`](`StatusSeverity::Success`),
    /// returns `Ok(on_success())`;
    /// otherwise returns `Err(self)`.
    #[inline]
    pub(crate) fn as_result<T, F: FnOnce() -> T>(&self, on_success: F) -> Result<T, Status> {
        match self.severity() {
            StatusSeverity::Success => Ok(on_success()),
            _ => Err(*self),
        }
    }

    /// Returns the status as a 32-bit number.
    #[inline]
    pub fn as_num(&self) -> u32 {
        self.status as u32
    }
}

/// Format string for using a Rust [`str`] in
/// [`printf(3)`](https://www.freebsd.org/cgi/man.cgi?printf%283%29)-style C functions.
const RUST_STR_FMT: NullString = null_str!("%.*s");
