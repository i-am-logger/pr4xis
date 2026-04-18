//! Discourse Coherence — RST + SDRT.
//!
//! Rhetorical relations are MORPHISMS (typed edges), not objects.
//! An Elaboration is a relationship between a Nucleus and its Satellite —
//! it doesn't exist independently. This follows Mann & Thompson (1988)
//! and Asher & Lascarides (2003): relations have specific semantic
//! conditions and aren't arbitrary connections.
//!
//! Concepts (objects): TextSpan, Nucleus, Satellite, DiscourseSegment,
//!   DiscourseStructure, Topic.
//! Relations (morphism kinds): Elaboration, Sequence, Cause, Condition,
//!   Purpose, Background, Justify, Restatement, Concession, Contrast,
//!   Narration, Explanation, Parallel, Continuation.
//!
//! Source: Mann & Thompson (1988); Asher & Lascarides (2003); Hobbs (1979)

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Discourse",
    source: "Mann & Thompson (1988); Asher & Lascarides (2003); Hobbs (1979)",
    being: AbstractObject,

    concepts: [TextSpan, Nucleus, Satellite, DiscourseSegment, DiscourseStructure, Topic],

    labels: {
        TextSpan: ("en", "Text span", "A minimal unit of text that participates in discourse structure."),
        Nucleus: ("en", "Nucleus", "The more central part of a nucleus-satellite pair (RST)."),
        Satellite: ("en", "Satellite", "The supporting part — depends on the nucleus for interpretation."),
        DiscourseSegment: ("en", "Discourse segment", "An elementary discourse unit (SDRT)."),
        DiscourseStructure: ("en", "Discourse structure", "A complex discourse structure built from segments + relations."),
        Topic: ("en", "Topic", "What the discourse is about."),
    },

    edges: [
        // Nucleus-Satellite relations (RST core)
        (Nucleus, Satellite, Elaboration),
        (Nucleus, Satellite, Justify),
        (Nucleus, Satellite, Restatement),
        (Nucleus, Satellite, Concession),
        (Nucleus, Satellite, Background),
        (Nucleus, Satellite, Condition),
        (Nucleus, Satellite, Purpose),
        // Multinuclear relations (both spans are nuclei)
        (Nucleus, Nucleus, Sequence),
        (Nucleus, Nucleus, Contrast),
        (Nucleus, Nucleus, Parallel),
        (Nucleus, Nucleus, Continuation),
        // Causal relations (SDRT)
        (DiscourseSegment, DiscourseSegment, Cause),
        (DiscourseSegment, DiscourseSegment, Explanation),
        (DiscourseSegment, DiscourseSegment, Narration),
        // Structural containment
        (DiscourseStructure, DiscourseSegment, Contains),
        (DiscourseStructure, Nucleus, Contains),
        (DiscourseStructure, Satellite, Contains),
        (DiscourseStructure, Topic, Contains),
    ],

    is_a: [
        (Nucleus, TextSpan),
        (Satellite, TextSpan),
        (DiscourseSegment, TextSpan),
    ],

    opposes: [
        (Nucleus, Satellite),
    ],
}

/// Whether a concept is structural vs a text unit.
#[derive(Debug, Clone)]
pub struct IsStructuralElement;

impl Quality for IsStructuralElement {
    type Individual = DiscourseConcept;
    type Value = bool;

    fn get(&self, individual: &DiscourseConcept) -> Option<bool> {
        Some(matches!(
            individual,
            DiscourseConcept::DiscourseStructure | DiscourseConcept::Topic
        ))
    }
}

/// Nucleus-Satellite relations are asymmetric (RST core).
#[derive(Debug)]
pub struct NucleusSatelliteAsymmetric;

impl Axiom for NucleusSatelliteAsymmetric {
    fn description(&self) -> &str {
        "Nucleus→Satellite relations exist but Satellite→Nucleus do not (RST asymmetry)"
    }
    fn holds(&self) -> bool {
        let m = DiscourseCategory::morphisms();
        let has_nuc_sat = m.iter().any(|r| {
            r.from == DiscourseConcept::Nucleus
                && r.to == DiscourseConcept::Satellite
                && r.kind == DiscourseRelationKind::Elaboration
        });
        let has_sat_nuc_elab = m.iter().any(|r| {
            r.from == DiscourseConcept::Satellite
                && r.to == DiscourseConcept::Nucleus
                && r.kind == DiscourseRelationKind::Elaboration
        });
        has_nuc_sat && !has_sat_nuc_elab
    }
}
pr4xis::register_axiom!(
    NucleusSatelliteAsymmetric,
    "Mann & Thompson (1988); Asher & Lascarides (2003); Hobbs (1979)"
);

/// Multinuclear relations connect Nucleus to Nucleus (RST).
#[derive(Debug)]
pub struct MultinuclearExists;

impl Axiom for MultinuclearExists {
    fn description(&self) -> &str {
        "Multinuclear relations: Sequence, Contrast, Parallel connect Nucleus→Nucleus (RST)"
    }
    fn holds(&self) -> bool {
        let m = DiscourseCategory::morphisms();
        m.iter().any(|r| {
            r.from == DiscourseConcept::Nucleus
                && r.to == DiscourseConcept::Nucleus
                && r.kind == DiscourseRelationKind::Sequence
        })
    }
}
pr4xis::register_axiom!(
    MultinuclearExists,
    "Mann & Thompson (1988); Asher & Lascarides (2003); Hobbs (1979)"
);

impl Ontology for DiscourseOntology {
    type Cat = DiscourseCategory;
    type Qual = IsStructuralElement;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        DiscourseOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(NucleusSatelliteAsymmetric),
            Box::new(MultinuclearExists),
        ]
    }
}
