// Copyright (c) 2023 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Task-related APIs.

use cfs_sys::*;
use core::ffi::CStr;

use super::*;
use crate::utils::CStrBuf;

/// An identifier for an OSAL task.
///
/// Wraps `osal_id_t`.
#[doc(alias = "osal_id_t")]
#[derive(Clone, Debug)]
pub struct Task {
    id: osal_id_t,
}

impl Task {
    /// Tries to find an OSAL task given its name.
    ///
    /// Wraps `OS_TaskGetIdByName`.
    #[doc(alias = "OS_TaskGetIdByName")]
    #[inline]
    pub fn by_name<S: AsRef<CStr>>(task_name: &S) -> Result<Self, i32> {
        let task_name = task_name.as_ref().as_ptr();
        let mut id: osal_id_t = X_OS_OBJECT_ID_UNDEFINED;

        let result = unsafe { OS_TaskGetIdByName(&mut id, task_name) };

        if result >= 0 && (ObjectId { id }).obj_type() == OS_OBJECT_TYPE_OS_TASK {
            Ok(Self { id })
        } else {
            Err(result)
        }
    }

    /// Returns information about the task.
    ///
    /// Wraps `OS_TaskGetInfo`.
    #[doc(alias = "OS_TaskGetInfo")]
    #[inline]
    pub fn info(&self) -> Result<TaskProperties, i32> {
        let mut props = OS_task_prop_t {
            name:       [0; { OS_MAX_API_NAME as usize }],
            creator:    0,
            stack_size: 0,
            priority:   0,
        };

        let result = unsafe { OS_TaskGetInfo(self.id, &mut props) };

        if result >= 0 {
            Ok(TaskProperties {
                name:       CStrBuf::new_into(props.name),
                stack_size: props.stack_size,
                priority:   props.priority,
                creator:    ObjectId { id: props.creator },
            })
        } else {
            Err(result)
        }
    }

    /// Sets the priority of the task.
    ///
    /// Wraps `OS_TaskSetPriority`.
    #[doc(alias = "OS_TaskSetPriority")]
    #[inline]
    pub fn set_priority(&self, new_priority: TaskPriority) -> Result<(), i32> {
        let result = unsafe { OS_TaskSetPriority(self.id, new_priority) };

        if result < 0 {
            Err(result)
        } else {
            Ok(())
        }
    }

    /// Deletes the task.
    ///
    /// Wraps `OS_TaskDelete`.
    #[doc(alias = "OS_TaskDelete")]
    #[inline]
    pub fn delete(self) -> Result<(), i32> {
        let result = unsafe { OS_TaskDelete(self.id) };

        if result >= 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    /// Returns the [`ObjectId`] for the task.
    #[inline]
    pub fn as_id(&self) -> ObjectId {
        ObjectId { id: self.id }
    }
}

/// Converts an `ObjectId` to a `Task` if the object ID represents a task.
impl TryFrom<ObjectId> for Task {
    type Error = ObjectTypeConvertError;

    #[inline]
    fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
        if value.obj_type() == OS_OBJECT_TYPE_OS_TASK {
            Ok(Task { id: value.id })
        } else {
            Err(ObjectTypeConvertError {})
        }
    }
}

/// An OSAL task priority.
///
/// This is in reverse numeric order, so 0 is the highest priority
/// and 255 the lowest.
///
/// This is the same as `osal_priority_t`.
#[doc(alias = "osal_priority_t")]
#[doc(inline)]
pub use cfs_sys::osal_priority_t as TaskPriority;

/// Information about an OSAL task.
///
/// Corresponds to `OS_task_prop_t`.
#[doc(alias = "OS_task_prop_t")]
#[derive(Clone, Copy, Debug)]
pub struct TaskProperties {
    /// The task's name.
    pub name: CStrBuf<{ OS_MAX_API_NAME as usize }>,

    /// The size of the task's stack.
    pub stack_size: usize,

    /// The task's priority.
    pub priority: TaskPriority,

    /// The task's creator.
    pub creator: ObjectId,
}

/// Returns the task ID for the current task if successful.
///
/// Wraps `OS_TaskGetId`.
#[doc(alias = "OS_TaskGetId")]
#[inline]
pub fn get_id() -> Result<Task, ()> {
    let task_id = unsafe { OS_TaskGetId() };

    if task_id != 0 && (ObjectId { id: task_id }).obj_type() == OS_OBJECT_TYPE_OS_TASK {
        Ok(Task { id: task_id })
    } else {
        Err(())
    }
}

/// Exits the current task.
///
/// Does not return, so Rust objects owned by this thread's stack
/// won't get dropped.
///
/// Wraps `OS_TaskExit`.
#[doc(alias = "OS_TaskExit")]
#[inline]
pub fn exit() -> ! {
    unsafe {
        OS_TaskExit();
    }

    // we should never get here, but if we do:
    panic!("OS_TaskExit returned, somehow");
}

/// Stops execution of this task for `millis` milliseconds.
///
/// Wraps `OS_TaskDelay`.
#[doc(alias = "OS_TaskDelay")]
#[inline]
pub fn delay(millis: u32) -> Result<(), i32> {
    let result = unsafe { OS_TaskDelay(millis) };

    if result < 0 {
        Err(result)
    } else {
        Ok(())
    }
}
