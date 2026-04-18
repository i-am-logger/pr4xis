//! Kind-tags for arrows at every cell-level of Cat (issue #153).
//!
//! Per Gruber (1993) KAS 5 — "ontology = formally-named relations" —
//! every arrow in pr4xis carries a relation-kind tag. At 1-cell level
//! this is `RelationKind` (Subsumption, Parthood, etc. from the
//! Relations ontology). At the 1-cells-in-Cat, 2-cells-in-Cat, and
//! structured-2-cell-pair levels, we need analogous kind enums.
//!
//! References:
//! - Mac Lane (1971) *Categories for the Working Mathematician* I.3, IV.1, I.4
//! - Awodey (2010) *Category Theory* §7, §9
//! - Smith et al. (2005) OBO Relation Ontology (principle: every
//!   relation has a canonical named type)

/// Classification of a functor F: C → D (Mac Lane 1971 I.3; Awodey §7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctorKind {
    /// Injective on each Hom-set: F: Hom(A,B) → Hom(F A, F B) is injective.
    Faithful,
    /// Surjective on each Hom-set.
    Full,
    /// Both faithful and full — the strongest "embedding" classification.
    FullyFaithful,
    /// Discards structure (e.g., Group → Set, forgetting the group operation).
    Forgetful,
    /// Left adjoint to a forgetful functor (Mac Lane IV.1 free-forgetful paradigm).
    Free,
    /// Trivial self-functor `Id_C: C → C`.
    Identity,
    /// `Hom(A, -): C → Set` for some fixed A — the Yoneda-representable functor.
    Representable,
    /// Inclusion functor from a subcategory.
    Inclusion,
    /// Not further classified.
    Generic,
}

/// Classification of a natural transformation η: F ⇒ G
/// (Mac Lane 1971 I.4; Awodey §7.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NatTransKind {
    /// Every component η_A is iso in the target category.
    NaturalIsomorphism,
    /// Every component η_A is a monomorphism.
    NaturalMonomorphism,
    /// Every component η_A is an epimorphism.
    NaturalEpimorphism,
    /// The canonical unit η: Id_C ⇒ G∘F of an adjunction.
    AdjunctionUnit,
    /// The canonical counit ε: F∘G ⇒ Id_D of an adjunction.
    AdjunctionCounit,
    /// Not further classified.
    Generic,
}

/// Classification of an adjunction F ⊣ G
/// (Mac Lane 1971 IV.3; Awodey §9.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdjunctionKind {
    /// Unit η is a natural isomorphism — C embeds as reflective subcategory of D.
    Reflection,
    /// Counit ε is a natural isomorphism — D embeds as coreflective subcategory of C.
    Coreflection,
    /// Both unit and counit are natural isos — C and D are categorically equivalent.
    Equivalence,
    /// Free-forgetful paradigm: F constructs the free object, G forgets structure.
    FreeForgetful,
    /// Not further classified.
    Generic,
}
