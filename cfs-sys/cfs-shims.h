/* Copyright (c) 2021 The Pennsylvania State University. All rights reserved.
 */

/* Shim functions for cFE and OSAL functionality that is implemented by
 * C inline functions, for said functionality's use in Rust.
 */

#include <cfe.h>
#include <osapi.h>

/* The following macros shim<n> (if not pre-defined)
 * generate prototypes for shim functions of <n> arguments.
 */

#if !defined(shim1)
#define shim1(rettype, fname, arg1type, arg1) \
    rettype SHIM_ ## fname(arg1type arg1);
#endif

#if !defined(shim2)
#define shim2(rettype, fname, arg1type, arg1, arg2type, arg2) \
    rettype SHIM_ ## fname(arg1type arg1, arg2type arg2);
#endif

shim1(unsigned long, CFE_ResourceId_ToInteger, CFE_ResourceId_t, id)
shim1(CFE_ResourceId_t, CFE_ResourceId_FromInteger, unsigned long, Value)
shim2(bool, CFE_ResourceId_Equal, CFE_ResourceId_t, id1, CFE_ResourceId_t, id2)
shim1(bool, CFE_ResourceId_IsDefined, CFE_ResourceId_t, id)

shim2(bool, CFE_SB_MsgId_Equal, CFE_SB_MsgId_t, MsgId1, CFE_SB_MsgId_t, MsgId2)
shim1(CFE_SB_MsgId_Atom_t, CFE_SB_MsgIdToValue, CFE_SB_MsgId_t, MsgId)
shim1(CFE_SB_MsgId_t, CFE_SB_ValueToMsgId, CFE_SB_MsgId_Atom_t, MsgIdValue)

shim1(int64, OS_TimeGetTotalSeconds, OS_time_t, tm)
shim1(int64, OS_TimeGetTotalMilliseconds, OS_time_t, tm)
shim1(int64, OS_TimeGetTotalMicroseconds, OS_time_t, tm)
shim1(int64, OS_TimeGetTotalNanoseconds, OS_time_t, tm)
shim1(int64, OS_TimeGetFractionalPart, OS_time_t, tm)
shim1(uint32, OS_TimeGetSubsecondsPart, OS_time_t, tm)
shim1(uint32, OS_TimeGetMillisecondsPart, OS_time_t, tm)
shim1(uint32, OS_TimeGetMicrosecondsPart, OS_time_t, tm)
shim1(uint32, OS_TimeGetNanosecondsPart, OS_time_t, tm)
shim2(OS_time_t, OS_TimeAssembleFromNanoseconds, int64, seconds, uint32, nanoseconds)
shim2(OS_time_t, OS_TimeAssembleFromMicroseconds, int64, seconds, uint32, microseconds)
shim2(OS_time_t, OS_TimeAssembleFromMilliseconds, int64, seconds, uint32, milliseconds)
shim2(OS_time_t, OS_TimeAssembleFromSubseconds, int64, seconds, uint32, subseconds)
shim2(OS_time_t, OS_TimeAdd, OS_time_t, time1, OS_time_t, time2)
shim2(OS_time_t, OS_TimeSubtract, OS_time_t, time1, OS_time_t, time2)

shim1(unsigned long, OS_ObjectIdToInteger, osal_id_t, object_id)
shim1(osal_id_t, OS_ObjectIdFromInteger, unsigned long, value)
shim2(bool, OS_ObjectIdEqual, osal_id_t, object_id1, osal_id_t, object_id2)
shim1(bool, OS_ObjectIdDefined, osal_id_t, object_id)
