use super::ontology::*;
use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::validate::check_category_laws;

#[test]
fn category_laws() {
    check_category_laws::<ProvenanceCategory>().unwrap();
}

#[test]
fn has_prov_o_core_relations() {
    let morphisms = ProvenanceCategory::morphisms();
    // prov:wasGeneratedBy
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == ProvenanceConcept::Artifact
                && m.to == ProvenanceConcept::Activity
                && m.kind == ProvenanceRelationKind::WasGeneratedBy)
    );
    // prov:wasDerivedFrom
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == ProvenanceConcept::Artifact
                && m.to == ProvenanceConcept::Artifact
                && m.kind == ProvenanceRelationKind::WasDerivedFrom)
    );
    // prov:wasAttributedTo
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == ProvenanceConcept::Artifact
                && m.to == ProvenanceConcept::Agent
                && m.kind == ProvenanceRelationKind::WasAttributedTo)
    );
}

#[test]
fn has_version_control_relations() {
    let morphisms = ProvenanceCategory::morphisms();
    assert!(morphisms.iter().any(|m| m.from == ProvenanceConcept::Commit
        && m.to == ProvenanceConcept::Repository
        && m.kind == ProvenanceRelationKind::BelongsTo));
    assert!(morphisms.iter().any(|m| m.from == ProvenanceConcept::Branch
        && m.to == ProvenanceConcept::Commit
        && m.kind == ProvenanceRelationKind::PointsTo));
    assert!(morphisms.iter().any(|m| m.from == ProvenanceConcept::Tag
        && m.to == ProvenanceConcept::Commit
        && m.kind == ProvenanceRelationKind::Marks));
}

#[test]
fn has_knowledge_source_relations() {
    let morphisms = ProvenanceCategory::morphisms();
    assert!(
        morphisms
            .iter()
            .any(|m| m.from == ProvenanceConcept::Artifact
                && m.to == ProvenanceConcept::Source
                && m.kind == ProvenanceRelationKind::DefinedBy)
    );
}

#[test]
fn ten_concepts() {
    assert_eq!(ProvenanceConcept::variants().len(), 10);
}
