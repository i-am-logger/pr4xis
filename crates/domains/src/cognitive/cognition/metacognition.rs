//! Metacognition — the system observing its own reasoning.
//!
//! Second-order cybernetics formalized: the observer IS part of the system.
//! The meta-level monitors, evaluates, and controls the object level.
//!
//! When the grammar fails to parse, the meta-level:
//! 1. Detects the failure (monitoring)
//! 2. Diagnoses the cause (evaluation — which type reduction failed?)
//! 3. Decides what to do (control — ask for clarification? attempt repair?)
//!
//! References:
//! - von Foerster, Observing Systems (1981)
//! - Glanville, Second Order Cybernetics
//! - Olivares-Alarcos et al., MOI: Meta-Ontology for Introspection (2023)

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pr4xis::ontology! {
    name: "MetaCognition",
    source: "von Foerster (1981); Olivares-Alarcos MOI (2023)",
    being: MentalObject,

    concepts: [
        ObjectLevel,
        MetaLevel,
        Monitoring,
        Evaluation,
        Control,
        Trace,
        Gap,
        Repair,
        Clarification,
        EpistemicAssessment,
    ],

    labels: {
        ObjectLevel: ("en", "Object level", "The actual reasoning happening (grammar, semantics, queries)."),
        MetaLevel: ("en", "Meta level", "The observer of the reasoning (monitors, evaluates, controls)."),
        Monitoring: ("en", "Monitoring", "What the meta-level watches: the trace of object-level steps."),
        Evaluation: ("en", "Evaluation", "Assessing whether the object level succeeded or failed."),
        Control: ("en", "Control", "Deciding what to do based on evaluation (continue, repair, ask)."),
        Trace: ("en", "Trace", "The record of what happened at the object level (the Engine Trace)."),
        Gap: ("en", "Gap", "A detected gap — something the object level couldn't handle."),
        Repair: ("en", "Repair", "An attempt to fix a gap without external help."),
        Clarification: ("en", "Clarification", "A request for external help (asking the user to clarify)."),
        EpistemicAssessment: ("en", "Epistemic assessment", "The epistemic state assessment (from epistemics.rs)."),
    },

    edges: [
        // MetaLevel observes ObjectLevel
        (MetaLevel, ObjectLevel, Observes),
        // MetaLevel orchestrates its three sub-processes — monitoring,
        // evaluation, control. These make transitive closure reach the
        // leaf concepts (Trace, Gap, Repair, Clarification).
        (MetaLevel, Monitoring, Orchestrates),
        (MetaLevel, Evaluation, Orchestrates),
        (MetaLevel, Control, Orchestrates),
        (MetaLevel, EpistemicAssessment, Orchestrates),
        // Monitoring records Trace; Evaluation feeds on Trace.
        (Monitoring, Trace, Records),
        (Monitoring, Evaluation, Feeds),
        // Evaluation assesses Trace and detects Gap.
        (Evaluation, Trace, Assesses),
        (Evaluation, Gap, Detects),
        (Evaluation, Control, Feeds),
        // Control decides Repair or Clarification.
        (Control, Repair, Decides),
        (Control, Clarification, Decides),
        // Gap triggers Repair / Clarification.
        (Gap, Repair, Triggers),
        (Gap, Clarification, Triggers),
        // EpistemicAssessment classifies Gap.
        (EpistemicAssessment, Gap, Classifies),
    ],
}
