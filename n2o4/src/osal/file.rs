// Copyright (c) 2022-2023 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Types and methods for interacting with files.

use cfs_sys::*;
use core::convert::TryFrom;
use core::ffi::{c_void, CStr};
use core::ops::{BitOr, BitOrAssign, Deref, DerefMut};

use super::*;

/// A file handle.
///
/// Wraps `osal_id_t`.
#[doc(alias = "osal_id_t")]
#[derive(Clone, Debug)]
pub struct File {
    id: osal_id_t,
}

impl File {
    /// Opens a handle to a file, possibly creating the file if [`FileFlags::CREATE`] is set.
    ///
    /// Wraps `OS_OpenCreate`.
    #[doc(alias = "OS_OpenCreate")]
    #[inline]
    pub fn open_create<S: AsRef<CStr> + ?Sized>(
        path: &S,
        flags: FileFlags,
        access_mode: AccessMode,
    ) -> Result<Self, OsalError> {
        let mut id: osal_id_t = X_OS_OBJECT_ID_UNDEFINED;

        unsafe {
            OS_OpenCreate(&mut id, path.as_ref().as_ptr(), flags.flag as i32, access_mode as i32)
        }
        .as_osal_status()?;

        // some sanity checking of our own:
        if (ObjectId { id }).obj_type() == OS_OBJECT_TYPE_OS_STREAM {
            Ok(File { id })
        } else {
            Err(OsalError::OS_ERR_INVALID_ID)
        }
    }

    /// Reads up to `buf.len()` bytes from the file handle `self`
    /// into the beginning of `buf`.
    ///
    /// Returns the number of bytes actually read into `buf` if successful,
    /// the error code if not.
    ///
    /// Wraps `OS_read`.
    #[doc(alias = "OS_read")]
    #[inline]
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, OsalError> {
        let buffer = buf.as_mut_ptr() as *mut c_void;
        let retval = unsafe { OS_read(self.id, buffer, buf.len()) }.as_osal_status()?;

        Ok(retval as usize)
    }

    /// Writes up to `buf.len()` bytes from `buf`
    /// to the file handle `self`.
    ///
    /// Returns the number of bytes written to the file if successful,
    /// the error code if not.
    ///
    /// Wraps `OS_write`.
    #[doc(alias = "OS_write")]
    #[inline]
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, OsalError> {
        let buffer = buf.as_ptr() as *const c_void;
        let retval = unsafe { OS_write(self.id, buffer, buf.len()) }.as_osal_status()?;

        Ok(retval as usize)
    }

    /// Seeks the file handle `self`
    /// to the specified location in the file.
    ///
    /// Returns the offset from the file start if successful,
    /// the error code if not.
    ///
    /// Wraps `OS_lseek`.
    #[doc(alias = "OS_lseek")]
    #[inline]
    pub fn lseek(&mut self, offset: i32, whence: SeekReference) -> Result<u32, OsalError> {
        let retval = unsafe { OS_lseek(self.id, offset, whence as u32) }.as_osal_status()?;

        Ok(retval as u32)
    }

    /// Closes the file handle `self`.
    ///
    /// Wraps `OS_close`.
    #[doc(alias = "OS_close")]
    #[inline]
    pub fn close(self) -> Result<(), OsalError> {
        unsafe { OS_close(self.id) }.as_osal_status()?;

        Ok(())
    }

    /// Returns the [`ObjectId`] for the file.
    #[inline]
    pub fn as_id(&self) -> ObjectId {
        ObjectId { id: self.id }
    }
}

/// Converts an `ObjectId` to a `File` if sensible.
impl TryFrom<ObjectId> for File {
    type Error = ObjectTypeConvertError;

    #[inline]
    fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
        if value.obj_type() == OS_OBJECT_TYPE_OS_STREAM {
            Ok(File { id: value.id })
        } else {
            Err(ObjectTypeConvertError {})
        }
    }
}

/// A wrapper for [`File`] that automatically closes its file handle when dropped.
#[derive(Debug)]
pub struct OwnedFile {
    f: File,
}

impl OwnedFile {
    /// Like [`File::open_create`], but returning an [`OwnedFile`] on success instead.
    #[doc(alias = "OS_OpenCreate")]
    #[inline]
    pub fn open_create<S: AsRef<CStr> + ?Sized>(
        path: &S,
        flags: FileFlags,
        access_mode: AccessMode,
    ) -> Result<Self, OsalError> {
        File::open_create(path, flags, access_mode).map(|f| OwnedFile { f })
    }
}

impl Deref for OwnedFile {
    type Target = File;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.f
    }
}

impl DerefMut for OwnedFile {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.f
    }
}

/// Wraps `OS_close`.
impl Drop for OwnedFile {
    #[inline]
    fn drop(&mut self) {
        let _ = unsafe { OS_close(self.f.id) };
    }
}

/// Takes the wrapped [`File`] out of the [`OwnedFile`] wrapper
/// without closing it.
impl From<OwnedFile> for File {
    #[inline]
    fn from(o_f: OwnedFile) -> Self {
        let x = File { id: o_f.f.id };
        let _ = core::mem::ManuallyDrop::new(o_f);
        x
    }
}

/// The access mode a file should be opened with.
///
/// Used with [`File::open_create`].
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum AccessMode {
    /// Read-only access.
    ///
    /// Wraps `OS_READ_ONLY`.
    #[doc(alias = "OS_READ_ONLY")]
    ReadOnly  = OS_READ_ONLY as i32,

    /// Write-only access.
    ///
    /// Wraps `OS_WRITE_ONLY`.
    #[doc(alias = "OS_WRITE_ONLY")]
    WriteOnly = OS_WRITE_ONLY as i32,

    /// Read-write access.
    ///
    /// Wraps `OS_READ_WRITE`.
    #[doc(alias = "OS_READ_WRITE")]
    ReadWrite = OS_READ_WRITE as i32,
}

/// Flags for opening/creating a [`File`].
///
/// This is a bitfield; elements may be combined using the `|` operator.
///
/// Wraps `OS_file_flag_t`.
#[doc(alias = "OS_file_flag_t")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FileFlags {
    flag: OS_file_flag_t,
}

impl FileFlags {
    /// No flags set.
    ///
    /// Wraps `OS_FILE_FLAG_NONE`.
    #[doc(alias = "OS_FILE_FLAG_NONE")]
    pub const NONE: FileFlags = Self {
        flag: OS_file_flag_t_OS_FILE_FLAG_NONE,
    };

    /// If the file doesn't exist, create it.
    ///
    /// Wraps `OS_FILE_FLAG_CREATE`.
    #[doc(alias = "OS_FILE_FLAG_CREATE")]
    pub const CREATE: FileFlags = Self {
        flag: OS_file_flag_t_OS_FILE_FLAG_CREATE,
    };

    /// If the file exists, truncate it on opening.
    ///
    /// Wraps `OS_FILE_FLAG_TRUNCATE`.
    #[doc(alias = "OS_FILE_FLAG_TRUNCATE")]
    pub const TRUNCATE: FileFlags = Self {
        flag: OS_file_flag_t_OS_FILE_FLAG_TRUNCATE,
    };
}

impl BitOr<FileFlags> for FileFlags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: FileFlags) -> Self::Output {
        FileFlags { flag: self.flag | rhs.flag }
    }
}

impl BitOrAssign for FileFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

/// The reference point for a seek offset.
///
/// Used as the `whence` argument of [`File::lseek`].
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum SeekReference {
    /// Seek from the beginning of the file.
    ///
    /// Wraps `OS_SEEK_SET`.
    #[doc(alias = "OS_SEEK_SET")]
    Beginning = OS_SEEK_SET,

    /// Seek from the current location in the file.
    ///
    /// Wraps `OS_SEEK_CUR`.
    #[doc(alias = "OS_SEEK_CUR")]
    Current   = OS_SEEK_CUR,

    /// Seek from the end of the file.
    ///
    /// Wraps `OS_SEEK_END`.
    #[doc(alias = "OS_SEEK_END")]
    End       = OS_SEEK_END,
}
