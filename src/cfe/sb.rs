// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

use cfs_sys::*;
use printf_wrap::NullString;
use super::Status;
use super::msg::Message;

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

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)]
pub enum QosPriority {
    High = CFE_SB_QosPriority_CFE_SB_QosPriority_HIGH as u8,
    Low = CFE_SB_QosPriority_CFE_SB_QosPriority_LOW as u8,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)]
pub enum QosReliability {
    High = CFE_SB_QosReliability_CFE_SB_QosReliability_HIGH as u8,
    Low = CFE_SB_QosReliability_CFE_SB_QosReliability_LOW as u8,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(C)]
pub struct Qos {
    priority: u8,
    reliability: u8,
}

impl Qos {
    #[inline]
    pub const fn new(priority: QosPriority, reliability: QosReliability) -> Qos {
        Qos {
            priority: priority as u8,
            reliability: reliability as u8,
        }
    }

    pub const DEFAULT: Qos = Qos {
        priority: X_CFE_SB_DEFAULT_QOS_PRIORITY,
        reliability: X_CFE_SB_DEFAULT_QOS_RELIABILITY,
    };
}

impl From<Qos> for CFE_SB_Qos_t {
    #[inline]
    fn from(x: Qos) -> CFE_SB_Qos_t {
        CFE_SB_Qos_t {
            Priority: x.priority,
            Reliability: x.reliability,
        }
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum TimeOut {
    Millis(u32),
    Poll,
    PendForever,
}

impl From<TimeOut> for i32 {
    fn from(tmo: TimeOut) -> i32 {
        use TimeOut::*;

        match tmo {
            Millis(n) => (n & !0x8000_0000) as i32,
            Poll => CFE_SB_POLL as i32,
            PendForever => CFE_SB_PEND_FOREVER as i32,
        }
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
    pub fn subscribe_ex(&mut self, msg_id: MsgId, quality: Qos, msg_lim: u16) -> Result<(), Status> {
        let qos: CFE_SB_Qos_t = quality.into();

        let s: Status = unsafe {
            CFE_SB_SubscribeEx(msg_id.id, self.id, qos, msg_lim)
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

    #[inline]
    pub fn unsubscribe(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = unsafe {
            CFE_SB_Unsubscribe(msg_id.id, self.id)
        }.into();

        s.as_result(|| { () })
    }

    #[inline]
    pub fn unsubscribe_local(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = unsafe {
            CFE_SB_UnsubscribeLocal(msg_id.id, self.id)
        }.into();

        s.as_result(|| { () })
    }

    #[inline]
    pub fn receive_buffer<T, F>(&mut self, time_out: TimeOut, closure: F) -> Result<T, Status>
        where F: for<'a> FnOnce(&'a CFE_SB_Buffer_t) -> Result<T, Status> {

        let mut buf: *mut CFE_SB_Buffer_t = core::ptr::null_mut();

        let s: Status = unsafe {
            CFE_SB_ReceiveBuffer(&mut buf, self.id, time_out.into())
        }.into();

        if s.severity() != super::StatusSeverity::Success {
            return Err(s);
        }

        match unsafe { buf.as_ref() } {
            None => Err(Status::SB_BUFFER_INVALID),
            Some(b) => closure(b),
        }
    }
}

pub struct Buffer<'a> {
    b: &'a CFE_SB_Buffer_t
}

impl<'a> Buffer<'a> {
    pub fn as_message(&self) -> &'a Message {
        let p: &CFE_MSG_Message_t = unsafe { &self.b.Msg };
        Message::from_cfe(p)
    }
}
