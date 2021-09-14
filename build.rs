extern crate bindgen;

use std::env;
use std::path;

fn env_unwrap(key: &str) -> String {
    println!("cargo:rerun-if-env-changed={}", key);
    env::var(key)
        .expect(&format!("Environment variable {} non-existent or unusable", key))
}

fn main() {

    println!("cargo:rerun-if-changed=cfs-all.h");

    let bindings = bindgen::builder()
        .header("cfs-all.h")
        .clang_args(env_unwrap("RUST_CFS_SYS_COMPILE_DEFINITIONS")
            .split('@').map(|s| { String::from("-D") + s }))
        .clang_args(env_unwrap("RUST_CFS_SYS_INCLUDE_DIRECTORIES")
            .split('@').map(|s| { String::from("-I") + s }))
        .clang_args(env_unwrap("RUST_CFS_SYS_COMPILE_OPTIONS").split('@'))
        //.use_core()
        .generate().expect("Unable to generate cFS bindings");

    let mut out_file = path::PathBuf::from(env_unwrap("OUT_DIR"));
    out_file.push("cfs-all.rs");

    bindings.write_to_file(&out_file)
        .expect("Unable to write out cFS bindings");
}
