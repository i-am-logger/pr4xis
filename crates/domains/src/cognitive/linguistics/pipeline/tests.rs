use super::ontology::*;
use pr4xis::category::entity::Concept;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    check_category_laws::<PipelineCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    PipelineOntology::validate().unwrap();
}

#[test]
fn twelve_concepts() {
    assert_eq!(PipelineConcept::variants().len(), 12);
}

#[test]
fn shared_lexicon() {
    assert!(SharedLexicon.holds());
}

#[test]
fn parse_generate_adjoint() {
    assert!(ParseGenerateAdjoint.holds());
}

#[test]
fn surface_meaning_opposed() {
    assert!(SurfaceMeaningOpposed.holds());
}

#[test]
fn parse_has_three_stages() {
    use pr4xis::ontology::reasoning::mereology::MereologyDef;
    let parts = PipelineMereology::relations();
    let stages = [
        PipelineConcept::SurfaceForm,
        PipelineConcept::SyntacticStructure,
        PipelineConcept::SemanticRepresentation,
    ];
    for s in &stages {
        assert!(
            parts
                .iter()
                .any(|(w, p)| *w == PipelineConcept::Parse && p == s),
            "Parse missing stage {:?}",
            s
        );
    }
}

#[test]
fn generate_has_three_stages() {
    use pr4xis::ontology::reasoning::mereology::MereologyDef;
    let parts = PipelineMereology::relations();
    let stages = [
        PipelineConcept::SemanticRepresentation,
        PipelineConcept::SyntacticStructure,
        PipelineConcept::SurfaceForm,
    ];
    for s in &stages {
        assert!(
            parts
                .iter()
                .any(|(w, p)| *w == PipelineConcept::Generate && p == s),
            "Generate missing stage {:?}",
            s
        );
    }
}

#[test]
fn causal_chain_surface_to_meaning() {
    use pr4xis::ontology::reasoning::causation::CausalDef;
    let rels = PipelineCausation::relations();
    assert!(
        rels.iter().any(|(c, e)| *c == PipelineConcept::SurfaceForm
            && *e == PipelineConcept::SyntacticStructure)
    );
    assert!(
        rels.iter()
            .any(|(c, e)| *c == PipelineConcept::SyntacticStructure
                && *e == PipelineConcept::SemanticRepresentation)
    );
}
