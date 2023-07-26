// Copyright (c) 2022-2023 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Synchronization primitives.

use cfs_sys::*;

use super::*;
use crate::utils::CStrBuf;
use core::ffi::{c_char, CStr};

/// A handle for a binary semaphore.
///
/// Wraps `osal_id_t`.
#[doc(alias = "osal_id_t")]
#[derive(Clone, Debug)]
pub struct BinSem {
    pub(crate) id: osal_id_t,
}

impl BinSem {
    /// Attempts to create a new binary semaphore with name `name`,
    /// initial value `initial_value`, and default options; if successful, returns it.
    ///
    /// Wraps `OS_BinSemCreate`.
    #[doc(alias = "OS_BinSemCreate")]
    #[inline]
    pub fn new<S: AsRef<CStr> + ?Sized>(name: &S, initial_value: BinSemState) -> Result<Self, i32> {
        let mut id: osal_id_t = X_OS_OBJECT_ID_UNDEFINED;

        let retval =
            unsafe { OS_BinSemCreate(&mut id, name.as_ref().as_ptr(), initial_value as u32, 0) };

        if retval == I_OS_SUCCESS && id != X_OS_OBJECT_ID_UNDEFINED {
            Ok(Self { id })
        } else {
            Err(retval)
        }
    }

    /// If a binary semaphore with the name `name` exists, returns `Ok(Some(`a handle to it`))`.
    ///
    /// If no binary semaphore with the name exists, returns `Ok(None)`.
    /// If an error occurred, returns `Err(err_code)`.
    ///
    /// Wraps `OS_BinSemGetIdByName`.
    #[doc(alias = "OS_BinSemGetIdByName")]
    #[inline]
    pub fn find_by_name<S: AsRef<CStr> + ?Sized>(name: &S) -> Result<Option<Self>, i32> {
        let mut id: osal_id_t = X_OS_OBJECT_ID_UNDEFINED;

        match unsafe { OS_BinSemGetIdByName(&mut id, name.as_ref().as_ptr()) } {
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

    /// Decrements the semaphore value, blocking until it is non-zero if needed.
    ///
    /// Wraps `OS_BinSemTake`.
    #[doc(alias = "OS_BinSemTake")]
    #[inline]
    pub fn take(&self) -> Result<(), i32> {
        match unsafe { OS_BinSemTake(self.id) } {
            I_OS_SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    /// Decrements the semaphore value, blocking for up to `timeout_ms` milliseconds if need be.
    ///
    /// Returns `Ok(true)` if a lock was obtained before timing out,
    /// `Ok(false)` if the request timed out,
    /// or `Err(err_code)` if an error occurred.
    ///
    /// Wraps `OS_BinSemTimedWait`.
    #[doc(alias = "OS_BinSemTimedWait")]
    #[inline]
    pub fn timed_wait(&self, timeout_ms: u32) -> Result<bool, i32> {
        match unsafe { OS_BinSemTimedWait(self.id, timeout_ms) } {
            I_OS_SUCCESS => Ok(true),
            OS_SEM_TIMEOUT => Ok(false),
            err => Err(err),
        }
    }

    /// Increments the semaphore value, waking up a blocked thread (if any).
    ///
    /// Wraps `OS_BinSemGive`.
    #[doc(alias = "OS_BinSemGive")]
    #[inline]
    pub fn give(&self) -> Result<(), i32> {
        match unsafe { OS_BinSemGive(self.id) } {
            I_OS_SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    /// Unblocks all tasks blocking on the semaphore without incrementing or decrementing its value.
    ///
    /// Wraps `OS_BinSemFlush`.
    #[doc(alias = "OS_BinSemFlush")]
    #[inline]
    pub fn flush(&self) -> Result<(), i32> {
        match unsafe { OS_BinSemFlush(self.id) } {
            I_OS_SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    /// Deletes the binary semaphore.
    ///
    /// Wraps `OS_BinSemDelete`.
    #[doc(alias = "OS_BinSemDelete")]
    #[inline]
    pub fn delete(self) -> Result<(), i32> {
        match unsafe { OS_BinSemDelete(self.id) } {
            I_OS_SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    /// If successful, returns details about the binary semaphore.
    ///
    /// Wraps `OS_BinSemGetInfo`.
    #[doc(alias = "OS_BinSemGetInfo")]
    #[inline]
    pub fn info(&self) -> Result<BinSemProperties, i32> {
        let mut props = OS_bin_sem_prop_t {
            name:    [b'\0' as c_char; MAX_NAME_LEN],
            creator: X_OS_OBJECT_ID_UNDEFINED,
            value:   0,
        };

        match unsafe { OS_BinSemGetInfo(self.id, &mut props) } {
            I_OS_SUCCESS => Ok(BinSemProperties {
                name:    CStrBuf::new(&props.name),
                creator: ObjectId { id: props.creator },
                value:   props.value,
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

impl TryFrom<ObjectId> for BinSem {
    type Error = ObjectTypeConvertError;

    #[inline]
    fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
        match value.obj_type() {
            OS_OBJECT_TYPE_OS_BINSEM => Ok(BinSem { id: value.id }),
            _ => Err(ObjectTypeConvertError {}),
        }
    }
}

/// The initial state of a semaphore.
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum BinSemState {
    /// Full state.
    Full  = OS_SEM_FULL,

    /// Empty state.
    Empty = OS_SEM_EMPTY,
}

/// The properties associated with a [`BinSem`].
///
/// Substitutes for `OS_bin_sem_prop_t`.
#[doc(alias = "OS_bin_sem_prop_t")]
#[derive(Debug)]
pub struct BinSemProperties {
    /// The semaphore's name.
    pub name: CStrBuf<{ MAX_NAME_LEN }>,

    /// The semaphore's creator.
    pub creator: ObjectId,

    /// The semaphore's value.
    pub value: i32,
}

/// A handle for a counting semaphore.
///
/// Wraps `osal_id_t`.
#[doc(alias = "osal_id_t")]
#[derive(Clone, Debug)]
pub struct CountSem {
    pub(crate) id: osal_id_t,
}

impl CountSem {
    /// Attempts to create a new counting semaphore with name `sem_name`,
    /// initial value `initial_value`, and default options;
    /// if successful, returns a handle to it.
    ///
    /// Wraps `OS_CountSemCreate`.
    #[doc(alias = "OS_CountSemCreate")]
    #[inline]
    pub fn new<S: AsRef<CStr> + ?Sized>(sem_name: &S, initial_value: u32) -> Result<Self, i32> {
        let mut id: osal_id_t = X_OS_OBJECT_ID_UNDEFINED;

        let retval =
            unsafe { OS_CountSemCreate(&mut id, sem_name.as_ref().as_ptr(), initial_value, 0) };

        if retval == I_OS_SUCCESS && id != X_OS_OBJECT_ID_UNDEFINED {
            Ok(Self { id })
        } else {
            Err(retval)
        }
    }

    /// If a counting semaphore with the name `name` exists, returns `Ok(Some(`a handle to it`))`.
    ///
    /// If no counting semaphore with the name exists, returns `Ok(None)`.
    /// If an error occurred, returns `Err(err_code)`.
    ///
    /// Wraps `OS_CountSemGetIdByName`.
    #[doc(alias = "OS_CountSemGetIdByName")]
    #[inline]
    pub fn find_by_name<S: AsRef<CStr> + ?Sized>(name: &S) -> Result<Option<Self>, i32> {
        let mut id: osal_id_t = X_OS_OBJECT_ID_UNDEFINED;

        match unsafe { OS_CountSemGetIdByName(&mut id, name.as_ref().as_ptr()) } {
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

    /// Decrements the semaphore value, blocking until it is non-zero if needed.
    ///
    /// Wraps `OS_CountSemTake`.
    #[doc(alias = "OS_CountSemTake")]
    #[inline]
    pub fn take(&self) -> Result<(), i32> {
        match unsafe { OS_CountSemTake(self.id) } {
            I_OS_SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    /// Decrements the semaphore value; if it is non-zero, waits for up to `timeout_ms` milliseconds to be able to decrement.
    ///
    /// Returns `Ok(true)` if a lock was obtained before timing out,
    /// `Ok(false)` if the request timed out,
    /// or `Err(err_code)` if an error occurred.
    ///
    /// Wraps `OS_CountSemTimedWait`.
    #[doc(alias = "OS_CountSemTimedWait")]
    #[inline]
    pub fn timed_wait(&self, timeout_ms: u32) -> Result<bool, i32> {
        match unsafe { OS_CountSemTimedWait(self.id, timeout_ms) } {
            I_OS_SUCCESS => Ok(true),
            OS_SEM_TIMEOUT => Ok(false),
            err => Err(err),
        }
    }

    /// Increments the semaphore value, waking up a blocked thread (if any).
    ///
    /// Wraps `OS_CountSemGive`.
    #[doc(alias = "OS_CountSemGive")]
    #[inline]
    pub fn give(&self) -> Result<(), i32> {
        match unsafe { OS_CountSemGive(self.id) } {
            I_OS_SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    /// Deletes the counting semaphore.
    ///
    /// Wraps `OS_CountSemDelete`.
    #[doc(alias = "OS_CountSemDelete")]
    #[inline]
    pub fn delete(self) -> Result<(), i32> {
        match unsafe { OS_CountSemDelete(self.id) } {
            I_OS_SUCCESS => Ok(()),
            err => Err(err),
        }
    }

    /// If successful, returns details about the counting semaphore.
    ///
    /// Wraps `OS_CountSemGetInfo`.
    #[doc(alias = "OS_CountSemGetInfo")]
    #[inline]
    pub fn info(&self) -> Result<CountSemProperties, i32> {
        let mut props = OS_count_sem_prop_t {
            name:    [b'\0' as c_char; MAX_NAME_LEN],
            creator: X_OS_OBJECT_ID_UNDEFINED,
            value:   0,
        };

        match unsafe { OS_CountSemGetInfo(self.id, &mut props) } {
            I_OS_SUCCESS => Ok(CountSemProperties {
                name:    CStrBuf::new(&props.name),
                creator: ObjectId { id: props.creator },
                value:   props.value,
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

impl TryFrom<ObjectId> for CountSem {
    type Error = ObjectTypeConvertError;

    #[inline]
    fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
        match value.obj_type() {
            OS_OBJECT_TYPE_OS_COUNTSEM => Ok(CountSem { id: value.id }),
            _ => Err(ObjectTypeConvertError {}),
        }
    }
}

/// The properties associated with a [`CountSem`].
///
/// Substitutes for `OS_count_sem_prop_t`.
#[doc(alias = "OS_count_sem_prop_t")]
#[derive(Debug)]
pub struct CountSemProperties {
    /// The semaphore's name.
    pub name: CStrBuf<{ MAX_NAME_LEN }>,

    /// The semaphore's creator.
    pub creator: ObjectId,

    /// The semaphore's value.
    pub value: i32,
}

/// A handle for a mutex semaphore.
///
/// Wraps `osal_id_t`.
#[doc(alias = "osal_id_t")]
#[derive(Clone, Debug)]
pub struct MutSem {
    pub(crate) id: osal_id_t,
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
    pub fn new<S: AsRef<CStr> + ?Sized>(sem_name: &S) -> Result<Self, i32> {
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
    pub fn find_by_name<S: AsRef<CStr> + ?Sized>(name: &S) -> Result<Option<Self>, i32> {
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
/// Substitutes for `OS_mut_sem_prop_t`.
#[doc(alias = "OS_mut_sem_prop_t")]
#[derive(Debug)]
pub struct MutSemProperties {
    /// The mutex's name.
    pub name: CStrBuf<{ super::MAX_NAME_LEN }>,

    /// The creator of the mutex.
    pub creator: ObjectId,
}

macro_rules! owned_sem_variant {
    ($type_name:ident, $wrapped_type:ty, $destructor:ident, $constructor:ident $(; $cparam:ident: $ctype:ty)*) => {
        #[doc = concat!("A wrapper around a [`", stringify!($wrapped_type), "`] that automatically deletes it when dropped.")]
        pub struct $type_name {
            sem: $wrapped_type,
        }

        impl $type_name {
            #[doc = concat!("Like [`", stringify!($wrapped_type), "::new`], but creates an owned semaphore instead.")]
            #[doc = "\n\n"]
            #[doc = concat!("Wraps `", stringify!($constructor), "`.")]
            #[inline]
            pub fn new<S: AsRef<CStr> + ?Sized>(sem_name: &S $(, $cparam: $ctype )*) -> Result<Self, i32> {
                <$wrapped_type>::new(sem_name $(, $cparam)*).map(|sem| $type_name { sem })
            }
        }

        impl core::ops::Deref for $type_name {
            type Target = $wrapped_type;

            #[inline]
            fn deref(&self) -> &$wrapped_type {
                &self.sem
            }
        }

        #[doc = concat!("Wraps `", stringify!($destructor), "`.")]
        impl Drop for $type_name {
            #[inline]
            fn drop(&mut self) {
                let _ = unsafe { $destructor(self.sem.id) };
            }
        }

        #[doc = concat!("Takes the wrapped [`", stringify!($wrapped_type), "`] out of the [`", stringify!($type_name), "`] wrapper without dropping it.")]
        impl From<$type_name> for $wrapped_type {
            #[inline]
            fn from(o_sem: $type_name) -> Self {
                let x = Self { id: o_sem.sem.id };
                let _ = core::mem::ManuallyDrop::new(o_sem);
                x
            }
        }
    };
}

owned_sem_variant!(OwnedBinSem, BinSem, OS_BinSemDelete, OS_BinSemCreate; initial_value: BinSemState);
owned_sem_variant!(OwnedCountSem, CountSem, OS_CountSemDelete, OS_CountSemCreate; initial_value: u32);
owned_sem_variant!(OwnedMutSem, MutSem, OS_MutSemDelete, OS_MutSemCreate);
