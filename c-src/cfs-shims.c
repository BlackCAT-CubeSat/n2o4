/* Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
 * SPDX-License-Identifier: Apache-2.0
 */

/* Shim functions for cFE and OSAL functionality that is implemented by
 * C inline functions, for said functionality's use in Rust.
 */


/* The following macros shim<n> implement shim functions of <n> arguments. */

#define shim1(rettype, fname, arg1type, arg1) \
    rettype SHIM_ ## fname(arg1type arg1) \
    { \
        return fname(arg1); \
    }

#define shim2(rettype, fname, arg1type, arg1, arg2type, arg2) \
    rettype SHIM_ ## fname(arg1type arg1, arg2type arg2) \
    { \
        return fname(arg1, arg2); \
    }

#include "cfs-shims.h"

/* Bindgen *really* doesn't want to handle pointer constants. This is a workaround for that. */
CFE_ES_StackPointer_t X_CFE_ES_TASK_STACK_ALLOCATE = CFE_ES_TASK_STACK_ALLOCATE;
