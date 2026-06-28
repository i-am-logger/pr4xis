use super::ontology::*;
use pr4xis::category::entity::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category};
use pr4xis::ontology::{Axiom, Ontology};

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn category_laws() {
    assert_category_laws::<PipelineCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ontology_validates() {
    PipelineOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn twelve_concepts() {
    assert_eq!(PipelineConcept::variants().len(), 12);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn shared_lexicon() {
    assert!(SharedLexicon.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_generate_adjoint() {
    assert!(ParseGenerateAdjoint.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn surface_meaning_opposed() {
    assert!(SurfaceMeaningOpposed.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
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
            // part→whole (BFO:0000050): each stage is a PART of Parse (stage→Parse).
            parts
                .iter()
                .any(|m| m.source() == *s && m.target() == PipelineConcept::Parse),
            "Parse missing stage {:?}",
            s
        );
    }
}

#[pr4xis::praxis_value(Verifiable)]
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
            // part→whole: each stage is a PART of Generate (stage→Generate).
            parts
                .iter()
                .any(|m| m.source() == *s && m.target() == PipelineConcept::Generate),
            "Generate missing stage {:?}",
            s
        );
    }
}

#[pr4xis::praxis_value(Verifiable)]
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
