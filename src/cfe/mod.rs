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

