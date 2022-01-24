# Copyright (c) 2022 The Pennsylvania State University. All rights reserved.
#
# Build configuration for Rust code in cFS. This file contains a top-level
# mission-wide build target, and generally should be included
# from mission_build_custom.cmake.

# Top-level target to build and install Rustdocs for all mission CPUs:
add_custom_target(rust-doc)

foreach(TGTSYS ${TGTSYS_LIST})
  # Cribbing from mission_build.cmake's process_arch() to construct the build directory...
  set(BUILD_CONFIG ${BUILD_CONFIG_${TGTSYS}})
  list(GET BUILD_CONFIG 0 ARCH_TOOLCHAIN_NAME)
  list(REMOVE_AT BUILD_CONFIG 0)
  string(REGEX REPLACE "[^A-Za-z0-9]" "_" ARCH_CONFIG_NAME "${BUILD_CONFIG}")
  set(ARCH_BINARY_DIR "${CMAKE_BINARY_DIR}/${ARCH_TOOLCHAIN_NAME}/${ARCH_CONFIG_NAME}")

  # install Rustdocs for this target:
  add_custom_target(${TGTSYS}_install_rust_docs
      COMMAND $(MAKE) install_rust_docs
      WORKING_DIRECTORY "${ARCH_BINARY_DIR}"
  )
  add_dependencies(rust-doc ${TGTSYS}_install_rust_docs)
endforeach()
