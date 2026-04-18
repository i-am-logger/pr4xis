#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

// OMV/MOD — Ontology Metadata Vocabulary.
//
// Describes ontologies as first-class objects: their formality level,
// engineering methodology, structural metrics, evaluation, and purpose.
//
// This is the "ontology about ontologies" — it tells you what KIND of
// ontology something is, how it was built, and what it's for.
//
// Grounded in:
// - Hartmann et al. "Ontology Metadata Vocabulary" (2005) — OMV
// - Dutta et al. "MOD: Metadata for Ontology Description" (2017) — MOD 1.2
// - FAIR-IMPACT MOD 2.0 (2021) — extends DCAT 2
//
// Composes with:
// - Knowledge (VoID) — structural metadata (class/property counts)
// - Lemon (Ontolex) — linguistic realization of ontology names/descriptions
// - OWL — the representation language itself

use pr4xis::category::{Category, Concept};
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Concepts in the OMV/MOD ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum OmvConcept {
    /// mod:SemanticArtefact — an ontology/vocabulary/terminology as a first-class object.
    SemanticArtefact,

    /// mod:FormalityLevel — degree of logical formalization.
    /// Ranges from informal taxonomy to axiomatized higher-order logic.
    FormalityLevel,

    /// mod:KnowledgeRepresentationParadigm — the formalism used.
    /// OWL, SKOS, RDF-S, Lambek calculus, etc.
    RepresentationParadigm,

    /// mod:EngineeringMethodology — how the ontology was built.
    /// METHONTOLOGY, NeOn, ontology-from-paper, etc.
    Methodology,

    /// mod:SemanticArtefactTask — what the ontology is designed for.
    /// Classification, question answering, NLG, etc.
    DesignedTask,

    /// mod:Analytics — structural metrics (class/property/axiom counts).
    /// Bridges to VoID void:classes, void:properties.
    Analytics,

    /// mod:Evaluation — quality assessment of the ontology.
    /// FAIR assessment, OQuaRE, OntoQA, etc.
    Evaluation,

    /// mod:SemanticArtefactCatalog — a registry/repository of ontologies.
    /// Maps to Knowledge::KnowledgeBase.
    Catalog,

    /// dcterms:LinguisticSystem — natural language of the ontology's content.
    /// Connects to Lemon's lime:Lexicon via language tag.
    NaturalLanguage,

    /// Competency question — what the ontology can answer.
    /// Gruninger & Fox (1995); Uschold & Gruninger (1996).
    CompetencyQuestion,
}

define_ontology! {
    /// OMV/MOD — ontology metadata vocabulary.
    pub OmvOntology for OmvCategory {
        concepts: OmvConcept,
        relation: OmvRelation,
        kind: OmvRelationKind,
        kinds: [
            /// mod:hasFormalityLevel — how formally axiomatized.
            HasFormalityLevel,
            /// mod:hasRepresentationLanguage — which formalism.
            HasRepresentation,
            /// mod:usedEngineeringMethodology — how it was built.
            UsedMethodology,
            /// mod:designedForTask — what it's for.
            DesignedFor,
            /// hasAnalytics — structural metrics.
            HasAnalytics,
            /// hasEvaluation — quality assessment.
            HasEvaluation,
            /// catalogs — catalog contains artefact.
            Catalogs,
            /// dcterms:language — natural language coverage.
            HasLanguage,
            /// competency question association.
            HasCompetencyQuestion,
        ],
        edges: [
            (SemanticArtefact, FormalityLevel, HasFormalityLevel),
            (SemanticArtefact, RepresentationParadigm, HasRepresentation),
            (SemanticArtefact, Methodology, UsedMethodology),
            (SemanticArtefact, DesignedTask, DesignedFor),
            (SemanticArtefact, Analytics, HasAnalytics),
            (SemanticArtefact, Evaluation, HasEvaluation),
            (SemanticArtefact, NaturalLanguage, HasLanguage),
            (SemanticArtefact, CompetencyQuestion, HasCompetencyQuestion),
            (Catalog, SemanticArtefact, Catalogs),
        ],
        composed: [
            (Catalog, FormalityLevel),
            (Catalog, RepresentationParadigm),
            (Catalog, Methodology),
            (Catalog, DesignedTask),
            (Catalog, Analytics),
            (Catalog, Evaluation),
            (Catalog, NaturalLanguage),
            (Catalog, CompetencyQuestion),
        ],

        being: AbstractObject,
        source: "Hartmann et al. (2005); Dutta et al. (2017); MOD 2.0 (2021)",
    }
}

/// Formality levels — from OMV omv:hasFormalityLevel.
/// Ranges from informal to highly axiomatized.
#[derive(Debug, Clone)]
pub struct FormalityLevelOf;

impl Quality for FormalityLevelOf {
    type Individual = OmvConcept;
    type Value = bool;

    fn get(&self, individual: &OmvConcept) -> Option<bool> {
        Some(matches!(individual, OmvConcept::FormalityLevel))
    }
}

/// Every SemanticArtefact must have a FormalityLevel (OMV core requirement).
#[derive(Debug)]
pub struct ArtefactHasFormalityLevel;

impl Axiom for ArtefactHasFormalityLevel {
    fn description(&self) -> &str {
        "every SemanticArtefact has a FormalityLevel (OMV core; Hartmann 2005)"
    }
    fn holds(&self) -> bool {
        let m = OmvCategory::morphisms();
        m.iter().any(|r| {
            r.from == OmvConcept::SemanticArtefact
                && r.to == OmvConcept::FormalityLevel
                && r.kind == OmvRelationKind::HasFormalityLevel
        })
    }
}
pr4xis::register_axiom!(ArtefactHasFormalityLevel);

/// Every SemanticArtefact must have Analytics (structural metrics).
#[derive(Debug)]
pub struct ArtefactHasAnalytics;

impl Axiom for ArtefactHasAnalytics {
    fn description(&self) -> &str {
        "every SemanticArtefact has Analytics (MOD 2.0; VoID statistics)"
    }
    fn holds(&self) -> bool {
        let m = OmvCategory::morphisms();
        m.iter().any(|r| {
            r.from == OmvConcept::SemanticArtefact
                && r.to == OmvConcept::Analytics
                && r.kind == OmvRelationKind::HasAnalytics
        })
    }
}
pr4xis::register_axiom!(ArtefactHasAnalytics);

/// Catalog reaches all artefact metadata transitively.
#[derive(Debug)]
pub struct CatalogReachesAll;

impl Axiom for CatalogReachesAll {
    fn description(&self) -> &str {
        "Catalog reaches all concepts transitively (DCAT completeness)"
    }
    fn holds(&self) -> bool {
        use pr4xis::category::entity::Concept;
        let m = OmvCategory::morphisms();
        OmvConcept::variants().iter().all(|c| {
            m.iter()
                .any(|r| r.from == OmvConcept::Catalog && r.to == *c)
        })
    }
}
pr4xis::register_axiom!(CatalogReachesAll);

impl Ontology for OmvOntology {
    type Cat = OmvCategory;
    type Qual = FormalityLevelOf;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        OmvOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(ArtefactHasFormalityLevel),
            Box::new(ArtefactHasAnalytics),
            Box::new(CatalogReachesAll),
        ]
    }
}
