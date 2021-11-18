/* Copyright (c) 2021 The Pennsylvania State University. All rights reserved.
 */

/* Shim functions for cFE and OSAL functionality that is implemented by
 * C inline functions, for said functionality's use in Rust.
 */

#include <cfe_resourceid.h>
#include <cfe_sb.h>
#include <osapi-clock.h>
#include <osapi-idmap.h>


/* The following macros shim<n> implement shim functions of <n> arguments. */

#define shim1(rettype, fname, arg1type) \
    rettype SHIM_ ## fname(arg1type arg1) \
    { \
        return fname(arg1); \
    }

#define shim2(rettype, fname, arg1type, arg2type) \
    rettype SHIM_ ## fname(arg1type arg1, arg2type arg2) \
    { \
        return fname(arg1, arg2); \
    }


shim1(unsigned long, CFE_ResourceId_ToInteger, CFE_ResourceId_t)
shim1(CFE_ResourceId_t, CFE_ResourceId_FromInteger, unsigned long)
shim2(bool, CFE_ResourceId_Equal, CFE_ResourceId_t, CFE_ResourceId_t)
shim1(bool, CFE_ResourceId_IsDefined, CFE_ResourceId_t)

shim2(bool, CFE_SB_MsgId_Equal, CFE_SB_MsgId_t, CFE_SB_MsgId_t)
shim1(CFE_SB_MsgId_Atom_t, CFE_SB_MsgIdToValue, CFE_SB_MsgId_t)
shim1(CFE_SB_MsgId_t, CFE_SB_ValueToMsgId, CFE_SB_MsgId_Atom_t)

shim1(int64, OS_TimeGetTotalSeconds, OS_time_t)
shim1(int64, OS_TimeGetTotalMilliseconds, OS_time_t)
shim1(int64, OS_TimeGetTotalMicroseconds, OS_time_t)
shim1(int64, OS_TimeGetTotalNanoseconds, OS_time_t)
shim1(int64, OS_TimeGetFractionalPart, OS_time_t)
shim1(uint32, OS_TimeGetSubsecondsPart, OS_time_t)
shim1(uint32, OS_TimeGetMillisecondsPart, OS_time_t)
shim1(uint32, OS_TimeGetMicrosecondsPart, OS_time_t)
shim1(uint32, OS_TimeGetNanosecondsPart, OS_time_t)
shim2(OS_time_t, OS_TimeAssembleFromNanoseconds, int64, uint32)
shim2(OS_time_t, OS_TimeAssembleFromMicroseconds, int64, uint32)
shim2(OS_time_t, OS_TimeAssembleFromMilliseconds, int64, uint32)
shim2(OS_time_t, OS_TimeAssembleFromSubseconds, int64, uint32)
shim2(OS_time_t, OS_TimeAdd, OS_time_t, OS_time_t)
shim2(OS_time_t, OS_TimeSubtract, OS_time_t, OS_time_t)

shim1(unsigned long, OS_ObjectIdToInteger, osal_id_t)
shim1(osal_id_t, OS_ObjectIdFromInteger, unsigned long)
shim2(bool, OS_ObjectIdEqual, osal_id_t, osal_id_t)
shim1(bool, OS_ObjectIdDefined, osal_id_t)
