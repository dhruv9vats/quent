// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{annotations::Annotations, data_type::DataType, identifier::Identifier};

/// A named, typed field of a [`crate::record::Record`] or
/// [`crate::event::Event`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Field {
    /// The name of the field.
    pub name: Identifier,
    /// The type of the field.
    pub ty: DataType,
    /// Annotations of this field.
    pub annotations: Annotations,
}
