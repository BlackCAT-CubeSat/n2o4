// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

use cfs_sys::*;
use libc::c_ulong;

pub mod es;
pub mod evs;
pub mod fs;
pub mod msg;
pub mod sb;
pub mod tbl;
pub mod time;

mod status_consts;

#[derive(Clone,Copy,Debug)]
pub struct ResourceId {
    pub id: CFE_ResourceId_t
}

impl ResourceId {
    #[inline]
    pub fn is_defined(&self) -> bool {
        unsafe { SHIM_CFE_ResourceId_IsDefined(self.id) }
    }

    pub const UNDEFINED: Self = ResourceId { id: X_CFE_RESOURCEID_UNDEFINED };
    pub const RESERVED: Self = ResourceId { id: X_CFE_RESOURCEID_RESERVED };
}

impl PartialEq<ResourceId> for ResourceId {
    #[inline]
    fn eq(&self, other: &ResourceId) -> bool {
        unsafe { SHIM_CFE_ResourceId_Equal(self.id, other.id) }
    }
}

impl Eq for ResourceId { }

impl From<c_ulong> for ResourceId {
    #[inline]
    fn from(val: c_ulong) -> ResourceId {
        let rid = unsafe { SHIM_CFE_ResourceId_FromInteger(val) };
        ResourceId { id: rid }
    }
}

impl From<ResourceId> for c_ulong {
    #[inline]
    fn from(id: ResourceId) -> c_ulong {
        unsafe { SHIM_CFE_ResourceId_ToInteger(id.id) }
    }
}

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub struct Status {
    pub status: CFE_Status_t
}

#[repr(u32)]
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum StatusSeverity {
    Success = 0b00,
    Informational = 0b01,
    Error = 0b11,
}

#[repr(u32)]
#[derive(Clone,Copy,PartialEq,Eq,Debug)]
pub enum StatusServiceId {
    NotCfe  = 0b000,
    EVS     = 0b001,
    ES      = 0b010,
    FS      = 0b011,
    Generic = 0b100,
    SB      = 0b101,
    TBL     = 0b110,
    TIME    = 0b111,
}

impl Status {
    #[inline]
    pub fn severity(&self) -> StatusSeverity {
        use StatusSeverity::*;

        match (self.status >> 30) & 0b0011 {
            0b00 => Success,
            0b01 => Informational,
            0b10 | 0b11 => Error,
        }
    }

    #[inline]
    pub fn service_identifier(&self) -> StatusServiceId {
        use StatusServiceId::*;

        match (self.status >> 25) & 0b0111 {
            0b000 => NotCfe,
            0b001 => EVS,
            0b010 => ES,
            0b011 => FS,
            0b100 => Generic,
            0b101 => SB,
            0b110 => TBL,
            0b111 => TIME,
        }
    }

    #[inline]
    pub fn mission_defined(&self) -> u16 {
        ((self.status >> 16) & 0x01ff) as u16
    }

    #[inline]
    pub fn code(&self) -> u16 {
        self.status as u16
    }

    #[inline]
    pub fn as_result<T, F: FnOnce() -> T>(&self, on_success: F) -> Result<T, Status> {
        match self.severity() {
            StatusSeverity::Success => Ok(on_success()),
            _ => Err(*self),
        }
    }
}
