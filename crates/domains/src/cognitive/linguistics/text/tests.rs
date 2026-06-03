use super::ontology::*;
use pr4xis::category::entity::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category};
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    assert_category_laws::<TextCategory>();
}

#[test]
fn ontology_validates() {
    TextOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[test]
fn nine_concepts() {
    assert_eq!(TextConcept::variants().len(), 9);
}

#[test]
fn word_is_fully_connected() {
    assert!(WordIsFullyConnected.verify().is_ok());
}

#[test]
fn two_level_containment() {
    assert!(TwoLevelContainment.verify().is_ok());
}

#[test]
fn word_is_a_span() {
    assert!(TextCategory::morphisms().iter().any(|m| {
        m.kind() == TextRelationKind::Subsumption
            && m.source() == TextConcept::Word
            && m.target() == TextConcept::Span
    }));
}
