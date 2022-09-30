// Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Software Bus system.

use core::ffi::CStr;
use core::marker::PhantomData;

use super::msg::{Message, MsgType};
use super::Status;
use cfs_sys::*;

/// The numeric value of a [message ID](`MsgId`).
///
/// This is the same as `CFE_SB_MsgId_Atom_t`.
#[doc(alias = "CFG_SB_MsgId_Atom_t")]
#[doc(inline)]
pub use cfs_sys::CFE_SB_MsgId_Atom_t as MsgId_Atom;

/// An encoded message ID.
///
/// SB uses this as a mapped version of the
/// [numeric message ID](`MsgId_Atom`).
///
/// Wraps `CFE_SB_MsgId_t`.
#[doc(alias = "CFG_SB_MsgId_t")]
#[derive(Clone, Copy, Debug)]
pub struct MsgId {
    pub(crate) id: CFE_SB_MsgId_t,
}

impl MsgId {
    /// Returns whether `self` is a valid message ID.
    ///
    /// Wraps `CFE_SB_IsValidMsgId`.
    #[doc(alias = "CFG_SB_IsValidMsgId")]
    #[inline]
    pub fn is_valid(self) -> bool {
        unsafe { CFE_SB_IsValidMsgId(self.id) }
    }

    /// Returns the message type this message ID corresponds to.
    ///
    /// Wraps `CFE_MSG_GetTypeFromMsgId`.
    #[doc(alias = "CFG_MSG_GetTypeFromMsgId")]
    #[inline]
    pub fn msg_type(self) -> Result<MsgType, Status> {
        let mut t: CFE_MSG_Type_t = CFE_MSG_Type_CFE_MSG_Type_Invalid;
        let s: Status = unsafe { CFE_MSG_GetTypeFromMsgId(self.id, &mut t) }.into();

        s.as_result(|| MsgType::from_cfe(t))
    }

    /// A reserved value that will not match any valid message ID.
    ///
    /// Wraps `CFE_SB_MSGID_RESERVED`.
    #[doc(alias = "CFG_SB_MSGID_RESERVED")]
    pub const RESERVED: MsgId = MsgId { id: X_CFE_SB_MSGID_RESERVED };

    /// Value representing an invalid message ID.
    ///
    /// Wraps `CFE_SB_INVALID_MSG_ID`.
    #[doc(alias = "CFG_SB_INVALID_MSG_ID")]
    pub const INVALID: MsgId = MsgId { id: X_CFE_SB_INVALID_MSG_ID };
}

/// Wraps `CFE_SB_MsgId_Equal`.
impl PartialEq<MsgId> for MsgId {
    #[doc(alias = "CFG_SB_MsgId_Equal")]
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        unsafe { SHIM_CFE_SB_MsgId_Equal(self.id, other.id) }
    }
}

impl Eq for MsgId {}

/// Wraps `CFE_SB_ValueToMsgId`.
impl From<MsgId_Atom> for MsgId {
    #[doc(alias = "CFG_SB_ValueToMsgId")]
    #[inline]
    fn from(val: MsgId_Atom) -> Self {
        let msg_id = unsafe { SHIM_CFE_SB_ValueToMsgId(val) };
        MsgId { id: msg_id }
    }
}

/// Wraps `CFE_SB_MsgIdToValue`.
impl From<MsgId> for MsgId_Atom {
    #[doc(alias = "CFG_SB_MsgIdToValue")]
    #[inline]
    fn from(id: MsgId) -> Self {
        unsafe { SHIM_CFE_SB_MsgIdToValue(id.id) }
    }
}

/// Message priority for off-system routing. Currently unused by cFE.
#[doc(alias = "CFG_SB_QosPriority")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum QosPriority {
    /// High priority.
    #[doc(alias = "CFG_SB_QosPriority_HIGH")]
    High = CFE_SB_QosPriority_CFE_SB_QosPriority_HIGH as u8,

    /// Normal priority level.
    #[doc(alias = "CFG_SB_QosPriority_LOW")]
    Low  = CFE_SB_QosPriority_CFE_SB_QosPriority_LOW as u8,
}

/// Message transfer reliability for off-instance routing. Currently unused by cFE.
#[doc(alias = "CFG_SB_QosReliability")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum QosReliability {
    /// High reliability.
    #[doc(alias = "CFG_SB_QosReliability_HIGH")]
    High = CFE_SB_QosReliability_CFE_SB_QosReliability_HIGH as u8,

    /// Normal (best-effort) reliability.
    #[doc(alias = "CFG_SB_QosReliability_LOW")]
    Low  = CFE_SB_QosReliability_CFE_SB_QosReliability_LOW as u8,
}

/// Quality-of-service information for message subscriptions on the software bus.
/// Currently unused by cFE.
///
/// Wraps `CFE_SB_Qos_t`.
#[doc(alias = "CFG_SB_Qos_t")]
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Qos {
    qos: CFE_SB_Qos_t,
}

impl Qos {
    /// Constructs a new QoS with the specified priority and reliability.
    #[inline]
    pub const fn new(priority: QosPriority, reliability: QosReliability) -> Qos {
        Qos {
            qos: CFE_SB_Qos_t {
                Priority:    priority as u8,
                Reliability: reliability as u8,
            },
        }
    }

    /// The default QoS. Most applications should use this.
    ///
    /// Wraps `CFE_SB_DEFAULT_QOS`.
    #[doc(alias = "CFG_SB_DEFAULT_QOS")]
    pub const DEFAULT: Qos = Qos {
        qos: CFE_SB_Qos_t {
            Priority:    X_CFE_SB_DEFAULT_QOS_PRIORITY,
            Reliability: X_CFE_SB_DEFAULT_QOS_RELIABILITY,
        },
    };
}

/// How long to wait for a new message if a pipe is empty.
#[derive(Clone, Copy, Debug)]
pub enum TimeOut {
    /// Wait for the specified number of milliseconds.
    Millis(u32),

    /// Non-blocking receive.
    #[doc(alias = "CFE_SB_POLL")]
    Poll,

    /// Wait forever for a message to arrive.
    #[doc(alias = "CFE_SB_PEND_FOREVER")]
    PendForever,
}

/// Converts a [`TimeOut`] into the `TimeOut` argument to `CFE_SB_ReceiveBuffer`.
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
///
/// Wraps `CFE_SB_PipeId_t`.
#[doc(alias = "CFG_SB_PipeId_t")]
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
    /// Wraps `CFE_SB_CreatePipe`.
    #[doc(alias = "CFG_SB_CreatePipe")]
    #[inline]
    pub fn new<S: AsRef<CStr> + ?Sized>(depth: u16, pipe_name: &S) -> Result<Pipe, Status> {
        let mut p: CFE_SB_PipeId_t = super::ResourceId::UNDEFINED.id;

        let s: Status =
            unsafe { CFE_SB_CreatePipe(&mut p, depth, pipe_name.as_ref().as_ptr()) }.into();

        if p == super::ResourceId::UNDEFINED.id {
            return Err(Status::SB_PIPE_CR_ERR);
        }

        s.as_result(|| Pipe { id: p, _pd: PhantomData })
    }

    /// Deletes the software bus pipe.
    ///
    /// Note that applications should not call this if the deletion
    /// is part of application shutdown;
    /// the framework will do the needed cleanup at application exit.
    ///
    /// Wraps `CFE_SB_DeletePipe`.
    #[doc(alias = "CFG_SB_DeletePipe")]
    #[inline]
    pub fn delete(self) {
        unsafe {
            CFE_SB_DeletePipe(self.id);
        }
    }

    /// Subscribes to messages with ID `msg_id`
    /// on the software bus with default parameters.
    ///
    /// Wraps `CFE_SB_Subscribe`.
    #[doc(alias = "CFG_SB_Subscribe")]
    #[inline]
    pub fn subscribe(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = unsafe { CFE_SB_Subscribe(msg_id.id, self.id) }.into();

        s.as_result(|| ())
    }

    /// Subscribes to messages with ID `msg_id` on the software bus
    /// with the specified quality of service (currently unused by cFE)
    /// and a limit to the number of messages with this ID
    /// allowed in the pipe at the same time.
    ///
    /// Wraps `CFE_SB_SubscribeEx`.
    #[doc(alias = "CFG_SB_SubscribeEx")]
    #[inline]
    pub fn subscribe_ex(
        &mut self,
        msg_id: MsgId,
        quality: Qos,
        msg_lim: u16,
    ) -> Result<(), Status> {
        let s: Status =
            unsafe { CFE_SB_SubscribeEx(msg_id.id, self.id, quality.qos, msg_lim) }.into();

        s.as_result(|| ())
    }

    /// Subscribes to messages with ID `msg_id`,
    /// but keep the subscription local to the current CPU.
    ///
    /// This is typically only used by the [SBN](https://github.com/nasa/SBN) application.
    ///
    /// Wraps `CFE_SB_SubscribeLocal`.
    #[doc(alias = "CFG_SB_SubscribeLocal")]
    #[inline]
    pub fn subscribe_local(&mut self, msg_id: MsgId, msg_lim: u16) -> Result<(), Status> {
        let s: Status = unsafe { CFE_SB_SubscribeLocal(msg_id.id, self.id, msg_lim) }.into();

        s.as_result(|| ())
    }

    /// Removes the current pipe's subscription to messages with ID `msg_id`.
    ///
    /// Wraps `CFE_SB_Unsubscribe`.
    #[doc(alias = "CFG_SB_Unsubscribe")]
    #[inline]
    pub fn unsubscribe(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = unsafe { CFE_SB_Unsubscribe(msg_id.id, self.id) }.into();

        s.as_result(|| ())
    }

    /// Removes the current pipe's subscription to messages with ID `msg_id`,
    /// keeping the unsubscription local to the current CPU.
    ///
    /// This is typically only used by the [SBN](https://github.com/nasa/SBN) application.
    ///
    /// Wraps `CFE_SB_UnsubscribeLocal`.
    #[doc(alias = "CFG_SB_UnsubscribeLocal")]
    #[inline]
    pub fn unsubscribe_local(&mut self, msg_id: MsgId) -> Result<(), Status> {
        let s: Status = unsafe { CFE_SB_UnsubscribeLocal(msg_id.id, self.id) }.into();

        s.as_result(|| ())
    }

    /// Receives a message from the pipe.
    ///
    /// Whether or not a message was received, `closure` gets called with
    /// the result of the reception attempt.
    ///
    /// Uses `time_out` to determine how long to wait for a message if the pipe is empty.
    ///
    /// Passing the message buffer to a closure rather than returning it ensures that
    /// the buffer's lifetime constraints are respected.
    ///
    /// Wraps `CFE_SB_ReceiveBuffer`.
    #[doc(alias = "CFG_SB_ReceiveBuffer")]
    #[inline]
    pub fn receive_buffer<T, F>(&mut self, time_out: TimeOut, closure: F) -> T
    where
        F: for<'a> FnOnce(Result<&'a Message, Status>) -> T,
    {
        let mut buf: *mut CFE_SB_Buffer_t = core::ptr::null_mut();

        let s: Status = unsafe { CFE_SB_ReceiveBuffer(&mut buf, self.id, time_out.into()) }.into();

        let result: Result<&Message, Status>;
        result = if s.severity() == super::StatusSeverity::Error {
            Err(s)
        } else {
            match unsafe { buf.as_ref() } {
                None => Err(Status::SB_BUFFER_INVALID),
                Some(b) => Ok(Message::from_cfe(unsafe { &(b.Msg) })),
            }
        };

        closure(result)
    }
}
