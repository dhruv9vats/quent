// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Constraint for entity reference restricting the type of entity it can point to.

use quent_constraints::{Constraint, utils::bullet_list};
use quent_schema::{
    DataType, Identifier,
    visitor::{Cursor, Element, Visitor},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The entity type an entity reference is restricted to point at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefTarget {
    /// Name of the entity type this reference must point at.
    pub target: Identifier,
}

/// Restricts the type of entities
/// [`quent_schema::DataType::EntityRef`] can point to.
///
/// ## Requirements
///
/// 1. The named target is an entity declared in the schema.
#[derive(Default)]
pub struct RefTargetConstraint {
    errors: Vec<RefTargetError>,
}

impl Visitor for RefTargetConstraint {
    type Output = Result<(), RefTargetError>;

    fn visit(&mut self, cursor: &Cursor) {
        if let Element::DataType(DataType::EntityRef { annotations, .. }) = cursor.current()
            && let Some(constraint) = annotations.constraint(RefTargetConstraint::NAME)
        {
            let location = cursor.to_string();
            match constraint.data() {
                None => self.errors.push(RefTargetError::InvalidData {
                    location,
                    message: "constraint data is missing".to_string(),
                }),
                Some(raw) => match serde_json::from_str::<RefTarget>(raw) {
                    Ok(ref_target) => {
                        if cursor.root().entity(&ref_target.target).is_none() {
                            self.errors.push(RefTargetError::UnknownTarget {
                                location,
                                target: ref_target.target,
                            });
                        }
                    }
                    Err(e) => self.errors.push(RefTargetError::InvalidData {
                        location,
                        message: format!("failed to decode ref-target: {e}"),
                    }),
                },
            }
        }
    }

    fn finish(self) -> Self::Output {
        match self.errors.len() {
            0 => Ok(()),
            1 => Err(self.errors.into_iter().next().unwrap()),
            _ => Err(RefTargetError::Multiple(self.errors)),
        }
    }
}

impl Constraint for RefTargetConstraint {
    const NAME: &'static str = "quent.ref-target.v1";
}

#[derive(Debug, Error)]
pub enum RefTargetError {
    #[error("{location}: invalid ref-target data: {message}")]
    InvalidData { location: String, message: String },
    #[error("{location}: ref-target points at unknown entity \"{target}\"")]
    UnknownTarget {
        location: String,
        target: Identifier,
    },
    #[error("multiple ref-target violations:\n{}", bullet_list(.0))]
    Multiple(Vec<RefTargetError>),
}
