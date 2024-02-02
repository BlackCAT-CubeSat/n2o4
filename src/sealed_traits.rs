// Copyright (c) 2022-2023 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Module for the creation of [sealed traits](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed).

/// Sealing trait for [`FilterScheme`](crate::cfe::evs::FilterScheme).
pub trait FilterSchemeSealed {}

/// Sealing trait for [`SocketDomain`](crate::osal::socket::SocketDomain).
pub trait SocketDomainSealed {
    const DOMAIN: crate::sys::OS_SocketDomain_t;
}

/// Sealing trait for [`SocketType`](crate::osal::socket::SocketType).
pub trait SocketTypeSealed {
    const SOCK_TYPE: crate::sys::OS_SocketType_t;
}

/// Sealing trait for [`SocketRole`](crate::osal::socket::SocketRole).
pub trait SocketRoleSealed {}
