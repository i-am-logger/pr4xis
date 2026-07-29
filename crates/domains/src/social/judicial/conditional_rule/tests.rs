//! Tests for the `ConditionalRuleFrame` ontology and `ConditionalRule`
//! value-type reduction (Sergot et al. 1986).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::Ontology;
use proptest::prelude::*;

use super::ontology::ConditionalRuleFrameCategory;
use super::ontology::ConditionalRuleFrameOntology;
use super::{Applicability, AppliedElement, ConditionalRule, FactValue, SlotState};
use crate::formal::meta::identifier_format::Identifier;
use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use crate::social::judicial::ontology::{
    EvidenceRequirement, EvidenceType, LegalTerm, RequirementLevel,
};
use crate::social::judicial::source_text::SourceTextRef;

// =============================================================================
// Category laws and validation
// =============================================================================

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn category_laws() {
    assert_category_laws::<ConditionalRuleFrameCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ontology_validates() {
    ConditionalRuleFrameOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// ConditionalRule::applicability() -- Sergot et al. (1986) reduction
// =============================================================================

fn requirement(field: &str, required: RequirementLevel) -> EvidenceRequirement {
    EvidenceRequirement {
        field: SourceTextRef::new(field),
        field_type: EvidenceType::Currency,
        required,
        description: None,
    }
}

fn term_with(evidence: Vec<EvidenceRequirement>) -> LegalTerm {
    LegalTerm {
        id: Identifier::curie("witness:test_term").expect("valid CURIE"),
        name: SourceTextRef::new("witness term"),
        definition: SourceTextRef::new("witness definition"),
        source_text: None,
        subsection: None,
        required_evidence: evidence,
        obligations: vec![],
        deadlines: vec![],
        rights: vec![],
        remedies: vec![],
        burdens: vec![],
        exceptions: vec![],
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn all_required_satisfied_applies() {
    let term = term_with(vec![requirement("income", RequirementLevel::Required)]);
    let mut rule = ConditionalRule::from_term(term, SourceTaxonomyConcept::UsFederalStatute);
    rule.elements[0].state = SlotState::Satisfied(FactValue::Boolean(true));
    assert_eq!(rule.applicability(), Applicability::Applies);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn one_unfilled_required_is_indeterminate_naming_it() {
    let req = requirement("income", RequirementLevel::Required);
    let term = term_with(vec![req.clone()]);
    let rule = ConditionalRule::from_term(term, SourceTaxonomyConcept::UsFederalStatute);
    match rule.applicability() {
        Applicability::Indeterminate { missing } => {
            assert_eq!(missing.len(), 1);
            assert_eq!(missing.head.requirement, req);
        }
        other => panic!("expected Indeterminate, got {other:?}"),
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn one_not_satisfied_required_does_not_apply_regardless_of_others() {
    let term = term_with(vec![
        requirement("income", RequirementLevel::Required),
        requirement("residency", RequirementLevel::Required),
    ]);
    let mut rule = ConditionalRule::from_term(term, SourceTaxonomyConcept::UsFederalStatute);
    rule.elements[0].state = SlotState::NotSatisfied(FactValue::Boolean(false));
    // Second element left Unfilled -- DoesNotApply must still win.
    assert_eq!(rule.applicability(), Applicability::DoesNotApply);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn unfilled_optional_element_never_blocks_or_appears_in_missing() {
    let term = term_with(vec![requirement(
        "supporting document",
        RequirementLevel::Optional,
    )]);
    let rule = ConditionalRule::from_term(term, SourceTaxonomyConcept::UsFederalStatute);
    assert_eq!(rule.applicability(), Applicability::Applies);
}

// =============================================================================
// Property: NotSatisfied on ANY Required element always wins over
// Indeterminate/Applies, regardless of the other elements' states or count
// (Sergot et al. 1986's conjunctive reduction -- one failed condition is
// decisive).
// =============================================================================

fn arb_slot_state() -> impl Strategy<Value = SlotState> {
    prop_oneof![
        Just(SlotState::Unfilled),
        Just(SlotState::Satisfied(FactValue::Boolean(true))),
        Just(SlotState::NotSatisfied(FactValue::Boolean(false))),
    ]
}

proptest! {
    /// One `NotSatisfied` Required element is decisive regardless of the
    /// other elements' states or count (Sergot et al. 1986's conjunctive
    /// reduction).
    #[test]
    fn not_satisfied_required_always_dominates(
        states in proptest::collection::vec(arb_slot_state(), 1..6),
        failing_index in 0usize..6,
    ) {
        let failing_index = failing_index % states.len();
        let term = term_with(
            (0..states.len())
                .map(|i| requirement(&format!("field{i}"), RequirementLevel::Required))
                .collect(),
        );
        let mut rule = ConditionalRule::from_term(term, SourceTaxonomyConcept::UsFederalStatute);
        for (el, state) in rule.elements.iter_mut().zip(states.iter()) {
            el.state = state.clone();
        }
        rule.elements[failing_index].state = SlotState::NotSatisfied(FactValue::Boolean(false));
        prop_assert_eq!(rule.applicability(), Applicability::DoesNotApply);
    }
}

pr4xis::register_praxis_value!(not_satisfied_required_always_dominates, Verifiable);

// =============================================================================
// AppliedElement::unfilled
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn unfilled_constructor_starts_unfilled() {
    let req = requirement("income", RequirementLevel::Required);
    let el = AppliedElement::unfilled(req.clone());
    assert_eq!(el.requirement, req);
    assert_eq!(el.state, SlotState::Unfilled);
}
