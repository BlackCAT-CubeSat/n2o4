// Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Table system.

use crate::cfe::time::SysTime;
use crate::cfe::Status;
use crate::utils::CStrBuf;
use cfs_sys::*;
use core::ffi::c_void;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use libc::c_char;
use printf_wrap::NullString;

/// A convenience trait for referring to which types can be
/// used as the contents of cFE tables.
pub trait TableType: Copy + Sync + Sized + 'static {}

/// Blanket implementation for all eligible types.
impl<T: Copy + Sync + Sized + 'static> TableType for T {}

/// Returns characteristics of, and information about, the table with name `table_name`.
///
/// Wraps `CFE_TBL_GetInfo`.
#[doc(alias = "CFE_TBL_GetInfo")]
#[inline]
pub fn info(table_name: NullString) -> Result<TblInfo, Status> {
    let mut info: CFE_TBL_Info_t = DEFAULT_TBL_INFO;

    let status: Status = unsafe { CFE_TBL_GetInfo(&mut info, table_name.as_ptr()) }.into();

    status.as_result(|| (&info).into())
}

/// A handle to a table.
///
/// Wraps a `CFE_TBL_Handle_t`.
#[doc(alias = "CFE_TBL_Handle_t")]
pub struct TblHandle<T: TableType> {
    hdl: CFE_TBL_Handle_t,
    _x:  PhantomData<T>,
}

impl<T: TableType> TblHandle<T> {
    /// Tries to register a loadable table with cFE,
    /// returning a handle if successful.
    ///
    /// Wraps `CFE_TBL_Register`.
    #[doc(alias = "CFE_TBL_Register")]
    #[inline]
    pub fn register(
        tbl_name: NullString,
        options: TblOptions,
        validation_fn: Option<TableValidationFn<T>>,
    ) -> Result<(Self, RegisterInfo), Status> {
        use RegisterInfo as RI;

        let mut hdl: CFE_TBL_Handle_t = X_CFE_TBL_BAD_TABLE_HANDLE;
        let struct_size = core::mem::size_of::<T>();
        let table_opts = options.as_u16();
        let validation_func_ptr = validation_fn.as_cfe_val();

        let status: Status = unsafe {
            CFE_TBL_Register(
                &mut hdl,
                tbl_name.as_ptr(),
                struct_size,
                table_opts,
                validation_func_ptr,
            )
        }
        .into();

        if hdl == X_CFE_TBL_BAD_TABLE_HANDLE {
            return Err(status);
        }

        let register_info = match status {
            Status::SUCCESS => RI::Normal,
            Status::TBL_WARN_DUPLICATE => RI::Duplicate,
            Status::TBL_INFO_RECOVERED_TBL => RI::Recovered,
            Status::TBL_WARN_NOT_CRITICAL => RI::NotCritical,
            _ => return Err(status),
        };

        Ok((Self { hdl, _x: PhantomData }, register_info))
    }

    /// Tries to obtain the current address of the table contents.
    /// If successful, passes a reference to the contents
    /// (and whether the table has been updated since the last
    /// time the application obtained its address or status)
    /// to `closure`, whose return value becomes the output.
    ///
    /// Wraps `CFE_TBL_GetAddress` and `CFE_TBL_ReleaseAddress`.
    #[doc(alias("CFE_TBL_GetAddress", "CFE_TBL_ReleaseAddress"))]
    #[inline]
    pub fn get_ref<F, V>(&mut self, closure: F) -> Result<V, Status>
    where
        F: for<'a> FnOnce(&'a T, bool) -> V,
    {
        let mut tbl_ptr: *mut c_void = core::ptr::null_mut();

        let status: Status = unsafe { CFE_TBL_GetAddress(&mut tbl_ptr, self.hdl) }.into();

        let updated_recently = match status {
            Status::SUCCESS => false,
            Status::TBL_INFO_UPDATED => true,
            _ => {
                return Err(status);
            }
        };

        let return_val = match unsafe { (tbl_ptr as *const T).as_ref() } {
            None => Err(Status::TBL_ERR_INVALID_HANDLE),
            Some(tbl_ref) => Ok(closure(tbl_ref, updated_recently)),
        };

        drop(tbl_ptr);

        let _ = unsafe { CFE_TBL_ReleaseAddress(self.hdl) };

        return_val
    }

    /// Tries to load the table with data from `source`.
    ///
    /// Wraps `CFE_TBL_Load`.
    #[doc(alias = "CFE_TBL_Load")]
    #[inline]
    pub fn load(&mut self, source: TblLoadSource<'_, T>) -> Result<(), Status> {
        use TblLoadSource as TLS;

        let (src_type, src_data_ptr) = match source {
            TLS::Ref(r) => (CFE_TBL_SrcEnum_CFE_TBL_SRC_ADDRESS, r as *const T as *const c_void),
            TLS::FileName(name) => {
                (CFE_TBL_SrcEnum_CFE_TBL_SRC_FILE, name.as_ptr() as *const c_void)
            }
        };

        let status: Status = unsafe { CFE_TBL_Load(self.hdl, src_type, src_data_ptr) }.into();

        status.as_result(|| ())
    }

    /// Notifies Table Services that this application
    /// has modified the contents of this table.
    ///
    /// Generally, applications using this crate won't need to call this explicitly.
    /// The only mutable access to table contents this crate provides is through
    /// [`DumpOnlyTblHandle::get_mut`], which calls `CFE_TBL_Modified` itself after
    /// any modification occurs.
    ///
    /// Wraps `CFE_TBL_Modified`.
    #[doc(alias = "CFE_TBL_Modified")]
    #[inline]
    pub fn modified(&mut self) -> Result<(), Status> {
        let status: Status = unsafe { CFE_TBL_Modified(self.hdl) }.into();

        status.as_result(|| ())
    }

    /// Performs the standard operations to maintain the table image.
    ///
    /// Applications should call this periodically to process pending
    /// requests for updates, validation, or dumping to a buffer.
    ///
    /// On success, returns whether a table update occurred.
    ///
    /// Wraps `CFE_TBL_Manage`.
    #[doc(alias = "CFE_TBL_Manage")]
    #[inline]
    pub fn manage(&mut self) -> Result<bool, Status> {
        let status: Status = unsafe { CFE_TBL_Manage(self.hdl) }.into();

        match status {
            Status::SUCCESS => Ok(false),
            Status::TBL_INFO_UPDATED => Ok(true),
            _ => Err(status),
        }
    }

    /// Updates the contents of the table image, if an update is pending.
    ///
    /// Applications should generally just use the [`manage`](Self::manage) method,
    /// which includes this bit of maintenance.
    ///
    /// Returns whether there was, in fact, an update pending.
    ///
    /// Wraps `CFE_TBL_Update`.
    #[doc(alias = "CFE_TBL_Update")]
    #[inline]
    pub fn update(&mut self) -> Result<bool, Status> {
        let status: Status = unsafe { CFE_TBL_Update(self.hdl) }.into();

        match status {
            Status::SUCCESS => Ok(true),
            Status::TBL_INFO_NO_UPDATE_PENDING => Ok(false),
            _ => Err(status),
        }
    }

    /// Validates the contents of the table image.
    ///
    /// Applications should generally just use the [`manage`](Self::manage) method,
    /// which includes this bit of maintenance.
    ///
    /// Returns whether there was, in fact, a validation request pending.
    ///
    /// Wraps `CFE_TBL_Validate`.
    #[doc(alias = "CFE_TBL_Validate")]
    #[inline]
    pub fn validate(&mut self) -> Result<bool, Status> {
        let status: Status = unsafe { CFE_TBL_Validate(self.hdl) }.into();

        match status {
            Status::SUCCESS => Ok(true),
            Status::TBL_INFO_NO_VALIDATION_PENDING => Ok(false),
            _ => Err(status),
        }
    }

    /// Copies the contents of a dump-only table to a buffer.
    ///
    /// Applications should generally just use the [`manage`](Self::manage) method,
    /// which includes this bit of maintenance.
    ///
    /// Wraps `CFE_TBL_DumpToBuffer`.
    #[doc(alias = "CFE_TBL_DumpToBuffer")]
    #[inline]
    pub fn dump_to_buffer(&mut self) -> Result<(), Status> {
        let status: Status = unsafe { CFE_TBL_DumpToBuffer(self.hdl) }.into();

        status.as_result(|| ())
    }

    /// Returns one of the pending actions required for the table, if any.
    ///
    /// Wraps `CFE_TBL_GetStatus`.
    #[doc(alias = "CFE_TBL_GetStatus")]
    #[inline]
    pub fn status(&self) -> Result<Option<PendingAction>, Status> {
        use PendingAction::*;

        let status: Status = unsafe { CFE_TBL_GetStatus(self.hdl) }.into();

        match status {
            Status::SUCCESS => Ok(None),
            Status::TBL_INFO_UPDATE_PENDING => Ok(Some(Update)),
            Status::TBL_INFO_VALIDATION_PENDING => Ok(Some(Validation)),
            Status::TBL_INFO_DUMP_PENDING => Ok(Some(Dump)),
            _ => Err(status),
        }
    }

    /// Instructs Table Services to notify the calling application with a message when the
    /// table requires management.
    ///
    /// The message will have message ID `msg_id`, function code `function_code`,
    /// and [`u32`] payload `payload`.
    ///
    /// Only the application that owns the table in question may successfully call this.
    ///
    /// Wraps `CFE_TBL_NotifyByMessage`.
    #[doc(alias = "CFE_TBL_NotifyByMessage")]
    #[inline]
    pub fn notify_by_message(
        &mut self,
        msg_id: super::sb::MsgId,
        function_code: super::msg::FunctionCode,
        payload: u32,
    ) -> Result<(), Status> {
        let status: Status =
            unsafe { CFE_TBL_NotifyByMessage(self.hdl, msg_id.id, function_code, payload) }.into();

        status.as_result(|| ())
    }

    /// Unregisters the table corresponding to this handle.
    ///
    /// Note that you generally shouldn't need to call this
    /// for a table that the current application registered, as
    /// cFE automatically unregisters all tables owned by
    /// an application when that application exits.
    ///
    /// Unregistering can be useful for handles to tables
    /// registered by other apps, as it frees up resources,
    /// but this is handled with the [`Drop`] `impl` on [`SharedTblHandle`].
    ///
    /// Wraps `CFE_TBL_Unregister`.
    #[doc(alias = "CFE_TBL_Unregister")]
    #[inline]
    pub fn unregister(self) -> Result<(), Status> {
        let status: Status = unsafe { CFE_TBL_Unregister(self.hdl) }.into();

        status.as_result(|| ())
    }
}

/// A handle to a dump-only table.
///
/// Wraps a `CFE_TBL_Handle_t`.
///
/// # Safety and Concurrency
///
/// As the only writer to the table will be the owner of
/// the `DumpOnlyTblHandle`, this is safe for the owner to use.
/// Other applications with a handle to the table may have
/// concurrency-related problems reading data unless
/// care is taken; that care is _not_ automatically provided
/// in full by `DumpOnlyTblHandle` (there is some non-comprehensive
/// assistance in [`get_mut`](Self::get_mut)).
#[doc(alias = "CFE_TBL_Handle_t")]
pub struct DumpOnlyTblHandle<T: TableType> {
    th:  TblHandle<T>,
    buf: Option<&'static mut T>,
}

impl<T: TableType> DumpOnlyTblHandle<T> {
    /// Tries to register a dump-only table
    /// (with optional user-defined address `tbl_buffer`)
    /// with cFE, returning a handle if successful.
    ///
    /// Wraps `CFE_TBL_Register`
    /// (and for tables with a user-defined address, `CFE_TBL_Load`).
    #[doc(alias("CFE_TBL_Register", "CFE_TBL_Load"))]
    #[inline]
    pub fn register_user_def(
        tbl_name: NullString,
        tbl_buffer: Option<&'static mut T>,
        validation_fn: Option<TableValidationFn<T>>,
    ) -> Result<Self, Status> {
        let mut hdl: CFE_TBL_Handle_t = X_CFE_TBL_BAD_TABLE_HANDLE;
        let struct_size = core::mem::size_of::<T>();
        let validation_func_ptr = validation_fn.as_cfe_val();

        let tbl_options = match tbl_buffer {
            Some(_) => CFE_TBL_OPT_USR_DEF_ADDR as u16,
            None => {
                (CFE_TBL_OPT_DUMP_ONLY | CFE_TBL_OPT_SNGL_BUFFER | CFE_TBL_OPT_NOT_CRITICAL) as u16
            }
        };

        let status: Status = unsafe {
            CFE_TBL_Register(
                &mut hdl,
                tbl_name.as_ptr(),
                struct_size,
                tbl_options,
                validation_func_ptr,
            )
        }
        .into();

        if hdl == X_CFE_TBL_BAD_TABLE_HANDLE {
            return Err(status);
        }

        match status {
            Status::SUCCESS | Status::TBL_WARN_DUPLICATE => (),
            _ => {
                return Err(status);
            }
        };

        let mut tbl_buffer = tbl_buffer;

        if let Some(_) = tbl_buffer {
            let buf_ptr: *mut T =
                tbl_buffer.as_mut().map_or(core::ptr::null_mut(), |x| (*x) as *mut T);

            // NOTE: it is safe for the app to use a `&'static mut T` as the address here,
            // as tables with user-defined addresses are dump-only;
            // there may be concurrency issues for any would-be sharers,
            // but that's not a problem for the current app.

            let s: Status = unsafe {
                CFE_TBL_Load(hdl, CFE_TBL_SrcEnum_CFE_TBL_SRC_ADDRESS, buf_ptr as *mut c_void)
            }
            .into();

            drop(buf_ptr);
            s.as_result(|| ())?;
        }

        Ok(Self {
            th:  TblHandle { hdl, _x: PhantomData },
            buf: tbl_buffer,
        })
    }

    /// Attempts to obtain the current address of the table contents.
    /// If successful, provides `closure` with a mutable reference
    /// to the data backing the table, returning `closure`'s return value.
    ///
    /// Calls `CFE_TBL_Modified` after `closure` finishes to let Table Services
    /// know the table has been modified.
    ///
    /// In the case when the table doesn't have a user-defined address, also
    /// wraps `CFE_TBL_GetAddress` and `CFE_TBL_ReleaseAddress`.
    #[doc(alias("CFE_TBL_Modified", "CFE_TBL_GetAddress", "CFE_TBL_ReleaseAddress"))]
    #[inline]
    pub fn get_mut<F, V>(&mut self, closure: F) -> Result<V, Status>
    where
        F: for<'a> FnOnce(&'a mut T) -> V,
    {
        use core::sync::atomic::{fence, Ordering::SeqCst};

        let buf_ref: Option<&'static mut T> = core::mem::replace(&mut self.buf, None);

        let return_val = if let Some(buf) = buf_ref {
            let rv = closure(buf);
            fence(SeqCst);
            self.buf = Some(buf);
            Ok(rv)
        } else {
            let mut tbl_ptr: *mut c_void = core::ptr::null_mut();

            let status: Status = unsafe { CFE_TBL_GetAddress(&mut tbl_ptr, self.th.hdl) }.into();

            match status {
                Status::SUCCESS | Status::TBL_INFO_UPDATED => (),
                _ => {
                    return Err(status);
                }
            }

            let rv = match unsafe { (tbl_ptr as *mut T).as_mut() } {
                None => Err(Status::TBL_ERR_INVALID_HANDLE),
                Some(tbl_mut) => {
                    let val = Ok(closure(tbl_mut));
                    fence(SeqCst);
                    val
                }
            };

            let _ = unsafe { CFE_TBL_ReleaseAddress(self.th.hdl) };

            rv
        };

        let _ = unsafe { CFE_TBL_Modified(self.th.hdl) };

        return return_val;
    }

    /// Unregisters the table corresponding to this handle.
    ///
    /// Note that you generally shouldn't need to call this,
    /// as cFE automatically unregisters all tables owned by
    /// an application when that application exits.
    ///
    /// In the case of a table with user-defined address,
    /// this consciously does _not_ return the `&'static mut T`
    /// to the backing data, as other applications may still
    /// have handles to the table.
    ///
    /// Wraps `CFE_TBL_Unregister`.
    #[doc(alias = "CFE_TBL_Unregister")]
    #[inline]
    pub fn unregister(self) -> Result<(), Status> {
        let status: Status = unsafe { CFE_TBL_Unregister(self.th.hdl) }.into();

        status.as_result(|| ())
    }
}

impl<T: TableType> Deref for DumpOnlyTblHandle<T> {
    type Target = TblHandle<T>;

    fn deref(&self) -> &Self::Target {
        &self.th
    }
}

impl<T: TableType> DerefMut for DumpOnlyTblHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.th
    }
}

/// A handle to a table registered by another application.
///
/// Wraps a `CFE_TBL_Handle_t` obtained from `CFE_TBL_Share`.
#[doc(alias("CFE_TBL_Handle_t", "CFE_TBL_Share"))]
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

        let status: Status = CFE_TBL_Share(&mut hdl, tbl_name.as_ptr()).into();

        if hdl == X_CFE_TBL_BAD_TABLE_HANDLE {
            return Err(status);
        }

        status.as_result(|| Self {
            th: TblHandle { hdl, _x: PhantomData },
        })
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
        let _ = unsafe { CFE_TBL_Unregister(self.th.hdl) };
    }
}

/// Alternative successful or partially-successful outcomes of [`TblHandle::register`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegisterInfo {
    /// Normal successful registration.
    Normal,

    /// Duplicate registration; table was already registered in a compatible fashion.
    Duplicate,

    /// Table was registered, and it has been initialized based on contents saved in the Critical Data Store.
    Recovered,

    /// Table was registered, but not as a critical table (as was requested).
    NotCritical,
}

/// Options available when registering a table using [`TblHandle::register`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TblOptions(pub TblBuffering, pub TblCriticality);

impl TblOptions {
    /// Returns the `u16` corresponding to `self` for use
    /// as the `TblOptionFlags` parameter to the `CFE_TBL_Register` function.
    #[inline]
    const fn as_u16(&self) -> u16 {
        (self.0 as u16) | (self.1 as u16)
    }
}

/// A good set of options for most tables: loadable, single-buffered, and not critical.
impl Default for TblOptions {
    fn default() -> Self {
        Self(TblBuffering::SingleBuffered, TblCriticality::NotCritical)
    }
}

/// Options regarding buffer use on table modifications.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
pub enum TblBuffering {
    /// Modifications to the table will use a shared memory space,
    /// copying to the actual table buffer when the table update occurs.
    ///
    /// This uses less space than [`DoubleBuffered`](Self::DoubleBuffered), but may be blocking.
    ///
    /// Corresponds to `CFE_TBL_OPT_SNGL_BUFFER`.
    #[doc(alias = "CFE_TBL_OPT_SNGL_BUFFER")]
    SingleBuffered = CFE_TBL_OPT_SNGL_BUFFER as u16,

    /// Modifications to the table will use a reserved buffer
    /// specific to this table, swapping "active" and "inactive"
    /// buffers when the table update occurs.
    ///
    /// This is non-blocking (unless the table is [critical](TblCriticality::Critical)),
    /// but uses more space than [`SingleBuffered`](Self::SingleBuffered).
    ///
    /// Corresponds to `CFE_TBL_OPT_DBL_BUFFER`.
    #[doc(alias = "CFE_TBL_OPT_DBL_BUFFER")]
    DoubleBuffered = CFE_TBL_OPT_DBL_BUFFER as u16,
}

/// Options regarding whether a copy of the table is
/// stored in the Critical Data Store (CDS).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
pub enum TblCriticality {
    /// Not critical; no copy of the table will be stored in the CDS.
    ///
    /// Corresponds to `CFE_TBL_OPT_NOT_CRITICAL`.
    #[doc(alias = "CFE_TBL_OPT_NOT_CRITICAL")]
    NotCritical = CFE_TBL_OPT_NOT_CRITICAL as u16,

    /// Critical; the contents of the active table buffer
    /// will be stored in the CDS.
    ///
    /// Corresponds to `CFE_TBL_OPT_CRITICAL`.
    #[doc(alias = "CFE_TBL_OPT_CRITICAL")]
    Critical    = CFE_TBL_OPT_CRITICAL as u16,
}

/// A source of table-update data for [`TblHandle::load`].
#[derive(Clone, Copy, Debug)]
pub enum TblLoadSource<'a, T> {
    /// Update the table to have the contents of the referred-to `T`.
    Ref(&'a T),

    /// Update the table using the table file at this filename.
    FileName(NullString),
}

/// A pending action for a table.
#[derive(Clone, Copy, Debug)]
pub enum PendingAction {
    /// An update is pending.
    Update,

    /// A validation is pending.
    Validation,

    /// A table dump is pending.
    Dump,
}

/// The characteristics of, and statistical information for,
/// a table.
///
/// Corresponds to `CFE_TBL_Info_t`.
#[doc(alias = "CFE_TBL_Info_t")]
#[derive(Clone, Copy, Debug)]
pub struct TblInfo {
    /// Size of the table in bytes.
    pub size: usize,
    /// The number of applications with access to the table.
    pub num_users: u32,
    /// The CRC most recently calculated by Table Services from the table's contents.
    pub crc: u32,
    /// The time the table was last updated.
    pub last_update_time: SysTime,
    /// Filename of the last file loaded into the table.
    pub last_file_loaded: CStrBuf<MAX_PATH_LEN>,
    /// The file creation time from the header of the last file loaded into the table.
    pub file_create_time: SysTime,
    /// Flag indicating whether the table has been loaded once or not.
    pub table_loaded_once: bool,
    /// Flag indicating whether loads to the table are forbidden.
    pub dump_only: bool,
    /// Flag indicating whether table as a dedicated "inactive" buffer.
    pub double_buffered: bool,
    /// Flag indicating whether the table address was defined by the owning application.
    pub user_def_addr: bool,
    /// Flag indicating whether a copy of the table contents is maintained in the Critical Data Store.
    pub critical: bool,
}

const MAX_PATH_LEN: usize = CFE_MISSION_MAX_PATH_LEN as usize;

#[doc(hidden)]
impl From<&CFE_TBL_Info_t> for TblInfo {
    #[inline]
    fn from(info: &CFE_TBL_Info_t) -> Self {
        Self {
            size: info.Size,
            num_users: info.NumUsers,
            file_create_time: SysTime {
                tm: CFE_TIME_SysTime_t {
                    Seconds:    info.FileCreateTimeSecs,
                    Subseconds: info.FileCreateTimeSubSecs,
                },
            },
            crc: info.Crc,
            last_update_time: SysTime { tm: info.TimeOfLastUpdate },
            table_loaded_once: info.TableLoadedOnce,
            dump_only: info.DumpOnly,
            double_buffered: info.DoubleBuffered,
            user_def_addr: info.UserDefAddr,
            critical: info.Critical,
            last_file_loaded: CStrBuf::new(&info.LastFileLoaded[..]),
        }
    }
}

const DEFAULT_TBL_INFO: CFE_TBL_Info_t = CFE_TBL_Info_t {
    Size: 0,
    NumUsers: 0,
    FileCreateTimeSecs: 0,
    FileCreateTimeSubSecs: 0,
    Crc: 0,
    TimeOfLastUpdate: CFE_TIME_SysTime_t { Seconds: 0, Subseconds: 0 },
    TableLoadedOnce: false,
    DumpOnly: false,
    DoubleBuffered: false,
    UserDefAddr: false,
    Critical: false,
    LastFileLoaded: [b'\0' as c_char; MAX_PATH_LEN],
};

/// A wrapped version of a static `fn` to
/// verify that a table (with contents of type `T`)
/// is in a valid state.
///
/// Users of this crate should not create these directly,
/// but should use the [`table_validation_fn`](crate::table_validation_fn) macro,
/// which expands to a `const`able `TableValidationFn<$t>`.
///
/// Wraps `CFE_TBL_CallbackFuncPtr_t`.
#[doc(alias = "CFE_TBL_CallbackFuncPtr_t")]
#[derive(Clone, Copy, Debug)]
pub struct TableValidationFn<T: TableType> {
    cfp: CFE_TBL_CallbackFuncPtr_t,
    _x:  PhantomData<T>,
}

impl<T: TableType> TableValidationFn<T> {
    /// **WARNING:** This is only meant to be used by the [`table_validation_fn`] macro.
    ///
    /// # Safety
    ///
    /// This function assumes, without checking, that the argument `vf`
    /// treats the `*mut c_void` it gets passed in as an [`Option`]`<&T>`;
    /// callers must ensure that this is actually the case.
    #[doc(hidden)]
    #[inline]
    pub const unsafe fn new(vf: unsafe extern "C" fn(*mut c_void) -> i32) -> Self {
        Self {
            cfp: Some(vf),
            _x:  PhantomData,
        }
    }
}

trait OptionExt {
    /// Returns `self` as a `CFE_TBL_CallbackFuncPtr_t`.
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

/// This is only exported for the use of [`table_validation_fn`](crate::table_validation_fn).
#[doc(hidden)]
pub const CFE_SUCCESS: i32 = cfs_sys::S_CFE_SUCCESS;

/// Creates a `const` [`TableValidationFn`]`<$t>` from
/// static function `$f_wrapped`,
/// a `fn(&$t) -> Result<(), i32>`
/// (or, if `$t` is prefixed by `^`, a `fn(&$t) -> Result<(), `[`NegativeI32`]`>`).
///
/// If `$f_wrapped` returns `Err(n)`, the error code `n`
/// should be negative to have the desired effect
/// (the type [`NegativeI32`] enforces this).
///
/// The type `$t` is assumed to be [`Sized`].
///
/// ```rust
/// use n2o4::{table_validation_fn, cfe::tbl::TableValidationFn};
///
/// const NEG_VALIDATOR: TableValidationFn<i64> = table_validation_fn!(i64, |x| if *x < 0 { Ok(()) } else { Err(-5) });
/// ```
///
/// [`NegativeI32`]: crate::utils::NegativeI32
#[macro_export]
macro_rules! table_validation_fn {
    ($t:ty, $f_wrapped:expr) => {{
        const F_WRAP: fn(&$t) -> ::core::result::Result<(), i32> = $f_wrapped;
        const CFE_SUCCESS: i32 = $crate::cfe::tbl::CFE_SUCCESS;
        unsafe extern "C" fn vf(tbl_ptr: *mut ::core::ffi::c_void) -> i32 {
            use ::core::{option::Option, option::Option::*, result::Result::*};

            let tbl_ptr: *mut $t = tbl_ptr as *mut $t;
            let t: Option<&$t> = unsafe { tbl_ptr.as_ref() };
            match t {
                None => -999,
                Some(rt) => match F_WRAP(rt) {
                    Ok(()) => CFE_SUCCESS,
                    Err(result) => {
                        if result < 0 {
                            result
                        } else {
                            CFE_SUCCESS
                        }
                    }
                },
            }
        }
        unsafe { $crate::cfe::tbl::TableValidationFn::<$t>::new(vf) }
    }};
    (^ $t:ty, $f_wrapped:expr) => {{
        const F_WRAP: fn(&$t) -> ::core::result::Result<(), $crate::utils::NegativeI32> =
            $f_wrapped;
        unsafe extern "C" fn vf(tbl_ptr: *mut ::core::ffi::c_void) -> i32 {
            use ::core::{option::Option, option::Option::*, result::Result::*};

            let tbl_ptr: *mut $t = tbl_ptr as *mut $t;
            let t: Option<&$t> = unsafe { tbl_ptr.as_ref() };
            match t {
                None => -999,
                Some(rt) => match F_WRAP(rt) {
                    Ok(()) => $crate::cfe::tbl::CFE_SUCCESS,
                    Err(result) => result.as_i32(),
                },
            }
        }
        unsafe { $crate::cfe::tbl::TableValidationFn::<$t>::new(vf) }
    }};
}
