# Using Rust and `n2o4` in a cFS Application

cFS and its build system are strongly oriented towards C.
However, with some finagling, you can write a cFS application in Rust
using the `n2o4` crate.

This is what you need to do:

## Prerequisites

First off, you will need the C toolchain and other build tools (_e.g._, CMake)
you are using to build the rest of your cFS system.

Next, you need to [install Rust], if you haven't done so already.
Make sure the `cargo` tool is in your `$PATH`.

For the time being, we need the `nightly` release channel
(refs [1], [2]) for a couple of features that aren't stable yet.
If you're using `rustup` to manage your Rust installation, this will add
that channel:

```sh
rustup toolchain install nightly
```

We also need the Rust standard library's source code
(for reasons relating to how we handle panics):

```sh
rustup +nightly component add rust-src
```

Finally, you will need to install `libclang` and the associated development files,
as they are required for automated generation of low-level bindings to cFS C APIs.
For example, on most Debian-like systems, you can run the following (as `root`):

```sh
apt install libclang-[n]-dev libclang1-[n]
```

where `[n]` is some version of `clang` (generally, the newer the better,
but [version 5 at minimum]).

## Build system additions

The build script,
which generates the low-level bindings to the cFS APIs that `n2o4` uses,
requires information about the C compiler used
and the locations of the cFE and OSAL include files.
In addition, Cargo needs to be instructed to write output files
to an appropriate place under the cFS project's `build/` directory.

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

When cross-compiling to a non-host CPU, or when you need custom C compiler flags, you'll
also need to set some variables in your toolchain file (`*_defs/toolchain-${CPU}.cmake`):

```cmake
# The Rust target, as you would pass to rustc or Cargo through the --target option:
SET(RUST_TARGET "armv7-unknown-linux-gnueabihf")

# Any additional compiler flags to pass to the `cc` crate,
# which is used by `n2o4`'s build script:
SET(RUST_CC_CFLAGS "-I/an/include/dir" "-DC_DEFINE" "-Wno-something")

# Any additional compiler flags to pass to the `bindgen` crate,
# which is used by the build script:
SET(RUST_BINDGEN_CFLAGS "-I/an/include/dir" "-DC_DEFINE" "-I/another/include_dir")
```

You can find a couple of example toolchain files in this repository
at `etc/toolchain-*.cmake.example`.

## Build caching (optional)

Each Rust-based cFS application is compiled separately,
by default not sharing compilation artifacts
even when they use the same version of the same crate.
This can lead to fresh builds taking some
time&mdash;a couple of minutes per app on a 2021-vintage laptop
(though incremental rebuilds are typically much faster&mdash;often a second or two).

With [a little configuration], Cargo can support [shared compile caches], notably with [sccache].
You can set the `RUSTC_WRAPPER_CMD` environment variable when CMake is run to enable its use, e.g.:

```sh
# From the top-level FSW directory, using the default cFS Makefile:
make RUSTC_WRAPPER_CMD=sccache prep
```

In practice, this can greatly reduce fresh build times for projects with many Rust-based cFS apps.

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

The default panic behavior on Rust is to unwind the stack&#8230;
which doesn't work well if you end up unwinding into C code like that your cFS application is called from.
As such, at least for now, we need to use the `abort` panic behavior:

```toml
[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

The `n2o4` crate isn't published on [crates.io] (at least for now)
as its ability to work is tightly bound to
`{rust_cfs_app,rust_mission_build}.cmake`
and a specific version of cFE and OSAL,
and there's many bindings still to add.
As such, you'll need to specify the crate using the Git repository
and revision you want to use:

```toml
[dependencies]
n2o4 = { git = "https://github.com/BlackCAT-CubeSat/n2o4.git", rev = "<commit ID>" }
```

Any functions that will be called directly from C code,
_including any application entry points_,
need to be made `extern "C"` and should have name mangling disabled:

```rust
/// The entry point for the "rustfsw" application.
#[no_mangle]
pub extern "C" fn RUSTFSW_AppMain() {
    // ...
}
```

The cFS CMake build system assumes your application has at least one C source file.
If all your logic is in Rust, you can use the [`placebo.c`](etc/placebo.c) file from `etc/` to
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

## Cargo features

One can set which [Cargo features] are enabled for your app by setting the `<app-name>_CARGO_FEATURES` variable in `targets.cmake`, where you set which apps are used.
Specify the set of features as a comma-separated list; prefix the list with `*` to disable the [default feature].
To give an example of what this looks like in `targets.cmake`:

```cmake
list(APPEND MISSION_GLOBAL_APPLIST rustfsw another_rust_app)
# enable features feature1 and feature2 for application rustfsw
set(rustfsw_CARGO_FEATURES "feature1,feature2")
# disable the default feature, and enable feature no-std for application another_rust_app
set(another_rust_app_CARGO_FEATURES "*no-std")
```

## Example

You can find a fully worked-out example of a Rust-using cFS application at
<https://github.com/BlackCAT-CubeSat/rust_sample_app>.

[install rust]: https://www.rust-lang.org/tools/install
[1]: https://rust-lang.github.io/rustup/concepts/channels.html
[2]: https://doc.rust-lang.org/book/appendix-07-nightly-rust.html
[version 5 at minimum]: https://rust-lang.github.io/rust-bindgen/requirements.html
[a little configuration]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-reads
[shared compile caches]: https://doc.rust-lang.org/cargo/guide/build-cache.html#shared-cache
[sccache]: https://github.com/mozilla/sccache
[crates.io]: https://crates.io/
[Cargo features]: https://doc.rust-lang.org/cargo/reference/features.html
[default feature]: https://doc.rust-lang.org/cargo/reference/features.html#the-default-feature
