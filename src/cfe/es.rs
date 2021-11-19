// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

use cfs_sys::*;
use super::Status;
use printf_wrap::null_str;
use libc::c_char;

#[derive(Clone,Copy,Debug)]
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

#[inline]
pub fn write_to_syslog_str(msg: &str) -> Result<(), Status> {
    let status: Status = unsafe {
        CFE_ES_WriteToSysLog(
            null_str!("%.*s").as_ptr(),
            msg.len(), msg.as_ptr() as *const c_char
        )
    }.into();
    status.as_result(|| { () })
}
