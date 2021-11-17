// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

//! Safe, higher-level bindings to the APIs of
//! [cFE](https://github.com/nasa/cFE)
//! and [OSAL](https://github.com/nasa/osal), the libraries used by
//! [Core Flight System](https://cfs.gsfc.nasa.gov/) applications.

#![cfg_attr(not(test), no_std)]

extern crate cfs_sys;

use cfs_sys as _;

pub mod osal;
pub mod cfe;
