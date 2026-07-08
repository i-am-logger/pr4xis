//! Lens ontology — the bidirectional transformation `(get, put)`, its
//! well-behaved-lens laws, and the *category* those lenses form under
//! sequential composition, made a first-class praxis ontology with runnable
//! axioms (North Star W3 slice 4;
//! `feedback_praxis_as_compiler_self_describing`).
//!
//! The lens is the central abstraction of praxis's load / emit / projection
//! architecture: [`WellBehavedLens`](crate::formal::meta::well_behaved_lens::WellBehavedLens)
//! is the *signature of understanding* for every loaded source (`bytes ⇆
//! ontology`), [`Lens`](crate::formal::meta::lens_composition::Lens) is the
//! general `Source ⇆ View` pair whose composite is the
//! `file ⇄ xml ⇄ xsd ⇄ statute` pipeline, and `ArchiveLens`
//! (`pr4xis_runtime`) is the `rkyv` runtime-cache lens. Their laws are proven
//! as registered axioms — `RoundTripHarnessAllVerified` (every registered
//! [`WellBehavedLens`] passes PutGet against `praxis.lock`), `EmitLoadWellBehaved`
//! (the archive-storage GetPut leg), and `ArchiveLensGetPut` / `ArchiveLensPutGet`
//! (the runtime cache lens) — but the LENS itself was named in no `ontology!`:
//! `Lens`, `WellBehavedLens`, `GetPut`, `PutGet`, `PutPut`,
//! `SequentialComposition` and `RoundTripFidelity` were not discoverable
//! concepts, and the general lens *algebra* — that lenses form a category
//! (composition preserves well-behavedness, is associative, with the identity
//! lens as unit; Foster et al. 2007 §3) — was proven only in `#[test]`
//! helpers, registered as no axiom.
//!
//! This module closes both gaps, exactly as [`super::canonical_codec`] did for
//! the codec:
//!
//! - [`ontology`](self) — the concepts and their kinded morphisms (a
//!   `WellBehavedLens` *depends on* the `GetPut` and `PutGet` laws; a
//!   `VeryWellBehavedLens` also on `PutPut`; a `SequentialComposition` depends
//!   on its component `Lens`es; a `RoundTripFidelity` grade refines which
//!   `PutGet` the harness holds a lens to). Always compiled — the general
//!   [`Lens`](crate::formal::meta::lens_composition::Lens) trait and its law
//!   checkers are unconditional domains code (no `feature = "prx"` gate).
//! - [`super::axioms`] — the six runnable predicates lifting the general lens
//!   algebra into registered, discoverable axioms: the three well-behaved-lens
//!   laws (`LensGetPutLaw`, `LensPutGetLaw`, `LensPutPutLaw`) and the three
//!   category laws (`LensCompositionWellBehaved`, `LensCompositionAssociative`,
//!   `LensIdentityUnit`), each `verify()`ing over REAL lens values with teeth
//!   (a deliberately mis-paired lens is rejected; a well-behaved-but-not-very-
//!   well-behaved lens fails `PutPut`).
//!
//! The byte-anchored and archive lens laws are NOT re-run here (they are owned
//! by, and verified against the real realisation in,
//! [`super::well_behaved_lens::harness`] and [`super::ontology_archive`], and
//! `RoundTripHarnessAllVerified` reads the on-disk corpus). They are registered
//! through the same `register_axiom!` / `axiom_by_name` machinery and cite the
//! same Foster et al. 2007 §3 laws, so the whole lens-law family — general
//! algebra, byte-anchored round-trip, archive round-trip — resolves through the
//! one registry the Lens concept vocabulary now names.
//!
//! # Literature
//!
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators for
//!   Bidirectional Tree Transformations: A Linguistic Approach to the
//!   View-Update Problem", *ACM TOPLAS* 29(3) Article 17 — §3 "Semantic
//!   Foundations": Def. 3.2 (the well-behaved-lens laws GetPut / PutGet; the
//!   very-well-behaved PutPut law follows in §3) and, later in §3, sequential
//!   composition, the identity lens, and that lenses form a category.
//! - **Pierce (2006)** *Lenses* lecture notes — the lens category.
//! - **Bancilhon & Spyratos (1981)** "Update Semantics of Relational Views",
//!   *ACM TODS* 6(4) — the constant complement the `RawBytesComplementFloor`
//!   fidelity records.
//!
//! The lens is treated here as an ALGEBRAIC structure (a `(get, put)` pair
//! obeying the well-behaved laws), not as an adjunction: `get ⊣ put` is NOT a
//! theorem of lens theory, so no adjunction / equivalence-of-categories law is
//! claimed.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Lens",
    source: "Foster, Greenwald, Moore, Pierce & Schmitt (2007) Combinators for Bidirectional Tree Transformations: A Linguistic Approach to the View-Update Problem, ACM TOPLAS 29(3) Article 17 §3 (Semantic Foundations: well-behaved-lens laws Def. 3.2; composition, identity, the lens category); Pierce (2006) Lenses lecture notes; Bancilhon & Spyratos (1981) Update Semantics of Relational Views, ACM TODS 6(4)",

    concepts: [
        Lens,
        GetPut,
        PutGet,
        PutPut,
        WellBehavedLens,
        VeryWellBehavedLens,
        SequentialComposition,
        IdentityLens,
        RoundTripFidelity,
    ],

    labels: {
        Lens: ("en", "Lens",
            "Foster et al. (2007) §3, Def. 3.2: a bidirectional transformation between a source S and a view V — a pair (get : S → V, put : V × S → S) that extracts a view and writes an updated view back into a source."),
        GetPut: ("en", "GetPut law",
            "Foster et al. (2007) §3, Def. 3.2: put(get(s), s) = s — putting back an unchanged view leaves the source untouched (no phantom edit)."),
        PutGet: ("en", "PutGet law",
            "Foster et al. (2007) §3, Def. 3.2: get(put(v, s)) = v — getting a just-put view returns exactly that view (the write is faithful up to the source-equivalence, e.g. canonical form or byte identity)."),
        PutPut: ("en", "PutPut law",
            "Foster et al. (2007) §3, Def. 3.2: put(v', put(v, s)) = put(v', s) — a later put overwrites an earlier one; successive puts are idempotent in source space. The extra law of a VERY well-behaved lens."),
        WellBehavedLens: ("en", "Well-behaved lens",
            "Foster et al. (2007) §3, Def. 3.2: a lens satisfying GetPut and PutGet. In praxis the runtime signature-of-understanding for a loaded source: get parses bytes into the ontology, put re-emits, and the round-trip preserves the source up to its published canonical form."),
        VeryWellBehavedLens: ("en", "Very well-behaved lens",
            "Foster et al. (2007) §3, Def. 3.2: a well-behaved lens that ALSO satisfies PutPut. A well-behaved lens need not be very well behaved — one whose put stashes prior state into a complement obeys GetPut and PutGet yet violates PutPut."),
        SequentialComposition: ("en", "Sequential composition",
            "Foster et al. (2007) §3: the composite l ; k of l : S ⇆ V and k : V ⇆ W, a lens S ⇆ W with get = k.get ∘ l.get and put(w, s) = l.put(k.put(w, l.get(s)), s). Composition of well-behaved lenses is well-behaved and is associative — the lenses form a category."),
        IdentityLens: ("en", "Identity lens",
            "Foster et al. (2007) §3: the lens S ⇆ S with get = id and put = fst; the unit of sequential composition (id ; l = l = l ; id)."),
        RoundTripFidelity: ("en", "Round-trip fidelity",
            "praxis M4.ι: the grade of PutGet a well-behaved lens is held to — ByteExactGraphFaithful (put(get(b)) = b byte-for-byte, reconstructed from the graph alone) or the RawBytesComplementFloor (byte identity via a stored constant complement, Bancilhon & Spyratos 1981)."),
    },

    // Subsumption: a well-behaved lens is a lens; a very-well-behaved lens is a
    // well-behaved one; the identity lens is a lens (Foster et al. 2007 §3).
    is_a: [
        (WellBehavedLens, Lens),
        (VeryWellBehavedLens, WellBehavedLens),
        (IdentityLens, Lens),
    ],

    // Kinded dependency morphisms (OBO-RO `depends on`, as in
    // `super::canonical_codec`'s ContentAddress → CanonicalEncoding). These are
    // traversable edges of the lens graph, NOT laws: well-behavedness DEPENDS ON
    // the GetPut/PutGet laws holding, and the very-well-behaved refinement on
    // PutPut; a composite DEPENDS ON its component lenses; the fidelity grade
    // refines which PutGet the harness enforces. The laws themselves are the
    // runnable axioms in `super::axioms` — deliberately not restated as
    // `f(x) == f(x)` equalities here.
    edges: [
        (WellBehavedLens, GetPut, Dependency),
        (WellBehavedLens, PutGet, Dependency),
        (VeryWellBehavedLens, PutPut, Dependency),
        (SequentialComposition, Lens, Dependency),
        (RoundTripFidelity, PutGet, Dependency),
    ],
}

/// Quality: a short symbolic description of each lens concept, matching the
/// citation column in the ontology header.
#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = LensConcept;
    type Value = &'static str;

    fn get(&self, c: &LensConcept) -> Option<&'static str> {
        use LensConcept as C;
        Some(match c {
            C::Lens => "a (get : S → V, put : V × S → S) pair (Foster et al. 2007 §3)",
            C::GetPut => "put(get(s), s) == s — no phantom edit (Foster 2007 §3)",
            C::PutGet => "get(put(v, s)) == v — the write is faithful (Foster 2007 §3)",
            C::PutPut => "put(v', put(v, s)) == put(v', s) — puts are idempotent (Foster 2007 §3)",
            C::WellBehavedLens => "a lens satisfying GetPut + PutGet (Foster 2007 §3)",
            C::VeryWellBehavedLens => {
                "a well-behaved lens that also satisfies PutPut (Foster 2007 §3)"
            }
            C::SequentialComposition => {
                "l ; k — well-behaved, associative; lenses form a category (Foster 2007 §3)"
            }
            C::IdentityLens => "get = id, put = fst — the composition unit (Foster 2007 §3)",
            C::RoundTripFidelity => "byte-exact vs raw-bytes-complement PutGet grade (praxis M4.ι)",
        })
    }
}

impl Ontology for LensOntology {
    type Cat = LensCategory;
    type Qual = ConceptDescription;

    fn axioms() -> alloc::vec::Vec<alloc::boxed::Box<dyn Axiom>> {
        // The general `Lens` trait and its law checkers
        // (`crate::formal::meta::lens_composition`) are unconditional domains
        // code, so these six predicates run against the real lens machinery in
        // EVERY build — no `feature = "prx"` gate (contrast
        // `super::ontology_archive`, whose realisation lives behind `.prx`).
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        use super::axioms::{
            LensCompositionAssociative, LensCompositionWellBehaved, LensGetPutLaw,
            LensIdentityUnit, LensPutGetLaw, LensPutPutLaw,
        };
        axioms.push(alloc::boxed::Box::new(LensGetPutLaw));
        axioms.push(alloc::boxed::Box::new(LensPutGetLaw));
        axioms.push(alloc::boxed::Box::new(LensPutPutLaw));
        axioms.push(alloc::boxed::Box::new(LensCompositionWellBehaved));
        axioms.push(alloc::boxed::Box::new(LensCompositionAssociative));
        axioms.push(alloc::boxed::Box::new(LensIdentityUnit));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::ontology::registry::{axiom_by_name, describe_knowledge_base};

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<LensCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        // Runs the category laws AND all six lens-algebra axioms' `verify()`
        // against the real `crate::formal::meta::lens_composition` machinery.
        LensOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nine_concepts() {
        assert_eq!(LensConcept::variants().len(), 9);
    }

    /// The lens is reasoned about through the SAME registry as any statute: the
    /// ontology is discoverable in `VOCABULARIES`, and each of this ontology's
    /// six lens-algebra axioms re-binds by name through `axiom_by_name` (the
    /// load-time rebind gate) — so "what laws does the lens obey?" is a graph
    /// query, not opaque runtime code. This is what making the lens a
    /// first-class self-describing concept buys.
    ///
    /// The pre-existing byte-anchored / archive lens laws
    /// (`RoundTripHarnessAllVerified`, `EmitLoadWellBehaved`,
    /// `ArchiveLensGetPut` / `ArchiveLensPutGet`) are registered through the
    /// same `register_axiom!` / `axiom_by_name` machinery and cite the same
    /// Foster et al. 2007 §3 laws, so they resolve through this one registry
    /// alongside these — the whole lens-law family is one query. They are not
    /// asserted here because their `register_axiom!` constructors are dead-strip
    /// eligible in a filtered unit-test binary that never references their
    /// modules (a linker artefact, not a registry gap); the `all_archive_axioms_hold`
    /// / `harness` tests own their in-binary verification.
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn discoverable_via_self_model() {
        assert!(
            describe_knowledge_base()
                .iter()
                .any(|v| v.name() == "LensOntology"),
            "Lens must be discoverable in the ontology registry"
        );
        // This ontology's own six lens-algebra axioms.
        for axiom in [
            "LensGetPutLaw",
            "LensPutGetLaw",
            "LensPutPutLaw",
            "LensCompositionWellBehaved",
            "LensCompositionAssociative",
            "LensIdentityUnit",
        ] {
            assert!(
                axiom_by_name(axiom).is_some(),
                "lens axiom {axiom} must re-bind through the registry (axiom_by_name)"
            );
        }
    }
}
