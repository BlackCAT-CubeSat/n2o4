// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

use core::default::Default;
use core::ops::{Deref,DerefMut};

use cfs_sys::*;
use super::Status;
use super::sb::MsgId;

#[repr(transparent)]
pub struct Message {
    pub(super) msg: CFE_MSG_Message_t
}

impl Message {
    #[inline]
    pub(super) fn from_cfe(m: &CFE_MSG_Message_t) -> &Message {
        let p = m as *const CFE_MSG_Message_t as *const Message;
        unsafe { &*p }
    }

    #[inline]
    pub(super) fn from_cfe_mut(m: &mut CFE_MSG_Message_t) -> &mut Message {
        let p = m as *mut CFE_MSG_Message_t as *mut Message;
        unsafe { &mut *p }
    }

    #[inline]
    pub fn msgid(&self) -> Result<MsgId, Status> {
        let mut mid: MsgId = MsgId::INVALID;

        let s: Status = unsafe {
            CFE_MSG_GetMsgId(&self.msg, &mut mid.id)
        }.into();

        s.as_result(|| { mid })
    }

    #[inline]
    pub fn set_msgid(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = unsafe {
            CFE_MSG_SetMsgId(&mut self.msg, msg_id.id)
        }.into();

        s.as_result(|| { () })
    }
}

#[repr(transparent)]
pub struct CommandHeader {
    pub(super) hdr: CFE_MSG_CommandHeader_t
}

#[repr(transparent)]
pub struct TelemetryHeader {
    pub(super) hdr: CFE_MSG_TelemetryHeader_t
}

impl Deref for CommandHeader {
    type Target = Message;

    #[inline]
    fn deref(&self) -> &Message {
        Message::from_cfe(&self.hdr.Msg)
    }
}

impl DerefMut for CommandHeader {
    #[inline]
    fn deref_mut(&mut self) -> &mut Message {
        Message::from_cfe_mut(&mut self.hdr.Msg)
    }
}

impl Deref for TelemetryHeader {
    type Target = Message;

    #[inline]
    fn deref(&self) -> &Message {
        Message::from_cfe(&self.hdr.Msg)
    }
}

impl DerefMut for TelemetryHeader {
    #[inline]
    fn deref_mut(&mut self) -> &mut Message {
        Message::from_cfe_mut(&mut self.hdr.Msg)
    }
}
