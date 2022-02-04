// Copyright (c) 2021-2022 The Pennsylvania State University. All rights reserved.

//! Safe, higher-level bindings to the APIs of
//! [cFE](https://github.com/nasa/cFE)
//! and [OSAL](https://github.com/nasa/osal), the libraries used by
//! [Core Flight System](https://cfs.gsfc.nasa.gov/) applications.

#![cfg_attr(not(test), no_std)]
#![feature(const_fn_trait_bound)]

extern crate cfs_sys;
extern crate libc;
extern crate printf_wrap;

pub mod cfe;
pub mod osal;

pub(crate) mod sealed_traits;
