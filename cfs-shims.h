/* Copyright (c) 2021 The Pennsylvania State University. All rights reserved.
 */

/* Shim functions for cFE and OSAL functionality that is implemented by
 * C inline functions, for said functionality's use in Rust.
 */

#include <cfe.h>
#include <osapi.h>

unsigned long SHIM_CFE_ResourceId_ToInteger(CFE_ResourceId_t id);
CFE_ResourceId_t SHIM_CFE_ResourceId_FromInteger(unsigned long Value);
bool SHIM_CFE_ResourceId_Equal(CFE_ResourceId_t id1, CFE_ResourceId_t id2);
bool SHIM_CFE_ResourceId_IsDefined(CFE_ResourceId_t id);
bool SHIM_CFE_SB_MsgId_Equal(CFE_SB_MsgId_t MsgId1, CFE_SB_MsgId_t MsgId2);
CFE_SB_MsgId_Atom_t SHIM_CFE_SB_MsgIdToValue(CFE_SB_MsgId_t MsgId);
CFE_SB_MsgId_t SHIM_CFE_SB_ValueToMsgId(CFE_SB_MsgId_Atom_t MsgIdValue);
int64 SHIM_OS_TimeGetTotalSeconds(OS_time_t tm);
int64 SHIM_OS_TimeGetTotalMilliseconds(OS_time_t tm);
int64 SHIM_OS_TimeGetTotalMicroseconds(OS_time_t tm);
int64 SHIM_OS_TimeGetTotalNanoseconds(OS_time_t tm);
int64 SHIM_OS_TimeGetFractionalPart(OS_time_t tm);
uint32 SHIM_OS_TimeGetSubsecondsPart(OS_time_t tm);
uint32 SHIM_OS_TimeGetMillisecondsPart(OS_time_t tm);
uint32 SHIM_OS_TimeGetMicrosecondsPart(OS_time_t tm);
uint32 SHIM_OS_TimeGetNanosecondsPart(OS_time_t tm);
OS_time_t SHIM_OS_TimeAssembleFromNanoseconds(int64 seconds, uint32 nanoseconds);
OS_time_t SHIM_OS_TimeAssembleFromMicroseconds(int64 seconds, uint32 microseconds);
OS_time_t SHIM_OS_TimeAssembleFromMilliseconds(int64 seconds, uint32 milliseconds);
OS_time_t SHIM_OS_TimeAssembleFromSubseconds(int64 seconds, uint32 subseconds);
OS_time_t SHIM_OS_TimeAdd(OS_time_t time1, OS_time_t time2);
OS_time_t SHIM_OS_TimeSubtract(OS_time_t time1, OS_time_t time2);
unsigned long SHIM_OS_ObjectIdToInteger(osal_id_t object_id);
osal_id_t SHIM_OS_ObjectIdFromInteger(unsigned long value);
bool SHIM_OS_ObjectIdEqual(osal_id_t object_id1, osal_id_t object_id2);
bool SHIM_OS_ObjectIdDefined(osal_id_t object_id);
