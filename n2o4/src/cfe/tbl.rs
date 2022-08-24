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
pub trait TableType: Copy + Sync + Sized + 'static {}

/// Blanket implementation for all eligible types.
impl<T: Copy + Sync + Sized + 'static> TableType for T {}

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

impl<T: TableType> TblHandle<T> {
    /// Tries to load the table with data from `source`.
    ///
    /// Wraps `CFE_TBL_Load`.
    #[doc(alias = "CFE_TBL_Load")]
    #[inline]
    pub fn load(&mut self, source: TblLoadSource<'_, T>) {
        use TblLoadSource as TLS;

        let (src_type, src_data_ptr) = match source {
            TLS::Ref(r) => (CFE_TBL_SrcEnum_CFE_TBL_SRC_ADDRESS, unsafe { r as *const T as *const c_void }),
            TLS::FileName(name) => (CFE_TBL_SrcEnum_CFE_TBL_SRC_FILE, unsafe { name.as_ptr() as *const c_void }),
        };
        unimplemented!("TODO: determine return signature, what to do for each status code");

        let status: Status = unsafe {
            CFE_TBL_Load(self.hdl, src_type, src_data_ptr)
        }.into();
    }

    // TODO: add docstring
    #[doc(alias("CFE_TBL_GetAddress", "CFE_TBL_ReleaseAddress"))]
    #[inline]
    pub fn with_address<F: FnOnce(&T)>(&mut self, closure: F) -> Result<(), Status> {
        unimplemented!("TODO: finish logic, determine return signature");

        let mut tbl_ptr: *mut c_void = core::ptr::null_mut();

        let status: Status = unsafe {
            CFE_TBL_GetAddress(&mut tbl_ptr, self.hdl)
        }.into();

        unimplemented!("TODO: figure out actual error handling here");
        status.as_result(|| ())?;

        match unsafe { (tbl_ptr as *const T).as_ref() } {
            None => unimplemented!("TODO: figure out good thing to do here -- do need to ReleaseAddress..."),
            Some(tbl_ref) => { closure(tbl_ref); }
        };

        drop(tbl_ptr);

        let status: Status = unsafe {
            CFE_TBL_ReleaseAddress(self.hdl)
        }.into();

        unimplemented!("TODO: is this the correct error handling here?");
        status.as_result(|| ())
    }
}

pub enum TblLoadSource<'a, T> {
    Ref(&'a T),
    FileName(NullString),
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
    pub fn register(tbl_name: NullString, options: TblOptions, validation_fn: Option<TableValidationFn<T>>) -> Result<(Self, RegisterInfo), Status> {
        use RegisterInfo as RI;

        let mut hdl: CFE_TBL_Handle_t = X_CFE_TBL_BAD_TABLE_HANDLE;
        let struct_size = core::mem::size_of::<T>();
        let table_opts = options.as_u16();
        let validation_func_ptr = validation_fn.as_cfe_val();

        let status: Status = unsafe {
            CFE_TBL_Register(&mut hdl, tbl_name.as_ptr(), struct_size, table_opts, validation_func_ptr)
        }.into();

        let register_info = match status {
            Status::SUCCESS => RI::Normal,
            Status::TBL_WARN_DUPLICATE => RI::Duplicate,
            Status::TBL_INFO_RECOVERED_TBL => RI::Recovered,
            Status::TBL_WARN_NOT_CRITICAL => RI::NotCritical,
            _ => { return Err(status) }
        };

        Ok((Self { th: TblHandle { hdl, _x: PhantomData }}, register_info))
    }

    /// Tries to register a table (with user-defined address) with cFE,
    /// returning a handle if successful.
    ///
    /// Wraps `CFE_TBL_Register` and `CFE_TBL_Load`.
    #[doc(alias("CFE_TBL_Register", "CFE_TBL_Load"))]
    #[inline]
    pub fn register_user_def(tbl_name: NullString, tbl_buffer: &'static mut T, validation_fn: Option<TableValidationFn<T>>) -> Result<Self, Status> {
        unimplemented!("TODO: is &'static mut T the correct type for tbl_buffer??");
        let mut hdl: CFE_TBL_Handle_t = X_CFE_TBL_BAD_TABLE_HANDLE;
        let struct_size = core::mem::size_of::<T>();
        let validation_func_ptr = validation_fn.as_cfe_val();

        let status: Status = unsafe {
            CFE_TBL_Register(&mut hdl, tbl_name.as_ptr(), struct_size, CFE_TBL_OPT_USR_DEF_ADDR as u16, validation_func_ptr)
        }.into();

        match status {
            Status::SUCCESS | Status::TBL_WARN_DUPLICATE => (),
            _ => { return Err(status); }
        };

        let s: Status = unsafe {
            CFE_TBL_Load(hdl, CFE_TBL_SrcEnum_CFE_TBL_SRC_ADDRESS, tbl_buffer as *mut T as *mut c_void)
        }.into();

        s.as_result(|| Self { th: TblHandle { hdl, _x: PhantomData } })
    }

    // TODO: write docstring
    #[doc(alias = "CFE_TBL_Unregister")]
    #[inline]
    pub fn unregister(self) -> Result<(), Status> {
        let status: Status = unsafe {
            CFE_TBL_Unregister(self.th.hdl)
        }.into();

        status.as_result(|| ())
    }
}

/// Alternative successful or partially-successful outcomes of [`OwnedTblHandle::register`].
pub enum RegisterInfo {
    /// Normal successful registration.
    Normal,

    /// Duplicate registration; table was already registered in a compatible fashion.
    Duplicate,

    /// Table was registered, and has been initialized based on contents saved in the Critical Data Store.
    Recovered,

    /// Table was registered, but not as a critical table (as was requested).
    NotCritical,
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
    ///
    /// # Safety
    ///
    /// The cFE API provides no way to ensure that the table named `tbl_name`
    /// is, in fact, a value of type `T`. This function just assumes that it
    /// is; this fact must be verified by the programmer.
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
    DumpOnly,
    LoadDump(TblBuffering, TblCriticality),
}

impl TblOptions {
    #[inline]
    pub const fn as_u16(&self) -> u16 {
        unimplemented!();
        match *self {
            Self::DumpOnly => (CFE_TBL_OPT_DUMP_ONLY | CFE_TBL_OPT_SNGL_BUFFER | CFE_TBL_OPT_NOT_CRITICAL) as u16,
            Self::LoadDump(buffering, criticality) => (buffering as u16) | (criticality as u16),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
pub enum TblBuffering {
    SingleBuffered = CFE_TBL_OPT_SNGL_BUFFER as u16,
    DoubleBuffered = CFE_TBL_OPT_DBL_BUFFER as u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
pub enum TblCriticality {
    NotCritical = CFE_TBL_OPT_NOT_CRITICAL as u16,
    Critical = CFE_TBL_OPT_CRITICAL as u16,
}

#[derive(Clone, Copy, Debug)]
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
    #[inline]
    pub const unsafe fn new(vf: unsafe extern "C" fn(*mut c_void) -> i32) -> Self {
        Self { cfp: Some(vf), _x: PhantomData }
    }
}

/// This is only exported for the use of [`table_validation_fn`].
#[doc(hidden)]
pub const CFE_SUCCESS: i32 = cfs_sys::S_CFE_SUCCESS;

trait OptionExt {
    fn as_cfe_val(&self) -> CFE_TBL_CallbackFuncPtr_t;
}

impl<T: TableType> OptionExt for Option<TableValidationFn<T>> {
    #[inline]
    fn as_cfe_val(&self) -> CFE_TBL_CallbackFuncPtr_t {
        match self {
            Some(vf) => vf.cfp,
            None => None,
        }
    }
}

/// Creates a `const` [`TableValidationFn`]`<$t>` from
/// static function `$f_wrapped`,
/// a `fn(&$t) -> Result<(), i32>`.
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
            const CFE_SUCCESS: i32 = $crate::cfe::tbl::CFE_SUCCESS;
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
