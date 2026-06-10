//! [`NamedCategory`] — a [`Category`] that knows its own ontology name.
//!
//! # Why a separate trait
//!
//! [`Category`] is the bare categorical structure:
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

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::category::Category;
use crate::logic::axiom::Axiom;
use crate::ontology::meta::OntologyName;

/// A [`Category`] that knows the [`OntologyName`] it was declared under.
///
/// The name is the ontology's identity at the connection layer: a functor
/// between two `NamedCategory`s serializes each endpoint's `ontology_name()`
/// into its `Connection`'s `source`/`target`, so the cross-ontology arrow is
/// content-addressed by the OTHER ontology's declared name — never by a
/// toolchain-bound type name. The declared name must agree with the name the
/// category's `Vocabulary` registers under (the same `name:` literal the author
/// wrote in `ontology!` / `define_category!`).
pub trait NamedCategory: Category {
    /// This category's ontology name — its declared identity.
    fn ontology_name() -> OntologyName;
}

/// A [`NamedCategory`] that can reach the DOMAIN axioms its ontology declared.
///
/// # Why a sibling trait (not a method on [`NamedCategory`])
///
/// Praxis distinguishes two axiom families ([`catalog`](crate::ontology::reasoning)):
///
/// - **Structural** axioms come from the category's typed relation-kinds —
///   `structural_axioms_for::<Cat>()` derives them from the morphism graph
///   alone (Smith 2005 OBO-RO: every kind carries canonical axioms), so a bare
///   [`Category`] already determines them.
/// - **Domain** axioms are the per-`axioms:`-clause claims an author declares
///   inside `ontology! { … axioms: { … } }` — claims *specific to the subject
///   matter* (Guarino 2009: the axiomatisation layer, distinct from the
///   structural commitment). These live on the generated `<Name>Ontology`
///   struct's `generated_domain_axioms()` and are NOT reachable from the `Cat`
///   type alone.
///
/// This trait is the typed bridge: it wires the generated `Cat` to its
/// ontology's declared domain axioms, the same way [`NamedCategory`] wires it to
/// its declared name. The `ontology!` proc-macro auto-implements it for every
/// category it generates (delegating to `<Name>Ontology::generated_domain_axioms()`),
/// so a projection ([`emit`](../../../pr4xis_runtime/emit/index.html)) can serialize
/// the domain axioms as content-addressed `Axiom` nodes — keyed by each
/// [`Axiom::name`], the stable wire identity the load gate rebinds against — with
/// no string-munged identity and no per-ontology hand-wiring.
///
/// A category with an EMPTY `axioms:` clause (or none) auto-implements this with
/// an empty `domain_axioms()` — honest absence, never a fabricated axiom.
///
/// # Literature
///
/// - Guarino (2009) *The Ontological Level* — separates ontological commitment
///   (the structural layer) from axiomatisation (the domain-axiom layer this
///   trait surfaces).
/// - Gruber (1993) *A Translation Approach to Portable Ontology Specifications*
///   KAS 5 — an ontology specifies concepts, relations, AND axioms uniformly.
pub trait DomainAxiomatized: NamedCategory {
    /// The domain axioms this ontology declared in its `axioms:` clause — each a
    /// runnable [`Axiom`] carrying its own typed [`name`](Axiom::name),
    /// [`description`](Axiom::description), and [`citation`](Axiom::citation).
    /// Empty when the ontology declares none (honest absence). The macro fills
    /// this from the author's `axioms:` clause; it is never synthesised.
    fn domain_axioms() -> Vec<Box<dyn Axiom>>;
}
