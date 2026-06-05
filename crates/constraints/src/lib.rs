// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! # Constraint trait and validation for [`Schema`]s.

use std::collections::{BTreeSet, HashMap, hash_map::Entry};

use quent_schema::{Schema, constraint::Constraint as SchemaConstraint, data_type::DataType};

/// A trait for types that implement a "constraint" of an application event
/// model.
///
/// A constraint is a rule imposed on an application event model. It is conveyed
/// through opaque data attached to the constituents of a [`Schema`] as
/// [`quent_schema::constraint::Constraint`]s.
///
/// By applying the constraint to a model, the model gains properties that need
/// to be validated against the entire schema, which is the main purpose of this
/// trait.
///
/// Constraints are leveraged for a wide variety of purposes. For more details,
/// see [`quent_schema`].
///
/// The canonical validation flow is orchestrated by the [`Validator`].
pub trait Constraint {
    /// A unique name for this constraint.
    ///
    /// While no restrictions are imposed on constraint names (other than that
    /// they are valid UTF-8 strings) it is recommended to follow the
    /// human-readable dot-separated pattern `project.constraint.version`. For
    /// example: `quent.fsm.v1`. This reduces the probability of name clashes
    /// between dependencies, and provides a means of easily detecting breaking
    /// changes to the constraint's own schema.
    const NAME: &'static str;

    /// Validate this constraint against `schema`.
    fn validate(&self, schema: &Schema) -> Result<(), Box<dyn std::error::Error>>;
}

/// The error type produced by this crate.
#[derive(Debug)]
pub enum Error {
    /// A [`Constraint`] was registered under a name already in use.
    DuplicateConstraint(&'static str),
    /// Validation failed.
    Invalid {
        /// Constraint names used by the schema with no registered validator.
        unregistered: BTreeSet<String>,
        /// Failures reported by registered constraints.
        failures: Vec<(&'static str, Box<dyn std::error::Error>)>,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DuplicateConstraint(name) => {
                write!(f, "duplicate registration of constraint: \"{name}\"")
            }
            Error::Invalid {
                unregistered,
                failures,
            } => {
                writeln!(f, "schema failed to validate:")?;
                for name in unregistered {
                    writeln!(f, "unregistered constraint: \"{name}\"")?;
                }
                for (name, source) in failures {
                    writeln!(f, "constraint \"{name}\": {source}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for Error {}

type ConstraintFn = Box<dyn Fn(&Schema) -> Result<(), Box<dyn std::error::Error>>>;

/// Validates registered [`Constraint`]s.
///
/// Validation will fail when:
/// - a constraint used by the schema isn't registered by the validator, or
/// - the rule of a registered constraint is unmet
///
/// # Example: validate in a `build.rs`
///
/// ```ignore
/// let validator = Validator::default()
///     .try_with(MyConstraint)?; // register a constraint implemented elsewhere
///
/// if let Err(error) = validator.validate(&schema) {
///     panic!("{error}");
/// }
/// ```
#[derive(Default)]
pub struct Validator {
    constraints: HashMap<&'static str, ConstraintFn>,
}

impl Validator {
    /// Register a [`Constraint`] to be validated.
    pub fn try_with<C: Constraint + 'static>(mut self, constraint: C) -> Result<Self, Error> {
        match self.constraints.entry(C::NAME) {
            Entry::Occupied(_) => Err(Error::DuplicateConstraint(C::NAME)),
            Entry::Vacant(entry) => {
                entry.insert(Box::new(move |schema: &Schema| constraint.validate(schema)));
                Ok(self)
            }
        }
    }

    /// Run validation of all registered constraints against `schema`.
    pub fn validate(&self, schema: &Schema) -> Result<(), Error> {
        // First, walk the entire schema to figure out if it uses any
        // unregistered constraints.
        let mut unregistered = BTreeSet::new();
        let mut check = |constraints: &[SchemaConstraint]| {
            for constraint in constraints {
                if !self.constraints.contains_key(constraint.name.as_str()) {
                    unregistered.insert(constraint.name.clone());
                }
            }
        };
        check(&schema.annotations.constraints);
        for entity in &schema.entities {
            check(&entity.annotations.constraints);
            for event in &entity.events {
                check(&event.annotations.constraints);
                for field in &event.payload {
                    check(&field.annotations.constraints);
                    check_entity_refs(&field.ty, &mut check);
                }
            }
        }
        for record in &schema.records {
            check(&record.annotations.constraints);
            for field in &record.fields {
                check(&field.annotations.constraints);
                check_entity_refs(&field.ty, &mut check);
            }
        }

        // Second, validate
        let mut failures = Vec::new();
        for (name, validate) in &self.constraints {
            if let Err(source) = validate(schema) {
                failures.push((*name, source));
            }
        }

        if unregistered.is_empty() && failures.is_empty() {
            Ok(())
        } else {
            Err(Error::Invalid {
                unregistered,
                failures,
            })
        }
    }
}

fn check_entity_refs(ty: &DataType, check: &mut impl FnMut(&[SchemaConstraint])) {
    match ty {
        DataType::Option(inner) | DataType::List(inner) => check_entity_refs(inner, check),
        DataType::EntityRef { data, annotations } => {
            check(&annotations.constraints);
            if let Some(inner) = data {
                check_entity_refs(inner, check);
            }
        }
        // this doesn't need to go into records as this is walked from the top-level
        _ => {}
    }
}
