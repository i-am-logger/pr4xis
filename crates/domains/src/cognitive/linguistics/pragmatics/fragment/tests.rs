use super::ontology::*;
use pr4xis::category::entity::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category};
use pr4xis::ontology::{Axiom, Ontology};

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn category_laws() {
    assert_category_laws::<FragmentCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ontology_validates() {
    FragmentOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn twelve_concepts() {
    assert_eq!(FragmentConcept::variants().len(), 12);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn all_fragments_classified() {
    assert!(AllFragmentsClassified.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn eight_fragment_types() {
    let count = FragmentCategory::morphisms()
        .iter()
        .filter(|m| {
            m.kind() == FragmentRelationKind::Subsumption && m.target() == FragmentConcept::Fragment
        })
        .count();
    assert_eq!(count, 8);
}
