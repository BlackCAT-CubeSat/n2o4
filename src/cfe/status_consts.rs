// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

//! Status constants used by cFE.

use super::Status;
use cfs_sys::*;

const fn mk_status(n: CFE_Status_t) -> Status {
    Status { status: n }
}

impl Status {
    pub const SUCCESS: Status = mk_status(S_CFE_SUCCESS);
    pub const STATUS_NO_COUNTER_INCREMENT: Status = mk_status(S_CFE_STATUS_NO_COUNTER_INCREMENT);
    pub const STATUS_WRONG_MSG_LENGTH: Status = mk_status(S_CFE_STATUS_WRONG_MSG_LENGTH);
    pub const STATUS_UNKNOWN_MSG_ID: Status = mk_status(S_CFE_STATUS_UNKNOWN_MSG_ID);
    pub const STATUS_BAD_COMMAND_CODE: Status = mk_status(S_CFE_STATUS_BAD_COMMAND_CODE);
    pub const STATUS_EXTERNAL_RESOURCE_FAIL: Status =
        mk_status(S_CFE_STATUS_EXTERNAL_RESOURCE_FAIL);
    pub const STATUS_REQUEST_ALREADY_PENDING: Status =
        mk_status(S_CFE_STATUS_REQUEST_ALREADY_PENDING);
    pub const STATUS_NOT_IMPLEMENTED: Status = mk_status(S_CFE_STATUS_NOT_IMPLEMENTED);
    pub const EVS_UNKNOWN_FILTER: Status = mk_status(S_CFE_EVS_UNKNOWN_FILTER);
    pub const EVS_APP_NOT_REGISTERED: Status = mk_status(S_CFE_EVS_APP_NOT_REGISTERED);
    pub const EVS_APP_ILLEGAL_APP_ID: Status = mk_status(S_CFE_EVS_APP_ILLEGAL_APP_ID);
    pub const EVS_APP_FILTER_OVERLOAD: Status = mk_status(S_CFE_EVS_APP_FILTER_OVERLOAD);
    pub const EVS_RESET_AREA_POINTER: Status = mk_status(S_CFE_EVS_RESET_AREA_POINTER);
    pub const EVS_EVT_NOT_REGISTERED: Status = mk_status(S_CFE_EVS_EVT_NOT_REGISTERED);
    pub const EVS_FILE_WRITE_ERROR: Status = mk_status(S_CFE_EVS_FILE_WRITE_ERROR);
    pub const EVS_INVALID_PARAMETER: Status = mk_status(S_CFE_EVS_INVALID_PARAMETER);
    pub const EVS_NOT_IMPLEMENTED: Status = mk_status(S_CFE_EVS_NOT_IMPLEMENTED);
    pub const ES_ERR_RESOURCEID_NOT_VALID: Status = mk_status(S_CFE_ES_ERR_RESOURCEID_NOT_VALID);
    pub const ES_ERR_NAME_NOT_FOUND: Status = mk_status(S_CFE_ES_ERR_NAME_NOT_FOUND);
    pub const ES_ERR_APP_CREATE: Status = mk_status(S_CFE_ES_ERR_APP_CREATE);
    pub const ES_ERR_CHILD_TASK_CREATE: Status = mk_status(S_CFE_ES_ERR_CHILD_TASK_CREATE);
    pub const ES_ERR_SYS_LOG_FULL: Status = mk_status(S_CFE_ES_ERR_SYS_LOG_FULL);
    pub const ES_ERR_MEM_BLOCK_SIZE: Status = mk_status(S_CFE_ES_ERR_MEM_BLOCK_SIZE);
    pub const ES_ERR_LOAD_LIB: Status = mk_status(S_CFE_ES_ERR_LOAD_LIB);
    pub const ES_BAD_ARGUMENT: Status = mk_status(S_CFE_ES_BAD_ARGUMENT);
    pub const ES_ERR_CHILD_TASK_REGISTER: Status = mk_status(S_CFE_ES_ERR_CHILD_TASK_REGISTER);
    pub const ES_CDS_ALREADY_EXISTS: Status = mk_status(S_CFE_ES_CDS_ALREADY_EXISTS);
    pub const ES_CDS_INSUFFICIENT_MEMORY: Status = mk_status(S_CFE_ES_CDS_INSUFFICIENT_MEMORY);
    pub const ES_CDS_INVALID_NAME: Status = mk_status(S_CFE_ES_CDS_INVALID_NAME);
    pub const ES_CDS_INVALID_SIZE: Status = mk_status(S_CFE_ES_CDS_INVALID_SIZE);
    pub const ES_CDS_INVALID: Status = mk_status(S_CFE_ES_CDS_INVALID);
    pub const ES_CDS_ACCESS_ERROR: Status = mk_status(S_CFE_ES_CDS_ACCESS_ERROR);
    pub const ES_FILE_IO_ERR: Status = mk_status(S_CFE_ES_FILE_IO_ERR);
    pub const ES_RST_ACCESS_ERR: Status = mk_status(S_CFE_ES_RST_ACCESS_ERR);
    pub const ES_ERR_APP_REGISTER: Status = mk_status(S_CFE_ES_ERR_APP_REGISTER);
    pub const ES_ERR_CHILD_TASK_DELETE: Status = mk_status(S_CFE_ES_ERR_CHILD_TASK_DELETE);
    pub const ES_ERR_CHILD_TASK_DELETE_MAIN_TASK: Status =
        mk_status(S_CFE_ES_ERR_CHILD_TASK_DELETE_MAIN_TASK);
    pub const ES_CDS_BLOCK_CRC_ERR: Status = mk_status(S_CFE_ES_CDS_BLOCK_CRC_ERR);
    pub const ES_MUT_SEM_DELETE_ERR: Status = mk_status(S_CFE_ES_MUT_SEM_DELETE_ERR);
    pub const ES_BIN_SEM_DELETE_ERR: Status = mk_status(S_CFE_ES_BIN_SEM_DELETE_ERR);
    pub const ES_COUNT_SEM_DELETE_ERR: Status = mk_status(S_CFE_ES_COUNT_SEM_DELETE_ERR);
    pub const ES_QUEUE_DELETE_ERR: Status = mk_status(S_CFE_ES_QUEUE_DELETE_ERR);
    pub const ES_FILE_CLOSE_ERR: Status = mk_status(S_CFE_ES_FILE_CLOSE_ERR);
    pub const ES_CDS_WRONG_TYPE_ERR: Status = mk_status(S_CFE_ES_CDS_WRONG_TYPE_ERR);
    pub const ES_CDS_OWNER_ACTIVE_ERR: Status = mk_status(S_CFE_ES_CDS_OWNER_ACTIVE_ERR);
    pub const ES_APP_CLEANUP_ERR: Status = mk_status(S_CFE_ES_APP_CLEANUP_ERR);
    pub const ES_TIMER_DELETE_ERR: Status = mk_status(S_CFE_ES_TIMER_DELETE_ERR);
    pub const ES_BUFFER_NOT_IN_POOL: Status = mk_status(S_CFE_ES_BUFFER_NOT_IN_POOL);
    pub const ES_TASK_DELETE_ERR: Status = mk_status(S_CFE_ES_TASK_DELETE_ERR);
    pub const ES_OPERATION_TIMED_OUT: Status = mk_status(S_CFE_ES_OPERATION_TIMED_OUT);
    pub const ES_LIB_ALREADY_LOADED: Status = mk_status(S_CFE_ES_LIB_ALREADY_LOADED);
    pub const ES_ERR_SYS_LOG_TRUNCATED: Status = mk_status(S_CFE_ES_ERR_SYS_LOG_TRUNCATED);
    pub const ES_NO_RESOURCE_IDS_AVAILABLE: Status = mk_status(S_CFE_ES_NO_RESOURCE_IDS_AVAILABLE);
    pub const ES_POOL_BLOCK_INVALID: Status = mk_status(S_CFE_ES_POOL_BLOCK_INVALID);
    pub const ES_ERR_DUPLICATE_NAME: Status = mk_status(S_CFE_ES_ERR_DUPLICATE_NAME);
    pub const ES_NOT_IMPLEMENTED: Status = mk_status(S_CFE_ES_NOT_IMPLEMENTED);
    pub const FS_BAD_ARGUMENT: Status = mk_status(S_CFE_FS_BAD_ARGUMENT);
    pub const FS_INVALID_PATH: Status = mk_status(S_CFE_FS_INVALID_PATH);
    pub const FS_FNAME_TOO_LONG: Status = mk_status(S_CFE_FS_FNAME_TOO_LONG);
    pub const FS_NOT_IMPLEMENTED: Status = mk_status(S_CFE_FS_NOT_IMPLEMENTED);
    pub const MSG_WRONG_MSG_TYPE: Status = mk_status(S_CFE_MSG_WRONG_MSG_TYPE);
    pub const SB_TIME_OUT: Status = mk_status(S_CFE_SB_TIME_OUT);
    pub const SB_NO_MESSAGE: Status = mk_status(S_CFE_SB_NO_MESSAGE);
    pub const SB_BAD_ARGUMENT: Status = mk_status(S_CFE_SB_BAD_ARGUMENT);
    pub const SB_MAX_PIPES_MET: Status = mk_status(S_CFE_SB_MAX_PIPES_MET);
    pub const SB_PIPE_CR_ERR: Status = mk_status(S_CFE_SB_PIPE_CR_ERR);
    pub const SB_PIPE_RD_ERR: Status = mk_status(S_CFE_SB_PIPE_RD_ERR);
    pub const SB_MSG_TOO_BIG: Status = mk_status(S_CFE_SB_MSG_TOO_BIG);
    pub const SB_BUF_ALOC_ERR: Status = mk_status(S_CFE_SB_BUF_ALOC_ERR);
    pub const SB_MAX_MSGS_MET: Status = mk_status(S_CFE_SB_MAX_MSGS_MET);
    pub const SB_MAX_DESTS_MET: Status = mk_status(S_CFE_SB_MAX_DESTS_MET);
    pub const SB_INTERNAL_ERR: Status = mk_status(S_CFE_SB_INTERNAL_ERR);
    pub const SB_WRONG_MSG_TYPE: Status = mk_status(S_CFE_SB_WRONG_MSG_TYPE);
    pub const SB_BUFFER_INVALID: Status = mk_status(S_CFE_SB_BUFFER_INVALID);
    pub const SB_NOT_IMPLEMENTED: Status = mk_status(S_CFE_SB_NOT_IMPLEMENTED);
    pub const TBL_ERR_INVALID_HANDLE: Status = mk_status(S_CFE_TBL_ERR_INVALID_HANDLE);
    pub const TBL_ERR_INVALID_NAME: Status = mk_status(S_CFE_TBL_ERR_INVALID_NAME);
    pub const TBL_ERR_INVALID_SIZE: Status = mk_status(S_CFE_TBL_ERR_INVALID_SIZE);
    pub const TBL_INFO_UPDATE_PENDING: Status = mk_status(S_CFE_TBL_INFO_UPDATE_PENDING);
    pub const TBL_ERR_NEVER_LOADED: Status = mk_status(S_CFE_TBL_ERR_NEVER_LOADED);
    pub const TBL_ERR_REGISTRY_FULL: Status = mk_status(S_CFE_TBL_ERR_REGISTRY_FULL);
    pub const TBL_WARN_DUPLICATE: Status = mk_status(S_CFE_TBL_WARN_DUPLICATE);
    pub const TBL_ERR_NO_ACCESS: Status = mk_status(S_CFE_TBL_ERR_NO_ACCESS);
    pub const TBL_ERR_UNREGISTERED: Status = mk_status(S_CFE_TBL_ERR_UNREGISTERED);
    pub const TBL_ERR_HANDLES_FULL: Status = mk_status(S_CFE_TBL_ERR_HANDLES_FULL);
    pub const TBL_ERR_DUPLICATE_DIFF_SIZE: Status = mk_status(S_CFE_TBL_ERR_DUPLICATE_DIFF_SIZE);
    pub const TBL_ERR_DUPLICATE_NOT_OWNED: Status = mk_status(S_CFE_TBL_ERR_DUPLICATE_NOT_OWNED);
    pub const TBL_INFO_UPDATED: Status = mk_status(S_CFE_TBL_INFO_UPDATED);
    pub const TBL_ERR_NO_BUFFER_AVAIL: Status = mk_status(S_CFE_TBL_ERR_NO_BUFFER_AVAIL);
    pub const TBL_ERR_DUMP_ONLY: Status = mk_status(S_CFE_TBL_ERR_DUMP_ONLY);
    pub const TBL_ERR_ILLEGAL_SRC_TYPE: Status = mk_status(S_CFE_TBL_ERR_ILLEGAL_SRC_TYPE);
    pub const TBL_ERR_LOAD_IN_PROGRESS: Status = mk_status(S_CFE_TBL_ERR_LOAD_IN_PROGRESS);
    pub const TBL_ERR_FILE_TOO_LARGE: Status = mk_status(S_CFE_TBL_ERR_FILE_TOO_LARGE);
    pub const TBL_WARN_SHORT_FILE: Status = mk_status(S_CFE_TBL_WARN_SHORT_FILE);
    pub const TBL_ERR_BAD_CONTENT_ID: Status = mk_status(S_CFE_TBL_ERR_BAD_CONTENT_ID);
    pub const TBL_INFO_NO_UPDATE_PENDING: Status = mk_status(S_CFE_TBL_INFO_NO_UPDATE_PENDING);
    pub const TBL_INFO_TABLE_LOCKED: Status = mk_status(S_CFE_TBL_INFO_TABLE_LOCKED);
    pub const TBL_INFO_VALIDATION_PENDING: Status = mk_status(S_CFE_TBL_INFO_VALIDATION_PENDING);
    pub const TBL_INFO_NO_VALIDATION_PENDING: Status =
        mk_status(S_CFE_TBL_INFO_NO_VALIDATION_PENDING);
    pub const TBL_ERR_BAD_SUBTYPE_ID: Status = mk_status(S_CFE_TBL_ERR_BAD_SUBTYPE_ID);
    pub const TBL_ERR_FILE_SIZE_INCONSISTENT: Status =
        mk_status(S_CFE_TBL_ERR_FILE_SIZE_INCONSISTENT);
    pub const TBL_ERR_NO_STD_HEADER: Status = mk_status(S_CFE_TBL_ERR_NO_STD_HEADER);
    pub const TBL_ERR_NO_TBL_HEADER: Status = mk_status(S_CFE_TBL_ERR_NO_TBL_HEADER);
    pub const TBL_ERR_FILENAME_TOO_LONG: Status = mk_status(S_CFE_TBL_ERR_FILENAME_TOO_LONG);
    pub const TBL_ERR_FILE_FOR_WRONG_TABLE: Status = mk_status(S_CFE_TBL_ERR_FILE_FOR_WRONG_TABLE);
    pub const TBL_ERR_LOAD_INCOMPLETE: Status = mk_status(S_CFE_TBL_ERR_LOAD_INCOMPLETE);
    pub const TBL_WARN_PARTIAL_LOAD: Status = mk_status(S_CFE_TBL_WARN_PARTIAL_LOAD);
    pub const TBL_ERR_PARTIAL_LOAD: Status = mk_status(S_CFE_TBL_ERR_PARTIAL_LOAD);
    pub const TBL_INFO_DUMP_PENDING: Status = mk_status(S_CFE_TBL_INFO_DUMP_PENDING);
    pub const TBL_ERR_INVALID_OPTIONS: Status = mk_status(S_CFE_TBL_ERR_INVALID_OPTIONS);
    pub const TBL_WARN_NOT_CRITICAL: Status = mk_status(S_CFE_TBL_WARN_NOT_CRITICAL);
    pub const TBL_INFO_RECOVERED_TBL: Status = mk_status(S_CFE_TBL_INFO_RECOVERED_TBL);
    pub const TBL_ERR_BAD_SPACECRAFT_ID: Status = mk_status(S_CFE_TBL_ERR_BAD_SPACECRAFT_ID);
    pub const TBL_ERR_BAD_PROCESSOR_ID: Status = mk_status(S_CFE_TBL_ERR_BAD_PROCESSOR_ID);
    pub const TBL_MESSAGE_ERROR: Status = mk_status(S_CFE_TBL_MESSAGE_ERROR);
    pub const TBL_ERR_SHORT_FILE: Status = mk_status(S_CFE_TBL_ERR_SHORT_FILE);
    pub const TBL_ERR_ACCESS: Status = mk_status(S_CFE_TBL_ERR_ACCESS);
    pub const TBL_BAD_ARGUMENT: Status = mk_status(S_CFE_TBL_BAD_ARGUMENT);
    pub const TBL_NOT_IMPLEMENTED: Status = mk_status(S_CFE_TBL_NOT_IMPLEMENTED);
    pub const TIME_NOT_IMPLEMENTED: Status = mk_status(S_CFE_TIME_NOT_IMPLEMENTED);
    pub const TIME_INTERNAL_ONLY: Status = mk_status(S_CFE_TIME_INTERNAL_ONLY);
    pub const TIME_OUT_OF_RANGE: Status = mk_status(S_CFE_TIME_OUT_OF_RANGE);
    pub const TIME_TOO_MANY_SYNCH_CALLBACKS: Status =
        mk_status(S_CFE_TIME_TOO_MANY_SYNCH_CALLBACKS);
    pub const TIME_CALLBACK_NOT_REGISTERED: Status = mk_status(S_CFE_TIME_CALLBACK_NOT_REGISTERED);
    pub const TIME_BAD_ARGUMENT: Status = mk_status(S_CFE_TIME_BAD_ARGUMENT);
}
