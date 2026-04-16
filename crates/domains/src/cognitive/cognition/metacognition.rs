use pr4xis::category::Entity;
use pr4xis::define_ontology;

// Metacognition — the system observing its own reasoning.
//
// Second-order cybernetics formalized: the observer IS part of the system.
// The meta-level monitors, evaluates, and controls the object level.
//
// When the grammar fails to parse, the meta-level:
// 1. Detects the failure (monitoring)
// 2. Diagnoses the cause (evaluation — which type reduction failed?)
// 3. Decides what to do (control — ask for clarification? attempt repair?)
//
// References:
// - von Foerster, Observing Systems (1981)
// - Glanville, Second Order Cybernetics (docs/papers/)
// - Olivares-Alarcos et al., MOI: Meta-Ontology for Introspection (2023)

/// Metacognitive concepts — the bi-level architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum MetaConcept {
    /// The actual reasoning happening (grammar, semantics, queries).
    ObjectLevel,
    /// The observer of the reasoning (monitors, evaluates, controls).
    MetaLevel,
    /// What the meta-level watches: the trace of object-level steps.
    Monitoring,
    /// Assessing whether the object level succeeded or failed.
    Evaluation,
    /// Deciding what to do based on evaluation (continue, repair, ask).
    Control,
    /// The record of what happened at the object level (the Engine Trace).
    Trace,
    /// A detected gap — something the object level couldn't handle.
    Gap,
    /// An attempt to fix a gap without external help.
    Repair,
    /// A request for external help (asking the user to clarify).
    Clarification,
    /// The epistemic state assessment (from epistemics.rs).
    EpistemicAssessment,
}

define_ontology! {
    /// Metacognition — the system observing its own reasoning (von Foerster 1981).
    pub MetaCognitionOntology for MetaCognitionCategory {
        concepts: MetaConcept,
        relation: MetaRelation,
        kind: MetaRelationKind,
        kinds: [
            /// MetaLevel observes ObjectLevel.
            Observes,
            /// Monitoring produces Trace.
            Records,
            /// Evaluation assesses Trace.
            Assesses,
            /// Evaluation detects Gap.
            Detects,
            /// Control decides action (Repair or Clarification).
            Decides,
            /// Gap triggers Repair or Clarification.
            Triggers,
            /// EpistemicAssessment classifies the state of knowledge.
            Classifies,
        ],
        edges: [
            // MetaLevel observes ObjectLevel
            (MetaLevel, ObjectLevel, Observes),
            // Monitoring records Trace
            (Monitoring, Trace, Records),
            // Evaluation assesses Trace
            (Evaluation, Trace, Assesses),
            // Evaluation detects Gap
            (Evaluation, Gap, Detects),
            // Control decides Repair or Clarification
            (Control, Repair, Decides),
            (Control, Clarification, Decides),
            // Gap triggers Repair
            (Gap, Repair, Triggers),
            // Gap triggers Clarification
            (Gap, Clarification, Triggers),
            // EpistemicAssessment classifies Gap
            (EpistemicAssessment, Gap, Classifies),
        ],
        composed: [
            // The second-order loop: MetaLevel → ... → everything
            (MetaLevel, EpistemicAssessment),
            (MetaLevel, Monitoring),
            (MetaLevel, Trace),
            (MetaLevel, Evaluation),
            (MetaLevel, Gap),
            (MetaLevel, Control),
            (MetaLevel, Repair),
            (MetaLevel, Clarification),
            (Monitoring, Evaluation),
            (Monitoring, Gap),
            (Evaluation, Repair),
            (Evaluation, Clarification),
            (Evaluation, Control),
        ],

        being: MentalObject,
        source: "von Foerster (1981); Olivares-Alarcos MOI (2023)",
    }
}
