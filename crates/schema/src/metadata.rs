// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

/// Opaque named metadata passed through the schema.
///
/// This is ignored by the canonical validator of the `quent-constraints` crate.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Metadata {
    /// The name of the metadata entry.
    pub name: String,
    /// The opaque metadata value.
    ///
    /// The format for this data is unrestricted, other than that it must form a
    /// valid UTF-8 string. It is encouraged to serialize Metadata data in a
    /// human-readable fashion for easier debugging.
    pub data: Option<String>,
}
