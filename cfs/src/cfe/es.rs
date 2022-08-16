// Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Executive Services system.

use super::{ResourceId, Status};
use cfs_sys::*;
use libc::c_char;
use printf_wrap::{NullString, PrintfArgument, PrintfFmt};

/// The status (or requested status) of a cFE application.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum RunStatus {
    /// Application is exiting with an error.
    AppError     = CFE_ES_RunStatus_CFE_ES_RunStatus_APP_ERROR,

    /// Application wants to exit normally.
    AppExit      = CFE_ES_RunStatus_CFE_ES_RunStatus_APP_EXIT,

    /// Application should continue to run.
    AppRun       = CFE_ES_RunStatus_CFE_ES_RunStatus_APP_RUN,

    /// Indication that the Core Application could not initialize.
    CoreAppInitError = CFE_ES_RunStatus_CFE_ES_RunStatus_CORE_APP_INIT_ERROR,

    /// Indication that the Core Application had a runtime failure.
    CoreAppRuntimeError = CFE_ES_RunStatus_CFE_ES_RunStatus_CORE_APP_RUNTIME_ERROR,

    /// Indication that the system is requesting that the application stop.
    SysDelete    = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_DELETE,

    /// Application caused an exception.
    SysException = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_EXCEPTION,

    /// The system is requesting a reload of the application.
    SysReload    = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_RELOAD,

    /// The system is requesting a restart of the application.
    SysRestart   = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_RESTART,

    /// Reserved value; should not be used.
    Undefined    = CFE_ES_RunStatus_CFE_ES_RunStatus_UNDEFINED,
}

/// The current state of the overall cFS system.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum SystemState {
    /// Single-threaded mode while setting up CFE itself.
    EarlyInit   = CFE_ES_SystemState_CFE_ES_SystemState_EARLY_INIT,

    /// Core apps are starting.
    CoreStartup = CFE_ES_SystemState_CFE_ES_SystemState_CORE_STARTUP,

    /// Core is ready, starting external apps/libraries.
    CoreReady   = CFE_ES_SystemState_CFE_ES_SystemState_CORE_READY,

    /// Startup apps have all completed early init, but are not necessarily operational yet.
    AppsInit    = CFE_ES_SystemState_CFE_ES_SystemState_APPS_INIT,

    /// Normal operation mode; all apps are running.
    Operational = CFE_ES_SystemState_CFE_ES_SystemState_OPERATIONAL,

    /// Reserved for future use; all apps would be stopped.
    Shutdown    = CFE_ES_SystemState_CFE_ES_SystemState_SHUTDOWN,
}

/// Logs an entry/exit marker for a specified ID
/// for use by
/// [the Software Performance Analysis tool](https://github.com/nasa/perfutils-java).
///
/// `marker` is a system-wide ID for some portion of code.
/// `entry_exit` should be `0` for an entry into the code in question,
/// and `1` for an exit.
///
/// Wraps `CFE_ES_PerfLogAdd`.
#[inline]
pub fn perf_log_add(marker: u32, entry_exit: u32) {
    unsafe { CFE_ES_PerfLogAdd(marker, entry_exit) };
}

/// Shortcut for [`perf_log_add`]`(marker, 0)`.
#[inline]
pub fn perf_log_entry(marker: u32) {
    perf_log_add(marker, 0);
}

/// Shortcut for [`perf_log_add`]`(marker, 1)`.
#[inline]
pub fn perf_log_exit(marker: u32) {
    perf_log_add(marker, 1);
}

/// Internal macro to generate _n_-adic wrappers around `CFE_ES_WriteToSysLog`.
macro_rules! wtsl_impl {
    (@ $doc_args:expr, $name:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        #[doc = concat!(
            "Writes a message to the cFE System Log using a format string and ",
            $doc_args, ".\n",
            "\n",
            "Wraps `CFE_ES_WriteToSysLog`.\n",
        )]
        #[inline]
        pub fn $name<$($t),*>(fmt: PrintfFmt<($($t,)*)>, $($var: $t),*) -> Status
            where $($t: PrintfArgument),* {

            unsafe {
                CFE_ES_WriteToSysLog(fmt.as_ptr() $(, $var.as_c_val())*)
            }.into()
        }
    };
    ($num:expr, $name:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        wtsl_impl!(@ concat!(stringify!($num), " format arguments"),
            $name, ( $($t),* ), ( $($var),* )
        );
    };
    ($name:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        wtsl_impl!(@ "1 format argument",
            $name, ( $($t),* ), ( $($var),* )
        );
    };
}

wtsl_impl!(0, write_to_syslog0, (), ());
#[rustfmt::skip]
wtsl_impl!(   write_to_syslog1, (A), (a));
wtsl_impl!(2, write_to_syslog2, (A, B), (a, b));
wtsl_impl!(3, write_to_syslog3, (A, B, C), (a, b, c));
wtsl_impl!(4, write_to_syslog4, (A, B, C, D), (a, b, c, d));
wtsl_impl!(5, write_to_syslog5, (A, B, C, D, E), (a, b, c, d, e));
wtsl_impl!(6, write_to_syslog6, (A, B, C, D, E, F), (a, b, c, d, e, f));
wtsl_impl!(7, write_to_syslog7, (A, B, C, D, E, F, G), (a, b, c, d, e, f, g));
wtsl_impl!(8, write_to_syslog8, (A, B, C, D, E, F, G, H), (a, b, c, d, e, f, g, h));

/// Writes the contents of a [`str`] to the cFE System Log.
///
/// Note that any embedded null characters and anything past them
/// will not get put into the log message.
///
/// Wraps `CFE_ES_WriteToSysLog`.
#[inline]
pub fn write_to_syslog_str(msg: &str) -> Status {
    unsafe {
        CFE_ES_WriteToSysLog(super::RUST_STR_FMT.as_ptr(), msg.len(), msg.as_ptr() as *const c_char)
    }
    .into()
}

/// Exits from the current application.
///
/// Wraps `CFE_ES_ExitApp`.
#[inline]
pub fn exit_app(exit_status: RunStatus) {
    unsafe { CFE_ES_ExitApp(exit_status as u32) };
}

/// Checks for exit requests from the cFE system
/// and possibly makes a request for app shutdown to the cFE system.
///
/// If `run_status` is set to
/// `Some(`[`AppExit`](`RunStatus::AppExit`)`)` or
/// `Some(`[`AppError`](`RunStatus::AppError`)`)`,
/// the cFE system treats the function call
/// as a shutdown request for this application.
///
/// Returns whether the app should continue running;
/// a return value of `false` means the application should
/// gracefully shut down.
///
/// Wraps `CFE_ES_RunLoop`.
#[inline]
pub fn run_loop(run_status: Option<RunStatus>) -> bool {
    let mut rs: u32 = run_status.map_or(0, |status| status as u32);
    let p: *mut u32 = match run_status {
        None => core::ptr::null_mut(),
        Some(_) => &mut rs,
    };
    unsafe { CFE_ES_RunLoop(p) }
}

/// An identifier for cFE applications.
///
/// Wraps `CFE_ES_AppId_t`.
#[derive(Clone, Copy, Debug)]
pub struct AppId {
    id: CFE_ES_AppId_t,
}

impl From<AppId> for ResourceId {
    #[inline]
    fn from(app_id: AppId) -> Self {
        ResourceId { id: app_id.id }
    }
}

/* TODO. Requires obtaining base resource-ID values from the cFE headers...
impl TryFrom<ResourceId> for AppId {
    type Error = ();

    #[inline]
    fn try_from(value: ResourceId) -> Result<Self, Self::Error> {
        if value.base() == CFE_ES_APPID_BASE {
            Ok(AppId { id: value.id })
        } else {
            Err(())
        }
    }
}
*/

/// Returns (if successful) the application ID for the calling cFE application.
///
/// Wraps `CFE_ES_GetAppID`.
#[inline]
pub fn get_app_id() -> Result<AppId, Status> {
    let mut app_id = AppId { id: 0 };
    let s: Status = unsafe { CFE_ES_GetAppID(&mut app_id.id) }.into();
    s.as_result(|| app_id)
}

/// Restarts a single cFE application.
///
/// Wraps `CFE_ES_RestartApp`.
#[inline]
pub fn restart_app(app_id: AppId) -> Result<(), Status> {
    let s: Status = unsafe { CFE_ES_RestartApp(app_id.id) }.into();
    s.as_result(|| ())
}

/// Stops a cFE application, then loads and starts it using the specified file.
///
/// Wraps `CFE_ES_ReloadApp`.
#[inline]
pub fn reload_app(app_id: AppId, app_file_name: NullString) -> Result<(), Status> {
    let s: Status = unsafe { CFE_ES_ReloadApp(app_id.id, app_file_name.as_ptr()) }.into();
    s.as_result(|| ())
}

/// Stops a cFE application, then deletes it from the cFE application table.
///
/// Wraps `CFE_ES_DeleteApp`.
#[inline]
pub fn delete_app(app_id: AppId) -> Result<(), Status> {
    let s: Status = unsafe { CFE_ES_DeleteApp(app_id.id) }.into();
    s.as_result(|| ())
}

/// Waits for a minimum state of the overall cFS system,
/// or a timeout (in milliseconds), whichever comes first.
///
/// Wraps `CFE_ES_WaitForSystemState`.
#[inline]
pub fn wait_for_system_state(min_system_state: SystemState, timeout_ms: u32) -> Result<(), Status> {
    let s: Status =
        unsafe { CFE_ES_WaitForSystemState(min_system_state as u32, timeout_ms) }.into();
    s.as_result(|| ())
}
