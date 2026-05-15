//! The `Arrow` trait — directed structures carrying kind and per-instance
//! provenance.
//!
//! # Ontological grounding
//!
//! Arrow is the Rust realisation of
//! [`crate::category::category_theory::CategoryTheoryConcept::Morphism`].
//! Mac Lane (1971) CWM Ch. I §1 uses "morphism" and "arrow" as synonyms;
//! Awodey (2010) uses "arrow" as primary. The Rust trait is `Arrow`
//! (avoids naming collision with the `Morphism` struct) and both names
//! point at the same ontological concept.
//!
//! The previous `Relationship` trait (Chen 1976 Entity-Relationship
//! database modelling vocabulary) has been absorbed — "relationship"
//! is ER modelling, not the category-theoretic or ontology-engineering
//! literature the rest of pr4xis cites. `Arrow` replaces it.
//!
//! # Per-instance provenance (required)
//!
//! Every arrow carries its own [`Provenance`] via [`Arrow::meta`]. Per
//! Gruber (1993) KAS 5 "formally-named relations", Smith et al. (2005)
//! OBO-RO "every relation-instance is named", W3C PROV-O (2013) "every
//! entity has provenance", W3C SKOS (2009) "every concept mapping has
//! labels + notes" — no anonymous arrows. `meta(&self)` is required
//! on every impl.
//!
//! # Dimensional scope
//!
//! Arrow is for **morphism-level** cells (0-cells to 0-cells within a
//! category). Higher cells — functors (1-cells in Cat), natural
//! transformations (2-cells), adjunctions (structured pairs) — keep
//! their own traits (`Functor`, `NaturalTransformation`, `Adjunction`)
//! with type-level `fn meta()`. Bénabou (1967) *Introduction to
//! Bicategories* distinguishes cells by dimension; their Rust encoding
//! follows.
//!
//! Literature:
//! - Mac Lane (1971) CWM Ch. I §1 — morphism / arrow
//! - Awodey (2010) *Category Theory* — "arrow" as primary
//! - Gruber (1993) KAS 5 — formally-named relations
//! - Smith et al. (2005) OBO-RO — relation-kinds with instance-level
//!   provenance
//! - W3C PROV-O (2013) — every entity has provenance
//! - W3C SKOS (2009) — every concept mapping has labels/notes

use core::fmt::Debug;

use super::entity::Concept;
use crate::ontology::meta::Provenance;

/// A directed structure between objects, carrying a relation kind and
/// per-instance provenance.
pub trait Arrow: Sized + Clone + Debug + Eq {
    /// The type of objects this arrow connects.
    type Object: Concept;

    /// The relation-kind tag — Subsumption, Parthood, Causation, etc.
    /// Per OBO-RO (Smith et al. 2005), every arrow has a canonical kind.
    type Kind: Copy + Debug + Eq;

    /// The domain of this arrow.
    fn source(&self) -> Self::Object;

    /// The codomain of this arrow.
    fn target(&self) -> Self::Object;

    /// The relation-kind carried by this arrow.
    fn kind(&self) -> Self::Kind;

    /// Per-instance provenance — name, description, citation, module
    /// path. Per Gruber / OBO-RO / PROV-O / SKOS — every arrow is a
    /// named, cited entity.
    ///
    /// The default uses `std::any::type_name::<Self>()` as an honest
    /// placeholder identifier with an empty citation — "no ontology
    /// source declared at this impl". The `ontology!` and
    /// `define_category!` macros override this to carry their
    /// ontology's source citation. Hand-written impls that represent
    /// a specific literature-grounded relation should override too.
    fn meta(&self) -> Provenance {
        let tn = core::any::type_name::<Self>();
        Provenance {
            name: crate::ontology::meta::OntologyName::new_static(tn),
            description: crate::ontology::meta::Label::new_static(tn),
            citation: crate::ontology::meta::Citation::EMPTY,
            module_path: crate::ontology::meta::ModulePath::new_static(module_path!()),
        }
    }
}
