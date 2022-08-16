# Copyright (c) 2022 The Pennsylvania State University. All rights reserved.
#
# Build configuration for Rust code in cFS. This file contains a top-level
# mission-wide build target, and generally should be included
# from mission_build_custom.cmake.

# Target for running Rust tests for all mission CPUs.
#
# WARNING: right now, this probably does nonsensical things on non-native builds!
#
# TODO: figure out what to do for non-native builds
add_custom_target(rust-test)

# Top-level target to build and install Rustdocs for all mission CPUs:
add_custom_target(rust-doc)

set(RUSTDOC_TOP_INDEX_PATH "${MISSION_BINARY_DIR}/docs/rustdocs.html")

file(
  WRITE  "${RUSTDOC_TOP_INDEX_PATH}"
  "<!DOCTYPE html>\n<html>\n<head><title>${MISSION_NAME} Rustdocs</title></head>\n"
)
file(
  APPEND "${RUSTDOC_TOP_INDEX_PATH}" "<body>\n<h1>Targets</h1>\n<ul>\n")

foreach(TGTSYS ${TGTSYS_LIST})
  file(APPEND "${RUSTDOC_TOP_INDEX_PATH}" "<li><a href=\"${TGTSYS}/index.html\">${TGTSYS}</a></li>\n")

  # Cribbing from mission_build.cmake's process_arch() to construct the build directory...
  set(BUILD_CONFIG ${BUILD_CONFIG_${TGTSYS}})
  list(GET BUILD_CONFIG 0 ARCH_TOOLCHAIN_NAME)
  list(REMOVE_AT BUILD_CONFIG 0)
  string(REGEX REPLACE "[^A-Za-z0-9]" "_" ARCH_CONFIG_NAME "${BUILD_CONFIG}")
  set(ARCH_BINARY_DIR "${CMAKE_BINARY_DIR}/${ARCH_TOOLCHAIN_NAME}/${ARCH_CONFIG_NAME}")

  # run Rust tests for this target:
  add_custom_target(${TGTSYS}_run_rust_tests
    COMMAND $(MAKE) run_rust_tests
    WORKING_DIRECTORY "${ARCH_BINARY_DIR}"
  )
  add_dependencies(rust-test "${TGTSYS}_run_rust_tests")

  # install Rustdocs for this target:
  add_custom_target(${TGTSYS}_install_rust_docs
    COMMAND $(MAKE) install_rust_docs
    WORKING_DIRECTORY "${ARCH_BINARY_DIR}"
  )
  add_dependencies(rust-doc "${TGTSYS}_install_rust_docs")
endforeach()

file(APPEND "${RUSTDOC_TOP_INDEX_PATH}" "</ul>\n</body>\n</html>\n")
