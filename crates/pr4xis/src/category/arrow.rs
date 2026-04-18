//! The `Arrow` trait — type-level unification of pr4xis's 2-category cells.
//!
//! Per Mac Lane (1971) *Categories for the Working Mathematician* XII.3,
//! Cat is a 2-category whose 0-cells are categories, 1-cells are functors,
//! and 2-cells are natural transformations. Adjunctions are structured
//! 2-cell pairs. **Every cell at every dimension has a source, a target,
//! and a kind.** This trait names that shared shape.
//!
//! # Coverage
//!
//! - **Object-level morphisms** (`Category::Morphism`) — Arrow impl emitted
//!   by `pr4xis::ontology!` / `define_category!` alongside `Relationship`.
//! - **Functors** (1-cells in Cat) — Arrow via blanket `impl<F: Functor>`.
//! - **Adjunctions** (structured 2-cell pairs in Cat) — Arrow impl emitted
//!   by `pr4xis::adjunction!` and written manually for hand-rolled adjunctions.
//! - **Natural transformations** (2-cells in Cat) — Arrow impl via the
//!   `pr4xis::natural_transformation!` macro or manual.
//!
//! # Literature
//!
//! - Mac Lane (1971) XII.3 — Cat as a 2-category
//! - Bénabou (1967) *Introduction to Bicategories* — first systematic treatment
//! - Awodey (2010) *Category Theory* §7 — 2-categories and Cat
//! - Leinster (2004) *Higher Operads, Higher Categories* — modern n-category textbook
//! - Gruber (1993) KAS 5 — "ontology = formally-named relations" at every dimension

use core::fmt::Debug;

use super::functor::Functor;
use super::kinds::FunctorKind;
use crate::ontology::meta::RelationshipMeta;

/// A cell in a 2-category — shared shape across morphisms, functors,
/// natural transformations, and adjunctions.
///
/// Each cell declares:
/// - [`Source`](Arrow::Source) — the domain / starting object
/// - [`Target`](Arrow::Target) — the codomain / ending object
/// - [`Kind`](Arrow::Kind) — the relation-kind classification
/// - [`meta`](Arrow::meta) — the unified Lemon + PROV-O record
///
/// `Source` / `Target` live at different ambient levels depending on the cell:
/// - For `Category::Morphism` — objects (0-cells in the ambient category)
/// - For `Functor` — categories (0-cells in Cat)
/// - For `NaturalTransformation` and `Adjunction` — functors (1-cells in Cat)
pub trait Arrow: Sized {
    /// Domain of the arrow — what it comes from.
    type Source;
    /// Codomain of the arrow — what it goes to.
    type Target;
    /// Relation-kind tag — `RelationKind` for 1-cell morphisms, `FunctorKind`
    /// for functors, `NatTransKind` for natural transformations, `AdjunctionKind`
    /// for adjunctions.
    type Kind: Copy + Debug + Eq;

    /// Structured metadata — Lemon + PROV-O record shared across every arrow.
    fn meta() -> RelationshipMeta;
}

/// Every [`Functor`] is an [`Arrow`] — 1-cell in the 2-category Cat.
///
/// Blanket impl: Mac Lane XII.3 says Cat has functors as 1-cells; this
/// impl realises that claim at the Rust trait level. Every Functor impl
/// in the workspace automatically picks up Arrow without any per-site
/// boilerplate.
///
/// # Why asymmetric?
///
/// `Adjunction` and `NaturalTransformation` don't get blanket Arrow impls
/// because Rust's coherence rules reject multiple blanket impls over a
/// single trait — even when the trait bounds are in practice disjoint.
/// Per-impl `Arrow for XxxAdjunction` / `Arrow for YyyNatTrans` blocks
/// are emitted by the `pr4xis::adjunction!` / `pr4xis::natural_transformation!`
/// macros, and added manually for hand-rolled impls.
impl<F: Functor> Arrow for F {
    type Source = <F as Functor>::Source;
    type Target = <F as Functor>::Target;
    type Kind = FunctorKind;

    fn meta() -> RelationshipMeta {
        <F as Functor>::meta()
    }
}
