// Copyright (c) 2022 The Pennsylvania State University and the project contributors.
// SPDX-License-Identifier: Apache-2.0

//! Module for the creation of [sealed traits](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed).

/// Sealing trait for [`FilterScheme`](crate::cfe::evs::FilterScheme).
pub trait FilterSchemeSealed {}
