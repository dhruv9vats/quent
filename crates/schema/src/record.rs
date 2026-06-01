// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{annotations::Annotations, data_type::DataType, identifier::Identifier};

/// Definition of a field in a record.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecordField {
    /// The name of this record field.
    pub name: Identifier,
    /// The type of this record field.
    pub ty: DataType,
    /// Annotations of this record field.
    pub annotations: Annotations,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Record {
    /// The name of the record.
    pub name: Identifier,
    /// The fields of the record.
    pub fields: Vec<RecordField>,
    /// Annotations of this record.
    pub annotations: Annotations,
}
