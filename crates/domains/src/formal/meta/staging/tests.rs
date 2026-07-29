//! Tests for the staging ontology — category laws, taxonomy, causation,
//! domain axioms, and property-based tests over the staging level quality.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    EachProjectionRaisesStagingByOne, FutamuraChainIsComplete, FutamuraStagingLevel,
    StagingCategory, StagingConcept, StagingLevel, StagingOntology, StagingRelationKind,
    Temporality, TemporalityTag,
};
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws and ontology validation
// =============================================================================

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn category_laws() {
    assert_category_laws::<StagingCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ontology_validates() {
    StagingOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Entity surface
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn fifteen_stage_concepts() {
    // 10 staging concepts + 5 pipeline-stage concepts.
    assert_eq!(StagingConcept::variants().len(), 15);
}

// =============================================================================
// Taxonomy: every program-kind is-a Program
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn interpreter_is_a_program() {
    let sub: Vec<_> = StagingCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == StagingRelationKind::Subsumption)
        .map(|m| (m.source(), m.target()))
        .collect();
    assert!(sub.contains(&(StagingConcept::Interpreter, StagingConcept::Program)));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn source_object_residual_are_programs() {
    let sub: Vec<_> = StagingCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == StagingRelationKind::Subsumption)
        .map(|m| (m.source(), m.target()))
        .collect();
    for c in [
        StagingConcept::SourceProgram,
        StagingConcept::ObjectProgram,
        StagingConcept::ResidualProgram,
    ] {
        assert!(sub.contains(&(c, StagingConcept::Program)));
    }
}

// =============================================================================
// Causation: Futamura projection chain
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn futamura_chain_reaches_cogen() {
    // The causal closure (OBO-RO transitive_over) includes the full chain.
    let caus: Vec<_> = StagingCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == StagingRelationKind::Causation)
        .map(|m| (m.source(), m.target()))
        .collect();
    assert!(caus.contains(&(
        StagingConcept::WriteInterpreter,
        StagingConcept::GenerateCompilerGenerator
    )));
}

// =============================================================================
// Qualities
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn static_input_is_static() {
    assert_eq!(
        TemporalityTag.get(&StagingConcept::StaticInput),
        Some(Temporality::Static)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dynamic_input_is_dynamic() {
    assert_eq!(
        TemporalityTag.get(&StagingConcept::DynamicInput),
        Some(Temporality::Dynamic)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn cogen_is_at_staging_level_three() {
    assert_eq!(
        StagingLevel.get(&StagingConcept::CompilerGenerator),
        Some(FutamuraStagingLevel::ThirdProjection)
    );
}

// =============================================================================
// Domain axioms
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_futamura_chain_is_complete() {
    assert!(FutamuraChainIsComplete.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_each_projection_raises_staging_by_one() {
    assert!(EachProjectionRaisesStagingByOne.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn all_axioms_hold() {
    for axiom in StagingOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!(
                "axiom failed: {} - {}",
                c.meta().name.as_str(),
                c.meta().description.as_str()
            );
        }
    }
}

// =============================================================================
// Property-based tests
// =============================================================================

fn arb_stage_concept() -> impl Strategy<Value = StagingConcept> {
    proptest::sample::select(StagingConcept::variants())
}

fn arb_futamura_ladder_pair() -> impl Strategy<Value = (StagingConcept, StagingConcept, usize)> {
    use StagingConcept::*;
    prop_oneof![
        Just((Interpreter, ObjectProgram, 1)),
        Just((Interpreter, Compiler, 2)),
        Just((Interpreter, CompilerGenerator, 3)),
        Just((ObjectProgram, Compiler, 1)),
        Just((Compiler, CompilerGenerator, 1)),
    ]
}

proptest! {
    /// Every program-or-input concept has a defined temporality.
    #[test]
    fn prop_program_concept_has_temporality(c in arb_stage_concept()) {
        use StagingConcept as S;
        let core = matches!(c,
            S::Program | S::Interpreter | S::Compiler | S::Specializer | S::CompilerGenerator
            | S::SourceProgram | S::ObjectProgram | S::ResidualProgram
            | S::StaticInput | S::DynamicInput
        );
        let v = TemporalityTag.get(&c);
        prop_assert_eq!(v.is_some(), core);
    }

    /// Every program-or-input concept has a staging level in [0, 3] —
    /// bounded above by the third (final) Futamura projection.
    #[test]
    fn prop_staging_level_bounded(c in arb_stage_concept()) {
        if let Some(level) = StagingLevel.get(&c) {
            prop_assert!(level <= FutamuraStagingLevel::ThirdProjection);
        }
    }

    /// Each Futamura projection along the ladder raises the level by the
    /// expected delta exactly — `expected` applications of `successor()`
    /// from `pre` land exactly on `post`.
    #[test]
    fn prop_futamura_ladder_deltas_are_exact(pair in arb_futamura_ladder_pair()) {
        let (pre, post, expected) = pair;
        let pre_level = StagingLevel.get(&pre).unwrap();
        let post_level = StagingLevel.get(&post).unwrap();
        let mut stepped = pre_level;
        for _ in 0..expected {
            stepped = stepped
                .successor()
                .expect("ladder step stays within the Futamura projection range");
        }
        prop_assert_eq!(stepped, post_level);
    }

    #[test]
    fn prop_structural_axioms_hold(_seed in any::<u32>()) {
        for axiom in StagingOntology::axioms() {
            if let Err(c) = axiom.verify() {
                prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
            }
        }
    }
}

pr4xis::register_praxis_value!(prop_program_concept_has_temporality, Verifiable);
pr4xis::register_praxis_value!(prop_staging_level_bounded, Verifiable);
pr4xis::register_praxis_value!(prop_futamura_ladder_deltas_are_exact, Verifiable);
pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
