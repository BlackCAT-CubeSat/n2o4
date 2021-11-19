// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

use cfs_sys::*;
use super::Status;
use printf_wrap::{PrintfFmt, PrintfArgument};
use libc::c_char;
use core::ffi::c_void;
use core::convert::TryFrom;
use core::marker::PhantomData;

/// A marker type to ensure you [`register`] before sending events.
pub struct EventSender {
    _x: PhantomData<u8>
}

pub type BinFilter = CFE_EVS_BinFilter;

pub trait FilterScheme {
    const SCHEME_ID: u16;
}

impl FilterScheme for BinFilter {
    const SCHEME_ID: u16 = CFE_EVS_EventFilter_CFE_EVS_EventFilter_BINARY as u16;
}

#[inline]
pub fn register<T: FilterScheme>(filters: &[T]) -> Result<EventSender, Status> {
    let num_filters = match u16::try_from(filters.len()) {
        Ok(n)  => n,
        Err(_) => { return Err(Status::EVS_APP_FILTER_OVERLOAD); }
    };

    let s: Status = unsafe {
        CFE_EVS_Register(
            filters.as_ptr() as *const c_void, num_filters, T::SCHEME_ID
        )
    }.into();
    s.as_result(|| { EventSender { _x: PhantomData } })
}

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
#[repr(u16)]
pub enum EventType {
    Debug = CFE_EVS_EventType_CFE_EVS_EventType_DEBUG as u16,
    Information = CFE_EVS_EventType_CFE_EVS_EventType_INFORMATION as u16,
    Error = CFE_EVS_EventType_CFE_EVS_EventType_ERROR as u16,
    Critical = CFE_EVS_EventType_CFE_EVS_EventType_CRITICAL as u16,
}

macro_rules! send_impl {
    (@ $doc_end:expr, $se:ident, $sewai:ident, $ste:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        #[doc = concat!("CFE_EVS_SendEvent with ", $doc_end)]
        #[inline]
        pub fn $se<$($t),*>(&self, event_id: u16, event_type: EventType, fmt: PrintfFmt<($($t,)*)>, $($var: $t),*) -> Result<(), Status>
            where $($t: PrintfArgument),* {

            let status: Status = unsafe {
                CFE_EVS_SendEvent(
                    event_id, event_type as u16, fmt.as_ptr()
                    $(, $var.as_c_val() )*
                )
            }.into();
            status.as_result(|| { () })
        }

        #[doc = concat!("CFE_EVS_SendEventWithAppID with ", $doc_end)]
        #[inline]
        pub fn $sewai<$($t),*>(&self, event_id: u16, event_type: EventType, app_id: CFE_ES_AppId_t, fmt: PrintfFmt<($($t,)*)>, $($var: $t),*) -> Result<(), Status>
            where $($t: PrintfArgument),* {

            let status: Status = unsafe {
                CFE_EVS_SendEventWithAppID(
                    event_id, event_type as u16, app_id, fmt.as_ptr()
                    $(, $var.as_c_val() )*
                )
            }.into();
            status.as_result(|| { () })
        }

        #[doc = concat!("CFE_EVS_SendTimedEvent with ", $doc_end)]
        #[inline]
        pub fn $ste<$($t),*>(&self, time: CFE_TIME_SysTime_t, event_id: u16, event_type: EventType, fmt: PrintfFmt<($($t,)*)>, $($var: $t),*) -> Result<(), Status>
            where $($t: PrintfArgument),* {

            let status: Status = unsafe {
                CFE_EVS_SendTimedEvent(
                    time, event_id, event_type as u16, fmt.as_ptr()
                    $(, $var.as_c_val() )*
                )
            }.into();
            status.as_result(|| { () })
        }
    };
    ($num:expr, $se:ident, $sewai:ident, $ste:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        send_impl!(@ concat!(stringify!($num), " format arguments."),
            $se, $sewai, $ste, ( $($t),* ), ( $($var),* )
        );
    };
    ($se:ident, $sewai:ident, $ste:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        send_impl!(@ "1 format argument.",
            $se, $sewai, $ste, ( $($t),* ), ( $($var),* )
        );
    };
}

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

    /// CFE_EVS_SendEvent with a `str` argument.
    #[inline]
    pub fn send_event_str(&self, event_id: u16, event_type: EventType, msg: &str) -> Result<(), Status> {
        let status: Status = unsafe {
            CFE_EVS_SendEvent(
                event_id, event_type as u16, super::RUST_STR_FMT.as_ptr(),
                msg.len(), msg.as_ptr() as *const c_char
            )
        }.into();
        status.as_result(|| { () })
    }

    /// CFE_EVS_SendEventWithAppID with a `str` argument.
    #[inline]
    pub fn send_event_with_app_id_str(&self, event_id: u16, event_type: EventType, app_id: CFE_ES_AppId_t, msg: &str) -> Result<(), Status> {
        let status: Status = unsafe {
            CFE_EVS_SendEventWithAppID(
                event_id, event_type as u16, app_id, super::RUST_STR_FMT.as_ptr(),
                msg.len(), msg.as_ptr() as *const c_char
            )
        }.into();
        status.as_result(|| { () })
    }

    /// CFE_EVS_SendTimedEvent with a `str` argument.
    #[inline]
    pub fn send_timed_event_str(&self, time: CFE_TIME_SysTime_t, event_id: u16, event_type: EventType, msg: &str) -> Result<(), Status> {
        let status: Status = unsafe {
            CFE_EVS_SendTimedEvent(
                time, event_id, event_type as u16, super::RUST_STR_FMT.as_ptr(),
                msg.len(), msg.as_ptr() as *const c_char
            )
        }.into();
        status.as_result(|| { () })
    }
}
