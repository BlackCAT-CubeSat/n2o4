// Copyright (c) 2023 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Error-related constants, utilities, etc.

use super::OsalError;
use crate::utils::NegativeI32;

const fn err_or_panic(err_code: i32) -> OsalError {
    OsalError {
        code: NegativeI32::new_or_panic(err_code),
    }
}

macro_rules! osal_err_consts {
    ($($error_code:ident , $doc:expr),+ $(,)?) => {
        impl OsalError {
            $(
                #[doc = concat!($doc, ".\n\nWraps `", stringify!($error_code), "`.")]
                pub const $error_code: Self = err_or_panic(crate::sys::$error_code);
            )+
        }
    };
}

osal_err_consts! {
    OS_ERROR, "Failed execution",
    OS_INVALID_POINTER, "Invalid pointer",
    OS_ERROR_ADDRESS_MISALIGNED, "Address misalignment",
    OS_ERROR_TIMEOUT, "Error timeout",
    OS_INVALID_INT_NUM, "Invalid Interrupt number",
    OS_SEM_FAILURE, "Semaphore failure",
    OS_SEM_TIMEOUT, "Semaphore timeout",
    OS_QUEUE_EMPTY, "Queue empty",
    OS_QUEUE_FULL, "Queue full",
    OS_QUEUE_TIMEOUT, "Queue timeout",
    OS_QUEUE_INVALID_SIZE, "Queue invalid size",
    OS_QUEUE_ID_ERROR, "Queue ID error",
    OS_ERR_NAME_TOO_LONG, "Name length including null terminator greater than #OS_MAX_API_NAME",
    OS_ERR_NO_FREE_IDS, "No free IDs",
    OS_ERR_NAME_TAKEN, "Name taken",
    OS_ERR_INVALID_ID, "Invalid ID",
    OS_ERR_NAME_NOT_FOUND, "Name not found",
    OS_ERR_SEM_NOT_FULL, "Semaphore not full",
    OS_ERR_INVALID_PRIORITY, "Invalid priority",
    OS_INVALID_SEM_VALUE, "Invalid semaphore value",
    OS_ERR_FILE, "File error",
    OS_ERR_NOT_IMPLEMENTED, "Not implemented",
    OS_TIMER_ERR_INVALID_ARGS, "Timer invalid arguments",
    OS_TIMER_ERR_TIMER_ID, "Timer ID error",
    OS_TIMER_ERR_UNAVAILABLE, "Timer unavailable",
    OS_TIMER_ERR_INTERNAL, "Timer internal error",
    OS_ERR_OBJECT_IN_USE, "Object in use",
    OS_ERR_BAD_ADDRESS, "Bad address",
    OS_ERR_INCORRECT_OBJ_STATE, "Incorrect object state",
    OS_ERR_INCORRECT_OBJ_TYPE, "Incorrect object type",
    OS_ERR_STREAM_DISCONNECTED, "Stream disconnected",
    OS_ERR_OPERATION_NOT_SUPPORTED, "Requested operation not support on supplied object(s)",
    OS_ERR_INVALID_SIZE, "Invalid Size",
    OS_ERR_OUTPUT_TOO_LARGE, "Size of output exceeds limit ",
    OS_ERR_INVALID_ARGUMENT, "Invalid argument value (other than ID or size)",

    OS_FS_ERR_PATH_TOO_LONG, "FS path too long",
    OS_FS_ERR_NAME_TOO_LONG, "FS name too long",
    OS_FS_ERR_DRIVE_NOT_CREATED, "FS drive not created",
    OS_FS_ERR_DEVICE_NOT_FREE, "FS device not free",
    OS_FS_ERR_PATH_INVALID, "FS path invalid",
}

pub(crate) trait I32Ext {
    /// If the `i32` represents an OSAL error value, returns `Err`;
    /// otherwise, returns `Ok`.
    fn as_osal_status(self) -> Result<i32, OsalError>;
}

impl I32Ext for i32 {
    #[inline]
    fn as_osal_status(self) -> Result<i32, OsalError> {
        match NegativeI32::new(self) {
            Some(code) => Err(OsalError { code }),
            None => Ok(self),
        }
    }
}
