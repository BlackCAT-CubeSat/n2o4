// Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Table system.

use core::ffi::c_void;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use cfs_sys::*;
use printf_wrap::NullString;
use super::Status;

/// A convenience trait for referring to which types can be
/// used as the contents of cFE tables.
pub trait TableType: Copy + Sync + Sized {}

/// Blanket implementation for all eligible types.
impl<T: Copy + Sync + Sized> TableType for T {}

/// A handle to a table.
///
/// Users of this crate don't directly create instances of this `struct`;
/// they create [`OwnedTblHandle`]s and/or [`SharedTblHandle`]s instead.
///
/// Wraps a `CFE_TBL_Handle_t`.
#[doc(alias = "CFE_TBL_Handle_t")]
pub struct TblHandle<T: TableType> {
    hdl: CFE_TBL_Handle_t,
    _x: PhantomData<T>,
}

/// A handle to a table registered by the current application.
///
/// Wraps a `CFE_TBL_Handle_t` obtained through `CFE_TBL_Register`.
#[doc(alias = "CFE_TBL_Handle_t")]
pub struct OwnedTblHandle<T: TableType> {
    th: TblHandle<T>,
}

impl<T: TableType> OwnedTblHandle<T> {
    /// Tries to register a table with cFE, returning a handle if successful.
    ///
    /// Wraps `CFE_TBL_Register`.
    #[doc(alias = "CFE_TBL_Register")]
    #[inline]
    pub fn register(tbl_name: NullString, options: TblOptions, validation_fn: Option<TableValidationFn<T>>) -> () {
        let mut hdl: CFE_TBL_Handle_t = X_CFE_TBL_BAD_TABLE_HANDLE;
        let struct_size = core::mem::size_of::<T>();
        let table_opts = options.as_u16();
        let validation_func_ptr: CFE_TBL_CallbackFuncPtr_t = match validation_fn {
            Some(vf) => vf.cfp,
            None => None,
        };
        unimplemented!("TODO: finish signature, implement actual body");

        let status: Status = unsafe {
            CFE_TBL_Register(&mut hdl, tbl_name.as_ptr(), struct_size, table_opts, None)
        }.into();
    }
}

impl<T: TableType> Deref for OwnedTblHandle<T> {
    type Target = TblHandle<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.th
    }
}

impl<T: TableType> DerefMut for OwnedTblHandle<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.th
    }
}

/// A handle to a table registered by another application.
///
/// Wraps a `CFE_TBL_Handle_t` obtained through `CFE_TBL_Share`.
#[doc(alias = "CFE_TBL_Handle_t")]
pub struct SharedTblHandle<T: TableType> {
    th: TblHandle<T>,
}

impl<T: TableType> SharedTblHandle<T> {
    /// Tries to obtain a handle to a table registered by another application.
    ///
    /// Wraps `CFE_TBL_Share`.
    #[doc(alias = "CFE_TBL_Share")]
    #[inline]
    pub unsafe fn share(tbl_name: NullString) -> Result<Self, Status> {
        let mut hdl: CFE_TBL_Handle_t = X_CFE_TBL_BAD_TABLE_HANDLE;

        let status: Status = unsafe {
            CFE_TBL_Share(&mut hdl, tbl_name.as_ptr())
        }.into();

        match status {
            r @ Status::SUCCESS => {
                if hdl != X_CFE_TBL_BAD_TABLE_HANDLE {
                    Ok(SharedTblHandle { th: TblHandle { hdl, _x: PhantomData } })
                } else {
                    Err(r)
                }
            }
            r => Err(r)
        }
    }
}

impl<T: TableType> Deref for SharedTblHandle<T> {
    type Target = TblHandle<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.th
    }
}

impl<T: TableType> DerefMut for SharedTblHandle<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.th
    }
}

/// Wraps `CFE_TBL_Unregister`.
impl<T: TableType> Drop for SharedTblHandle<T> {
    #[doc(alias = "CFE_TBL_Unregister")]
    #[inline]
    fn drop(&mut self) {
        let _ = unsafe { CFE_TBL_Unregister(self.hdl) };
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TblOptions {
    // TODO: implement (as enum or struct)
}

impl TblOptions {
    pub fn as_u16(&self) -> u16 {
        unimplemented!();
    }
}

pub struct TableValidationFn<T: TableType> {
    cfp: CFE_TBL_CallbackFuncPtr_t,
    _x: PhantomData<T>,
}

impl<T: TableType> TableValidationFn<T> {
    /// This is only meant to be used by the [`table_validation_fn`] macro.
    ///
    /// # Safety
    ///
    /// This function assumes, without checking, that the argument `vf`
    /// treats the `*mut c_void` it gets passed in as an [`Option`]`<&T>`;
    /// callers must ensure that this is actually the case.
    #[doc(hidden)]
    pub const unsafe fn new(vf: unsafe extern "C" fn(*mut c_void) -> i32) -> Self {
        Self { cfp: Some(vf), _x: PhantomData }
    }

    #[doc(hidden)]
    pub const CFE_SUCCESS: i32 = cfs_sys::S_CFE_SUCCESS;
}

/// Creates a `const` [`TableValidationFn`]`<$t>` from
/// static function `$f_wrapped`,
/// a static `fn(&$t) -> Result<(), i32>`.
///
/// If `$f_wrapped` returns `Err(n)`, the error code `n` should negative to have the desired effect.
///
/// The type `$t` is assumed to be [`Sized`].
///
/// ```rust
/// use n2o4::{table_validation_fn, cfe::tbl::TableValidationFn};
///
/// const NEG_VALIDATOR: TableValidationFn<i64> = table_validation_fn!(i64, |x| if *x < 0 { Ok(()) } else { Err(-5) });
/// ```
#[macro_export]
macro_rules! table_validation_fn {
    ($t:ty, $f_wrapped:expr) => {
        {
            const F_WRAP: fn(&$t) -> ::core::result::Result<(), i32> = $f_wrapped;
            const CFE_SUCCESS: i32 = $crate::cfe::tbl::TableValidationFn::<$t>::CFE_SUCCESS;
            unsafe extern "C" fn vf(tbl_ptr: *mut ::core::ffi::c_void) -> i32 {
                let tbl_ptr: *mut $t = tbl_ptr as *mut $t;
                let t: ::core::option::Option<&$t> = unsafe { tbl_ptr.as_ref() };
                match t {
                    None => -999,
                    Some(rt) => match F_WRAP(rt) {
                        Ok(()) => CFE_SUCCESS,
                        Err(result) => if result < 0 { result } else { CFE_SUCCESS },
                    },
                }
            }
            unsafe { $crate::cfe::tbl::TableValidationFn::<$t>::new(vf) }
        }
    };
}
