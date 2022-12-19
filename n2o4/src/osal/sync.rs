// Copyright (c) 2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Synchronization primitives.

use cfs_sys::*;

use super::*;
use crate::utils::CStrBuf;
use core::ffi::{c_char, CStr};
use core::mem::ManuallyDrop;
use core::ops::Deref;

/// A handle for a mutex semaphore.
///
/// Wraps `osal_id_t`.
#[doc(alias = "osal_id_t")]
#[derive(Clone, Debug)]
pub struct MutSem {
    id: osal_id_t,
}

impl MutSem {
    /// Tries to create a new mutex semaphore with default options;
    /// if successful, returns it.
    ///
    /// Per the cFE Users Guide, mutexes are always created in the unlocked state.
    ///
    /// Wraps `OS_MutSemCreate`.
    #[doc(alias = "OS_MutSemCreate")]
    #[inline]
    pub fn new<S: AsRef<CStr>>(sem_name: &S) -> Result<Self, i32> {
        let mut id: osal_id_t = X_OS_OBJECT_ID_UNDEFINED;

        let retval = unsafe { OS_MutSemCreate(&mut id, sem_name.as_ref().as_ptr(), 0) };

        if retval == I_OS_SUCCESS && id != X_OS_OBJECT_ID_UNDEFINED {
            Ok(Self { id })
        } else {
            Err(retval)
        }
    }

    /// If a mutex with the name `name` exists, returns `Ok(Some(`a handle to it`)`.
    ///
    /// If no mutex with the name exists, returns `Ok(None)`.
    /// If an error occurred, returns `Err(err_code)`.
    ///
    /// Wraps `OS_MutSemGetIdByName`.
    #[doc(alias = "OS_MutSemGetIdByName")]
    #[inline]
    pub fn find_by_name<S: AsRef<CStr>>(name: &S) -> Result<Option<Self>, i32> {
        let mut id: osal_id_t = X_OS_OBJECT_ID_UNDEFINED;

        match unsafe { OS_MutSemGetIdByName(&mut id, name.as_ref().as_ptr()) } {
            I_OS_SUCCESS => {
                if id != X_OS_OBJECT_ID_UNDEFINED {
                    Ok(Some(Self { id }))
                } else {
                    Err(I_OS_SUCCESS)
                }
            }
            OS_ERR_NAME_NOT_FOUND => Ok(None),
            err => Err(err),
        }
    }

    /// Attempts to acquire the mutex, blocking until it does.
    /// Assuming nothing went wrong acquiring, runs the closure, then releases the mutex.
    ///
    /// Wraps `OS_MutSemTake` and `OS_MutSemGive`.
    #[doc(alias("OS_MutSemTake", "OS_MutSemGive"))]
    #[inline]
    pub fn lock<T, F: FnOnce() -> T>(&self, closure: F) -> Result<T, i32> {
        self.take()?;

        struct MutGuard {
            x: MutSem,
        }
        impl Drop for MutGuard {
            fn drop(&mut self) {
                let _ = self.x.give();
            }
        }

        let guard = MutGuard { x: self.clone() };

        let val = closure();

        drop(guard);
        Ok(val)
    }

    // TODO: determine if this should be `pub`
    /// If successful, acquires the mutex; if the mutex is currently acquired, this thread will block until it does acquire it.
    ///
    /// Wraps `OS_MutSemTake`.
    #[doc(alias = "OS_MutSemTake")]
    #[inline]
    fn take(&self) -> Result<(), i32> {
        match unsafe { OS_MutSemTake(self.id) } {
            I_OS_SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    // TODO: determine if this should be `pub`
    /// If successful, releases the mutex, unblocking a thread (if any) waiting to acquire it.
    ///
    /// Wraps `OS_MutSemGive`.
    #[doc(alias = "OS_MutSemGive")]
    #[inline]
    fn give(&self) -> Result<(), i32> {
        match unsafe { OS_MutSemGive(self.id) } {
            I_OS_SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    /// Deletes the mutex.
    ///
    /// Wraps `OS_MutSemDelete`.
    #[doc(alias = "OS_MutSemDelete")]
    #[inline]
    pub fn delete(self) -> Result<(), i32> {
        match unsafe { OS_MutSemDelete(self.id) } {
            I_OS_SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    /// If successful, returns details about the mutex.
    ///
    /// Wraps `OS_MutSemGetInfo`.
    #[doc(alias = "OS_MutSemGetInfo")]
    #[inline]
    pub fn info(&self) -> Result<MutSemProperties, i32> {
        let mut info: OS_mut_sem_prop_t = OS_mut_sem_prop_t {
            name:    [b'\0' as c_char; super::MAX_NAME_LEN],
            creator: X_OS_OBJECT_ID_UNDEFINED,
        };

        match unsafe { OS_MutSemGetInfo(self.id, &mut info) } {
            I_OS_SUCCESS => Ok(MutSemProperties {
                name:    CStrBuf::new(&info.name),
                creator: ObjectId { id: info.creator },
            }),
            err => Err(err),
        }
    }

    /// Returns the [`ObjectId`] for the mutex.
    #[inline]
    pub fn as_id(&self) -> ObjectId {
        ObjectId { id: self.id }
    }
}

impl TryFrom<ObjectId> for MutSem {
    type Error = ObjectTypeConvertError;

    #[inline]
    fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
        match value.obj_type() {
            OS_OBJECT_TYPE_OS_MUTEX => Ok(MutSem { id: value.id }),
            _ => Err(ObjectTypeConvertError {}),
        }
    }
}

/// The properties associated with a [`MutSem`].
///
/// Substitutes for `os_mut_sem_prop_t`.
#[doc(alias = "os_mut_sem_prop_t")]
pub struct MutSemProperties {
    /// The mutex's name.
    pub name: CStrBuf<{ super::MAX_NAME_LEN }>,

    /// The creator of the mutex.
    pub creator: ObjectId,
}

/// A wrapper around a [`MutSem`] that automatically deletes the mutex when dropped.
pub struct OwnedMutSem {
    ms: MutSem,
}

impl OwnedMutSem {
    /// Like [`MutSem::new`], but creates an owned mutex semaphore instead.
    ///
    /// Wraps `OS_MutSemCreate`.
    #[doc(alias = "OS_MutSemCreate")]
    #[inline]
    pub fn new<S: AsRef<CStr>>(sem_name: &S) -> Result<Self, i32> {
        MutSem::new(sem_name).map(|ms| OwnedMutSem { ms })
    }
}

impl Deref for OwnedMutSem {
    type Target = MutSem;

    #[inline]
    fn deref(&self) -> &MutSem {
        &self.ms
    }
}

/// Wraps `OS_MutSemDelete`.
impl Drop for OwnedMutSem {
    #[inline]
    fn drop(&mut self) {
        let _ = unsafe { OS_MutSemDelete(self.ms.id) };
    }
}

/// Takes the wrapped [`MutSem`] out of the [`OwnedMutSem`] wrapper without dropping it.
impl From<OwnedMutSem> for MutSem {
    #[inline]
    fn from(o_ms: OwnedMutSem) -> Self {
        let x = MutSem { id: o_ms.ms.id };
        let _ = ManuallyDrop::new(o_ms);
        x
    }
}
