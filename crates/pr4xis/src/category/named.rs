//! [`NamedCategory`] — a [`Category`] that knows its own ontology name.
//!
//! # Why a separate trait
//!
//! [`Category`](super::category::Category) is the bare categorical structure:
//! objects, morphisms, identity, composition. It deliberately carries no name —
//! a category is a mathematical object, not a named ontology, and the 41
//! hand-rolled `impl Category` blocks across the workspace must stay valid
//! without a new required method.
//!
//! But a [`Functor`](super::functor::Functor) `F: C → D` between two *ontologies*
//! needs both endpoints' ontology NAMES to be serialized as a cross-ontology
//! [`Connection`](../../../pr4xis_runtime/connection/struct.Connection.html):
//! its `source`/`target` reference the OTHER ontology by name, so a peer can
//! rebind it by content-address agreement (the p2p-ready wire form). The name
//! cannot come from `core::any::type_name` (toolchain-bound, and the praxis-way
//! forbids string-munged identity); it must be the ontology's own DECLARED
//! [`OntologyName`].
//!
//! `NamedCategory` carries exactly that — `Category` + the declared name. The
//! `ontology!` proc-macro and the `define_category!` declarative macro
//! auto-implement it for every category they generate (from the `name:` the
//! author already wrote), so the common path needs no hand-written impl. A
//! hand-rolled `impl Category` that participates in a functor/adjunction as a
//! source/target adds a one-line `impl NamedCategory` declaring its name — the
//! same `OntologyName` its `Vocabulary` registers under.
//!
//! # Literature
//!
//! - Gruber (1993) *A Translation Approach to Portable Ontology Specifications*
//!   KAS 5 — an ontology is a named, formally-specified conceptualization.
//! - ONTOLEX-Lemon (W3C 2016) — a name is a lexical entry with a canonical form.

use super::category::Category;
use crate::ontology::meta::OntologyName;

/// A [`Category`] that knows the [`OntologyName`] it was declared under.
///
/// The name is the ontology's identity at the connection layer: a functor
/// between two `NamedCategory`s serializes each endpoint's `ontology_name()`
/// into its [`Connection`]'s `source`/`target`, so the cross-ontology arrow is
/// content-addressed by the OTHER ontology's declared name — never by a
/// toolchain-bound type name. The declared name must agree with the name the
/// category's `Vocabulary` registers under (the same `name:` literal the author
/// wrote in `ontology!` / `define_category!`).
pub trait NamedCategory: Category {
    /// This category's ontology name — its declared identity.
    fn ontology_name() -> OntologyName;
}
