// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Constraint for entity reference restricting the type of entity it can point to.

use quent_constraints::Constraint;
use quent_schema::{Schema, data_type::DataType, entity::Entity, identifier::Identifier};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The entity type an entity reference is restricted to point at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefTarget {
    /// Name of the entity type this reference must point at.
    pub target: Identifier,
}

/// Restricts the type of entities
/// [`quent_schema::data_type::DataType::EntityRef`] can point to.
///
/// ## Requirements
///
/// 1. The named target is an entity declared in the schema.
pub struct RefTargetConstraint;

impl Constraint for RefTargetConstraint {
    const NAME: &'static str = "quent.ref-target.v1";

    fn validate(&self, schema: &Schema) -> Result<(), Box<dyn std::error::Error>> {
        let mut errors = Vec::new();

        for entity in &schema.entities {
            for event in &entity.events {
                for field in &event.payload {
                    check_targets(
                        &field.ty,
                        &|| {
                            format!(
                                "entity \"{}\" event \"{}\" field \"{}\"",
                                entity.name, event.name, field.name
                            )
                        },
                        &schema.entities,
                        &mut errors,
                    );
                }
            }
        }
        for record in &schema.records {
            for field in &record.fields {
                check_targets(
                    &field.ty,
                    &|| format!("record \"{}\" field \"{}\"", record.name, field.name),
                    &schema.entities,
                    &mut errors,
                );
            }
        }

        match errors.len() {
            0 => Ok(()),
            1 => Err(errors.pop().unwrap().into()),
            _ => Err(RefTargetError::Multiple(errors).into()),
        }
    }
}

fn check_targets<F: Fn() -> String>(
    ty: &DataType,
    location: &F,
    entities: &[Entity],
    errors: &mut Vec<RefTargetError>,
) {
    match ty {
        DataType::Option(inner) | DataType::List(inner) => {
            check_targets(inner, location, entities, errors);
        }
        DataType::EntityRef { data, annotations } => {
            if let Some(constraint) = annotations
                .constraints
                .iter()
                .find(|c| c.name == RefTargetConstraint::NAME)
            {
                match constraint.data.as_deref() {
                    None => errors.push(RefTargetError::InvalidData {
                        location: location(),
                        message: "constraint data is missing".to_string(),
                    }),
                    Some(raw) => match serde_json::from_str::<RefTarget>(raw) {
                        Ok(ref_target) => {
                            if !entities.iter().any(|e| e.name == ref_target.target) {
                                errors.push(RefTargetError::UnknownTarget {
                                    location: location(),
                                    target: ref_target.target,
                                });
                            }
                        }
                        Err(e) => errors.push(RefTargetError::InvalidData {
                            location: location(),
                            message: format!("failed to decode ref-target: {e}"),
                        }),
                    },
                }
            }
            if let Some(inner) = data {
                check_targets(inner, location, entities, errors);
            }
        }
        _ => {}
    }
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
    #[error("multiple ref-target violations:\n{}", join_errors(.0))]
    Multiple(Vec<RefTargetError>),
}

fn join_errors(errors: &[RefTargetError]) -> String {
    errors
        .iter()
        .map(|e| format!("  - {e}"))
        .collect::<Vec<_>>()
        .join("\n")
}
