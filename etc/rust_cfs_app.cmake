# Copyright (c) 2021-2022 The Pennsylvania State University and the project contributors.
# SPDX-License-Identifier: Apache-2.0
#
# Build configuration for Rust code in cFS applications. This is used in
# arch-specific builds (different CPUs), and generally should be included
# from arch_build_custom.cmake.
#
# If cross-compiling, you should set the variable RUST_TARGET
# to the appropriate Rust target in toolchain-<CPU>.cmake.
# You'll probably also want to set the list-valued variable
# RUST_{CC,BINDGEN}_CFLAGS in the same place to pass in compiler flags
# for the appropriate target CPU, sysroot, etc. for use by the bindgen
# and cc crates.


# Target for running Rust tests for this arch.
#
# WARNING: currently, this probably does nothing good on non-native builds
#
# TODO: figure out what to do for non-native builds
add_custom_target(run_rust_tests)

# Target for building and installing Rustdocs for this arch.
add_custom_target(install_rust_docs)

set(RUSTDOC_TARGET_INDEX_PATH "${MISSION_BINARY_DIR}/docs/${TARGETSYSTEM}/index.html")
file(
  WRITE  "${RUSTDOC_TARGET_INDEX_PATH}"
  "<!DOCTYPE html>\n<html>\n<head><title>${MISSION_NAME}: ${TARGETSYSTEM}: Rust applications</title></head>\n"
)
file(
  APPEND "${RUSTDOC_TARGET_INDEX_PATH}"
  "<body>\n<h1>Rust applications on ${TARGETSYSTEM}</h1>\n<ul>\n"
)

if(NOT DEFINED RUST_TARGET)
  # If we're not given a target, use the default target

  # Thanks to <https://stackoverflow.com/q/52996949> for pointing out
  # how Cargo itself does this...

  execute_process(
    COMMAND "rustc" "-vV"
    COMMAND "sed" "-n" "-e" "s/^host: *//p"
    COMMAND "head" "-n" "1"
    OUTPUT_VARIABLE RUST_TARGET
    OUTPUT_STRIP_TRAILING_WHITESPACE
    ERROR_QUIET
    COMMAND_ERROR_IS_FATAL ANY
  )
endif()

# Sets the variable CARGO_ENV_VARIABLES in the caller's scope
# to a list of environment-variable settings for use in invocations of
# Cargo. Takes the cFS app name as a required first argument.
#
function(generate_cargo_vars CFS_APP)
  # The RUST_CFS_SYS_* environment variables are used for generating
  # the cFE/OSAL/PSP bindings.
  set(CEV
    "RUST_CFS_SYS_COMPILE_DEFINITIONS=$<JOIN:$<TARGET_PROPERTY:${CFS_APP},COMPILE_DEFINITIONS>,@>"
    "RUST_CFS_SYS_INCLUDE_DIRECTORIES=$<JOIN:$<TARGET_PROPERTY:${CFS_APP},INCLUDE_DIRECTORIES>,@>"
    "RUST_CFS_SYS_COMPILE_OPTIONS=$<JOIN:$<TARGET_PROPERTY:${CFS_APP},COMPILE_OPTIONS>,@>"
  )

  list(APPEND CEV "CC=${CMAKE_C_COMPILER}")

  if(DEFINED RUST_CC_CFLAGS)
    list(JOIN RUST_CC_CFLAGS " " RCCF)
    list(APPEND CEV "CFLAGS=${RCCF}")
    list(APPEND CEV "CRATE_CC_NO_DEFAULTS=true")
  endif()

  if(DEFINED RUST_BINDGEN_CFLAGS)
    list(JOIN RUST_BINDGEN_CFLAGS " " RBCF)
    list(APPEND CEV "BINDGEN_EXTRA_CLANG_ARGS=${RBCF}")
  endif()

  set(CARGO_ENV_VARIABLES ${CEV} PARENT_SCOPE)
endfunction()

# Builds the Rust crate CRATE_NAME and links it into cFS application CFS_APP.
#
# This function has a couple of requirements:
#
# * The crate manifest (Cargo.toml) is in the application source's
#   rust-fsw/ directory.
#
# * The crate compiles to a static library
#   (lib.crate-type includes "staticlib").
#
function(cfe_rust_crate CFS_APP CRATE_NAME)

  set(RUST_SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/rust-fsw)

  set(CARGO_TARGET_DIR ${CMAKE_CURRENT_BINARY_DIR}/target)

  string(REGEX REPLACE "-" "_" CRATE_FILE_STEM "${CRATE_NAME}")

  # Build the crate as a static library, to be linked into the cFS application:

  set(LIB_BUILD_DIR ${CARGO_TARGET_DIR}/${RUST_TARGET}/release)
  set(LIB_FILE ${LIB_BUILD_DIR}/lib${CRATE_FILE_STEM}.a)

  generate_cargo_vars(${CFS_APP})

  set(CARGO_OUTPUT_FLAGS
    --release --target ${RUST_TARGET} --target-dir ${CARGO_TARGET_DIR} --quiet
  )

  add_custom_command(
    OUTPUT ${LIB_FILE}
    WORKING_DIRECTORY ${RUST_SOURCE_DIR}
    COMMAND ${CMAKE_COMMAND} -E env
      ${CARGO_ENV_VARIABLES}
      cargo +nightly build -Z build-std=std,panic_abort
      ${CARGO_OUTPUT_FLAGS}
    DEPFILE ${LIB_BUILD_DIR}/lib${CRATE_FILE_STEM}.d
    DEPENDS ${RUST_SOURCE_DIR}/Cargo.toml
    VERBATIM
  )

  add_custom_target(${CFS_APP}_rust_build DEPENDS ${LIB_FILE})

  add_library(${CFS_APP}_rust_lib STATIC IMPORTED)
  add_dependencies(${CFS_APP}_rust_lib ${CFS_APP}_rust_build)
  set_target_properties(${CFS_APP}_rust_lib
    PROPERTIES
    IMPORTED_LOCATION ${LIB_FILE}
  )

  target_link_libraries(${CFS_APP} ${CFS_APP}_rust_lib m)


  # Now that we've set up building the Rust code for the application,
  # we set up crate-level testing:
  add_custom_target(${CFS_APP}_run_rust_tests
    COMMAND ${CMAKE_COMMAND} -E env
      ${CARGO_ENV_VARIABLES}
      cargo +nightly test
      ${CARGO_OUTPUT_FLAGS}
    WORKING_DIRECTORY ${RUST_SOURCE_DIR}
    VERBATIM
  )

  add_dependencies(run_rust_tests ${CFS_APP}_run_rust_tests)

  # and now we build its Rustdocs:
  set(DOC_DIR ${CARGO_TARGET_DIR}/${RUST_TARGET}/doc)

  add_custom_target(${CFS_APP}_rustdocs_build
    COMMAND ${CMAKE_COMMAND} -E env
      ${CARGO_ENV_VARIABLES}
      cargo +nightly doc --document-private-items
      ${CARGO_OUTPUT_FLAGS}
    WORKING_DIRECTORY ${RUST_SOURCE_DIR}
    VERBATIM
  )

  # Install the Rustdocs in the docs directory:
  set(DOC_INSTALL_DIR ${MISSION_BINARY_DIR}/docs/${TARGETSYSTEM}/rust-${CFS_APP})

  add_custom_target(${CFS_APP}_rustdocs_install
    COMMAND ${CMAKE_COMMAND} -E rm -rf ${DOC_INSTALL_DIR}
    COMMAND ${CMAKE_COMMAND} -E copy_directory ${DOC_DIR} ${DOC_INSTALL_DIR}
    DEPENDS ${CFS_APP}_rustdocs_build
    VERBATIM
  )

  add_dependencies(install_rust_docs ${CFS_APP}_rustdocs_install)

  # Add an entry to the Rustdocs index:
  file(
    APPEND "${RUSTDOC_TARGET_INDEX_PATH}"
    "<li><a href=\"rust-${CFS_APP}/${CRATE_FILE_STEM}/index.html\">${CFS_APP}</a></li>\n"
  )

  # When cleaning, clean up the build directory:
  set_directory_properties(
    PROPERTIES
    ADDITIONAL_CLEAN_FILES ${CARGO_TARGET_DIR}
  )
endfunction()
