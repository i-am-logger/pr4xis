//! Response Generation ontology — the right adjoint of parsing.
//!
//! Parsing: Text → Syntax → Semantics (left adjoint, F)
//! Generation: Semantics → Syntax → Text (right adjoint, G)
//! Together: Parse ⊣ Generate (adjunction)
//!
//! The unit η: Id → G∘F means: parse then generate ≈ paraphrase.
//! The counit ε: F∘G → Id means: generate then parse ≈ comprehension check.
//! G∘F ≠ Id: paraphrase loses information (many texts → one meaning).
//! F∘G ≠ Id: a meaning can be expressed many ways.
//!
//! This ontology defines the concepts for generating responses from
//! ontological states. It composes:
//! - Metacognition (what happened: gap, repair, clarification)
//! - Epistemics (what the system knows: KK, KU, UK, UU)
//! - SelfModel (what the system IS: components, capabilities)
//! - Speech acts (what kind of response: assertion, question)
//!
//! The response is NOT a hardcoded string. It is composed from
//! ontological concepts that are then realized through the Language.
//!
//! References:
//! - Reiter & Dale, "Building Natural Language Generation Systems" (2000)
//! - White, "Efficient Realization of Coordinate Structures in CCG" (2006)
//! - Lambek & Scott, "Introduction to Higher Order Categorical Logic" (1986)

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pr4xis::ontology! {
    name: "Response",
    source: "Reiter & Dale (2000); Lambek & Scott (1986)",

    concepts: [Intent, EpistemicFrame, Content, SpeechActType, SurfaceForm, Context],

    labels: {
        Intent: ("en", "Intent", "The communicative intent — what the system wants to express. Determined by metacognition."),
        EpistemicFrame: ("en", "Epistemic frame", "The system's knowledge state about this exchange (KK/KU/UK/UU)."),
        Content: ("en", "Content", "The actual information to convey. From self-model, knowledge base, or query results."),
        SpeechActType: ("en", "Speech act type", "Assertion, question, directive, etc. From speech_act.rs."),
        SurfaceForm: ("en", "Surface form", "The linguistic realization. The right adjoint maps intent → text through the Language."),
        Context: ("en", "Context", "What was said before, what the user asked. From dialogue state."),
    },

    edges: [
        // Context constrains Intent
        (Context, Intent, Constrains),
        // EpistemicFrame determines Intent (knowledge state → response intent)
        (EpistemicFrame, Intent, Determines),
        // Epistemics frames Content (KK→assert, KU→ask, UK→suggest, UU→admit)
        (EpistemicFrame, Content, Frames),
        // Intent selects SpeechActType (clarification→question, assertion→statement)
        (Intent, SpeechActType, Selects),
        // Content realizes as SurfaceForm (the generation functor)
        (Content, SurfaceForm, Realizes),
        // SpeechActType shapes SurfaceForm (question → interrogative form)
        (SpeechActType, SurfaceForm, Shapes),
    ],
}

// =========================================================================
// Functor: Epistemics → ResponseFrame
// =========================================================================
//
// Maps each epistemic state to its response framing:
// - KnownKnown → assert with confidence
// - KnownUnknown → acknowledge gap, ask for clarification
// - UnknownKnown → partial understanding, suggest interpretation
// - UnknownUnknown → admit complete lack of understanding

/// The response frame for each epistemic state.
/// These are NOT hardcoded strings — they are semantic frames
/// that the Language realizes into text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFrame {
    /// System knows and can answer confidently.
    AssertKnowledge,
    /// System does not know the VOCABULARY — a genuine `KnownUnknown`: one or
    /// more surfaces resolve to no loaded concept. Realized as "I do not know the
    /// word(s) …". (Formerly the conflated `AcknowledgeGap`, split by FIX-4 so it
    /// no longer masquerades as an unproven-relation abstain.)
    UnknownVocabulary,
    /// System knows BOTH concepts as vocabulary but cannot DERIVE the queried
    /// relation between them from the loaded closure — the fifth epistemic cell,
    /// *vocabulary-known, proposition-open* (distinct from the four-cell
    /// KK/KU/UK/UU state). Honest abstention (Reiter 1978 closed-world): names
    /// both concepts as KNOWN and the relation as unconfirmed, never claiming
    /// ignorance of a word it knows. Realized from a LOADED realization-frame
    /// template (Reiter & Dale 2000, realization stage), not an inline literal.
    UnprovenRelation,
    /// System has partial knowledge — suggest interpretation.
    SuggestInterpretation,
    /// System has no relevant knowledge — admit and guide.
    AdmitLimitation,
    /// Phatic communion (Malinowski 1923): the utterance's function is
    /// establishing/maintaining social contact, not conveying information —
    /// Jakobson's (1960) `Phatic` communicative function. Info-content = ∅ by
    /// design (Malinowski: "phatic communion... in which ties of union are
    /// created by a mere exchange of words" — the words' propositional
    /// content is beside the point). Realized as a reciprocal social
    /// response, never routed through the KK/KU/UK/UU epistemic frames —
    /// there is no proposition here to be known or unknown.
    PhaticReturn,
    /// The rule governing this question is fully known and cited; one or
    /// more of its Slots (Fillmore 1976/1982 Frame Elements) await a Filler
    /// (Bobrow, Kaplan, Norman, Thompson & Winograd 1977, "GUS, A
    /// Frame-Driven Dialog System", *Artificial Intelligence* 8(2):155-173 —
    /// ask, don't guess). Distinct from `UnprovenRelation`'s fifth epistemic
    /// cell (relation itself unknown) — here the relation (the RULE) is
    /// known; what's missing is a fact ABOUT THE ASKER, never derivable from
    /// the loaded closure. Not reachable via `from_epistemic`/`from_jakobson`
    /// — set directly at its call site, same as `UnprovenRelation` and
    /// `PhaticReturn`.
    RuleAwaitingFact,
    /// The multi-turn slot-filling layer (task #17) resolved every
    /// `RequirementLevel::Required` `AppliedElement` (`social::judicial::
    /// conditional_rule`) to `Satisfied` — Sergot et al. (1986)'s reduction
    /// reaches `Applicability::Applies`. Not reachable via
    /// `from_epistemic`/`from_jakobson` — set directly at its call site.
    RuleApplies,
    /// The multi-turn slot-filling layer found a `Required` element
    /// `NotSatisfied` — Sergot et al. (1986)'s reduction reaches
    /// `Applicability::DoesNotApply`. Not reachable via
    /// `from_epistemic`/`from_jakobson` — set directly at its call site.
    RuleDoesNotApply,
    /// A "difference between X and Y" question (Barker 2011 DERIVED
    /// relational noun, `comparison_relation_lexicon`) realized as Mann &
    /// Thompson's (1988) RST Contrast relation — a MULTINUCLEAR relation
    /// (both spans equally important; contrast the mononuclear
    /// Nucleus/Satellite relations like `Elaboration`/`Concession`), the
    /// same `(Nucleus, Nucleus, Contrast)` shape this crate's own
    /// `pragmatics::discourse::ontology` already declares and axiom-checks
    /// (`MultinuclearExists`). Recites each named term's own loaded gloss
    /// side by side rather than asserting a fact ABOUT their relationship:
    /// there is no Contrast edge between two arbitrary defined terms in a
    /// materialized closure — this is a request to RECITE, not a
    /// proposition to VERIFY. Distinct from `AssertKnowledge`'s existing
    /// `entities.len() >= 2` branch, which produces a relation-ASSERTION
    /// sentence ("Yes. {subject} {pred} {object}.") — flatly wrong for this
    /// shape ("Yes. difference relates to Medicare." is not an answer to
    /// "what is the difference between Medicare and Medicaid?"). Not
    /// reachable via `from_epistemic`/`from_jakobson` — set directly at its
    /// call site (`pr4xis_chat::answer_question`), same as
    /// `UnprovenRelation` and `PhaticReturn`.
    Comparison,
}

impl ResponseFrame {
    /// Map from epistemic state to response frame.
    pub fn from_epistemic(
        state: &crate::cognitive::cognition::epistemics::EpistemicConcept,
    ) -> Self {
        use crate::cognitive::cognition::epistemics::EpistemicConcept;
        match state {
            EpistemicConcept::KnownKnown => Self::AssertKnowledge,
            EpistemicConcept::KnownUnknown => Self::UnknownVocabulary,
            EpistemicConcept::UnknownKnown => Self::SuggestInterpretation,
            EpistemicConcept::UnknownUnknown => Self::AdmitLimitation,
        }
    }

    /// The classifier edge `JakobsonFunction::Phatic → ResponseFrame::PhaticReturn`
    /// (R-2 A2): the ONLY Jakobson function with a dedicated response frame —
    /// the other five (Referential/Emotive/Conative/Metalingual/Poetic)
    /// classify what an utterance is FOR, not what epistemic state the
    /// system's ANSWER is in, so they have no honest counterpart here and are
    /// deliberately left unmapped (`None`) rather than forced onto an
    /// unrelated frame.
    pub fn from_jakobson(
        function: crate::formal::information::communication::ontology::JakobsonFunction,
    ) -> Option<Self> {
        use crate::formal::information::communication::ontology::JakobsonFunction;
        match function {
            JakobsonFunction::Phatic => Some(Self::PhaticReturn),
            JakobsonFunction::Referential
            | JakobsonFunction::Emotive
            | JakobsonFunction::Conative
            | JakobsonFunction::Metalingual
            | JakobsonFunction::Poetic => None,
        }
    }
}

/// Ameka's (1992) interjection-function classifier, read onto Jakobson's
/// (1960) function typology — the "loaded phatic lexeme inventory" the R-2
/// A2 generated test walks. `Greeting`/`Farewell`/`Response` are Ameka's own
/// PHATIC subtypes and `Politeness` his phatic-interactional routine
/// (`lexicon/pos.rs` `InterjectionKind` doc); Jakobson's PHATIC function is
/// the same Malinowski (1923) concept both authors name directly, so this
/// mapping is a direct terminological identification, not an invented
/// correspondence. `Expressive` (Ameka's EXPRESSIVE) and `Conative` (Ameka's
/// CONATIVE) are deliberately left unmapped (`None`): Jakobson's Emotive and
/// Conative functions are plausible near-matches but their equivalence to
/// Ameka's categories is not independently cited here, so this classifier
/// only asserts what it can ground.
pub fn jakobson_function_of_interjection(
    kind: crate::cognitive::linguistics::lexicon::pos::InterjectionKind,
) -> Option<crate::formal::information::communication::ontology::JakobsonFunction> {
    use crate::cognitive::linguistics::lexicon::pos::InterjectionKind;
    use crate::formal::information::communication::ontology::JakobsonFunction;
    match kind {
        InterjectionKind::Greeting
        | InterjectionKind::Farewell
        | InterjectionKind::Response
        | InterjectionKind::Politeness => Some(JakobsonFunction::Phatic),
        InterjectionKind::Expressive | InterjectionKind::Conative => None,
    }
}

/// The speech-act-level counterpart of [`jakobson_function_of_interjection`]:
/// `SpeechAct::Greeting` is documented at its own declaration site as
/// "Social/phatic: maintaining social connection... No propositional
/// content — purely relational" — the same Malinowski/Jakobson phatic
/// function, reached via the utterance's illocutionary force rather than its
/// lexical content. Every other `SpeechAct` variant is left unmapped: none
/// of Searle's other four illocutionary categories are phatic.
pub fn jakobson_function_of_speech_act(
    act: crate::cognitive::linguistics::pragmatics::speech_act::SpeechAct,
) -> Option<crate::formal::information::communication::ontology::JakobsonFunction> {
    use crate::cognitive::linguistics::pragmatics::speech_act::SpeechAct;
    use crate::formal::information::communication::ontology::JakobsonFunction;
    match act {
        SpeechAct::Greeting => Some(JakobsonFunction::Phatic),
        SpeechAct::Assertion
        | SpeechAct::Question
        | SpeechAct::Command
        | SpeechAct::Request
        | SpeechAct::Promise
        | SpeechAct::Exclamation
        | SpeechAct::Declaration => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    // The canonical, shared category-law check every sibling ontology file
    // uses (Pharmacology, Counting, Peano, MereologyTheory, ...): Closure +
    // Identity (both directions) + Associativity. Replaces two hand-rolled
    // tests that only checked identity's from/to fields and one-directional
    // left-composition -- a strictly weaker, ad hoc subset of this helper
    // that an adversarial audit correctly flagged as reduced coverage with
    // no indication in the names that it was a partial check.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ResponseCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn has_six_concepts() {
        assert_eq!(ResponseConcept::variants().len(), 6);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn content_realizes_as_surface_form() {
        assert!(
            ResponseCategory::morphisms()
                .iter()
                .any(|m| m.from == ResponseConcept::Content
                    && m.to == ResponseConcept::SurfaceForm
                    && m.kind == ResponseRelationKind::Realizes)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn context_reaches_surface_form() {
        // Per #166 the auto-generated kind no longer emits `Composed`;
        // verify that the direct edges along the path (Context → Intent →
        // SpeechActType → SurfaceForm) all exist.
        let m = ResponseCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == ResponseConcept::Context && r.to == ResponseConcept::Intent)
        );
        assert!(
            m.iter().any(
                |r| r.from == ResponseConcept::Intent && r.to == ResponseConcept::SpeechActType
            )
        );
        assert!(
            m.iter().any(|r| r.from == ResponseConcept::SpeechActType
                && r.to == ResponseConcept::SurfaceForm)
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn epistemic_frames_map_correctly() {
        use crate::cognitive::cognition::epistemics::EpistemicConcept;
        assert_eq!(
            ResponseFrame::from_epistemic(&EpistemicConcept::KnownKnown),
            ResponseFrame::AssertKnowledge
        );
        assert_eq!(
            ResponseFrame::from_epistemic(&EpistemicConcept::KnownUnknown),
            ResponseFrame::UnknownVocabulary
        );
        assert_eq!(
            ResponseFrame::from_epistemic(&EpistemicConcept::UnknownKnown),
            ResponseFrame::SuggestInterpretation
        );
        assert_eq!(
            ResponseFrame::from_epistemic(&EpistemicConcept::UnknownUnknown),
            ResponseFrame::AdmitLimitation
        );
    }

    // === A2 (R-2): phatic dialogue routing ===

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_loaded_phatic_interjection_classifies_as_phatic_return() {
        // The GENERATED probe list: every interjection in the loaded
        // function-word inventory (`build_english_function_words`), never a
        // hand-picked word list. Every Ameka PHATIC subtype
        // (Greeting/Farewell/Response/Politeness) must route through
        // JakobsonFunction::Phatic to ResponseFrame::PhaticReturn; the two
        // non-phatic kinds (Expressive/Conative) must NOT.
        use crate::cognitive::linguistics::language::build_english_function_words;
        use crate::cognitive::linguistics::lexicon::pos::{InterjectionKind, LexicalEntry};

        let words = build_english_function_words();
        let mut phatic_probed = 0usize;
        let mut non_phatic_probed = 0usize;

        for entries in words.values() {
            for entry in entries {
                let LexicalEntry::Interjection(i) = entry else {
                    continue;
                };
                let function = jakobson_function_of_interjection(i.kind);
                let frame = function.and_then(ResponseFrame::from_jakobson);
                let is_phatic_kind = matches!(
                    i.kind,
                    InterjectionKind::Greeting
                        | InterjectionKind::Farewell
                        | InterjectionKind::Response
                        | InterjectionKind::Politeness
                );
                if is_phatic_kind {
                    phatic_probed += 1;
                    assert_eq!(
                        frame,
                        Some(ResponseFrame::PhaticReturn),
                        "{:?} (kind {:?}) must classify as PhaticReturn",
                        i.text,
                        i.kind
                    );
                } else {
                    non_phatic_probed += 1;
                    assert_eq!(
                        frame, None,
                        "{:?} (kind {:?}) must NOT classify as phatic",
                        i.text, i.kind
                    );
                }
            }
        }

        // The loaded inventory must actually EXERCISE both branches — an
        // empty probe set would make the assertions above vacuously true.
        assert!(
            phatic_probed > 0,
            "no loaded phatic interjection was probed"
        );
        assert!(
            non_phatic_probed > 0,
            "no loaded non-phatic interjection was probed"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn speech_act_greeting_classifies_as_phatic_return() {
        use crate::cognitive::linguistics::pragmatics::speech_act::SpeechAct;

        let function = jakobson_function_of_speech_act(SpeechAct::Greeting);
        assert_eq!(
            function.and_then(ResponseFrame::from_jakobson),
            Some(ResponseFrame::PhaticReturn)
        );

        // Every OTHER speech act is honestly left unmapped — phatic-hood is
        // not smuggled in as a default.
        for act in [
            SpeechAct::Assertion,
            SpeechAct::Question,
            SpeechAct::Command,
            SpeechAct::Request,
            SpeechAct::Promise,
            SpeechAct::Exclamation,
            SpeechAct::Declaration,
        ] {
            assert_eq!(
                jakobson_function_of_speech_act(act),
                None,
                "{act:?} must not classify as phatic"
            );
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn non_phatic_jakobson_functions_have_no_response_frame() {
        // The five non-phatic Jakobson functions are deliberately unmapped —
        // ResponseFrame classifies the SYSTEM's epistemic state about an
        // answer, which none of Referential/Emotive/Conative/Metalingual/
        // Poetic have an honest counterpart for.
        use crate::formal::information::communication::ontology::JakobsonFunction;
        for function in [
            JakobsonFunction::Referential,
            JakobsonFunction::Emotive,
            JakobsonFunction::Conative,
            JakobsonFunction::Metalingual,
            JakobsonFunction::Poetic,
        ] {
            assert_eq!(ResponseFrame::from_jakobson(function), None);
        }
    }
}
