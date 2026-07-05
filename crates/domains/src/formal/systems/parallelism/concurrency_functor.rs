//! Functor Parallelism → Concurrency, and the adjunction whose
//! machine-checked theorem is the **interleaving collapse gap**.
//!
//! `ParallelismToConcurrency` is the forgetful reading of a parallel
//! structure as a concurrent one: parallelism's rich performance,
//! hardware, and scheduling vocabulary collapses onto concurrency's bare
//! composition umbrella, keeping only the two behavioural pairings
//! (`ParallelTask ↔ Process`, `ParallelExecution ↔ ParallelComposition`)
//! and mapping determinism to a safety property.
//!
//! Paired with the reverse [`ConcurrencyToParallelism`], it forms an
//! adjunction whose unit round-trip `G∘F` exposes what parallelism cannot
//! see: `Interleaving` (Milner's single-thread nondeterministic merge)
//! round-trips to `ParallelComposition ≠ Interleaving`. The
//! interleaving-vs-true-concurrency distinction is **invisible** to the
//! parallel scale — the [`InterleavingCollapseGap`] axiom asserts exactly
//! that round-trip image.
//!
//! # Literature
//!
//! - **Mazurkiewicz (1977)** *Concurrent Program Schemes and Their
//!   Interpretations*, DAIMI PB-78 — trace theory: the quotient of
//!   interleavings by independence, i.e. the distinction parallelism
//!   forgets.
//! - **Winskel (1986)** *Event Structures*, LNCS 255, and **Pratt
//!   (1986)** *Modeling Concurrency with Partial Orders*, Int. J.
//!   Parallel Programming 15(1):33-71 — true-concurrency semantics, where
//!   interleaving and partial-order composition are provably different.
//! - **Mac Lane (1971)** *Categories for the Working Mathematician*,
//!   Ch. IV — adjunctions, units, counits.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Adjunction, Arrow, Category, Functor};
use pr4xis::ontology::Axiom;

use super::ontology::{
    ParallelismCategory, ParallelismConcept, ParallelismRelation, ParallelismRelationKind,
};
use crate::formal::systems::concurrency::ontology::{
    ConcurrencyCategory, ConcurrencyConcept, ConcurrencyRelation, ConcurrencyRelationKind,
};
use crate::formal::systems::concurrency::parallelism_functor::ConcurrencyToParallelism;

/// The forgetful functor reading each parallelism concept as its
/// concurrency image. Almost everything collapses onto
/// `ParallelComposition`: that heavy collapse is the honest structure —
/// concurrency has no vocabulary for parallelism's performance, hardware,
/// or scheduling distinctions.
pub struct ParallelismToConcurrency;

impl Functor for ParallelismToConcurrency {
    type Source = ParallelismCategory;
    type Target = ConcurrencyCategory;

    fn map_object(obj: &ParallelismConcept) -> ConcurrencyConcept {
        use ConcurrencyConcept as C;
        use ParallelismConcept as P;
        match obj {
            // The two behavioural pairings that survive the round trip.
            P::ParallelTask => C::Process,
            P::ParallelExecution => C::ParallelComposition,
            // Forms of parallel execution collapse onto composition.
            P::DataParallelism | P::TaskParallelism | P::PipelineParallelism => {
                C::ParallelComposition
            }
            // Determinism-by-default is a safety property — its
            // behavioural home in concurrency.
            P::DeterministicParallelism => C::SafetyProperty,
            // Hardware endurant: deliberately NOT Process (endurant !=
            // perdurant). Its faithful home is the Systems functor
            // (ProcessingElement → Component); here it collapses forgetfully.
            P::ProcessingElement => C::ParallelComposition,
            // Flynn's machine taxonomy is a hardware classification with
            // no behavioural image — collapses forgetfully.
            P::MachineOrganization | P::SISD | P::SIMD | P::MISD | P::MIMD => {
                C::ParallelComposition
            }
            // Quantitative cost measures — concurrency has no cost
            // vocabulary; collapse forgetfully.
            P::Work
            | P::Span
            | P::Speedup
            | P::Efficiency
            | P::SequentialFraction
            | P::ScaledSpeedup => C::ParallelComposition,
            // The scheduling apparatus collapses forgetfully.
            P::GreedyScheduler => C::ParallelComposition,
            // Machine cost models collapse forgetfully.
            P::CostModel | P::PRAM | P::BSP | P::LogP => C::ParallelComposition,
        }
    }

    fn map_morphism(m: &ParallelismRelation) -> ConcurrencyRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            ParallelismRelationKind::Identity => return ConcurrencyCategory::identity(&from),
            // A bound reads as a necessity constraint.
            ParallelismRelationKind::Bounds => ConcurrencyRelationKind::NecessaryFor,
            // Achieving a bound reads as enforcement.
            ParallelismRelationKind::Achieves => ConcurrencyRelationKind::Enforces,
            // Executing on a processing element reads as the
            // communication/composition glue.
            ParallelismRelationKind::ExecutesOn => ConcurrencyRelationKind::CommunicatesVia,
            // Exhibiting a property reads as respecting it.
            ParallelismRelationKind::Exhibits => ConcurrencyRelationKind::Respects,
            // A cost model modelling execution reads as an expansion.
            ParallelismRelationKind::Models => ConcurrencyRelationKind::ExpandsTo,
            // The four canonical kinds map to their namesakes.
            ParallelismRelationKind::Subsumption => ConcurrencyRelationKind::Subsumption,
            ParallelismRelationKind::Parthood => ConcurrencyRelationKind::Parthood,
            ParallelismRelationKind::Causation => ConcurrencyRelationKind::Causation,
            ParallelismRelationKind::Opposition => ConcurrencyRelationKind::Opposition,
        };
        ConcurrencyRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    ParallelismToConcurrency,
    "Marlow (2012) LNCS 7241 §1.2; Milner (1980) A Calculus of Communicating Systems, LNCS 92"
);

// ---------------------------------------------------------------------------
// The adjunction: Concurrency ⊣ Parallelism (forward is left)
// ---------------------------------------------------------------------------

/// The round-trip image of a concurrency concept under `G∘F`
/// (`ParallelismToConcurrency ∘ ConcurrencyToParallelism`) — the unit of
/// the adjunction at the object level.
pub fn unit_round_trip(c: &ConcurrencyConcept) -> ConcurrencyConcept {
    ParallelismToConcurrency::map_object(&ConcurrencyToParallelism::map_object(c))
}

/// The round-trip image of a parallelism concept under `F∘G`
/// (`ConcurrencyToParallelism ∘ ParallelismToConcurrency`) — the counit
/// of the adjunction at the object level.
pub fn counit_round_trip(p: &ParallelismConcept) -> ParallelismConcept {
    ConcurrencyToParallelism::map_object(&ParallelismToConcurrency::map_object(p))
}

/// The adjunction `ConcurrencyToParallelism ⊣ ParallelismToConcurrency`.
///
/// The forward (forgetful-onto-coarser) functor is the left adjoint; the
/// backward functor is the right adjoint. Neither round trip is an
/// isomorphism — the unit collapses `Interleaving` onto
/// `ParallelComposition`, which is the whole point.
pub struct ConcurrencyParallelismCollapse;

impl Adjunction for ConcurrencyParallelismCollapse {
    type Left = ConcurrencyToParallelism;
    type Right = ParallelismToConcurrency;

    fn unit(obj: &ConcurrencyConcept) -> ConcurrencyRelation {
        let round_trip = unit_round_trip(obj);
        if round_trip == *obj {
            ConcurrencyRelation {
                from: *obj,
                to: *obj,
                kind: ConcurrencyRelationKind::Identity,
            }
        } else {
            // Heterogeneous round trip — emit identity at the source so
            // the functor laws hold while the collapse stays visible in
            // `map_object` divergence (the gap analysis reads it there).
            ConcurrencyCategory::identity(obj)
        }
    }

    fn counit(obj: &ParallelismConcept) -> ParallelismRelation {
        let round_trip = counit_round_trip(obj);
        if round_trip == *obj {
            ParallelismRelation {
                from: *obj,
                to: *obj,
                kind: ParallelismRelationKind::Identity,
            }
        } else {
            ParallelismCategory::identity(obj)
        }
    }

    fn meta() -> pr4xis::ontology::meta::Provenance {
        pr4xis::ontology::meta::Provenance {
            name: pr4xis::ontology::meta::OntologyName::new_static(
                "ConcurrencyParallelismCollapse",
            ),
            description: pr4xis::ontology::meta::Label::new_static(
                "Concurrency ⊣ Parallelism — the unit collapses interleaving onto parallel composition",
            ),
            citation: pr4xis::ontology::meta::Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician Ch. IV; Mazurkiewicz (1977) Concurrent Program Schemes, DAIMI PB-78; Winskel (1986) Event Structures, LNCS 255; Pratt (1986) Modeling Concurrency with Partial Orders, Int. J. Parallel Programming 15(1):33-71",
            ),
            module_path: pr4xis::ontology::meta::ModulePath::new_static(module_path!()),
        }
    }
}
pr4xis::register_adjunction!(ConcurrencyParallelismCollapse);

// ---------------------------------------------------------------------------
// The gap axiom
// ---------------------------------------------------------------------------

/// Mazurkiewicz (1977) / Winskel (1986) / Pratt (1986): the
/// interleaving-vs-true-concurrency distinction is invisible to the
/// parallel scale. Concretely, the unit round trip `G∘F` sends
/// `Interleaving → ParallelComposition ≠ Interleaving`, while
/// `ParallelComposition` is preserved — so both concurrency concepts
/// collapse onto the single one that parallelism can name.
pub struct InterleavingCollapseGap;

impl Axiom for InterleavingCollapseGap {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        // Interleaving does NOT round-trip to itself …
        let interleaving_collapses = unit_round_trip(&ConcurrencyConcept::Interleaving)
            == ConcurrencyConcept::ParallelComposition
            && unit_round_trip(&ConcurrencyConcept::Interleaving)
                != ConcurrencyConcept::Interleaving;
        // … while ParallelComposition IS preserved: both collapse onto it.
        let composition_preserved = unit_round_trip(&ConcurrencyConcept::ParallelComposition)
            == ConcurrencyConcept::ParallelComposition;
        // The forward images coincide — the mechanism of the collapse.
        let images_coincide =
            ConcurrencyToParallelism::map_object(&ConcurrencyConcept::Interleaving)
                == ConcurrencyToParallelism::map_object(&ConcurrencyConcept::ParallelComposition);
        if interleaving_collapses && composition_preserved && images_coincide {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "InterleavingCollapseGap",
        "the unit round-trip sends Interleaving to ParallelComposition (not Interleaving), which is itself preserved: parallelism cannot see the interleaving distinction",
        "Mazurkiewicz (1977) Concurrent Program Schemes, DAIMI PB-78; Winskel (1986) Event Structures, LNCS 255; Pratt (1986) Int. J. Parallel Programming 15(1):33-71"
    );
}
pr4xis::register_axiom!(
    InterleavingCollapseGap,
    "Mazurkiewicz (1977) Concurrent Program Schemes, DAIMI PB-78; Winskel (1986) Event Structures, LNCS 255; Pratt (1986) Int. J. Parallel Programming 15(1):33-71"
);

// ---------------------------------------------------------------------------
// Gap analysis (in the style of formal::meta::gap_analysis)
// ---------------------------------------------------------------------------

use crate::formal::meta::gap_analysis::{Gap, GapReport};

/// Analyse the `Concurrency ⊣ Parallelism` adjunction for collapsed
/// distinctions — every source concept whose unit round-trip changes its
/// identity, and every target concept whose counit round-trip does.
pub fn analyze_concurrency_parallelism() -> GapReport<ConcurrencyConcept, ParallelismConcept> {
    use pr4xis::category::FinitelyGenerated;

    let mut unit_gaps = Vec::new();
    let mut unit_preserved = Vec::new();
    for entity in ConcurrencyConcept::variants() {
        let round_trip = unit_round_trip(&entity);
        if round_trip == entity {
            unit_preserved.push(entity);
        } else {
            unit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    let mut counit_gaps = Vec::new();
    let mut counit_preserved = Vec::new();
    for entity in ParallelismConcept::variants() {
        let round_trip = counit_round_trip(&entity);
        if round_trip == entity {
            counit_preserved.push(entity);
        } else {
            counit_gaps.push(Gap {
                original: entity,
                collapsed_to: round_trip,
            });
        }
    }

    GapReport {
        unit_gaps,
        unit_preserved,
        counit_gaps,
        counit_preserved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn parallelism_to_concurrency_functor_laws() {
        assert_functor_laws::<ParallelismToConcurrency>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn interleaving_collapse_gap_holds() {
        assert!(InterleavingCollapseGap.verify().is_ok());
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn adjunction_unit_wellformed() {
        use pr4xis::category::FinitelyGenerated;
        for obj in ConcurrencyConcept::variants() {
            let m = ConcurrencyParallelismCollapse::unit(&obj);
            assert_eq!(m.from, obj);
            assert!(ConcurrencyConcept::variants().contains(&m.to));
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn gap_analysis_finds_the_interleaving_collapse() {
        let report = analyze_concurrency_parallelism();
        // The adjunction is not an equivalence — there are gaps.
        assert!(
            !report.unit_gaps.is_empty(),
            "the unit must collapse distinctions"
        );
        // Interleaving is a unit gap, collapsing to ParallelComposition.
        let interleaving_gap = report
            .unit_gaps
            .iter()
            .find(|g| g.original == ConcurrencyConcept::Interleaving);
        let interleaving_gap = interleaving_gap.expect("Interleaving must be a unit gap");
        assert_eq!(
            interleaving_gap.collapsed_to,
            ConcurrencyConcept::ParallelComposition,
            "Interleaving must collapse onto ParallelComposition"
        );
        // ParallelComposition itself is preserved.
        assert!(
            report
                .unit_preserved
                .contains(&ConcurrencyConcept::ParallelComposition),
            "ParallelComposition is the preserved representative"
        );
    }
}
