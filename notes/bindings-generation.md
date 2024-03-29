# Low-level bindings generation

The `n2o4::sys` module is a set of low-level bindings to
[cFS](https://cfs.gsfc.nasa.gov/)'s
[cFE](https://github.com/nasa/cFE)
and [OSAL](https://github.com/nasa/osal) libraries,
generated by the build script using
[bindgen](https://github.com/rust-lang/rust-bindgen).

## Build setup

**IMPORTANT:** the build script requires several environment variables to be set
at build time in order to find and interpret the cFE and OSAL header files
properly:

<dl>
<dt>RUST_CFS_SYS_COMPILE_DEFINITIONS</dt>
<dd>A list of C preprocessor definitions, delimited by <code>@</code> characters.
E.g.: <code>_XOPEN_SOURCE=600@DEBUG@ARCH=x86_64</code></dd>

<dt>RUST_CFS_SYS_INCLUDE_DIRECTORIES</dt>
<dd>A list of directories to put in the include path, delimited by <code>@</code>
characters.
E.g.: <code>/opt/custom-cc/include@/home/build/proj/include</code></dd>

<dt>RUST_CFS_SYS_COMPILE_OPTIONS</dt>
<dd>A list of command-line options to the C compiler, delimited by <code>@</code> characters.
E.g.: <code>-std=c99@-pedantic@-Wall@-Wextra@-Werror</code></dd>
</dl>

These should match what the build system uses in compiling C files for the
cFS application `n2o4` is used with. For instance,
in the current (at time of writing) CMake-based build system provided by cFE
for use in cFS-based software, if one was building a top-level crate named
`x-crate` (which has `n2o4` as a transitive dependency),
which is located in the `rust-src` directory of cFS application
`app_x`, the following snippet in the application's `CMakeFiles.txt`
would set the environment variables correctly:

```cmake
add_custom_command(
  OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/target/release/libx_crate.a
  WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/rust-src
  COMMAND ${CMAKE_COMMAND} -E env
    "RUST_CFS_SYS_COMPILE_DEFINITIONS=$<JOIN:$<TARGET_PROPERTY:app_x,COMPILE_DEFINITIONS>,@>"
    "RUST_CFS_SYS_INCLUDE_DIRECTORIES=$<JOIN:$<TARGET_PROPERTY:app_x,INCLUDE_DIRECTORIES>,@>"
    "RUST_CFS_SYS_COMPILE_OPTIONS=$<JOIN:$<TARGET_PROPERTY:app_x,COMPILE_OPTIONS>,@>"
    cargo build --release --target ${CMAKE_CURRENT_BINARY_DIR}/target
  DEPFILE ${CMAKE_CURRENT_BINARY_DIR}/target/release/libx_crate.d
  VERBATIM
)
```

This is taken care of by `etc/rust_cfs_app.cmake`
(see [USING.md](../USING.md) for instructions),
so usually you don't have to worry about this.
