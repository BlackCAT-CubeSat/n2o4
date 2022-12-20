/* Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
 * SPDX-License-Identifier: Apache-2.0
 */

/* The include files for cFE's core API: */
#include <cfe.h>
#include <cfe_version.h>
#include <cfe_tbl_filedef.h>

/* For the components' messages and events: */
#include <cfe_es_events.h>
#include <cfe_es_msg.h>

#include <cfe_evs_events.h>
#include <cfe_evs_msg.h>

#include <cfe_sb_events.h>
#include <cfe_sb_msg.h>

#include <cfe_tbl_events.h>
#include <cfe_tbl_msg.h>

#include <cfe_time_events.h>
#include <cfe_time_msg.h>

/* For OSAL: */
#include <osapi.h>

/* For PSP: */
#include <cfe_psp.h>


/* The configuration for our target platform: */
#include <cfe_mission_cfg.h>
#include <cfe_platform_cfg.h>
#include <cfe_msgids.h>
#include <cfe_perfids.h>


/* Bindgen sometimes isn't able to determine that a #define is actually */
/* a constant, so we need to help it along... */
#define S(IDENT) const CFE_Status_t S_ ## IDENT = IDENT;

S(CFE_SEVERITY_BITMASK)
S(CFE_SEVERITY_SUCCESS)
S(CFE_SEVERITY_INFO)
S(CFE_SEVERITY_ERROR)
S(CFE_SERVICE_BITMASK)
S(CFE_EVENTS_SERVICE)
S(CFE_EXECUTIVE_SERVICE)
S(CFE_FILE_SERVICE)
S(CFE_GENERIC_SERVICE)
S(CFE_SOFTWARE_BUS_SERVICE)
S(CFE_TABLE_SERVICE)
S(CFE_TIME_SERVICE)
S(CFE_SUCCESS)
S(CFE_STATUS_NO_COUNTER_INCREMENT)
S(CFE_STATUS_WRONG_MSG_LENGTH)
S(CFE_STATUS_UNKNOWN_MSG_ID)
S(CFE_STATUS_BAD_COMMAND_CODE)
S(CFE_STATUS_EXTERNAL_RESOURCE_FAIL)
S(CFE_STATUS_REQUEST_ALREADY_PENDING)
S(CFE_STATUS_NOT_IMPLEMENTED)
S(CFE_EVS_UNKNOWN_FILTER)
S(CFE_EVS_APP_NOT_REGISTERED)
S(CFE_EVS_APP_ILLEGAL_APP_ID)
S(CFE_EVS_APP_FILTER_OVERLOAD)
S(CFE_EVS_RESET_AREA_POINTER)
S(CFE_EVS_EVT_NOT_REGISTERED)
S(CFE_EVS_FILE_WRITE_ERROR)
S(CFE_EVS_INVALID_PARAMETER)
S(CFE_EVS_NOT_IMPLEMENTED)
S(CFE_ES_ERR_RESOURCEID_NOT_VALID)
S(CFE_ES_ERR_NAME_NOT_FOUND)
S(CFE_ES_ERR_APP_CREATE)
S(CFE_ES_ERR_CHILD_TASK_CREATE)
S(CFE_ES_ERR_SYS_LOG_FULL)
S(CFE_ES_ERR_MEM_BLOCK_SIZE)
S(CFE_ES_ERR_LOAD_LIB)
S(CFE_ES_BAD_ARGUMENT)
S(CFE_ES_ERR_CHILD_TASK_REGISTER)
S(CFE_ES_CDS_ALREADY_EXISTS)
S(CFE_ES_CDS_INSUFFICIENT_MEMORY)
S(CFE_ES_CDS_INVALID_NAME)
S(CFE_ES_CDS_INVALID_SIZE)
S(CFE_ES_CDS_INVALID)
S(CFE_ES_CDS_ACCESS_ERROR)
S(CFE_ES_FILE_IO_ERR)
S(CFE_ES_RST_ACCESS_ERR)
S(CFE_ES_ERR_APP_REGISTER)
S(CFE_ES_ERR_CHILD_TASK_DELETE)
S(CFE_ES_ERR_CHILD_TASK_DELETE_MAIN_TASK)
S(CFE_ES_CDS_BLOCK_CRC_ERR)
S(CFE_ES_MUT_SEM_DELETE_ERR)
S(CFE_ES_BIN_SEM_DELETE_ERR)
S(CFE_ES_COUNT_SEM_DELETE_ERR)
S(CFE_ES_QUEUE_DELETE_ERR)
S(CFE_ES_FILE_CLOSE_ERR)
S(CFE_ES_CDS_WRONG_TYPE_ERR)
S(CFE_ES_CDS_OWNER_ACTIVE_ERR)
S(CFE_ES_APP_CLEANUP_ERR)
S(CFE_ES_TIMER_DELETE_ERR)
S(CFE_ES_BUFFER_NOT_IN_POOL)
S(CFE_ES_TASK_DELETE_ERR)
S(CFE_ES_OPERATION_TIMED_OUT)
S(CFE_ES_LIB_ALREADY_LOADED)
S(CFE_ES_ERR_SYS_LOG_TRUNCATED)
S(CFE_ES_NO_RESOURCE_IDS_AVAILABLE)
S(CFE_ES_POOL_BLOCK_INVALID)
S(CFE_ES_ERR_DUPLICATE_NAME)
S(CFE_ES_NOT_IMPLEMENTED)
S(CFE_FS_BAD_ARGUMENT)
S(CFE_FS_INVALID_PATH)
S(CFE_FS_FNAME_TOO_LONG)
S(CFE_FS_NOT_IMPLEMENTED)
S(CFE_MSG_WRONG_MSG_TYPE)
S(CFE_SB_TIME_OUT)
S(CFE_SB_NO_MESSAGE)
S(CFE_SB_BAD_ARGUMENT)
S(CFE_SB_MAX_PIPES_MET)
S(CFE_SB_PIPE_CR_ERR)
S(CFE_SB_PIPE_RD_ERR)
S(CFE_SB_MSG_TOO_BIG)
S(CFE_SB_BUF_ALOC_ERR)
S(CFE_SB_MAX_MSGS_MET)
S(CFE_SB_MAX_DESTS_MET)
S(CFE_SB_INTERNAL_ERR)
S(CFE_SB_WRONG_MSG_TYPE)
S(CFE_SB_BUFFER_INVALID)
S(CFE_SB_NOT_IMPLEMENTED)
S(CFE_TBL_ERR_INVALID_HANDLE)
S(CFE_TBL_ERR_INVALID_NAME)
S(CFE_TBL_ERR_INVALID_SIZE)
S(CFE_TBL_INFO_UPDATE_PENDING)
S(CFE_TBL_ERR_NEVER_LOADED)
S(CFE_TBL_ERR_REGISTRY_FULL)
S(CFE_TBL_WARN_DUPLICATE)
S(CFE_TBL_ERR_NO_ACCESS)
S(CFE_TBL_ERR_UNREGISTERED)
S(CFE_TBL_ERR_HANDLES_FULL)
S(CFE_TBL_ERR_DUPLICATE_DIFF_SIZE)
S(CFE_TBL_ERR_DUPLICATE_NOT_OWNED)
S(CFE_TBL_INFO_UPDATED)
S(CFE_TBL_ERR_NO_BUFFER_AVAIL)
S(CFE_TBL_ERR_DUMP_ONLY)
S(CFE_TBL_ERR_ILLEGAL_SRC_TYPE)
S(CFE_TBL_ERR_LOAD_IN_PROGRESS)
S(CFE_TBL_ERR_FILE_TOO_LARGE)
S(CFE_TBL_WARN_SHORT_FILE)
S(CFE_TBL_ERR_BAD_CONTENT_ID)
S(CFE_TBL_INFO_NO_UPDATE_PENDING)
S(CFE_TBL_INFO_TABLE_LOCKED)
S(CFE_TBL_INFO_VALIDATION_PENDING)
S(CFE_TBL_INFO_NO_VALIDATION_PENDING)
S(CFE_TBL_ERR_BAD_SUBTYPE_ID)
S(CFE_TBL_ERR_FILE_SIZE_INCONSISTENT)
S(CFE_TBL_ERR_NO_STD_HEADER)
S(CFE_TBL_ERR_NO_TBL_HEADER)
S(CFE_TBL_ERR_FILENAME_TOO_LONG)
S(CFE_TBL_ERR_FILE_FOR_WRONG_TABLE)
S(CFE_TBL_ERR_LOAD_INCOMPLETE)
S(CFE_TBL_WARN_PARTIAL_LOAD)
S(CFE_TBL_ERR_PARTIAL_LOAD)
S(CFE_TBL_INFO_DUMP_PENDING)
S(CFE_TBL_ERR_INVALID_OPTIONS)
S(CFE_TBL_WARN_NOT_CRITICAL)
S(CFE_TBL_INFO_RECOVERED_TBL)
S(CFE_TBL_ERR_BAD_SPACECRAFT_ID)
S(CFE_TBL_ERR_BAD_PROCESSOR_ID)
S(CFE_TBL_MESSAGE_ERROR)
S(CFE_TBL_ERR_SHORT_FILE)
S(CFE_TBL_ERR_ACCESS)
S(CFE_TBL_BAD_ARGUMENT)
S(CFE_TBL_NOT_IMPLEMENTED)
S(CFE_TIME_NOT_IMPLEMENTED)
S(CFE_TIME_INTERNAL_ONLY)
S(CFE_TIME_OUT_OF_RANGE)
S(CFE_TIME_TOO_MANY_SYNCH_CALLBACKS)
S(CFE_TIME_CALLBACK_NOT_REGISTERED)
S(CFE_TIME_BAD_ARGUMENT)

#define X(IDENT, TYPE) const TYPE X_ ## IDENT = IDENT;

X(CFE_ES_TASK_STACK_ALLOCATE, CFE_ES_StackPointer_t)
X(CFE_RESOURCEID_RESERVED, CFE_ResourceId_t)
X(CFE_RESOURCEID_UNDEFINED, CFE_ResourceId_t)
X(CFE_SB_MSGID_RESERVED, CFE_SB_MsgId_t)
X(CFE_SB_INVALID_MSG_ID, CFE_SB_MsgId_t)
X(CFE_TBL_BAD_TABLE_HANDLE, CFE_TBL_Handle_t)
X(OS_OBJECT_ID_UNDEFINED, osal_id_t)

const uint8 X_CFE_SB_DEFAULT_QOS_PRIORITY = CFE_SB_DEFAULT_QOS.Priority;
const uint8 X_CFE_SB_DEFAULT_QOS_RELIABILITY = CFE_SB_DEFAULT_QOS.Reliability;
