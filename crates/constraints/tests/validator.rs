// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use quent_constraints::{Constraint, Error, Validator};
use quent_schema::{
    Schema,
    annotations::Annotations,
    constraint::Constraint as SchemaConstraint,
    data_type::DataType,
    entity::Entity,
    event::{Cardinality, Event},
    field::Field,
    identifier::Identifier,
    metadata::Metadata,
    record::Record,
};

fn ident(s: &str) -> Identifier {
    Identifier::try_new(s).unwrap()
}

fn constraint(name: &str) -> SchemaConstraint {
    SchemaConstraint {
        name: name.to_string(),
        data: None,
    }
}

fn metadata(name: &str) -> Metadata {
    Metadata {
        name: name.to_string(),
        data: None,
    }
}

fn empty_schema() -> Schema {
    Schema {
        name: ident("TestSchema"),
        entities: vec![],
        records: vec![],
        annotations: Annotations::default(),
    }
}

// A constraint's name is now type-level (`const NAME`), so distinct names
// require distinct types.
struct NoopA;
impl Constraint for NoopA {
    const NAME: &'static str = "a";
    fn validate(&self, _schema: &Schema) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

struct NoopB;
impl Constraint for NoopB {
    const NAME: &'static str = "b";
    fn validate(&self, _schema: &Schema) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

// A constraint reports failures through any error type; here a minimal one.
#[derive(Debug)]
struct Boom(&'static str);
impl std::fmt::Display for Boom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for Boom {}

struct Failing(&'static str);
impl Constraint for Failing {
    const NAME: &'static str = "a";
    fn validate(&self, _schema: &Schema) -> Result<(), Box<dyn std::error::Error>> {
        Err(Box::new(Boom(self.0)))
    }
}

#[test]
fn empty_validator_on_empty_schema_passes() {
    assert!(Validator::default().validate(&empty_schema()).is_ok());
}

#[test]
fn try_with_rejects_duplicate_name() {
    let err = Validator::default()
        .try_with(NoopA)
        .unwrap()
        .try_with(NoopA)
        .err();
    assert!(matches!(err, Some(Error::DuplicateConstraint("a"))));
}

#[test]
fn try_with_accepts_distinct_names() {
    Validator::default()
        .try_with(NoopA)
        .unwrap()
        .try_with(NoopB)
        .unwrap();
}

#[test]
fn constraint_without_validator_is_unregistered() {
    let schema = Schema {
        annotations: Annotations {
            constraints: vec![constraint("unknown")],
            ..Default::default()
        },
        ..empty_schema()
    };
    let Error::Invalid {
        unregistered,
        failures,
    } = Validator::default().validate(&schema).unwrap_err()
    else {
        panic!("expected Error::Invalid");
    };
    assert_eq!(unregistered.len(), 1);
    assert!(unregistered.contains("unknown"));
    assert!(failures.is_empty());
}

#[test]
fn metadata_is_never_validated() {
    let schema = Schema {
        annotations: Annotations {
            metadata: vec![metadata("not_validated")],
            ..Default::default()
        },
        ..empty_schema()
    };
    assert!(Validator::default().validate(&schema).is_ok());
}

#[test]
fn unregistered_constraint_is_reported_once() {
    let unknown = || Annotations {
        constraints: vec![constraint("unknown")],
        ..Default::default()
    };
    let schema = Schema {
        name: ident("S"),
        annotations: unknown(),
        entities: vec![Entity {
            name: ident("E"),
            annotations: unknown(),
            events: vec![Event {
                name: ident("Ev"),
                cardinality: Cardinality::Once,
                annotations: unknown(),
                payload: vec![Field {
                    name: ident("ef"),
                    ty: DataType::U64,
                    annotations: unknown(),
                }],
            }],
        }],
        records: vec![Record {
            name: ident("R"),
            annotations: unknown(),
            fields: vec![Field {
                name: ident("rf"),
                ty: DataType::U64,
                annotations: unknown(),
            }],
        }],
    };
    let Error::Invalid { unregistered, .. } = Validator::default().validate(&schema).unwrap_err()
    else {
        panic!("expected Error::Invalid");
    };
    // The same name used at six sites is deduplicated to a single entry.
    assert_eq!(
        unregistered.into_iter().collect::<Vec<_>>(),
        vec!["unknown".to_string()]
    );
}

#[test]
fn constraint_failures_are_collected() {
    let Error::Invalid { failures, .. } = Validator::default()
        .try_with(Failing("boom"))
        .unwrap()
        .try_with(NoopB)
        .unwrap()
        .validate(&empty_schema())
        .unwrap_err()
    else {
        panic!("expected Error::Invalid");
    };
    assert_eq!(failures.len(), 1);
    let (name, source) = &failures[0];
    assert_eq!(*name, "a");
    assert_eq!(source.to_string(), "boom");
}

#[test]
fn unregistered_and_constraint_failures_aggregate() {
    let schema = Schema {
        annotations: Annotations {
            constraints: vec![constraint("unknown")],
            ..Default::default()
        },
        ..empty_schema()
    };
    let Error::Invalid {
        unregistered,
        failures,
    } = Validator::default()
        .try_with(Failing("boom"))
        .unwrap()
        .validate(&schema)
        .unwrap_err()
    else {
        panic!("expected Error::Invalid");
    };
    assert!(unregistered.contains("unknown"));
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].0, "a");
}
