// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::{annotations::Annotations, data_type::DataType, identifier::Identifier};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Cardinality {
    /// The event can be emitted zero or one time.
    Once,
    /// The event can be emitted zero or multiple times.
    Multi,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventField {
    /// The name of the event field.
    pub name: Identifier,
    /// The type of the event field.
    pub ty: DataType,
    /// Annotations of this event field.
    pub annotations: Annotations,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Event {
    /// The name of the event.
    pub name: Identifier,
    /// The [`Cardinality`] of the event.
    pub cardinality: Cardinality,
    /// The fields of the event.
    pub payload: Vec<EventField>,
    /// Annotations of this event.
    pub annotations: Annotations,
}
