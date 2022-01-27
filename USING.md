# Using Rust and `cfs` in a cFS App

cFS and its build system are strongly oriented towards C.
However, with some finagling, you can write a cFS application in Rust
the `cfs` crate.

This is what you have to do:

## Build system additions

The `cfs-sys` crate,
which provides the low-level bindings to the cFS APIs that `cfs` uses,
requires information about the C compiler used
and the locations of the cFE and OSAL include files.
In addition, generating Rustdocs for all the projects

This all is handled by a couple of CMake files
you can find in the `etc/` directory of this repository:
`rust_cfs_app.cmake` and `rust_mission_build.cmake`.
Copy those files to your cFS project's `*_defs` directory,
then add the following line to `*_defs/mission_build_custom.cmake`:

```cmake
include("${MISSION_DEFS}/rust_mission_build.cmake")
```

and the following line to `*_defs/arch_build_custom.cmake`:

```cmake
include("${MISSION_DEFS}/rust_cfs_app.cmake")
```

When cross-compiling to a non-host CPU, or when you need custom compiler flags, you'll
also need to set some variables in `*_defs/toolchain-${CPU}.cmake`:

```cmake
# The Rust target, as you would pass to rustc or Cargo through the --target option:
SET(RUST_TARGET "armv7-unknown-linux-gnueabihf")

# Any additional compiler flags to pass to the `cc` crate, used by `cfs-sys`:
SET(RUST_CC_CFLAGS "-I/an/include/dir" "-DC_DEFINE" "-Wno-something")

# Any additional compiler flags to pass to the `bindgen` crate, used by `cfs-sys`:
SET(RUST_BINDGEN_CFLAGS "-I/an/include/dir" "-DC_DEFINE" "-I/another/include_dir")
```

## Application layout

`rust_cfs_app.cmake` requires that the Rust code for the application be
in a crate whose `Cargo.toml` is in the `rust-fsw/` directory under the application root
(or, of course, in crates pulled in as dependencies of that crate).

To incorporate the compiled Rust code into the application binary, it must be
compiled as a static library.
As such, you'll need to set the crate type to `"staticlib"` in `rust-fsw/Cargo.toml`:

```toml
[lib]
crate-type = ["staticlib"]
```

Any functions that will be called from C code, including any application entry points,
need to be made `extern "C"`, and almost certainly should have name mangling disabled:

```rust
/// The entry point for the "rustfsw" application.
#[no_mangle]
pub extern "C" fn RUSTFSW_AppMain() {
    // ...
}
```

The cFS CMake build system assumes your application has at least one C source file.
If all your logic is in Rust, you can use the `placebo.c` file in `etc/` to
satisfy that assumption.

Finally, to tie everything together, use the `cfe_rust_crate` function in your
application's `CMakeLists.txt`. Be sure to ensure your entry points get
linked in!

```cmake
cmake_minimum_required(VERSION 3.13.0)

project(CFE_RUSTFSW_APP C)

add_cfe_app(rustfsw fsw/src/placebo.c)

# cfe_rust_crate takes two arguments: the name of the cFE app
# and the name of the crate at rust-fsw/Cargo.toml:
cfe_rust_crate(rustfsw thecratename)

# Since the Rust code is compiled to a static library, you need to
# ensure the application entry point is in the linked-together app:
target_link_options(rustfsw PUBLIC LINKER:--require-defined=RUSTFSW_AppMain)
```
