// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

use cfs_sys::*;

#[derive(Clone,Copy)]
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
