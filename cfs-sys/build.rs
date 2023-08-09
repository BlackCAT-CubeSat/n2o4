// Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    let in_dir = env_unwrap("CARGO_MANIFEST_DIR");
    let out_dir = env_unwrap("OUT_DIR");

    let api_header = pb(&[&in_dir, "cfs-api.h"]).to_string_unwrap();
    let shims_header = pb(&[&in_dir, "cfs-shims.h"]).to_string_unwrap();
    let out_file = pb(&[&out_dir, "cfs-all.rs"]).to_string_unwrap();
    let shims_c = pb(&[&out_dir, "cfs-shims.c"]).to_string_unwrap();

    for f in [&api_header, &shims_header, &shims_c] {
        println!("cargo:rerun-if-changed={}", f);
    }

    let compile_defs = env_unwrap("RUST_CFS_SYS_COMPILE_DEFINITIONS");
    let include_dirs = env_unwrap("RUST_CFS_SYS_INCLUDE_DIRECTORIES");
    let compile_opts = env_unwrap("RUST_CFS_SYS_COMPILE_OPTIONS");

    let bindings = bindgen::builder()
        .header(&api_header)
        .header(&shims_header)
        .clang_args(compile_defs.split('@').map(|s| String::from("-D") + s))
        .clang_args(include_dirs.split('@').map(|s| String::from("-I") + s))
        .clang_args(compile_opts.split('@'))
        .allowlist_recursively(true)
        .allowlist_type("(CFE|OS|OSAL|CFE_PSP|CCSDS)_.*")
        .allowlist_function("(CFE|OS|OSAL|CFE_PSP|SHIM)_.*")
        .allowlist_var("(X_|S_)?(CFE|OS|OSAL|CFE_PSP)_.*")
        .blocklist_function("CFE_ES_Main") // only to be called by the BSP
        .blocklist_function("OS_BSP_.*") // ditto
        .use_core()
        .ctypes_prefix("::core::ffi")
        .size_t_is_usize(true)
        .generate_comments(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate cFS bindings");

    bindings.write_to_file(&out_file).expect("Unable to write out cFS bindings");

    let mut builder = cc::Build::new();
    builder.includes(include_dirs.split('@'));

    for def in compile_defs.split('@') {
        builder.flag(&(String::from("-D") + def));
    }
    for opt in compile_opts.split('@') {
        builder.flag(opt);
    }

    builder.file("cfs-shims.c").compile("cfs-shims");
}

fn env_unwrap(key: &str) -> String {
    println!("cargo:rerun-if-env-changed={}", key);
    env::var(key).expect(&format!("Environment variable {} non-existent or unusable", key))
}

/// Given a slice of path components, return the corresponding [`PathBuf`].
fn pb(components: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(components[0]);
    path.extend(&components[1..]);
    path
}

trait PathBufExt {
    fn to_string_unwrap(&self) -> String;
}

impl PathBufExt for PathBuf {
    fn to_string_unwrap(&self) -> String {
        self.to_str().unwrap().to_owned()
    }
}
