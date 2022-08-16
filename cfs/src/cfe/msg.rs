// Copyright (c) 2021-2022 The Pennsylvania State University. All rights reserved.

//! Message utilities

use core::default::Default;
use core::mem;
use core::ops::{Deref, DerefMut};

use super::sb::MsgId;
use super::Status;
use cfs_sys::*;

/// Returns the number of items in array field `$field` of `$type`.
///
/// The `unsafe` variant may be used for fields in unions.
///
/// A tip of the hat to [https://stackoverflow.com/a/70224634] for the basic idea.
macro_rules! array_field_len {
    ($type:ty, $field:ident) => {{
        const SZ: usize = length_of_return_array(&|s: $type| s.$field);
        SZ
    }};
    ($type:ty, $field:ident, unsafe) => {{
        const SZ: usize = length_of_return_array(&|s: $type| unsafe { s.$field });
        SZ
    }};
}

/// Returns the array length of the return type of a function,
/// when the function returns an array.
const fn length_of_return_array<F, T, U, const N: usize>(_f: &F) -> usize
where
    F: FnOnce(T) -> [U; N],
{
    N
}

/// Returns the byte offset from the beginning of structure `$x` to
/// the beginning its `$type`'s field.
macro_rules! offset_of {
    ($x:expr, $field:ident) => {{
        let struct_addr = core::ptr::addr_of!(*$x);
        let field_addr = core::ptr::addr_of!($x.$field);
        (field_addr as usize) - (struct_addr as usize)
    }};
}

/// A [`Message`]'s function code.
///
/// This is the same as `CFE_MSG_FcnCode_t`.
#[doc(inline)]
pub use cfs_sys::CFE_MSG_FcnCode_t as FunctionCode;

/// Represents the size of a [`Message`].
///
/// This is the same as `CFE_MSG_Size_t`.
#[doc(inline)]
pub use cfs_sys::CFE_MSG_Size_t as Size;

/// An instance of the common header for cFE software bus messages.
///
/// Wraps `CFE_MSG_Message_t`.
#[repr(transparent)]
pub struct Message {
    pub(super) msg: CFE_MSG_Message_t,
}

/// A command message for use with the cFE software bus.
///
/// Wraps `CFE_MSG_CommandHeader_t`, with a user-specified payload following.
#[repr(C)]
pub struct Command<T: Copy> {
    /// The command header.
    header: CFE_MSG_CommandHeader_t,

    /// The message's payload. As messages are copied
    /// willy-nilly by cFE, `T` needs to be [`Copy`].
    pub payload: T,
}

/// A telemetry message for use with the cFE software bus.
///
/// Wraps `CFE_MSG_TelemetryHeader_t`, with a user-specified payload following.
#[repr(C)]
pub struct Telemetry<T: Copy> {
    /// The telemetry header.
    header: CFE_MSG_TelemetryHeader_t,

    /// The message's payload. As messages are copied
    /// willy-nilly by cFE, `T` needs to be [`Copy`].
    pub payload: T,
}

impl Message {
    /// An instance of [`CFE_MSG_Message_t`] for use when constructing instances.
    const ZERO_MESSAGE: CFE_MSG_Message_t = CFE_MSG_Message_t {
        Byte: [0; array_field_len!(CFE_MSG_Message_t, Byte, unsafe)],
    };

    /// Initialize a [`Message`]. As doing this arbitrarily can result in
    /// invalid state (e.g., a message with a command message ID but a telemetry
    /// secondary header), this is an unsafe operation.
    ///
    /// Wraps `CFE_MSG_Init`.
    #[inline]
    unsafe fn init(&mut self, msg_id: MsgId, size: Size) -> Result<(), Status> {
        let s: Status = CFE_MSG_Init(&mut self.msg, msg_id.id, size).into();
        s.as_result(|| ())
    }

    /// Convenience function for creating a higher-level wrapper from the
    /// [`cfs_sys`]-provided type.
    #[inline]
    pub(super) fn from_cfe(m: &CFE_MSG_Message_t) -> &Message {
        let p = m as *const CFE_MSG_Message_t as *const Message;
        unsafe { &*p }
    }

    /// Convenience function for creating a higher-level wrapper from the
    /// [`cfs_sys`]-provided type.
    #[inline]
    pub(super) fn from_cfe_mut(m: &mut CFE_MSG_Message_t) -> &mut Message {
        let p = m as *mut CFE_MSG_Message_t as *mut Message;
        unsafe { &mut *p }
    }

    /// Returns the [`Message`]'s function code (if it has one).
    ///
    /// Wraps `CFE_MSG_GetFcnCode`.
    #[inline]
    pub fn fcn_code(&self) -> Result<FunctionCode, Status> {
        let mut fc: FunctionCode = 0;
        let s: Status = unsafe { CFE_MSG_GetFcnCode(&self.msg, &mut fc) }.into();

        s.as_result(|| fc)
    }

    /// Returns the message ID.
    ///
    /// Wraps `CFE_MSG_GetMsgId`.
    #[inline]
    pub fn msgid(&self) -> Result<MsgId, Status> {
        let mut mid: MsgId = MsgId::INVALID;

        let s: Status = unsafe { CFE_MSG_GetMsgId(&self.msg, &mut mid.id) }.into();

        s.as_result(|| mid)
    }

    /// Tries to set the message ID, provided doing so would not change
    /// the message's type (e.g., telemetry to command).
    ///
    /// Wraps `CFE_MSG_SetMsgId`.
    #[inline]
    pub fn set_msgid(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let old_msg_id = self.msgid()?;

        if msg_id.msg_type()? != old_msg_id.msg_type()? {
            return Err(Status::SB_BAD_ARGUMENT);
        }

        unsafe { self.set_msgid_unchecked(msg_id) }
    }

    /// [`set_msgid`](`Self::set_msgid`) without the message-type check.
    ///
    /// As this can change the semantics of secondary headers without
    /// appropriately modifying them, this is an unsafe operation.
    ///
    /// Wraps `CFE_MSG_SetMsgId`.
    #[inline]
    pub unsafe fn set_msgid_unchecked(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = CFE_MSG_SetMsgId(&mut self.msg, msg_id.id).into();

        s.as_result(|| ())
    }

    /// Returns the total size of the message this [`Message`] is the header for.
    ///
    /// Wraps `CFE_MSG_GetSize`.
    #[inline]
    pub fn size(&self) -> Result<Size, Status> {
        let mut sz: Size = 0;
        let s: Status = unsafe { CFE_MSG_GetSize(&self.msg, &mut sz) }.into();

        s.as_result(|| sz)
    }

    /// Sets the total size of the message this [`Message`] is the header for.
    ///
    /// As this can change what does and doesn't get copied when a message is
    /// transmitted, this is an unsafe operation.
    ///
    /// Wraps `CFE_MSG_SetSize`.
    #[inline]
    pub unsafe fn set_size(&mut self, sz: Size) -> Result<(), Status> {
        let s: Status = CFE_MSG_SetSize(&mut self.msg, sz).into();
        s.as_result(|| ())
    }

    /// The backend of [`try_cast_cmd`](`Self::try_cast_cmd`)
    /// and [`try_cast_tlm`](`Self::try_cast_tlm`).
    #[inline]
    fn try_cast<T: Sized>(&self, msg_type: MsgType) -> Result<&T, Status> {
        if self.msgid()?.msg_type()? != msg_type {
            return Err(Status::MSG_WRONG_MSG_TYPE);
        }

        if self.size()? as usize != core::mem::size_of::<T>() {
            return Err(Status::STATUS_WRONG_MSG_LENGTH);
        }

        let p = &(self.msg) as *const CFE_MSG_Message_t as usize;
        if p % core::mem::align_of::<T>() != 0 {
            return Err(Status::SB_BAD_ARGUMENT);
        }

        let pkt: &T = unsafe { &*(p as *const T) };
        Ok(pkt)
    }

    /// If it makes sense to do so (the message is the right size,
    /// aligned correctly in memory, and has a compatible message ID),
    /// returns a reference to the message as a [`Command<T>`].
    #[inline]
    pub fn try_cast_cmd<T: Copy + Sized>(&self) -> Result<&Command<T>, Status> {
        self.try_cast::<Command<T>>(MsgType::Cmd)
    }

    /// If it makes sense to do so (the message is the right size,
    /// aligned correctly in memory, and has a compatible message ID),
    /// returns a reference to the message as a [`Telemetry<T>`].
    #[inline]
    pub fn try_cast_tlm<T: Copy + Sized>(&self) -> Result<&Telemetry<T>, Status> {
        self.try_cast::<Telemetry<T>>(MsgType::Tlm)
    }

    /// Returns the payload of the message as a byte slice.
    ///
    /// This can be useful when the payload isn't a C structure.
    #[inline]
    pub fn payload(&self) -> Result<&[u8], Status> {
        let size = self.size()? as usize;
        let header_length = match self.msgid()?.msg_type()? {
            MsgType::Cmd => mem::size_of::<CFE_MSG_CommandHeader_t>(),
            MsgType::Tlm => mem::size_of::<CFE_MSG_TelemetryHeader_t>(),
            _ => {
                return Err(Status::MSG_WRONG_MSG_TYPE);
            }
        };

        let slice: Option<&[u8]> = unsafe {
            let base: *const u8 =
                (self as *const Message as *const u8).offset(header_length as isize);
            core::ptr::slice_from_raw_parts(base, size - header_length).as_ref()
        };

        match slice {
            Some(s) => Ok(s),
            None => Err(Status::SB_NO_MESSAGE),
        }
    }

    /// Sets the [`Message`]'s time field to the current spacecraft time.
    ///
    /// Wraps `CFE_SB_TimeStampMsg`.
    #[inline]
    pub fn time_stamp(&mut self) {
        unsafe { CFE_SB_TimeStampMsg(&mut self.msg) }
    }

    /// Transmits onto the software bus the message this [`Message`] is a header for.
    ///
    /// The software bus makes a copy of the message,
    /// so the current instance of the message may be freely modified after
    /// calling this method.
    ///
    /// Wraps `CFE_SB_TransmitMsg`.
    #[inline]
    pub fn transmit(&mut self, increment_sequence_count: bool) -> Result<(), Status> {
        let s: Status =
            unsafe { CFE_SB_TransmitMsg(&mut self.msg, increment_sequence_count) }.into();

        s.as_result(|| ())
    }
}

impl<T: Copy + Sized> Command<T> {
    /// An instance of the command header for use when constructing instances.
    const ZERO_HEADER: CFE_MSG_CommandHeader_t = CFE_MSG_CommandHeader_t {
        Msg: Message::ZERO_MESSAGE,
        Sec: CFE_MSG_CommandSecondaryHeader_t {
            FunctionCode: 0,
            Checksum:     0,
        },
    };

    /// Tries to create a new command message, setting the message ID and function code
    /// along the way.
    ///
    /// Wraps `CFE_MSG_Init`, `CFE_MSG_GetTypeFromMsgId`, and `CFE_MSG_SetFcnCode`.
    #[inline]
    pub fn new(msg_id: MsgId, fcn_code: FunctionCode, payload: T) -> Result<Self, Status> {
        let mut cmd = Command {
            header:  Self::ZERO_HEADER,
            payload: payload,
        };
        let sz: Size = mem::size_of::<Self>() as Size;

        if msg_id.msg_type() != Ok(MsgType::Cmd) {
            return Err(Status::SB_BAD_ARGUMENT);
        }

        unsafe { Message::from_cfe_mut(&mut cmd.header.Msg).init(msg_id, sz) }?;

        cmd.set_fcn_code(fcn_code)?;

        Ok(cmd)
    }
}

impl<T: Copy + Sized + Default> Command<T> {
    /// [`new`](`Self::new`) using `T::default()` as the payload.
    #[inline]
    pub fn new_default(msg_id: MsgId, fcn_code: FunctionCode) -> Result<Self, Status> {
        Self::new(msg_id, fcn_code, T::default())
    }
}

impl<T: Copy + Sized> Command<T> {
    /// Sets the message's function code.
    ///
    /// Wraps `CFE_MSG_SetFcnCode`.
    #[inline]
    pub fn set_fcn_code(&mut self, fcn_code: FunctionCode) -> Result<(), Status> {
        let s: Status = unsafe { CFE_MSG_SetFcnCode(&mut self.header.Msg, fcn_code) }.into();

        s.as_result(|| ())
    }
}

impl<T: Copy + Sized, const SIZE: usize> Command<[T; SIZE]> {
    /// Transmits onto the software bus
    /// the header and first min(`len`,&nbsp;`SIZE`) elements
    /// of [`payload`](`Command::payload`).
    ///
    /// After transmission, attempts to set the message size (in the header)
    /// back to the structure's full length.
    ///
    /// Wraps `CFE_MSG_SetSize` and `CFE_SB_TransmitMsg`.
    #[inline]
    pub fn transmit_partial(
        &mut self,
        increment_sequence_count: bool,
        len: usize,
    ) -> Result<(), Status> {
        let len = len.min(SIZE);
        let sz = (offset_of!(self, payload) + (len * mem::size_of::<T>())) as Size;

        unsafe { self.set_size(sz) }?;
        let ret_val = self.transmit(increment_sequence_count);
        let _ = unsafe { self.set_size(mem::size_of::<Self>() as Size) };

        ret_val
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
    /// An instance of the telemetry header for use when constructing instances.
    const ZERO_HEADER: CFE_MSG_TelemetryHeader_t = CFE_MSG_TelemetryHeader_t {
        Msg:   Message::ZERO_MESSAGE,
        Sec:   CFE_MSG_TelemetrySecondaryHeader_t {
            Time: [0; array_field_len!(CFE_MSG_TelemetrySecondaryHeader_t, Time)],
        },
        Spare: [0; array_field_len!(CFE_MSG_TelemetryHeader_t, Spare)],
    };

    /// Tries to create a new telemetry message, setting the message ID
    /// along the way.
    ///
    /// Wraps `CFE_MSG_Init`, `CFE_MSG_GetTypeFromMsgId`, and `CFE_MSG_SetFcnCode`.
    #[inline]
    pub fn new(msg_id: MsgId, payload: T) -> Result<Self, Status> {
        let mut tlm = Telemetry {
            header:  Self::ZERO_HEADER,
            payload: payload,
        };
        let sz: Size = mem::size_of::<Self>() as Size;

        if msg_id.msg_type() != Ok(MsgType::Tlm) {
            return Err(Status::SB_BAD_ARGUMENT);
        }

        unsafe { Message::from_cfe_mut(&mut tlm.header.Msg).init(msg_id, sz) }?;

        Ok(tlm)
    }
}

impl<T: Copy + Sized + Default> Telemetry<T> {
    /// [`new`](`Self::new`) using `T::default()` as the payload.
    #[inline]
    pub fn new_default(msg_id: MsgId) -> Result<Self, Status> {
        Self::new(msg_id, T::default())
    }
}

impl<T: Copy + Sized, const SIZE: usize> Telemetry<[T; SIZE]> {
    /// Transmits onto the software bus
    /// the header and first min(`len`,&nbsp;`SIZE`) elements
    /// of [`payload`](`Telemetry::payload`).
    ///
    /// After transmission, attempts to set the message size (in the header)
    /// back to the structure's full length.
    ///
    /// Wraps `CFE_MSG_SetSize` and `CFE_SB_TransmitMsg`.
    #[inline]
    pub fn transmit_partial(
        &mut self,
        increment_sequence_count: bool,
        len: usize,
    ) -> Result<(), Status> {
        let len = len.min(SIZE);
        let sz = (offset_of!(self, payload) + (len * mem::size_of::<T>())) as Size;

        unsafe { self.set_size(sz) }?;
        let ret_val = self.transmit(increment_sequence_count);
        let _ = unsafe { self.set_size(mem::size_of::<Self>() as Size) };

        ret_val
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

/// The type of a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MsgType {
    /// Command message.
    Cmd     = CFE_MSG_Type_CFE_MSG_Type_Cmd,

    /// Telemetry message.
    Tlm     = CFE_MSG_Type_CFE_MSG_Type_Tlm,

    /// Invalid message type.
    Invalid = CFE_MSG_Type_CFE_MSG_Type_Invalid,
}

impl MsgType {
    /// Construct a [`MsgType`] from the corresponding cFE type.
    #[inline]
    #[allow(non_upper_case_globals)]
    pub(crate) fn from_cfe(ty: CFE_MSG_Type_t) -> Self {
        match ty {
            CFE_MSG_Type_CFE_MSG_Type_Cmd => Self::Cmd,
            CFE_MSG_Type_CFE_MSG_Type_Tlm => Self::Tlm,
            _ => Self::Invalid,
        }
    }
}
