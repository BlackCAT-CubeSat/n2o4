// Copyright (c) 2024 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! File system-level APIs.

use super::{I32Ext, OsalError};
use crate::sys::*;
use crate::utils::CStrBuf;

use core::ffi::{c_char, CStr};

/// The maximum allowed length of an OSAL path name,
/// including directory name, file name, and terminating NUL character.
///
/// Wraps `OS_MAX_PATH_LEN`.
#[doc(alias = "OS_MAX_PATH_LEN")]
pub const MAX_PATH_LEN: usize = OS_MAX_PATH_LEN as usize;

/// Translates an OSAL virtual file-system path
/// to a path name in the underlying system being
/// abstracted over.
///
/// Wraps `OS_TranslatePath`.
#[doc(alias = "OS_TranslatePath")]
#[inline]
pub fn translate_path<S: AsRef<CStr>>(
    virtual_path: &S,
) -> Result<CStrBuf<MAX_PATH_LEN>, OsalError> {
    let virtual_path: *const c_char = virtual_path.as_ref().as_ptr();
    let mut local_path = [b'\0' as c_char; MAX_PATH_LEN];

    // Safety: virtual_path is the start of a null-terminated string,
    // local_path is long enough for all filenames OS_TranslatePath will output,
    // and virtual_path and local_path both outlast the unsafe block.
    unsafe { OS_TranslatePath(virtual_path, local_path.as_mut_ptr()) }.as_osal_status()?;

    Ok(CStrBuf::new_into(local_path))
}
