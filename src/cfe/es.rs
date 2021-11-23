// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

use cfs_sys::*;
use super::Status;
use printf_wrap::{PrintfFmt, PrintfArgument};
use libc::c_char;

#[derive(Clone,Copy,PartialEq,Eq,Debug)]
#[repr(u32)]
pub enum RunStatus {
    AppError = CFE_ES_RunStatus_CFE_ES_RunStatus_APP_ERROR,
    AppExit = CFE_ES_RunStatus_CFE_ES_RunStatus_APP_EXIT,
    AppRun = CFE_ES_RunStatus_CFE_ES_RunStatus_APP_RUN,
    CoreAppInitError = CFE_ES_RunStatus_CFE_ES_RunStatus_CORE_APP_INIT_ERROR,
    CoreAppRuntimeError = CFE_ES_RunStatus_CFE_ES_RunStatus_CORE_APP_RUNTIME_ERROR,
    SysDelete = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_DELETE,
    SysException = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_EXCEPTION,
    SysReload = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_RELOAD,
    SysRestart = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_RESTART,
    Undefined = CFE_ES_RunStatus_CFE_ES_RunStatus_UNDEFINED,
}

#[inline]
pub fn perf_log_add(marker: u32, entry_exit: u32) {
    unsafe { CFE_ES_PerfLogAdd(marker, entry_exit) };
}

#[inline]
pub fn perf_log_entry(marker: u32) { perf_log_add(marker, 0); }

#[inline]
pub fn perf_log_exit(marker: u32) { perf_log_add(marker, 1); }

macro_rules! wtsl_impl {
    (@ $doc_end:expr, $name:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        #[doc = concat!("CFE_ES_WriteToSysLog with ", $doc_end)]
        #[inline]
        pub fn $name<$($t),*>(fmt: PrintfFmt<($($t,)*)>, $($var: $t),*) -> Result<(), Status>
            where $($t: PrintfArgument),* {

            let status: Status = unsafe {
                CFE_ES_WriteToSysLog(fmt.as_ptr() $(, $var.as_c_val())*)
            }.into();
            status.as_result(|| { () })
        }
    };
    ($num:expr, $name:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        wtsl_impl!(@ concat!(stringify!($num), " format arguments."),
            $name, ( $($t),* ), ( $($var),* )
        );
    };
    ($name:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        wtsl_impl!(@ "1 format argument.",
            $name, ( $($t),* ), ( $($var),* )
        );
    };
}

wtsl_impl!( 0, write_to_syslog0, (), () );
wtsl_impl!(    write_to_syslog1, (A), (a) );
wtsl_impl!( 2, write_to_syslog2, (A, B), (a, b) );
wtsl_impl!( 3, write_to_syslog3, (A, B, C), (a, b, c) );
wtsl_impl!( 4, write_to_syslog4, (A, B, C, D), (a, b, c, d) );
wtsl_impl!( 5, write_to_syslog5, (A, B, C, D, E), (a, b, c, d, e) );
wtsl_impl!( 6, write_to_syslog6, (A, B, C, D, E, F), (a, b, c, d, e, f) );
wtsl_impl!( 7, write_to_syslog7, (A, B, C, D, E, F, G), (a, b, c, d, e, f, g) );
wtsl_impl!( 8, write_to_syslog8, (A, B, C, D, E, F, G, H), (a, b, c, d, e, f, g, h) );

/// CFE_ES_WriteToSysLog with a `str` argument.
///
/// Note that any embedded null characters and anything past them
/// will not get put into the log message.
#[inline]
pub fn write_to_syslog_str(msg: &str) -> Result<(), Status> {
    let status: Status = unsafe {
        CFE_ES_WriteToSysLog(
            super::RUST_STR_FMT.as_ptr(),
            msg.len(), msg.as_ptr() as *const c_char
        )
    }.into();
    status.as_result(|| { () })
}

#[inline]
pub fn exit_app(exit_status: RunStatus) {
    unsafe { CFE_ES_ExitApp(exit_status as u32) };
}

#[inline]
pub fn run_loop(run_status: RunStatus) -> bool {
    let mut rs: u32 = run_status as u32;
    let result = unsafe { CFE_ES_RunLoop(&mut rs) };
    result
}
