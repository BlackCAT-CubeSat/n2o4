// Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Safe, higher-level bindings to the APIs of
//! [cFE](https://github.com/nasa/cFE)
//! and [OSAL](https://github.com/nasa/osal), the libraries used by
//! [Core Flight System](https://cfs.gsfc.nasa.gov/) applications.

#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

extern crate cfs_sys;
extern crate libc;
extern crate printf_wrap;

pub mod cfe;
pub mod osal;

pub(crate) mod sealed_traits;
