//! Tests for the LegalActor ontology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    CarriesBurden, CounselWitnessOpposition, LegalActorCategory, LegalActorConcept,
    LegalActorOntology, OnlyPartiesCarryBurden, PartyAdjudicatorOpposition, adjudicator_leaves,
    is_leaf, is_party, parse_actor, party_leaves, witness_leaves,
};
use pr4xis::category::Concept;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws + validation
// =============================================================================

#[test]
fn category_laws() {
    assert_category_laws::<LegalActorCategory>();
}

#[test]
fn ontology_validates() {
    LegalActorOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[test]
fn nineteen_concepts() {
    // Root + 4 families + 7 Party + 5 Adjudicator + 2 Witness = 19.
    // Counsel is a family with no leaves declared (it's a flat role).
    assert_eq!(LegalActorConcept::variants().len(), 19);
}

#[test]
fn seven_party_leaves() {
    assert_eq!(party_leaves().len(), 7);
    for leaf in party_leaves() {
        assert!(is_party(leaf));
        assert!(is_leaf(leaf));
    }
}

#[test]
fn five_adjudicator_leaves() {
    assert_eq!(adjudicator_leaves().len(), 5);
    for leaf in adjudicator_leaves() {
        assert!(!is_party(leaf));
        assert!(is_leaf(leaf));
    }
}

#[test]
fn two_witness_leaves() {
    assert_eq!(witness_leaves().len(), 2);
}

// =============================================================================
// Actor-name parser
// =============================================================================

#[test]
fn parse_plaintiff() {
    assert_eq!(parse_actor("plaintiff"), Some(LegalActorConcept::Plaintiff));
    assert_eq!(parse_actor("Plaintiff"), Some(LegalActorConcept::Plaintiff));
    assert_eq!(parse_actor("PLAINTIFF"), Some(LegalActorConcept::Plaintiff));
}

#[test]
fn parse_employer_not_recognized() {
    // "employer" is a SOX-domain-specific term, not a general litigation
    // role. It doesn't classify as a legal actor at this layer.
    assert_eq!(parse_actor("employer"), None);
}

#[test]
fn parse_attorney_aliases() {
    assert_eq!(parse_actor("attorney"), Some(LegalActorConcept::Counsel));
    assert_eq!(parse_actor("lawyer"), Some(LegalActorConcept::Counsel));
    assert_eq!(parse_actor("counsel"), Some(LegalActorConcept::Counsel));
}

#[test]
fn parse_expert_witness_compound() {
    assert_eq!(
        parse_actor("expert witness"),
        Some(LegalActorConcept::ExpertWitness)
    );
    assert_eq!(
        parse_actor("expert"),
        Some(LegalActorConcept::ExpertWitness)
    );
}

// =============================================================================
// CarriesBurden quality
// =============================================================================

#[test]
fn plaintiff_carries_burden() {
    assert_eq!(CarriesBurden.get(&LegalActorConcept::Plaintiff), Some(true));
}

#[test]
fn judge_does_not_carry_burden() {
    assert_eq!(CarriesBurden.get(&LegalActorConcept::Judge), Some(false));
}

#[test]
fn expert_witness_does_not_carry_burden() {
    assert_eq!(
        CarriesBurden.get(&LegalActorConcept::ExpertWitness),
        Some(false)
    );
}

#[test]
fn family_concepts_are_abstract() {
    assert_eq!(CarriesBurden.get(&LegalActorConcept::Party), None);
    assert_eq!(CarriesBurden.get(&LegalActorConcept::Adjudicator), None);
    assert_eq!(CarriesBurden.get(&LegalActorConcept::LegalActor), None);
}

// =============================================================================
// Axioms
// =============================================================================

#[test]
fn axiom_party_adjudicator_opposition() {
    assert!(PartyAdjudicatorOpposition.verify().is_ok());
}

#[test]
fn axiom_counsel_witness_opposition() {
    assert!(CounselWitnessOpposition.verify().is_ok());
}

#[test]
fn axiom_only_parties_carry_burden() {
    assert!(OnlyPartiesCarryBurden.verify().is_ok());
}

#[test]
fn all_axioms_hold() {
    for axiom in LegalActorOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Property-based
// =============================================================================

fn arb_concept() -> impl Strategy<Value = LegalActorConcept> {
    proptest::sample::select(LegalActorConcept::variants())
}

proptest! {
    /// CarriesBurden is total on leaves and None on family/root.
    #[test]
    fn prop_burden_total_on_leaves(c in arb_concept()) {
        let v = CarriesBurden.get(&c);
        if is_leaf(c) {
            prop_assert!(v.is_some());
        } else {
            prop_assert_eq!(v, None);
        }
    }

    /// Every leaf is either a Party (carries burden) or not (doesn't).
    #[test]
    fn prop_burden_iff_party(c in arb_concept()) {
        if is_leaf(c) {
            let bears = CarriesBurden.get(&c) == Some(true);
            prop_assert_eq!(is_party(c), bears);
        }
    }
}
