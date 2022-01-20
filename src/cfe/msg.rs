// Copyright (c) 2021-2022 The Pennsylvania State University. All rights reserved.

//! Message utilities

use core::default::Default;
use core::mem;
use core::ops::{Deref,DerefMut};

use cfs_sys::*;
use super::Status;
use super::sb::MsgId;

pub type FunctionCode = CFE_MSG_FcnCode_t;
pub type Size = CFE_MSG_Size_t;

#[repr(transparent)]
pub struct Message {
    pub(super) msg: CFE_MSG_Message_t
}

#[repr(C)]
pub struct Command<T: Copy> {
    header: CFE_MSG_CommandHeader_t,
    pub payload: T,
}

#[repr(C)]
pub struct Telemetry<T: Copy> {
    header: CFE_MSG_TelemetryHeader_t,
    pub payload: T,
}

impl Message {
    const ZERO_MESSAGE: CFE_MSG_Message_t = CFE_MSG_Message_t { Byte: [0; 6] };

    #[inline]
    unsafe fn init(&mut self, msg_id: MsgId, size: Size) -> Result<(), Status> {
        let s: Status = CFE_MSG_Init(&mut self.msg, msg_id.id, size).into();
        s.as_result(|| { () })
    }

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
    pub fn fcn_code(&self) -> Result<FunctionCode, Status> {
        let mut fc: FunctionCode = 0;
        let s: Status = unsafe {
            CFE_MSG_GetFcnCode(&self.msg, &mut fc)
        }.into();

        s.as_result(|| { fc })
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
        let old_msg_id = self.msgid()?;

        if msg_id.msg_type()? != old_msg_id.msg_type()? {
            return Err(Status::SB_BAD_ARGUMENT);
        }

        unsafe { self.set_msgid_unchecked(msg_id) }
    }

    #[inline]
    pub unsafe fn set_msgid_unchecked(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = CFE_MSG_SetMsgId(&mut self.msg, msg_id.id).into();

        s.as_result(|| { () })
    }

    #[inline]
    pub fn size(&self) -> Result<Size, Status> {
        let mut sz: Size = 0;
        let s: Status = unsafe {
            CFE_MSG_GetSize(&self.msg, &mut sz)
        }.into();

        s.as_result(|| { sz })
    }

    #[inline]
    pub unsafe fn set_size(&mut self, sz: Size) -> Result<(), Status> {
        let s: Status = CFE_MSG_SetSize(&mut self.msg, sz).into();
        s.as_result(|| { () })
    }

    #[inline]
    pub fn time_stamp(&mut self) {
        unsafe { CFE_SB_TimeStampMsg(&mut self.msg) }
    }

    #[inline]
    pub fn transmit(&mut self, increment_sequence_count: bool) -> Result<(), Status> {
        let s: Status = unsafe {
            CFE_SB_TransmitMsg(&mut self.msg, increment_sequence_count)
        }.into();

        s.as_result(|| { () })
    }
}

impl<T: Copy + Sized> Command<T> {
    const ZERO_SECONDARY: CFE_MSG_CommandSecondaryHeader_t = CFE_MSG_CommandSecondaryHeader_t {
        FunctionCode: 0,
        Checksum: 0,
    };

    #[inline]
    pub fn new(msg_id: MsgId, fcn_code: FunctionCode, payload: T) -> Result<Self, Status> {
        let mut cmd = Command {
            header: CFE_MSG_CommandHeader_t {
                Msg: Message::ZERO_MESSAGE,
                Sec: Self::ZERO_SECONDARY,
            },
            payload: payload,
        };
        let sz: Size = mem::size_of::<Self>() as Size;

        if msg_id.msg_type() != Ok(MsgType::Cmd) { return Err(Status::SB_BAD_ARGUMENT); }

        unsafe { Message::from_cfe_mut(&mut cmd.header.Msg).init(msg_id, sz) }?;

        cmd.set_fcn_code(fcn_code)?;

        Ok(cmd)
    }
}

impl<T: Copy + Sized + Default> Command<T> {
    #[inline]
    pub fn new_default(msg_id: MsgId, fcn_code: FunctionCode) -> Result<Self, Status> {
        Self::new(msg_id, fcn_code, T::default())
    }
}

impl<T: Copy + Sized> Command<T> {
    #[inline]
    pub fn set_fcn_code(&mut self, fcn_code: FunctionCode) -> Result<(), Status> {
        let s: Status = unsafe {
            CFE_MSG_SetFcnCode(&mut self.header.Msg, fcn_code)
        }.into();

        s.as_result(|| { () })
    }
}

impl<T: Copy> Deref for Command<T> {
    type Target = Message;

    #[inline]
    fn deref(&self) -> &Message {
        Message::from_cfe(&self.header.Msg)
    }
}

impl<T: Copy> DerefMut for Command<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Message {
        Message::from_cfe_mut(&mut self.header.Msg)
    }
}

impl<T: Copy + Sized> Telemetry<T> {
    const ZERO_SECONDARY: CFE_MSG_TelemetrySecondaryHeader_t = CFE_MSG_TelemetrySecondaryHeader_t {
        Time: [0; 6],
    };

    #[inline]
    pub fn new(msg_id: MsgId, payload: T) -> Result<Self, Status> {
        let mut tlm = Telemetry {
            header: CFE_MSG_TelemetryHeader_t {
                Msg: Message::ZERO_MESSAGE,
                Sec: Self::ZERO_SECONDARY,
                Spare: [0; 4], //CFE_MSG_TelemetryHeader_t::Spare::size_of()],
            },
            payload: payload,
        };
        let sz: Size = mem::size_of::<Self>() as Size;

        if msg_id.msg_type() != Ok(MsgType::Tlm) { return Err(Status::SB_BAD_ARGUMENT); }

        unsafe { Message::from_cfe_mut(&mut tlm.header.Msg).init(msg_id, sz) }?;

        Ok(tlm)
    }
}

impl<T: Copy + Sized + Default> Telemetry<T> {
    #[inline]
    pub fn new_default(msg_id: MsgId) -> Result<Self, Status> {
        Self::new(msg_id, T::default())
    }
}

impl<T: Copy> Deref for Telemetry<T> {
    type Target = Message;

    #[inline]
    fn deref(&self) -> &Message {
        Message::from_cfe(&self.header.Msg)
    }
}

impl<T: Copy> DerefMut for Telemetry<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Message {
        Message::from_cfe_mut(&mut self.header.Msg)
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u32)]
pub enum MsgType {
    Cmd = CFE_MSG_Type_CFE_MSG_Type_Cmd,
    Tlm = CFE_MSG_Type_CFE_MSG_Type_Tlm,
    Invalid = CFE_MSG_Type_CFE_MSG_Type_Invalid,
}

impl From<CFE_MSG_Type_t> for MsgType {
    #[inline]
    #[allow(non_upper_case_globals)]
    fn from(ty: CFE_MSG_Type_t) -> MsgType {
        match ty {
            CFE_MSG_Type_CFE_MSG_Type_Cmd => Self::Cmd,
            CFE_MSG_Type_CFE_MSG_Type_Tlm => Self::Tlm,
            _ => Self::Invalid,
        }
    }
}

impl From<MsgType> for CFE_MSG_Type_t {
    #[inline]
    fn from(ty: MsgType) -> CFE_MSG_Type_t {
        ty as CFE_MSG_Type_t
    }
}
