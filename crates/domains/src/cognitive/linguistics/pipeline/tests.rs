use super::ontology::*;
use pr4xis::category::entity::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category};
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    assert_category_laws::<PipelineCategory>();
}

#[test]
fn ontology_validates() {
    PipelineOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[test]
fn twelve_concepts() {
    assert_eq!(PipelineConcept::variants().len(), 12);
}

#[test]
fn shared_lexicon() {
    assert!(SharedLexicon.verify().is_ok());
}

#[test]
fn parse_generate_adjoint() {
    assert!(ParseGenerateAdjoint.verify().is_ok());
}

#[test]
fn surface_meaning_opposed() {
    assert!(SurfaceMeaningOpposed.verify().is_ok());
}

#[test]
fn parse_has_three_stages() {
    let parts: Vec<_> = PipelineCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == PipelineRelationKind::Parthood)
        .collect();
    let stages = [
        PipelineConcept::SurfaceForm,
        PipelineConcept::SyntacticStructure,
        PipelineConcept::SemanticRepresentation,
    ];
    for s in &stages {
        assert!(
            parts
                .iter()
                .any(|m| m.source() == PipelineConcept::Parse && m.target() == *s),
            "Parse missing stage {:?}",
            s
        );
    }
}

#[test]
fn generate_has_three_stages() {
    let parts: Vec<_> = PipelineCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == PipelineRelationKind::Parthood)
        .collect();
    let stages = [
        PipelineConcept::SemanticRepresentation,
        PipelineConcept::SyntacticStructure,
        PipelineConcept::SurfaceForm,
    ];
    for s in &stages {
        assert!(
            parts
                .iter()
                .any(|m| m.source() == PipelineConcept::Generate && m.target() == *s),
            "Generate missing stage {:?}",
            s
        );
    }
}

#[test]
fn causal_chain_surface_to_meaning() {
    let rels: Vec<_> = PipelineCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == PipelineRelationKind::Causation)
        .collect();
    assert!(
        rels.iter()
            .any(|m| m.source() == PipelineConcept::SurfaceForm
                && m.target() == PipelineConcept::SyntacticStructure)
    );
    assert!(
        rels.iter()
            .any(|m| m.source() == PipelineConcept::SyntacticStructure
                && m.target() == PipelineConcept::SemanticRepresentation)
    );
}
