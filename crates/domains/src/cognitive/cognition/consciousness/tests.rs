use super::ontology::*;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[test]
fn category_laws() {
    check_category_laws::<ConsciousnessCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    ConsciousnessOntology::validate().unwrap();
}

#[test]
fn fourteen_concepts() {
    assert_eq!(ConsciousnessConcept::variants().len(), 14);
}

#[test]
fn attention_causes_access() {
    assert!(AttentionCausesAccess.holds());
}

#[test]
fn conscious_unconscious_opposed() {
    assert!(ConsciousUnconsciousOpposed.holds());
}

#[test]
fn integration_produces_structure() {
    assert!(IntegrationProducesStructure.holds());
}

#[test]
fn all_concepts_have_theory_origin() {
    for c in ConsciousnessConcept::variants() {
        assert!(
            TheoryOrigin.get(&c).is_some(),
            "{:?} has no theory origin",
            c
        );
    }
}

#[test]
fn global_workspace_has_parts() {
    use pr4xis::ontology::reasoning::mereology::MereologyDef;
    let parts = ConsciousnessMereology::relations();
    assert!(parts.iter().any(|(w, p)| {
        *w == ConsciousnessConcept::GlobalWorkspace && *p == ConsciousnessConcept::BroadcastMessage
    }));
    assert!(parts.iter().any(|(w, p)| {
        *w == ConsciousnessConcept::GlobalWorkspace && *p == ConsciousnessConcept::Coalition
    }));
}
