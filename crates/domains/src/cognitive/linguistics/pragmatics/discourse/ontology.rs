// Discourse Coherence — RST + SDRT.
//
// Rhetorical relations are MORPHISMS (typed edges), not objects.
// An Elaboration is a relationship between a Nucleus and its Satellite —
// it doesn't exist independently. This follows Mann & Thompson (1988)
// and Asher & Lascarides (2003): relations have specific semantic
// conditions and aren't arbitrary connections.
//
// Concepts (objects): TextSpan, Nucleus, Satellite, DiscourseSegment,
//   DiscourseStructure, Topic.
// Relations (morphism kinds): Elaboration, Sequence, Cause, Condition,
//   Purpose, Background, Justify, Restatement, Concession, Contrast,
//   Narration, Explanation, Parallel, Continuation.
//
// Source: Mann & Thompson (1988); Asher & Lascarides (2003); Hobbs (1979)

use pr4xis::category::{Category, Entity};
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Structural concepts in discourse — what text IS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum DiscourseConcept {
    /// A minimal unit of text that participates in discourse structure.
    TextSpan,
    /// The more central part of a nucleus-satellite pair.
    Nucleus,
    /// The supporting part — depends on the nucleus for interpretation.
    Satellite,
    /// An elementary discourse unit (SDRT).
    DiscourseSegment,
    /// A complex discourse structure built from segments + relations.
    DiscourseStructure,
    /// What the discourse is about.
    Topic,
}

define_ontology! {
    /// Discourse Coherence — RST + SDRT with typed rhetorical relations.
    pub DiscourseOntology for DiscourseCategory {
        concepts: DiscourseConcept,
        relation: DiscourseRelation,
        kind: DiscourseRelationKind,
        kinds: [
            // === Subject-matter relations (RST, informational) ===
            /// Adding detail to the nucleus.
            Elaboration,
            /// Temporal or logical sequence.
            Sequence,
            /// Causal connection.
            Cause,
            /// A condition under which the nucleus holds.
            Condition,
            /// The goal behind an action.
            Purpose,
            /// Setting the context.
            Background,

            // === Presentational relations (RST, affect attitude) ===
            /// Making the nucleus more believable.
            Justify,
            /// Restating in different words.
            Restatement,
            /// Acknowledging incompatibility then overriding.
            Concession,
            /// Highlighting difference.
            Contrast,

            // === SDRT-specific relations ===
            /// Temporal progression (Asher & Lascarides).
            Narration,
            /// Causal explanation of a previous event.
            Explanation,
            /// Similar content in parallel structure.
            Parallel,
            /// Continuing without new rhetorical structure.
            Continuation,

            // === Structural relations ===
            /// Structure contains segments.
            Contains,
        ],
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
        composed: [
            (DiscourseStructure, TextSpan),
        ],

        being: AbstractObject,
        source: "Mann & Thompson (1988); Asher & Lascarides (2003); Hobbs (1979)",

        is_a: DiscourseTaxonomy [
            (Nucleus, TextSpan),
            (Satellite, TextSpan),
            (DiscourseSegment, TextSpan),
        ],

        opposes: DiscourseOpposition [
            (Nucleus, Satellite),
        ],
    }
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
