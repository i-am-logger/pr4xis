//! OMV/MOD — Ontology Metadata Vocabulary. The "ontology about
//! ontologies"; describes ontologies as first-class objects, their
//! formality, methodology, structural metrics, evaluation, and purpose.
//!
//! # Literature
//!
//! - **Hartmann, Palma & Sure (2005)** "OMV — Ontology Metadata
//!   Vocabulary", *ISWC 2005 Workshop on Ontology Patterns* — the
//!   original OMV core.
//! - **Dutta, Toulet, Emonet & Jonquet (2017)** "New Generation
//!   Metadata Vocabulary for Ontology Description and Publication",
//!   *MTSR 2017* — MOD 1.2.
//! - **FAIR-IMPACT (2021)** *MOD 2.0* — extends DCAT 2 with FAIR
//!   evaluation.
//! - **Gruninger & Fox (1995)** "Methodology for the Design and
//!   Evaluation of Ontologies", *IJCAI Workshop on Basic Ontological
//!   Issues in Knowledge Sharing* — competency questions.
//! - **Uschold & Gruninger (1996)** "Ontologies: Principles, Methods
//!   and Applications", *Knowledge Engineering Review* 11(2):93-136.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Omv",
    source: "Hartmann, Palma & Sure (2005) OMV - Ontology Metadata Vocabulary, ISWC Workshop on Ontology Patterns; Dutta, Toulet, Emonet & Jonquet (2017) MOD 1.2, MTSR 2017; FAIR-IMPACT (2021) MOD 2.0; Gruninger & Fox (1995) Methodology for the Design and Evaluation of Ontologies, IJCAI Workshop; Uschold & Gruninger (1996) Ontologies: Principles, Methods and Applications, KER 11(2):93-136",

    concepts: [
        SemanticArtefact,
        FormalityLevel,
        RepresentationParadigm,
        Methodology,
        DesignedTask,
        Analytics,
        Evaluation,
        Catalog,
        NaturalLanguage,
        CompetencyQuestion,
    ],

    labels: {
        SemanticArtefact: ("en", "Semantic artefact",
            "MOD 2.0: an ontology / vocabulary / terminology as a first-class object."),
        FormalityLevel: ("en", "Formality level",
            "OMV omv:hasFormalityLevel: degree of logical formalisation - from informal taxonomy to axiomatised higher-order logic."),
        RepresentationParadigm: ("en", "Representation paradigm",
            "MOD: the formalism used (OWL, SKOS, RDF-S, Lambek calculus, ...)."),
        Methodology: ("en", "Methodology",
            "MOD: the engineering methodology (METHONTOLOGY, NeOn, ontology-from-paper, ...)."),
        DesignedTask: ("en", "Designed task",
            "MOD: what the ontology is designed for (classification, QA, NLG, ...)."),
        Analytics: ("en", "Analytics",
            "MOD 2.0: structural metrics - class / property / axiom counts. Bridges to VoID statistics."),
        Evaluation: ("en", "Evaluation",
            "MOD: quality assessment (FAIR, OQuaRE, OntoQA, ...)."),
        Catalog: ("en", "Catalog",
            "MOD: a registry / repository of semantic artefacts."),
        NaturalLanguage: ("en", "Natural language",
            "dcterms:language: the natural language of the ontology's content."),
        CompetencyQuestion: ("en", "Competency question",
            "Gruninger & Fox (1995): a question the ontology can answer."),
    },

    edges: [
        // Hartmann (2005) OMV core: every artefact has these properties.
        (SemanticArtefact, FormalityLevel, HasFormalityLevel),
        (SemanticArtefact, RepresentationParadigm, HasRepresentation),
        (SemanticArtefact, Methodology, UsedMethodology),
        (SemanticArtefact, DesignedTask, DesignedFor),
        (SemanticArtefact, Analytics, HasAnalytics),
        (SemanticArtefact, Evaluation, HasEvaluation),
        (SemanticArtefact, NaturalLanguage, HasLanguage),
        (SemanticArtefact, CompetencyQuestion, HasCompetencyQuestion),
        // MOD 2.0: Catalog contains SemanticArtefacts.
        (Catalog, SemanticArtefact, Catalogs),
    ],
}

/// Quality: which concepts are formality-level descriptors.
#[derive(Debug, Clone)]
pub struct FormalityLevelOf;

impl Quality for FormalityLevelOf {
    type Individual = OmvConcept;
    type Value = bool;

    fn get(&self, c: &OmvConcept) -> Option<bool> {
        Some(matches!(c, OmvConcept::FormalityLevel))
    }
}

/// Every SemanticArtefact has a FormalityLevel (Hartmann 2005 OMV core).
pub struct ArtefactHasFormalityLevel;

impl Axiom for ArtefactHasFormalityLevel {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let found = OmvCategory::morphisms().iter().any(|m| {
            m.source() == OmvConcept::SemanticArtefact
                && m.target() == OmvConcept::FormalityLevel
                && m.kind() == OmvRelationKind::HasFormalityLevel
        });
        if found {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ArtefactHasFormalityLevel",
        "every SemanticArtefact has a FormalityLevel",
        "Hartmann, Palma & Sure (2005) OMV - Ontology Metadata Vocabulary, ISWC Workshop on Ontology Patterns"
    );
}

pr4xis::register_axiom!(
    ArtefactHasFormalityLevel,
    "Hartmann, Palma & Sure (2005) OMV - Ontology Metadata Vocabulary, ISWC Workshop on Ontology Patterns"
);

/// Every SemanticArtefact has Analytics (MOD 2.0 — VoID statistics).
pub struct ArtefactHasAnalytics;

impl Axiom for ArtefactHasAnalytics {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let found = OmvCategory::morphisms().iter().any(|m| {
            m.source() == OmvConcept::SemanticArtefact
                && m.target() == OmvConcept::Analytics
                && m.kind() == OmvRelationKind::HasAnalytics
        });
        if found {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ArtefactHasAnalytics",
        "every SemanticArtefact has Analytics",
        "FAIR-IMPACT (2021) MOD 2.0"
    );
}

pr4xis::register_axiom!(ArtefactHasAnalytics, "FAIR-IMPACT (2021) MOD 2.0");

impl Ontology for OmvOntology {
    type Cat = OmvCategory;
    type Qual = FormalityLevelOf;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ArtefactHasFormalityLevel));
        axioms.push(Box::new(ArtefactHasAnalytics));
        axioms
    }
}
