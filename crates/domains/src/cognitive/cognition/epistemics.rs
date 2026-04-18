//! Epistemics — what you know about what you know.
//!
//! The Rumsfeld matrix formalized as an ontology:
//!   Known Known    — knowledge exists AND is accessible
//!   Known Unknown  — absence of knowledge is detected
//!   Unknown Known  — knowledge exists but is NOT accessible (ontology gap)
//!   Unknown Unknown — absence is not even detectable
//!
//! Second-order cybernetics moves things between these states:
//!   Self-observation turns Unknown Unknowns → Known Unknowns
//!   Ontology repair turns Unknown Knowns → Known Knowns
//!   Learning turns Known Unknowns → Known Knowns
//!
//! Open World Assumption:
//!   Failure to find ≠ falsity. "I don't know" is a valid epistemic state.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pr4xis::ontology! {
    name: "Epistemic",
    source: "von Foerster (1981)",
    being: MentalObject,

    concepts: [KnownKnown, KnownUnknown, UnknownKnown, UnknownUnknown],

    labels: {
        KnownKnown: ("en", "Known known", "I know it and can access it. 'Dogs are mammals' — in taxonomy, query works."),
        KnownUnknown: ("en", "Known unknown", "I know I don't know. 'What is a quark?' — not in my ontology, I detect the gap."),
        UnknownKnown: ("en", "Unknown known", "I know it but can't access it. Answer exists but grammar can't parse the question."),
        UnknownUnknown: ("en", "Unknown unknown", "I don't know that I don't know. Not even aware of the gap."),
    },

    edges: [
        // Self-observation: UU → KU (now I know I don't know)
        (UnknownUnknown, KnownUnknown, Observation),
        // Learning: KU → KK (acquired knowledge)
        (KnownUnknown, KnownKnown, Learning),
        // Repair: UK → KK (fixed the access gap)
        (UnknownKnown, KnownKnown, Repair),
        // Discovery: UK → KK (realized I had the answer)
        (UnknownKnown, KnownKnown, Discovery),
        // Forgetting: KK → UK (lost access)
        (KnownKnown, UnknownKnown, Forgetting),
    ],
}

/// Classify the epistemic state of a query result.
pub fn classify_result<T>(
    query_parsed: bool,
    knowledge_exists: bool,
    result: Option<T>,
) -> EpistemicConcept {
    match (query_parsed, knowledge_exists, result.is_some()) {
        (true, true, true) => EpistemicConcept::KnownKnown,
        (true, false, false) => EpistemicConcept::KnownUnknown,
        (false, true, false) => EpistemicConcept::UnknownKnown,
        (false, false, false) => EpistemicConcept::UnknownUnknown,
        (true, true, false) => EpistemicConcept::UnknownKnown,
        (true, false, true) => EpistemicConcept::KnownKnown,
        (false, true, true) => EpistemicConcept::KnownKnown,
        (false, false, true) => EpistemicConcept::KnownKnown,
    }
}
