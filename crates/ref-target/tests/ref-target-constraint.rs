// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use quent_constraints::{Constraint as _, Error as ValidatorError, Validator};
use quent_ref_target::{RefTarget, RefTargetConstraint, RefTargetError};
use quent_schema::{
    Schema,
    annotations::Annotations,
    constraint::Constraint,
    data_type::DataType,
    entity::Entity,
    event::{Cardinality, Event},
    field::Field,
    identifier::Identifier,
};

fn ident(s: &str) -> Identifier {
    Identifier::try_new(s).unwrap()
}

fn target_constraint(target: &str) -> Constraint {
    Constraint {
        name: RefTargetConstraint::NAME.to_string(),
        data: Some(
            serde_json::to_string(&RefTarget {
                target: ident(target),
            })
            .unwrap(),
        ),
    }
}

fn ref_with(constraint: Constraint) -> DataType {
    DataType::EntityRef {
        data: None,
        annotations: Annotations {
            constraints: vec![constraint],
            ..Default::default()
        },
    }
}

fn field(name: &str, ty: DataType) -> Field {
    Field {
        name: ident(name),
        ty,
        annotations: Annotations::default(),
    }
}

fn event(name: &str, payload: Vec<Field>) -> Event {
    Event {
        name: ident(name),
        cardinality: Cardinality::Once,
        payload,
        annotations: Annotations::default(),
    }
}

fn entity(name: &str, events: Vec<Event>) -> Entity {
    Entity {
        name: ident(name),
        events,
        annotations: Annotations::default(),
    }
}

fn schema_with(entities: Vec<Entity>) -> Schema {
    Schema {
        name: ident("S"),
        entities,
        records: vec![],
        annotations: Annotations::default(),
    }
}

fn validate(schema: &Schema) -> Vec<RefTargetError> {
    match Validator::default()
        .try_with(RefTargetConstraint)
        .unwrap()
        .validate(schema)
    {
        Ok(()) => Vec::new(),
        Err(ValidatorError::Invalid { failures, .. }) => {
            let (_, source) = failures.into_iter().next().unwrap();
            match *source.downcast::<RefTargetError>().unwrap() {
                RefTargetError::Multiple(errors) => errors,
                single => vec![single],
            }
        }
        Err(_) => unreachable!(),
    }
}

#[test]
fn ref_to_existing_entity_passes() {
    let worker = entity("Worker", vec![]);
    let task = entity(
        "Task",
        vec![event(
            "created",
            vec![field("on", ref_with(target_constraint("Worker")))],
        )],
    );
    assert!(validate(&schema_with(vec![worker, task])).is_empty());
}

#[test]
fn ref_to_unknown_entity_is_rejected() {
    let task = entity(
        "Task",
        vec![event(
            "created",
            vec![field("on", ref_with(target_constraint("ghost")))],
        )],
    );
    let errors = validate(&schema_with(vec![task]));
    assert!(
        errors.iter().any(
            |e| matches!(e, RefTargetError::UnknownTarget { target, .. } if target == "ghost")
        ),
    );
}

#[test]
fn missing_data_is_rejected() {
    let bad = ref_with(Constraint {
        name: RefTargetConstraint::NAME.to_string(),
        data: None,
    });
    let task = entity("Task", vec![event("created", vec![field("on", bad)])]);
    let errors = validate(&schema_with(vec![task]));
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, RefTargetError::InvalidData { .. })),
    );
}

#[test]
fn invalid_json_is_rejected() {
    let bad = ref_with(Constraint {
        name: RefTargetConstraint::NAME.to_string(),
        data: Some("{ trash".to_string()),
    });
    let task = entity("Task", vec![event("created", vec![field("on", bad)])]);
    let errors = validate(&schema_with(vec![task]));
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, RefTargetError::InvalidData { .. })),
    );
}
