// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! # Constraint trait and validation for [`Schema`]s.

pub mod utils;

mod recursive_record;
mod unregistered_constraints;
mod unresolved_refs;

use quent_schema::{Schema, visitor::Visitor};

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
/// The canonical validation flow is orchestrated by [`validate`].
pub trait Constraint: Visitor + Default {
    /// A unique name for this constraint.
    ///
    /// While no restrictions are imposed on constraint names (other than that
    /// they are valid UTF-8 strings) it is recommended to follow the
    /// human-readable dot-separated pattern `project.constraint.version`. For
    /// example: `quent.fsm.v1`. This reduces the probability of name clashes
    /// between dependencies, and provides a means of easily detecting breaking
    /// changes to the constraint's own schema.
    const NAME: &'static str;
}

/// The outcome of [`validate`].
pub struct Report<R> {
    /// Constraint names referenced by the schema that no validated constraint
    /// handles.
    pub unregistered_constraints: Vec<String>,
    /// Invalid references
    pub invalid_references: Vec<String>,
    /// Records that are recursive
    pub recursive_records: Vec<String>,
    /// Each constraint's own result, in tuple order matching the validated
    /// constraints.
    pub results: R,
}

/// Validates (a tuple of) [`Constraint`]s against `schema`.
///
/// The returned validation [`Report`] always includes:
/// - the result of an internal consistency check of the schema (i.e. internal
///   references properly resolve)
/// - a list of unregistered constraints
///
/// Results from additional constraints are gathered in [`Report::results`].
///
/// ```
/// use quent_constraints::validate;
/// use quent_schema::{Identifier, Schema, builder::SchemaBuilder};
/// # use quent_constraints::Constraint;
/// # use quent_schema::visitor::{Cursor, Visitor};
/// #
/// # #[derive(Default)]
/// # struct DocConstraint;
/// # impl Visitor for DocConstraint {
/// #     type Output = Result<(), String>;
/// #     fn visit(&mut self, _cursor: &Cursor) {}
/// #     fn finish(self) -> Self::Output {
/// #         Ok(())
/// #     }
/// # }
/// # impl Constraint for DocConstraint {
/// #     const NAME: &'static str = "quent.docs.constraint.v1";
/// # }
/// # type ConstraintA = DocConstraint;
/// # type ConstraintB = DocConstraint;
/// #
/// # let schema: Schema =
/// #     SchemaBuilder::new(Identifier::try_new("MySchema").unwrap()).build();
///
/// let report = validate::<(ConstraintA, ConstraintB)>(&schema);
/// let (result_a, result_b) = report.results;
/// assert!(result_a.is_ok());
/// assert!(result_b.is_ok());
/// assert!(report.unregistered_constraints.is_empty());
/// assert!(report.invalid_references.is_empty());
/// assert!(report.recursive_records.is_empty());
/// ```
pub fn validate<C: Constraints>(schema: &Schema) -> Report<C::Output> {
    let (invalid_references, unregistered_constraints, recursive_records, results) = schema.walk((
        unresolved_refs::UnresolvedReferences::default(),
        unregistered_constraints::UnregisteredConstraints::new(C::NAMES),
        recursive_record::RecursiveRecords::default(),
        C::default(),
    ));
    Report {
        unregistered_constraints: unregistered_constraints.into_iter().collect(),
        invalid_references,
        recursive_records,
        results,
    }
}

/// A tuple of [`Constraint`]s that can be validated together in one walk.
pub trait Constraints: Visitor + Default {
    /// The [`Constraint::NAME`] of every constraint in the tuple.
    const NAMES: &'static [&'static str];
}

// Enables validate::<()>(schema)
impl Constraints for () {
    const NAMES: &'static [&'static str] = &[];
}
macro_rules! constraints_impls {
    ($($T:ident),+) => {
        impl<$($T: Constraint),+> Constraints for ($($T,)+) {
            const NAMES: &'static [&'static str] = &[$($T::NAME),+];
        }
    };
}
constraints_impls!(A);
constraints_impls!(A, B);
constraints_impls!(A, B, C);
constraints_impls!(A, B, C, D);
constraints_impls!(A, B, C, D, E);
constraints_impls!(A, B, C, D, E, F);
constraints_impls!(A, B, C, D, E, F, G);
constraints_impls!(A, B, C, D, E, F, G, H);
constraints_impls!(A, B, C, D, E, F, G, H, I);
constraints_impls!(A, B, C, D, E, F, G, H, I, J);
constraints_impls!(A, B, C, D, E, F, G, H, I, J, K);
constraints_impls!(A, B, C, D, E, F, G, H, I, J, K, L);
