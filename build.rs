extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let in_dir = env_unwrap("CARGO_MANIFEST_DIR");
    let out_dir = env_unwrap("OUT_DIR");

    let c_header_file = pb(&[&in_dir,  "cfs-all.h"]).to_string_unwrap();
    let out_file      = pb(&[&out_dir, "cfs-all.rs"]).to_string_unwrap();

    println!("cargo:rerun-if-changed={}", &c_header_file);

    let bindings = bindgen::builder()
        .header(&c_header_file)
        .clang_args(env_unwrap("RUST_CFS_SYS_COMPILE_DEFINITIONS")
            .split('@').map(|s| { String::from("-D") + s }))
        .clang_args(env_unwrap("RUST_CFS_SYS_INCLUDE_DIRECTORIES")
            .split('@').map(|s| { String::from("-I") + s }))
        .clang_args(env_unwrap("RUST_CFS_SYS_COMPILE_OPTIONS").split('@'))
        .allowlist_recursively(true)
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate().expect("Unable to generate cFS bindings");

    bindings.write_to_file(&out_file)
        .expect("Unable to write out cFS bindings");
}

fn env_unwrap(key: &str) -> String {
    println!("cargo:rerun-if-env-changed={}", key);
    env::var(key)
        .expect(&format!("Environment variable {} non-existent or unusable", key))
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
