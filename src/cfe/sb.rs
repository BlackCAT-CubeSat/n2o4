// Copyright (c) 2021-2022 The Pennsylvania State University. All rights reserved.

//! Software Bus system

use core::marker::PhantomData;
use core::ops::Deref;

use cfs_sys::*;
use printf_wrap::NullString;
use super::Status;
use super::msg::{Message,MsgType,Command,Telemetry};

/// The numeric value of a [message ID](`MsgId`).
#[doc(inline)]
pub use cfs_sys::CFE_SB_MsgId_Atom_t as MsgId_Atom;

/// An encoded message ID.
///
/// SB uses this as a mapped version of the
/// [numeric message ID](`MsgId_Atom`).
#[derive(Clone,Copy,Debug)]
pub struct MsgId { pub(crate) id: CFE_SB_MsgId_t }

impl MsgId {
    /// Returns whether `self` is a valid message ID.
    ///
    /// Wraps CFE_SB_IsValidMsgId.
    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { CFE_SB_IsValidMsgId(self.id) }
    }

    /// Returns the message type this message ID corresponds to.
    ///
    /// Wraps CFE_MSG_GetTypeFromMsgId.
    #[inline]
    pub fn msg_type(self) -> Result<MsgType, Status> {
        let mut t: CFE_MSG_Type_t = CFE_MSG_Type_CFE_MSG_Type_Invalid;
        let s: Status = unsafe {
            CFE_MSG_GetTypeFromMsgId(self.id, &mut t)
        }.into();

        s.as_result(|| { MsgType::from_cfe(t) })
    }

    /// A reserved value that will not match any valid message ID.
    pub const RESERVED: MsgId = MsgId { id: X_CFE_SB_MSGID_RESERVED };

    /// Value representing an invalid message ID.
    pub const INVALID: MsgId = MsgId { id: X_CFE_SB_INVALID_MSG_ID };
}

/// Wraps CFE_SB_MsgId_Equal.
impl PartialEq<MsgId> for MsgId {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        unsafe { SHIM_CFE_SB_MsgId_Equal(self.id, other.id) }
    }
}

impl Eq for MsgId { }

/// Wraps CFE_SB_ValueToMsgId.
impl From<MsgId_Atom> for MsgId {
    #[inline]
    fn from(val: MsgId_Atom) -> Self {
        let msg_id = unsafe { SHIM_CFE_SB_ValueToMsgId(val) };
        MsgId { id: msg_id }
    }
}

/// Wraps CFE_SB_MsgIdToValue.
impl From<MsgId> for MsgId_Atom {
    #[inline]
    fn from(id: MsgId) -> Self {
        unsafe { SHIM_CFE_SB_MsgIdToValue(id.id) }
    }
}

/// Message priority for off-system routing. Currently unused by cFE.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)]
pub enum QosPriority {
    High = CFE_SB_QosPriority_CFE_SB_QosPriority_HIGH as u8,
    Low = CFE_SB_QosPriority_CFE_SB_QosPriority_LOW as u8,
}

/// Message transfer reliability for off-instance routing. Currently unused by cFE.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)]
pub enum QosReliability {
    High = CFE_SB_QosReliability_CFE_SB_QosReliability_HIGH as u8,
    Low = CFE_SB_QosReliability_CFE_SB_QosReliability_LOW as u8,
}

/// Quality-of-service information for message subscriptions on the software bus.
/// Currently unused by cFE.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(C)]
pub struct Qos {
    priority: u8,
    reliability: u8,
}

impl Qos {
    /// Constructs a new QoS with the specified priority and reliability.
    #[inline]
    pub const fn new(priority: QosPriority, reliability: QosReliability) -> Qos {
        Qos {
            priority: priority as u8,
            reliability: reliability as u8,
        }
    }

    /// The default QoS. Most applications should use this.
    pub const DEFAULT: Qos = Qos {
        priority: X_CFE_SB_DEFAULT_QOS_PRIORITY,
        reliability: X_CFE_SB_DEFAULT_QOS_RELIABILITY,
    };

    #[inline]
    fn into_cfe(self) -> CFE_SB_Qos_t {
        CFE_SB_Qos_t {
            Priority: self.priority,
            Reliability: self.reliability,
        }
    }
}

/// How long to wait for a new message if a pipe is empty.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum TimeOut {
    /// Wait for the specified number of milliseconds.
    Millis(u32),
    /// Non-blocking receive.
    Poll,
    /// Wait forever for a message to arrive.
    PendForever,
}

/// Converts a [`TimeOut`] into the `TimeOut` argument to CFE_SB_ReceiveBuffer.
impl From<TimeOut> for i32 {
    #[inline]
    fn from(tmo: TimeOut) -> i32 {
        use TimeOut::*;

        match tmo {
            Millis(n) => (n & !0x8000_0000) as i32,
            Poll => CFE_SB_POLL as i32,
            PendForever => CFE_SB_PEND_FOREVER as i32,
        }
    }
}

/// A software bus pipe; an application needs one of these to receive messages.
///
/// This may not be used from a different thread from the one it was created on.
#[derive(Debug)]
pub struct Pipe {
    /// cFE ID for the pipe.
    pub(crate) id: CFE_SB_PipeId_t,

    /// Marker field used to make this type [`!Send`](`Send`) and [`!Sync`](`Sync`).
    ///
    /// A cFE message pipe may not be used on any thread other than the one
    /// on which it was created, so we need to stop auto-derivation of
    /// {`Send`, `Sync`}.
    _pd: PhantomData<*const u8>,
}

impl Pipe {
    /// Creates a new pipe with space for `depth` yet-to-be-handled messages
    /// and the name `pipe_name`.
    ///
    /// Wraps CFE_SB_CreatePipe.
    #[inline]
    pub fn new(depth: u16, pipe_name: NullString) -> Result<Pipe, Status> {
        let mut p: CFE_SB_PipeId_t = super::ResourceId::UNDEFINED.id;

        let s: Status = unsafe {
            CFE_SB_CreatePipe(&mut p, depth, pipe_name.as_ptr())
        }.into();

        if p == super::ResourceId::UNDEFINED.id {
            return Err(Status::SB_PIPE_CR_ERR);
        }

        s.as_result(|| { Pipe { id: p, _pd: PhantomData } })
    }

    /// Deletes the software bus pipe.
    ///
    /// Note that applications should not call this as they are shutting down;
    /// the framework will do the needed cleanup at application exit.
    ///
    /// Wraps CFE_SB_DeletePipe.
    #[inline]
    pub fn delete(self) {
        unsafe { CFE_SB_DeletePipe(self.id); }
    }

    /// Subscribes to messages with ID `msg_id`
    /// on the software bus with default parameters.
    ///
    /// Wraps CFE_SB_Subscribe.
    #[inline]
    pub fn subscribe(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = unsafe {
            CFE_SB_Subscribe(msg_id.id, self.id)
        }.into();

        s.as_result(|| { () })
    }

    /// Subscribes to messages with ID `msg_id` on the software bus
    /// with the specified quality of service (currently unused by cFE)
    /// and a limit to the number of messages with this ID
    /// allowed in the pipe at the same time.
    ///
    /// Wraps CFE_SB_SubscribeEx.
    #[inline]
    pub fn subscribe_ex(&mut self, msg_id: MsgId, quality: Qos, msg_lim: u16) -> Result<(), Status> {
        let qos: CFE_SB_Qos_t = quality.into_cfe();

        let s: Status = unsafe {
            CFE_SB_SubscribeEx(msg_id.id, self.id, qos, msg_lim)
        }.into();

        s.as_result(|| { () })
    }

    /// Subscribes to messages with ID `msg_id`,
    /// but keep the subscription local to the current CPU.
    ///
    /// This is typically only used by the [SBN](https://github.com/nasa/SBN) application.
    ///
    /// Wraps CFE_SB_SubscribeLocal.
    #[inline]
    pub fn subscribe_local(&mut self, msg_id: MsgId, msg_lim: u16) -> Result<(), Status> {
        let s: Status = unsafe {
            CFE_SB_SubscribeLocal(msg_id.id, self.id, msg_lim)
        }.into();

        s.as_result(|| { () })
    }

    /// Removes the current pipe's subscription to messages with ID `msg_id`.
    ///
    /// Wraps CFE_SB_Unsubscribe.
    #[inline]
    pub fn unsubscribe(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = unsafe {
            CFE_SB_Unsubscribe(msg_id.id, self.id)
        }.into();

        s.as_result(|| { () })
    }

    /// Removes the current pipe's subscription to messages with ID `msg_id`,
    /// keeping the unsubscription local to the current CPU.
    ///
    /// This is typically only used by the [SBN](https://github.com/nasa/SBN) application.
    ///
    /// Wraps CFE_SB_UnsubscribeLocal.
    #[inline]
    pub fn unsubscribe_local(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = unsafe {
            CFE_SB_UnsubscribeLocal(msg_id.id, self.id)
        }.into();

        s.as_result(|| { () })
    }

    /// Receives a message from the pipe.
    ///
    /// Whether or not a message was received, `closure` gets called with
    /// the result of the reception attempt.
    ///
    /// Uses `time_out` to determine how long to wait for a message if the pipe is empty.
    ///
    /// Passing the buffer to a closure rather than returning it ensures that
    /// the buffer's lifetime constraints are respected.
    ///
    /// Wraps CFE_SB_ReceiveBuffer.
    #[inline]
    pub fn receive_buffer<T, F>(&mut self, time_out: TimeOut, closure: F) -> T
        where F: for<'a> FnOnce(Result<MessageBuffer<'a>, Status>) -> T {

        let mut buf: *mut CFE_SB_Buffer_t = core::ptr::null_mut();

        let s: Status = unsafe {
            CFE_SB_ReceiveBuffer(&mut buf, self.id, time_out.into())
        }.into();

        let result: Result<MessageBuffer, Status>;
        result = if s.severity() == super::StatusSeverity::Error {
            Err(s)
        } else {
            match unsafe { buf.as_ref() } {
                None => Err(Status::SB_BUFFER_INVALID),
                Some(b) => Ok(MessageBuffer { b: b }),
            }
        };

        closure(result)
    }
}

/// A message received from a software pipe.
pub struct MessageBuffer<'a> {
    b: &'a CFE_SB_Buffer_t
}

impl<'a> MessageBuffer<'a> {
    /// The backend of [`try_cast_cmd`] and [`try_cast_tlm`].
    #[inline]
    fn try_cast<T: Sized>(&self, msg_type: MsgType) -> Result<&'a T, Status> {
        if self.msgid()?.msg_type()? != msg_type {
            return Err(Status::MSG_WRONG_MSG_TYPE);
        }

        if self.size()? as usize != core::mem::size_of::<T>() {
            return Err(Status::STATUS_WRONG_MSG_LENGTH);
        }

        let p = self.b as *const CFE_SB_Buffer_t as usize;
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
    pub fn try_cast_cmd<T: Copy + Sized>(&self) -> Result<&'a Command<T>, Status> {
        self.try_cast::<Command<T>>(MsgType::Cmd)
    }

    /// If it makes sense to do so (the message is the right size,
    /// aligned correctly in memory, and has a compatible message ID),
    /// returns a reference to the message as a [`Telemetry<T>`].
    #[inline]
    pub fn try_cast_tlm<T: Copy + Sized>(&self) -> Result<&'a Telemetry<T>, Status> {
        self.try_cast::<Telemetry<T>>(MsgType::Tlm)
    }
}

impl<'a> Deref for MessageBuffer<'a> {
    type Target = Message;

    #[inline]
    fn deref(&self) -> &'a Message {
        let p: &CFE_MSG_Message_t = unsafe { &self.b.Msg };
        Message::from_cfe(p)
    }
}
