// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

use cfs_sys::*;
use printf_wrap::NullString;
use super::Status;

pub use cfs_sys::CFE_SB_MsgId_Atom_t as MsgId_Atom;

#[derive(Clone,Copy,Debug)]
pub struct MsgId { pub id: CFE_SB_MsgId_t }

impl PartialEq<MsgId> for MsgId {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        unsafe { SHIM_CFE_SB_MsgId_Equal(self.id, other.id) }
    }
}

impl Eq for MsgId { }

impl From<MsgId_Atom> for MsgId {
    #[inline]
    fn from(val: MsgId_Atom) -> Self {
        let msg_id = unsafe { SHIM_CFE_SB_ValueToMsgId(val) };
        MsgId { id: msg_id }
    }
}

impl From<MsgId> for MsgId_Atom {
    #[inline]
    fn from(id: MsgId) -> Self {
        unsafe { SHIM_CFE_SB_MsgIdToValue(id.id) }
    }
}

#[derive(Debug)]
pub struct Pipe { pub id: CFE_SB_PipeId_t }

impl Pipe {
    #[inline]
    pub fn new(depth: u16, pipe_name: NullString) -> Result<Pipe, Status> {
        let mut p: CFE_SB_PipeId_t = super::ResourceId::UNDEFINED.id;

        let s: Status = unsafe {
            CFE_SB_CreatePipe(&mut p, depth, pipe_name.as_ptr())
        }.into();

        if p == super::ResourceId::UNDEFINED.id {
            return Err(Status::SB_PIPE_CR_ERR);
        }

        s.as_result(|| { Pipe { id: p } })
    }

    #[inline]
    pub fn delete(self) {
        unsafe { CFE_SB_DeletePipe(self.id); }
    }

    #[inline]
    pub fn subscribe(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = unsafe {
            CFE_SB_Subscribe(msg_id.id, self.id)
        }.into();

        s.as_result(|| { () })
    }

    #[inline]
    pub fn subscribe_local(&mut self, msg_id: MsgId, msg_lim: u16) -> Result<(), Status> {
        let s: Status = unsafe {
            CFE_SB_SubscribeLocal(msg_id.id, self.id, msg_lim)
        }.into();

        s.as_result(|| { () })
    }
}
