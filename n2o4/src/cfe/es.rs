// Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Executive Services system.

use super::{ResourceId, Status};
use cfs_sys::*;
use core::ffi::{c_char, c_void, CStr};
use core::marker::PhantomData;
use printf_wrap::{PrintfArgument, PrintfFmt};

/// The status (or requested status) of a cFE application.
#[doc(alias = "CFE_ES_RunStatus")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum RunStatus {
    /// Application is exiting with an error.
    #[doc(alias = "CFE_ES_RunStatus_APP_ERROR")]
    AppError     = CFE_ES_RunStatus_CFE_ES_RunStatus_APP_ERROR,

    /// Application wants to exit normally.
    #[doc(alias = "CFE_ES_RunStatus_APP_EXIT")]
    AppExit      = CFE_ES_RunStatus_CFE_ES_RunStatus_APP_EXIT,

    /// Application should continue to run.
    #[doc(alias = "CFE_ES_RunStatus_APP_RUN")]
    AppRun       = CFE_ES_RunStatus_CFE_ES_RunStatus_APP_RUN,

    /// Indication that the Core Application could not initialize.
    #[doc(alias = "CFE_ES_RunStatus_CORE_APP_INIT_ERROR")]
    CoreAppInitError = CFE_ES_RunStatus_CFE_ES_RunStatus_CORE_APP_INIT_ERROR,

    /// Indication that the Core Application had a runtime failure.
    #[doc(alias = "CFE_ES_RunStatus_CORE_APP_RUNTIME_ERROR")]
    CoreAppRuntimeError = CFE_ES_RunStatus_CFE_ES_RunStatus_CORE_APP_RUNTIME_ERROR,

    /// Indication that the system is requesting that the application stop.
    #[doc(alias = "CFE_ES_RunStatus_SYS_DELETE")]
    SysDelete    = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_DELETE,

    /// Application caused an exception.
    #[doc(alias = "CFE_ES_RunStatus_SYS_EXCEPTION")]
    SysException = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_EXCEPTION,

    /// The system is requesting a reload of the application.
    #[doc(alias = "CFE_ES_RunStatus_SYS_RELOAD")]
    SysReload    = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_RELOAD,

    /// The system is requesting a restart of the application.
    #[doc(alias = "CFE_ES_RunStatus_SYS_RESTART")]
    SysRestart   = CFE_ES_RunStatus_CFE_ES_RunStatus_SYS_RESTART,

    /// Reserved value; should not be used.
    #[doc(alias = "CFE_ES_RunStatus_UNDEFINED")]
    Undefined    = CFE_ES_RunStatus_CFE_ES_RunStatus_UNDEFINED,
}

/// The current state of the overall cFS system.
#[doc(alias = "CFE_ES_SystemState")]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum SystemState {
    /// Single-threaded mode while setting up CFE itself.
    #[doc(alias = "CFE_ES_SystemState_EARLY_INIT")]
    EarlyInit   = CFE_ES_SystemState_CFE_ES_SystemState_EARLY_INIT,

    /// Core apps are starting.
    #[doc(alias = "CFE_ES_SystemState_CORE_STARTUP")]
    CoreStartup = CFE_ES_SystemState_CFE_ES_SystemState_CORE_STARTUP,

    /// Core is ready, starting external apps/libraries.
    #[doc(alias = "CFE_ES_SystemState_CORE_READY")]
    CoreReady   = CFE_ES_SystemState_CFE_ES_SystemState_CORE_READY,

    /// Startup apps have all completed early init, but are not necessarily operational yet.
    #[doc(alias = "CFE_ES_SystemState_APPS_INIT")]
    AppsInit    = CFE_ES_SystemState_CFE_ES_SystemState_APPS_INIT,

    /// Normal operation mode; all apps are running.
    #[doc(alias = "CFE_ES_SystemState_OPERATIONAL")]
    Operational = CFE_ES_SystemState_CFE_ES_SystemState_OPERATIONAL,

    /// Reserved for future use; all apps would be stopped.
    #[doc(alias = "CFE_ES_SystemState_SHUTDOWN")]
    Shutdown    = CFE_ES_SystemState_CFE_ES_SystemState_SHUTDOWN,
}

/// Logs an entry/exit marker for a specified ID
/// for use by
/// [the Software Performance Analysis tool](https://github.com/nasa/perfutils-java).
///
/// `marker` is a system-wide ID for some portion of code.
/// `entry_exit` should be `0` for an entry into the code in question,
/// and `1` for an exit.
///
/// Wraps `CFE_ES_PerfLogAdd`.
#[doc(alias = "CFE_ES_PerfLogAdd")]
#[inline]
pub fn perf_log_add(marker: u32, entry_exit: u32) {
    unsafe { CFE_ES_PerfLogAdd(marker, entry_exit) };
}

/// Shortcut for [`perf_log_add`]`(marker, 0)`.
#[doc(alias = "CFE_ES_PerfLogEntry")]
#[inline]
pub fn perf_log_entry(marker: u32) {
    perf_log_add(marker, 0);
}

/// Shortcut for [`perf_log_add`]`(marker, 1)`.
#[doc(alias = "CFE_ES_PerfLogExit")]
#[inline]
pub fn perf_log_exit(marker: u32) {
    perf_log_add(marker, 1);
}

/// Internal macro to generate _n_-adic wrappers around `CFE_ES_WriteToSysLog`.
macro_rules! wtsl_impl {
    (@ $doc_args:expr, $name:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        #[doc = concat!(
            "Writes a message to the cFE System Log using a format string and ",
            $doc_args, ".\n",
            "\n",
            "Wraps `CFE_ES_WriteToSysLog`.\n",
        )]
        #[doc(alias = "CFE_ES_WriteToSysLog")]
        #[inline]
        pub fn $name<$($t),*>(fmt: PrintfFmt<($($t,)*)>, $($var: $t),*) -> Status
            where $($t: PrintfArgument),* {

            unsafe {
                CFE_ES_WriteToSysLog(fmt.as_ptr() $(, $var.as_c_val())*)
            }.into()
        }
    };
    ($num:expr, $name:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        wtsl_impl!(@ concat!(stringify!($num), " format arguments"),
            $name, ( $($t),* ), ( $($var),* )
        );
    };
    ($name:ident, ( $($t:ident),* ), ( $($var:ident),* )) => {
        wtsl_impl!(@ "1 format argument",
            $name, ( $($t),* ), ( $($var),* )
        );
    };
}

wtsl_impl!(0, write_to_syslog0, (), ());
#[rustfmt::skip]
wtsl_impl!(   write_to_syslog1, (A), (a));
wtsl_impl!(2, write_to_syslog2, (A, B), (a, b));
wtsl_impl!(3, write_to_syslog3, (A, B, C), (a, b, c));
wtsl_impl!(4, write_to_syslog4, (A, B, C, D), (a, b, c, d));
wtsl_impl!(5, write_to_syslog5, (A, B, C, D, E), (a, b, c, d, e));
wtsl_impl!(6, write_to_syslog6, (A, B, C, D, E, F), (a, b, c, d, e, f));
wtsl_impl!(7, write_to_syslog7, (A, B, C, D, E, F, G), (a, b, c, d, e, f, g));
wtsl_impl!(8, write_to_syslog8, (A, B, C, D, E, F, G, H), (a, b, c, d, e, f, g, h));

/// Writes the contents of a [`str`] to the cFE System Log.
///
/// Note that any embedded null characters and anything after them
/// will not get put into the log message.
///
/// Wraps `CFE_ES_WriteToSysLog`.
#[doc(alias = "CFE_ES_WriteToSysLog")]
#[inline]
pub fn write_to_syslog_str(msg: &str) -> Status {
    unsafe {
        CFE_ES_WriteToSysLog(super::RUST_STR_FMT.as_ptr(), msg.len(), msg.as_ptr() as *const c_char)
    }
    .into()
}

/// Exits from the current application.
///
/// Wraps `CFE_ES_ExitApp`.
#[doc(alias = "CFE_ES_ExitApp")]
#[inline]
pub fn exit_app(exit_status: RunStatus) -> ! {
    unsafe { CFE_ES_ExitApp(exit_status as u32) };

    // If we get here, something's gone wrong with cFE:
    unreachable!("CFE_ES_ExitApp returned, somehow");
}

/// Checks for exit requests from the cFE system
/// and possibly makes a request for app shutdown to the cFE system.
///
/// If `run_status` is set to
/// `Some(`[`AppExit`](`RunStatus::AppExit`)`)` or
/// `Some(`[`AppError`](`RunStatus::AppError`)`)`,
/// the cFE system treats the function call
/// as a shutdown request for this application.
///
/// Returns whether the app should continue running;
/// a return value of `false` means the application should
/// gracefully shut down.
///
/// Wraps `CFE_ES_RunLoop`.
#[doc(alias = "CFE_ES_RunLoop")]
#[inline]
pub fn run_loop(run_status: Option<RunStatus>) -> bool {
    let mut rs: u32 = run_status.map_or(0, |status| status as u32);
    let p: *mut u32 = match run_status {
        None => core::ptr::null_mut(),
        Some(_) => &mut rs,
    };
    unsafe { CFE_ES_RunLoop(p) }
}

/// An identifier for cFE applications.
///
/// Wraps `CFE_ES_AppId_t`.
#[doc(alias = "CFE_ES_AppId_t")]
#[derive(Clone, Copy, Debug)]
pub struct AppId {
    pub(crate) id: CFE_ES_AppId_t,
}

impl From<AppId> for ResourceId {
    #[inline]
    fn from(app_id: AppId) -> Self {
        ResourceId { id: app_id.id }
    }
}

/* TODO. Requires obtaining base resource-ID values from the cFE headers...
impl TryFrom<ResourceId> for AppId {
    type Error = ();

    #[inline]
    fn try_from(value: ResourceId) -> Result<Self, Self::Error> {
        if value.base() == CFE_ES_APPID_BASE {
            Ok(AppId { id: value.id })
        } else {
            Err(())
        }
    }
}
*/

/// Returns (if successful) the application ID for the calling cFE application.
///
/// Wraps `CFE_ES_GetAppID`.
#[doc(alias = "CFE_ES_GetAppID")]
#[inline]
pub fn get_app_id() -> Result<AppId, Status> {
    let mut app_id = AppId { id: 0 };
    let s: Status = unsafe { CFE_ES_GetAppID(&mut app_id.id) }.into();
    s.as_result(|| app_id)
}

/// Restarts a single cFE application.
///
/// Wraps `CFE_ES_RestartApp`.
#[doc(alias = "CFE_ES_RestartApp")]
#[inline]
pub fn restart_app(app_id: AppId) -> Result<(), Status> {
    let s: Status = unsafe { CFE_ES_RestartApp(app_id.id) }.into();
    s.as_result(|| ())
}

/// Stops a cFE application, then loads and starts it using the specified file.
///
/// Wraps `CFE_ES_ReloadApp`.
#[doc(alias = "CFE_ES_ReloadApp")]
#[inline]
pub fn reload_app<S: AsRef<CStr> + ?Sized>(app_id: AppId, app_file_name: &S) -> Result<(), Status> {
    let s: Status = unsafe { CFE_ES_ReloadApp(app_id.id, app_file_name.as_ref().as_ptr()) }.into();
    s.as_result(|| ())
}

/// Stops a cFE application, then deletes it from the cFE application table.
///
/// Wraps `CFE_ES_DeleteApp`.
#[doc(alias = "CFE_ES_DeleteApp")]
#[inline]
pub fn delete_app(app_id: AppId) -> Result<(), Status> {
    let s: Status = unsafe { CFE_ES_DeleteApp(app_id.id) }.into();
    s.as_result(|| ())
}

/// Waits for a minimum state of the overall cFS system,
/// or a timeout (in milliseconds), whichever comes first.
///
/// Wraps `CFE_ES_WaitForSystemState`.
#[doc(alias = "CFE_ES_WaitForSystemState")]
#[inline]
pub fn wait_for_system_state(min_system_state: SystemState, timeout_ms: u32) -> Result<(), Status> {
    let s: Status =
        unsafe { CFE_ES_WaitForSystemState(min_system_state as u32, timeout_ms) }.into();
    s.as_result(|| ())
}

/// An identifier for cFE tasks.
///
/// Wraps `CFE_ES_TaskId_t`.
#[doc(alias = "CFE_ES_TaskId_t")]
#[derive(Clone, Copy, Debug)]
pub struct TaskId {
    pub(crate) id: CFE_ES_TaskId_t,
}

impl From<TaskId> for ResourceId {
    #[inline]
    fn from(app_id: TaskId) -> Self {
        ResourceId { id: app_id.id }
    }
}

/// A task priority; used for task scheduling.
///
/// Wraps `CFE_ES_TaskPriority_Atom_t`.
#[doc(alias = "CFE_ES_TaskPriority_Atom_t")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TaskPriority {
    prio: CFE_ES_TaskPriority_Atom_t,
}

impl TaskPriority {
    /// Creates a new [`TaskPriority`] with the given numerical priority.
    #[inline]
    pub fn new(priority: u8) -> Self {
        // Per the Users Guide, only values 0-255 are allowed for the priority, hence the u8 argument.
        Self {
            prio: priority as CFE_ES_TaskPriority_Atom_t,
        }
    }

    /// Returns the numeric value of this [`TaskPriority`].
    #[inline]
    pub fn val(self) -> u8 {
        self.prio as u8
    }
}

/// Flags for task creation, as used by [`create_child_task`].
///
/// At time of writing, no flags are defined, so we only have a default constructor.
#[derive(Clone, Copy, Debug)]
pub struct TaskFlags {
    _x: PhantomData<u8>,
}

impl TaskFlags {
    /// Creates a new [`TaskFlags`] with a default set of flags.
    #[inline]
    pub fn new_empty() -> Self {
        Self { _x: PhantomData }
    }
}

impl Default for TaskFlags {
    #[inline]
    fn default() -> Self {
        Self::new_empty()
    }
}

impl From<TaskFlags> for u32 {
    #[inline]
    fn from(_: TaskFlags) -> u32 {
        0
    }
}

/// A pointer used for cross-task transfer of data
/// by [`create_child_task`] and [`task_main_func`].
static mut TASK_FUNC_PTR: *const c_void = core::ptr::null();

/// Wrapper for a Rust [`FnOnce`] to run said function in a new task.
///
/// Handles the calling of `CFE_ES_ExitChildTask` so you don't have to!
#[doc(alias = "CFE_ES_ExitChildTask")]
extern "C" fn task_main_func<F: FnOnce() + Send + Sized + 'static>() {
    use core::ptr::read_volatile;
    use core::sync::atomic;

    let copy_completed_semaphore = match child_signal_sem() {
        Ok(sem) => sem,
        Err(_) => {
            unreachable!("The semaphore should have been created already!");
        }
    };

    // Before the parent task called us, it acquired a lock to use TASK_FUNC_PTR
    // and stored a pointer to the closure there. We copy it over:
    atomic::fence(atomic::Ordering::Acquire);
    let f: F = unsafe { read_volatile(TASK_FUNC_PTR as *const F) };

    // The parent task has been blocking in order to allow us to copy over `f`.
    // Now that we've completed that, we signal for it to continue.
    let _ = copy_completed_semaphore.give();

    // And, now that all that has been completed:
    f();

    // The thread closure has finished executing, so clean up:
    unsafe {
        CFE_ES_ExitChildTask();
    }

    unreachable!("CFE_ES_ExitChildTask didn't stop a child task, somehow");
}

/// Tries to create a new child task.
/// If successful, runs `function` in the child task and returns the child task's ID.
///
/// The child task will have name `task_name`, run on a stack with `stack_size` bytes,
/// run with priority `priority`, and have task flags `flags`.
///
/// Wraps `CFE_ES_CreateChildTask` (and `CFE_ES_ExitChildTask` in the child task).
#[doc(alias("CFE_ES_CreateChildTask", "CFE_ES_ExitChildTask"))]
#[inline]
pub fn create_child_task<F: FnOnce() + Send + Sized + 'static, S: AsRef<CStr>>(
    function: F,
    task_name: &S,
    stack_size: usize,
    priority: TaskPriority,
    flags: TaskFlags,
) -> Result<TaskId, Status> {
    use core::sync::atomic;

    let mut task_id = TaskId { id: X_CFE_RESOURCEID_UNDEFINED };
    let fptr: &F = &function;

    let copy_completed_semaphore = child_signal_sem()?;

    let s = child_mutex()?
        .lock(|| {
            // OK, we have the lock. Time to write a pointer to the closure into the shared space:
            unsafe {
                TASK_FUNC_PTR = (fptr as *const F) as *const c_void;
            }
            atomic::fence(atomic::Ordering::Release);

            let s: Status = unsafe {
                CFE_ES_CreateChildTask(
                    &mut task_id.id,
                    task_name.as_ref().as_ptr(),
                    Some(task_main_func::<F>),
                    X_CFE_ES_TASK_STACK_ALLOCATE,
                    stack_size,
                    priority.prio,
                    flags.into(),
                )
            }
            .into();

            if s.severity() != super::StatusSeverity::Success {
                return s;
            }

            // Wait for the child task to finish copying the closure, then return the status:
            let _ = copy_completed_semaphore.take();
            s
        })
        .map_err(|_| Status::STATUS_EXTERNAL_RESOURCE_FAIL)?;

    s.as_result(|| ())?;
    core::mem::drop(fptr);

    if task_id.id == X_CFE_RESOURCEID_UNDEFINED {
        return Err(Status::ES_ERR_RESOURCEID_NOT_VALID);
    }

    // If (and only if) we get here, the child task was successfully created
    // and has copied over the closure. As it has been logically moved over to
    // the new thread, we do *not* want to drop it here. As such:
    core::mem::forget(function);

    Ok(task_id)
}

type AtomicOsalId = <osal_id_t as crate::utils::AtomicVersion>::Atomic;
const BASE32_SYMBOLS: &[u8; 32] = b"0123456789abcdfghjklmnpqrstvwxyz";

/// Creates an atomic variable to hold an OSAL ID for some semaphore type
/// and a wrapper function for getting a handle to said semaphore.
macro_rules! get_shared_sem {
    ($fn_name:ident, $sem_type:ty, $atomic_id:ident, $initial_iter_value:expr $( ; $constructor_arg:expr )*) => {
        static $atomic_id: AtomicOsalId = AtomicOsalId::new(X_OS_OBJECT_ID_UNDEFINED);

        fn $fn_name() -> Result<$sem_type, Status> {
            use crate::utils::CStrBuf;
            use crate::osal::MAX_NAME_LEN;
            use core::sync::atomic::Ordering::{AcqRel, Acquire};
            type Sem = $sem_type;

            // First, check to see if someone's already created the semaphore in question:
            let old_id = $atomic_id.load(Acquire);
            if old_id != X_OS_OBJECT_ID_UNDEFINED {
                return Ok(Sem { id: old_id });
            }

            // If not, create it, and write its ID to the atomic variable
            // (if someone else doesn't write an ID first, in which case, use *that* ID).

            // First off, start work on a name:
            let mut name: [c_char; MAX_NAME_LEN] = [b'\0' as c_char; MAX_NAME_LEN];
            b"n2o4-".into_iter().enumerate().for_each(|(i, val)| name[i] = *val as c_char);
            let sp = psm::stack_pointer() as usize;
            let mut num_iter: usize = $initial_iter_value;

            let sem = loop {
                // Generate a name likely to be unique:
                let now = super::time::get_time();
                let mut pseudo_hash = sp
                    .wrapping_add(now.seconds() as usize)
                    .wrapping_add(now.subseconds().rotate_right(4) as usize)
                    .wrapping_add(num_iter);

                for i in 5..(MAX_NAME_LEN - 1) {
                    name[i] = BASE32_SYMBOLS[pseudo_hash % 32] as c_char;
                    pseudo_hash /= 32;
                }

                match Sem::new(&CStrBuf::<{MAX_NAME_LEN - 1}>::new(&name) $(, $constructor_arg)*) {
                    Ok(sem) => { break sem; }
                    Err(OS_ERR_NAME_TAKEN) => (), // go around for another attempt
                    Err(_) => { return Err(Status::STATUS_EXTERNAL_RESOURCE_FAIL); }
                }

                num_iter = num_iter.wrapping_add(0x5ed3_53bb); // random, largeish odd number
            };

            Ok(match $atomic_id.compare_exchange(X_OS_OBJECT_ID_UNDEFINED, sem.id, AcqRel, Acquire) {
                Ok(_) => sem,
                Err(first_sem_id) => {
                    // Someone beat us to writing a semaphore ID.
                    // We should use that one instead:
                    let _ = sem.delete();
                    Sem { id: first_sem_id }
                }
            })
        }
    };
}

get_shared_sem!(child_mutex, crate::osal::sync::MutSem, CHILD_MUTEX_ID, 42);
get_shared_sem!(child_signal_sem, crate::osal::sync::BinSem, CHILD_SIGNAL_SEM_ID, 143; crate::osal::sync::BinSemState::Empty);

/// Tries to create a new child task. See [`create_child_task`] for details about the arguments.
///
/// This is a little faster than [`create_child_task`] and uses less resources,
/// but unlike [`create_child_task`], this does not accept Rust-style closures as values of `function`.
///
/// `function` should call `CFE_ES_ExitChildTask` (or [`exit_child_task`] if written in Rust)
/// at the end of its execution.
///
/// Wraps `CFE_ES_CreateChildTask`.
#[doc(alias = "CFE_ES_CreateChildTask")]
#[inline]
pub fn create_child_task_c<S: AsRef<CStr>>(
    function: unsafe extern "C" fn(),
    task_name: &S,
    stack_size: usize,
    priority: TaskPriority,
    flags: TaskFlags,
) -> Result<TaskId, Status> {
    let mut task_id = TaskId { id: X_CFE_RESOURCEID_UNDEFINED };

    let s: Status = unsafe {
        CFE_ES_CreateChildTask(
            &mut task_id.id,
            task_name.as_ref().as_ptr(),
            Some(function),
            X_CFE_ES_TASK_STACK_ALLOCATE,
            stack_size,
            priority.prio,
            flags.into(),
        )
    }
    .into();

    match s.as_result(|| task_id) {
        Ok(task) => match task.id {
            X_CFE_RESOURCEID_UNDEFINED => Err(Status::ES_ERR_RESOURCEID_NOT_VALID),
            _ => Ok(task),
        },
        Err(e) => Err(e),
    }
}

/// When called from a child task, causes the child task to exit and be deleted by cFE.
///
/// Unless an error occurs, this does not return.
///
/// Tasks created by [`create_child_task`] already call this automatically at the end
/// of their execution, so functions passed to [`create_child_task`] do not need to
/// manually call this function.
///
/// Wraps `CFE_ES_ExitChildTask`.
#[doc(alias = "CFE_ES_ExitChildTask")]
#[inline]
pub fn exit_child_task() -> Result<crate::utils::Unconstructable, Status> {
    unsafe {
        CFE_ES_ExitChildTask();
    }

    Err(Status::ES_BAD_ARGUMENT)
}
