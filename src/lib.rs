// Copyright (c) 2021 The Pennsylvania State University. All rights reserved.

//! Low-level bindings to the C APIs of [cFE](https://github.com/nasa/cFE)
//! and [OSAL](https://github.com/nasa/osal).
//!
//! The definitions in this crate are generated by
//! [bindgen](https://github.com/rust-lang/rust-bindgen).

#![no_std]

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(improper_ctypes)]

pub type char = ::libc::c_char;

// ${OUT_DIR}/cfs-all.rs is generated by the build script
include!(concat!(env!("OUT_DIR"), "/cfs-all.rs"));
