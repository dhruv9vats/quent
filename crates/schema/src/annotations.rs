// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{constraint::Constraint, metadata::Metadata};

/// Annotations for [`crate::Schema`] constituents.
///
/// Annotations do not affect how events are read or written. They merely carry
/// documentation about core elements of the Schema, opaque [`Constraint`]s that
/// must be validated if they cannot be guaranteed to hold, and miscellaneous
/// opaque [`Metadata`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Annotations {
    /// Potential documentation that can e.g. be added in code generation.
    pub docs: Option<String>,
    /// Opaque constraints that must be validated against the schema.
    pub constraints: Vec<Constraint>,
    /// Opaque metadata passed through the schema.
    pub metadata: Vec<Metadata>,
}
