// Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Event system.

use super::Status;
use crate::cfe::{es::AppId, time::SysTime};
use crate::sealed_traits;
use cfs_sys::*;
use core::convert::TryFrom;
use core::ffi::c_void;
use core::marker::PhantomData;
use libc::c_char;
use printf_wrap::{PrintfArgument, PrintfFmt};

/// A marker type to ensure you [`register`] before sending events.
#[derive(Clone, Debug)]
pub struct EventSender {
    _x: PhantomData<u8>,
}

/// Event-message filter definition for the binary filter scheme.
///
/// `EventID` is an event ID as supplied to [`send_event_*`](`EventSender::send_event_str`).
/// `Mask` is used to control when an event message is sent as follows:
///
/// For each event ID supplied to [`register`], EVS maintains a (saturating) counter
/// which gets incremented after an event message with that ID is emitted
/// by the application.
/// This counter is binary AND'ed with the `Mask` value for that filter ID.
/// The message gets sent if (and only if) the result of the AND is zero.
///
/// See the [`bin_filter`] module for some possible values of `Mask`.
///
/// This is the same as `CFE_EVS_BinFilter`.
#[doc(alias = "CFE_EVS_BinFilter")]
#[doc(inline)]
pub use cfs_sys::CFE_EVS_BinFilter as BinFilter;

/// Values intended for use in the `Mask` field of [`BinFilter`].
pub mod bin_filter {
    use cfs_sys::*;

    /// Value for [`Mask`](`super::BinFilter::Mask`): all event messages are sent (until the event counter saturates).
    #[doc(alias = "CFE_EVS_NO_FILTER")]
    pub const NO_FILTER: u16 = CFE_EVS_NO_FILTER as u16;

    /// Value for [`Mask`](`super::BinFilter::Mask`): only the first event message is sent.
    #[doc(alias = "CFE_EVS_FIRST_ONE_STOP")]
    pub const FIRST_ONE_STOP: u16 = CFE_EVS_FIRST_ONE_STOP as u16;

    /// Value for [`Mask`](`super::BinFilter::Mask`): only the first two event messages are sent.
    #[doc(alias = "CFE_EVS_FIRST_TWO_STOP")]
    pub const FIRST_TWO_STOP: u16 = CFE_EVS_FIRST_TWO_STOP as u16;

    /// Value for [`Mask`](`super::BinFilter::Mask`): only the first four event messages are sent.
    #[doc(alias = "CFE_EVS_FIRST_4_STOP")]
    pub const FIRST_4_STOP: u16 = CFE_EVS_FIRST_4_STOP as u16;

    /// Value for [`Mask`](`super::BinFilter::Mask`): only the first eight event messages are sent.
    #[doc(alias = "CFE_EVS_FIRST_8_STOP")]
    pub const FIRST_8_STOP: u16 = CFE_EVS_FIRST_8_STOP as u16;

    /// Value for [`Mask`](`super::BinFilter::Mask`): only the first 16 event messages are sent.
    #[doc(alias = "CFE_EVS_FIRST_16_STOP")]
    pub const FIRST_16_STOP: u16 = CFE_EVS_FIRST_16_STOP as u16;

    /// Value for [`Mask`](`super::BinFilter::Mask`): only the first 32 event messages are sent.
    #[doc(alias = "CFE_EVS_FIRST_32_STOP")]
    pub const FIRST_32_STOP: u16 = CFE_EVS_FIRST_32_STOP as u16;

    /// Value for [`Mask`](`super::BinFilter::Mask`): only the first 64 event messages are sent.
    #[doc(alias = "CFE_EVS_FIRST_64_STOP")]
    pub const FIRST_64_STOP: u16 = CFE_EVS_FIRST_64_STOP as u16;

    /// Value for [`Mask`](`super::BinFilter::Mask`): every other event message is sent.
    #[doc(alias = "CFE_EVS_EVERY_OTHER_ONE")]
    pub const EVERY_OTHER_ONE: u16 = CFE_EVS_EVERY_OTHER_ONE as u16;

    /// Value for [`Mask`](`super::BinFilter::Mask`): sends two messages, filters out two, then repeats.
    #[doc(alias = "CFE_EVS_EVERY_OTHER_TWO")]
    pub const EVERY_OTHER_TWO: u16 = CFE_EVS_EVERY_OTHER_TWO as u16;

    /// Value for [`Mask`](`super::BinFilter::Mask`): every fourth event message is sent.
    #[doc(alias = "CFE_EVS_EVERY_FOURTH_ONE")]
    pub const EVERY_FOURTH_ONE: u16 = CFE_EVS_EVERY_FOURTH_ONE as u16;
}

/// A scheme for filtering event messages so that not all get recorded.
///
/// This is a [sealed trait](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed):
/// cFE only supports a fixed set of filter schemes.
pub trait FilterScheme: sealed_traits::FilterSchemeSealed {
    /// An integer identifying the scheme in question.
    ///
    /// This gets used as the `FilterScheme` argument to `CFE_EVS_Register`.
    const SCHEME_ID: u16;
}

impl sealed_traits::FilterSchemeSealed for BinFilter {}

impl FilterScheme for BinFilter {
    const SCHEME_ID: u16 = CFE_EVS_EventFilter_CFE_EVS_EventFilter_BINARY as u16;
}

/// Registers the application with event services.
///
/// This needs to be called before sending event messages, so "send an event"
/// operations are implemented as methods on [`EventSender`],
/// which is provided only by this function.
///
/// Wraps `CFE_EVS_Register`.
#[doc(alias = "CFE_EVS_Register")]
#[inline]
pub fn register<T: FilterScheme>(filters: &[T]) -> Result<EventSender, Status> {
    let num_filters = match u16::try_from(filters.len()) {
        Ok(n) => n,
        Err(_) => {
            return Err(Status::EVS_APP_FILTER_OVERLOAD);
        }
    };

    let s: Status =
        unsafe { CFE_EVS_Register(filters.as_ptr() as *const c_void, num_filters, T::SCHEME_ID) }
            .into();
    s.as_result(|| EventSender { _x: PhantomData })
}

/// The classification of an event message, analogous to the
/// [syslog](https://en.wikipedia.org/wiki/Syslog)
/// severity level.
#[doc(alias = "CFE_EVS_EventType")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
pub enum EventType {
    /// Events that are intended only for debugging, not nominal operations.
    #[doc(alias = "CFE_EVS_EventType_DEBUG")]
    Debug       = CFE_EVS_EventType_CFE_EVS_EventType_DEBUG as u16,

    /// Events that identify a state change or action that is not an error.
    #[doc(alias = "CFE_EVS_EventType_INFORMATION")]
    Information = CFE_EVS_EventType_CFE_EVS_EventType_INFORMATION as u16,

    /// Events that identify an error but are not catastrophic.
    #[doc(alias = "CFE_EVS_EventType_ERROR")]
    Error       = CFE_EVS_EventType_CFE_EVS_EventType_ERROR as u16,

    /// Events that identify errors that are unrecoverable autonomously.
    #[doc(alias = "CFE_EVS_EventType_CRITICAL")]
    Critical    = CFE_EVS_EventType_CFE_EVS_EventType_CRITICAL as u16,
}

/// Internal macro for generating _n_-adic wrappers around `CFE_EVS_Send*Event*`.
macro_rules! send_impl {
    (@ $doc_args:expr, $se:ident, $sewai:ident, $ste:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        #[doc = concat!(
            "Generates a software event using a format string and ",
            $doc_args, ".\n",
            "\n",
            "Wraps `CFE_EVS_SendEvent`.\n",
        )]
        #[doc(alias = "CFE_EVS_SendEvent")]
        #[inline]
        pub fn $se<$($t),*>(&self, event_id: u16, event_type: EventType, fmt: PrintfFmt<($($t,)*)>, $($var: $t),*) -> Status
            where $($t: PrintfArgument),* {

            unsafe {
                CFE_EVS_SendEvent(
                    event_id, event_type as u16, fmt.as_ptr()
                    $(, $var.as_c_val() )*
                )
            }.into()
        }

        #[doc = concat!(
            "Generates a software event (with the specified Application ID) ",
            "using a format string and ",
            $doc_args, ".\n",
            "\n",
            "Wraps `CFE_EVS_SendEventWithAppID`.\n",
        )]
        #[doc(alias = "CFE_EVS_SendEventWithAppID")]
        #[inline]
        pub fn $sewai<$($t),*>(&self, event_id: u16, event_type: EventType, app_id: AppId, fmt: PrintfFmt<($($t,)*)>, $($var: $t),*) -> Status
            where $($t: PrintfArgument),* {

            unsafe {
                CFE_EVS_SendEventWithAppID(
                    event_id, event_type as u16, app_id.id, fmt.as_ptr()
                    $(, $var.as_c_val() )*
                )
            }.into()
        }

        #[doc = concat!(
            "Generates a software event (with a specific time tag) ",
            "using a format string and ",
            $doc_args, ".\n",
            "\n",
            "Wraps `CFE_EVS_SendTimedEvent`.\n",
        )]
        #[doc(alias = "CFE_EVS_SendTimedEvent")]
        #[inline]
        pub fn $ste<$($t),*>(&self, time: SysTime, event_id: u16, event_type: EventType, fmt: PrintfFmt<($($t,)*)>, $($var: $t),*) -> Status
            where $($t: PrintfArgument),* {

            unsafe {
                CFE_EVS_SendTimedEvent(
                    time.tm, event_id, event_type as u16, fmt.as_ptr()
                    $(, $var.as_c_val() )*
                )
            }.into()
        }
    };
    ($num:expr, $se:ident, $sewai:ident, $ste:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        send_impl!(@ concat!(stringify!($num), " format arguments"),
            $se, $sewai, $ste, ( $($t),* ), ( $($var),* )
        );
    };
    ($se:ident, $sewai:ident, $ste:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        send_impl!(@ "1 format argument",
            $se, $sewai, $ste, ( $($t),* ), ( $($var),* )
        );
    };
}

#[rustfmt::skip]
impl EventSender {
    send_impl!(0, send_event0, send_event_with_app_id0, send_timed_event0,
               (), ());
    send_impl!(   send_event1, send_event_with_app_id1, send_timed_event1,
               (A), (a));
    send_impl!(2, send_event2, send_event_with_app_id2, send_timed_event2,
               (A, B), (a, b));
    send_impl!(3, send_event3, send_event_with_app_id3, send_timed_event3,
               (A, B, C), (a, b, c));
    send_impl!(4, send_event4, send_event_with_app_id4, send_timed_event4,
               (A, B, C, D), (a, b, c, d));
    send_impl!(5, send_event5, send_event_with_app_id5, send_timed_event5,
               (A, B, C, D, E), (a, b, c, d, e));
    send_impl!(6, send_event6, send_event_with_app_id6, send_timed_event6,
               (A, B, C, D, E, F), (a, b, c, d, e, f));
    send_impl!(7, send_event7, send_event_with_app_id7, send_timed_event7,
               (A, B, C, D, E, F, G), (a, b, c, d, e, f, g));
    send_impl!(8, send_event8, send_event_with_app_id8, send_timed_event8,
               (A, B, C, D, E, F, G, H), (a, b, c, d, e, f, g, h));
}

impl EventSender {
    /// Generates a software event using a [`str`] as the message.
    ///
    /// Note that any embedded null characters and anything past them
    /// will not get put into the event message.
    ///
    /// Wraps `CFE_EVS_SendEvent`.
    #[doc(alias = "CFE_EVS_SendEvent")]
    #[inline]
    pub fn send_event_str(&self, event_id: u16, event_type: EventType, msg: &str) -> Status {
        unsafe {
            CFE_EVS_SendEvent(
                event_id,
                event_type as u16,
                super::RUST_STR_FMT.as_ptr(),
                msg.len(),
                msg.as_ptr() as *const c_char,
            )
        }
        .into()
    }

    /// Generates a software event with the specified Application ID
    /// using a [`str`] as the message.
    ///
    /// Note that any embedded null characters and anything past them
    /// will not get put into the event message.
    ///
    /// Wraps `CFE_EVS_SendEventWithAppID`.
    #[doc(alias = "CFE_EVS_SendEventWithAppID")]
    #[inline]
    pub fn send_event_with_app_id_str(
        &self,
        event_id: u16,
        event_type: EventType,
        app_id: AppId,
        msg: &str,
    ) -> Status {
        unsafe {
            CFE_EVS_SendEventWithAppID(
                event_id,
                event_type as u16,
                app_id.id,
                super::RUST_STR_FMT.as_ptr(),
                msg.len(),
                msg.as_ptr() as *const c_char,
            )
        }
        .into()
    }

    /// Generates a software event with a specific time tag
    /// using a [`str`] as the message.
    ///
    /// Note that any embedded null characters and anything past them
    /// will not get put into the event message.
    ///
    /// Wraps `CFE_EVS_SendTimedEvent`.
    #[doc(alias = "CFE_EVS_SendTimedEvent")]
    #[inline]
    pub fn send_timed_event_str(
        &self,
        time: SysTime,
        event_id: u16,
        event_type: EventType,
        msg: &str,
    ) -> Status {
        unsafe {
            CFE_EVS_SendTimedEvent(
                time.tm,
                event_id,
                event_type as u16,
                super::RUST_STR_FMT.as_ptr(),
                msg.len(),
                msg.as_ptr() as *const c_char,
            )
        }
        .into()
    }
}
