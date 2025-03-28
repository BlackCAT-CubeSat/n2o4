// Copyright (c) 2022-2023 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Types and methods for interacting with files.

use crate::sys::*;
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

/// Information about a file or directory.
///
/// Semantically equivalent to `os_fstat_t`.
#[doc(alias = "os_fstat_t")]
pub struct FileStat {
    /// The file's mode bits.
    ///
    /// For the individual bits,
    /// see [`DIR`](Self::DIR), [`READ`](Self::READ),
    /// [`WRITE`](Self::WRITE), and [`EXEC`](Self::EXEC).
    pub file_mode_bits: u32,

    /// The time the file was last modified.
    pub file_time: super::OSTime,

    /// The size of the file, in bytes.
    pub file_size: usize,
}

impl FileStat {
    /// Set if the file is a directory.
    ///
    /// Wraps `OS_FILESTAT_MODE_DIR`.
    #[doc(alias = "OS_FILESTAT_MODE_DIR")]
    pub const DIR: u32 = OS_FILESTAT_MODE_DIR;

    /// Set if the file is readable.
    ///
    /// Wraps `OS_FILESTAT_MODE_READ`.
    #[doc(alias = "OS_FILESTAT_MODE_READ")]
    pub const READ: u32 = OS_FILESTAT_MODE_READ;

    /// Set if the file is writable.
    ///
    /// Wraps `OS_FILESTAT_MODE_WRITE`.
    #[doc(alias = "OS_FILESTAT_MODE_WRITE")]
    pub const WRITE: u32 = OS_FILESTAT_MODE_WRITE;

    /// Set if the file is executable.
    ///
    /// Wraps `OS_FILESTAT_MODE_EXEC`.
    #[doc(alias = "OS_FILESTAT_MODE_EXEC")]
    pub const EXEC: u32 = OS_FILESTAT_MODE_EXEC;
}

/// Obtains information about the file or directory at `path`.
///
/// Wraps `OS_stat`.
#[doc(alias = "OS_stat")]
#[inline]
pub fn stat<S: AsRef<CStr>>(path: &S) -> Result<FileStat, OsalError> {
    let path = path.as_ref().as_ptr();
    let mut filestats: os_fstat_t = os_fstat_t {
        FileModeBits: 0,
        FileTime:     OS_time_t { ticks: 0 },
        FileSize:     0,
    };

    // Safety: path isn't modified, and any possible bit-pattern is a valid
    // os_fstat_t.
    unsafe { OS_stat(path, &mut filestats) }.as_osal_status()?;

    Ok(FileStat {
        file_mode_bits: filestats.FileModeBits,
        file_time:      OSTime::from_os_time(filestats.FileTime),
        file_size:      filestats.FileSize,
    })
}

/// Removes the file at `path` from the file system.
///
/// This function's behavior is system-dependent if the file is open;
/// for maximum portability, make sure the file is closed before calling `remove`.
///
/// Wraps `OS_remove`.
#[doc(alias = "OS_remove")]
#[inline]
pub fn remove<S: AsRef<CStr>>(path: &S) -> Result<(), OsalError> {
    let path = path.as_ref().as_ptr();

    // Safety: the string pointed to by path lasts longer than this function invocation
    // and is not modified by the function.
    unsafe { OS_remove(path) }.as_osal_status()?;

    Ok(())
}

/// Changes the name of the file originally at `src` to `dest`.
///
/// `src` and `dest` must reside on the same file system.
///
/// This function's behavior is system-dependent if the file is open;
/// for maximum portability, make sure the file is closed before calling `rename`.
///
/// Wraps `OS_rename`.
#[doc(alias = "OS_rename")]
#[inline]
pub fn rename<S1, S2>(src: &S1, dest: &S2) -> Result<(), OsalError>
where
    S1: AsRef<CStr>,
    S2: AsRef<CStr>,
{
    let src = src.as_ref().as_ptr();
    let dest = dest.as_ref().as_ptr();

    // Safety: the strings pointed to by src and dest
    // are valid for longer than this function invocation
    // and are not modified by the function.
    unsafe { OS_rename(src, dest) }.as_osal_status()?;

    Ok(())
}

/// Copies the file at `src` to `dest`.
///
/// This function's behavior is system-dependent if the file is open;
/// for maximum portability, make sure the file is closed before calling `cp`.
///
/// Wraps `OS_cp`.
#[doc(alias = "OS_cp")]
#[inline]
pub fn cp<S1, S2>(src: &S1, dest: &S2) -> Result<(), OsalError>
where
    S1: AsRef<CStr>,
    S2: AsRef<CStr>,
{
    let src = src.as_ref().as_ptr();
    let dest = dest.as_ref().as_ptr();

    // Safety: the strings pointed to by src and dest
    // are valid for longer than this function invocation
    // and are not modified by the function.
    unsafe { OS_cp(src, dest) }.as_osal_status()?;

    Ok(())
}

/// Moves the file at `src` to `dest`.
///
/// This first attempts to rename the file,
/// which only works if `src` and `dest` are on the same file system.
/// Failing that, the function will copy the file, then remove the original.
///
/// This function's behavior is system-dependent if the file is open;
/// for maximum portability, make sure the file is closed before calling `mv`.
///
/// Wraps `OS_mv`.
#[doc(alias = "OS_mv")]
#[inline]
pub fn mv<S1, S2>(src: &S1, dest: &S2) -> Result<(), OsalError>
where
    S1: AsRef<CStr>,
    S2: AsRef<CStr>,
{
    let src = src.as_ref().as_ptr();
    let dest = dest.as_ref().as_ptr();

    // Safety: the strings pointed to by src and dest
    // are valid for longer than this function invocation
    // and are not modified by the function.
    unsafe { OS_mv(src, dest) }.as_osal_status()?;

    Ok(())
}

/// Determines whether the file `filename` is open within OSAL.
///
/// Wraps `OS_FileOpenCheck`.
#[doc(alias = "OS_FileOpenCheck")]
#[inline]
pub fn file_open_check<S: AsRef<CStr>>(filename: &S) -> Result<bool, OsalError> {
    let fname = filename.as_ref().as_ptr();

    // Safety: the string pointed to by fname lasts longer than this function invocation
    // and is not modified by the function.
    match unsafe { OS_FileOpenCheck(fname) } {
        OS_ERROR => Ok(false),
        status => {
            status.as_osal_status()?;
            Ok(true)
        }
    }
}
