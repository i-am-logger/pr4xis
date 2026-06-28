//! Tests for [`super::ontology`].

use super::*;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::Axiom;
use proptest::prelude::*;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn category_laws_hold() {
    assert_category_laws::<RuleAlgebraCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ontology_validates() {
    RuleAlgebraOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Domain axioms.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_implication_is_rule_shape_holds() {
    assert!(ImplicationIsRuleShape.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_deontic_square_opposes_holds() {
    assert!(DeonticSquareOpposes.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_strict_defeasible_oppose_holds() {
    assert!(StrictDefeasibleOppose.verify().is_ok());
}

// =============================================================================
// Structural spot checks.
// =============================================================================

#[pr4xis::praxis_value(Extensible, Verifiable)]
#[test]
fn rule_shapes_subsume_implication_or_rule_shape() {
    let sub: Vec<_> = RuleAlgebraCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == RuleAlgebraRelationKind::Subsumption)
        .map(|m| (m.source(), m.target()))
        .collect();
    use RuleAlgebraConcept as C;
    // StrictRule and DefeasibleRule subsume Implication.
    assert!(sub.contains(&(C::StrictRule, C::Implication)));
    assert!(sub.contains(&(C::DefeasibleRule, C::Implication)));
    // Implication subsumes RuleShape.
    assert!(sub.contains(&(C::Implication, C::RuleShape)));
}

#[pr4xis::praxis_value(Extensible, Verifiable)]
#[test]
fn deontic_flavours_all_subsume_deontic_flavour() {
    let sub: Vec<_> = RuleAlgebraCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == RuleAlgebraRelationKind::Subsumption)
        .map(|m| (m.source(), m.target()))
        .collect();
    use RuleAlgebraConcept as C;
    for d in [C::Obligation, C::Permission, C::Prohibition, C::Assertoric] {
        assert!(sub.contains(&(d, C::DeonticFlavour)));
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn pipeline_stages_form_chain() {
    let causation: Vec<_> = RuleAlgebraCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == RuleAlgebraRelationKind::Causation)
        .map(|m| (m.source(), m.target()))
        .collect();
    use RuleAlgebraConcept as C;
    let pipeline = [
        (C::Parsing, C::NormalizationStep),
        (C::NormalizationStep, C::SubsumptionTest),
        (C::SubsumptionTest, C::ConflictTest),
        (C::ConflictTest, C::ResolutionStep),
        (C::ResolutionStep, C::OutputAssembly),
    ];
    for edge in pipeline {
        assert!(causation.contains(&edge), "missing edge {:?}", edge);
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn deontic_square_is_symmetric() {
    let opp: Vec<_> = RuleAlgebraCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == RuleAlgebraRelationKind::Opposition)
        .map(|m| (m.source(), m.target()))
        .collect();
    use RuleAlgebraConcept as C;
    assert!(opp.contains(&(C::Obligation, C::Prohibition)));
    assert!(opp.contains(&(C::Prohibition, C::Obligation)));
}

#[pr4xis::praxis_value(Explainable)]
#[test]
fn lineage_quality_total_on_named_concepts() {
    use RuleAlgebraConcept as C;
    let lineage = RuleAlgebraLineage;
    use pr4xis::ontology::Quality;
    for c in [
        C::Implication,
        C::StrictRule,
        C::DefeasibleRule,
        C::Obligation,
        C::Subsumption,
        C::Normalization,
        C::ConflictDetection,
    ] {
        assert!(lineage.get(&c).is_some(), "no lineage for {c:?}");
    }
}

// =============================================================================
// Property tests.
// =============================================================================

fn arb_concept() -> impl Strategy<Value = RuleAlgebraConcept> {
    proptest::sample::select(RuleAlgebraConcept::variants())
}

proptest! {
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in RuleAlgebraCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    #[test]
    fn prop_lineage_returns_string(c in arb_concept()) {
        use pr4xis::ontology::Quality;
        let lineage = RuleAlgebraLineage;
        let v = lineage.get(&c);
        prop_assert!(v.is_some(), "lineage missing for {:?}", c);
    }

    #[test]
    fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
        let variants: Vec<_> = RuleAlgebraConcept::variants();
        for m in RuleAlgebraCategory::morphisms() {
            if m.kind() == RuleAlgebraRelationKind::Subsumption {
                prop_assert!(variants.contains(&m.source()));
                prop_assert!(variants.contains(&m.target()));
            }
        }
    }

    #[test]
    fn prop_opposition_symmetric(_seed in any::<u32>()) {
        let opp: std::collections::HashSet<_> = RuleAlgebraCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == RuleAlgebraRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        for (a, b) in &opp {
            prop_assert!(opp.contains(&(*b, *a)),
                "opposition not symmetric: {:?} → {:?}", a, b);
        }
    }
}

pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
pr4xis::register_praxis_value!(prop_lineage_returns_string, Explainable);
pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
pr4xis::register_praxis_value!(prop_opposition_symmetric, Verifiable);
