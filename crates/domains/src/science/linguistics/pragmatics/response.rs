use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

// Response Generation ontology — the right adjoint of parsing.
//
// Parsing: Text → Syntax → Semantics (left adjoint, F)
// Generation: Semantics → Syntax → Text (right adjoint, G)
// Together: Parse ⊣ Generate (adjunction)
//
// The unit η: Id → G∘F means: parse then generate ≈ paraphrase.
// The counit ε: F∘G → Id means: generate then parse ≈ comprehension check.
// G∘F ≠ Id: paraphrase loses information (many texts → one meaning).
// F∘G ≠ Id: a meaning can be expressed many ways.
//
// This ontology defines the concepts for generating responses from
// ontological states. It composes:
// - Metacognition (what happened: gap, repair, clarification)
// - Epistemics (what the system knows: KK, KU, UK, UU)
// - SelfModel (what the system IS: components, capabilities)
// - Speech acts (what kind of response: assertion, question)
//
// The response is NOT a hardcoded string. It is composed from
// ontological concepts that are then realized through the Language.
//
// References:
// - Reiter & Dale, "Building Natural Language Generation Systems" (2000)
// - White, "Efficient Realization of Coordinate Structures in CCG" (2006)
// - Lambek & Scott, "Introduction to Higher Order Categorical Logic" (1986)

/// Concepts in response generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResponseConcept {
    /// The communicative intent — what the system wants to express.
    /// Determined by metacognition (gap → clarify, repair → suggest).
    Intent,
    /// The epistemic frame — the system's knowledge state about this exchange.
    /// From epistemics: KnownKnown, KnownUnknown, UnknownKnown, UnknownUnknown.
    EpistemicFrame,
    /// The content — the actual information to convey.
    /// From self-model, knowledge base, or query results.
    Content,
    /// The speech act type — assertion, question, directive, etc.
    /// From speech_act.rs.
    SpeechActType,
    /// The surface form — the linguistic realization.
    /// The right adjoint maps intent → text through the Language.
    SurfaceForm,
    /// The context — what was said before, what the user asked.
    /// From dialogue state.
    Context,
}

impl Entity for ResponseConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Intent,
            Self::EpistemicFrame,
            Self::Content,
            Self::SpeechActType,
            Self::SurfaceForm,
            Self::Context,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResponseRelation {
    pub from: ResponseConcept,
    pub to: ResponseConcept,
    pub kind: ResponseRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResponseRelationKind {
    Identity,
    /// Metacognition determines Intent.
    Determines,
    /// Epistemics frames the Content.
    Frames,
    /// Intent selects SpeechActType.
    Selects,
    /// Content realizes as SurfaceForm (the generation step).
    Realizes,
    /// Context constrains Intent.
    Constrains,
    /// SpeechActType shapes SurfaceForm.
    Shapes,
    Composed,
}

impl Relationship for ResponseRelation {
    type Object = ResponseConcept;
    fn source(&self) -> ResponseConcept {
        self.from
    }
    fn target(&self) -> ResponseConcept {
        self.to
    }
}

pub struct ResponseCategory;

impl Category for ResponseCategory {
    type Object = ResponseConcept;
    type Morphism = ResponseRelation;

    fn identity(obj: &ResponseConcept) -> ResponseRelation {
        ResponseRelation {
            from: *obj,
            to: *obj,
            kind: ResponseRelationKind::Identity,
        }
    }

    fn compose(f: &ResponseRelation, g: &ResponseRelation) -> Option<ResponseRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == ResponseRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == ResponseRelationKind::Identity {
            return Some(f.clone());
        }
        Some(ResponseRelation {
            from: f.from,
            to: g.to,
            kind: ResponseRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<ResponseRelation> {
        use ResponseConcept as C;
        use ResponseRelationKind as R;
        let mut m = Vec::new();

        for c in ResponseConcept::variants() {
            m.push(ResponseRelation {
                from: c,
                to: c,
                kind: R::Identity,
            });
        }

        // Metacognition → Intent (gap/repair/clarification determines what to say)
        m.push(ResponseRelation {
            from: C::Context,
            to: C::Intent,
            kind: R::Constrains,
        });
        // Epistemics frames Content (KK→assert, KU→ask, UK→suggest, UU→admit)
        m.push(ResponseRelation {
            from: C::EpistemicFrame,
            to: C::Content,
            kind: R::Frames,
        });
        // Intent selects SpeechActType (clarification→question, assertion→statement)
        m.push(ResponseRelation {
            from: C::Intent,
            to: C::SpeechActType,
            kind: R::Selects,
        });
        // Content realizes as SurfaceForm (the generation functor)
        m.push(ResponseRelation {
            from: C::Content,
            to: C::SurfaceForm,
            kind: R::Realizes,
        });
        // SpeechActType shapes SurfaceForm (question → interrogative form)
        m.push(ResponseRelation {
            from: C::SpeechActType,
            to: C::SurfaceForm,
            kind: R::Shapes,
        });

        // Transitive
        m.push(ResponseRelation {
            from: C::Context,
            to: C::SpeechActType,
            kind: R::Composed,
        });
        m.push(ResponseRelation {
            from: C::Context,
            to: C::SurfaceForm,
            kind: R::Composed,
        });
        m.push(ResponseRelation {
            from: C::Intent,
            to: C::SurfaceForm,
            kind: R::Composed,
        });
        m.push(ResponseRelation {
            from: C::EpistemicFrame,
            to: C::SurfaceForm,
            kind: R::Composed,
        });

        for c in ResponseConcept::variants() {
            m.push(ResponseRelation {
                from: c,
                to: c,
                kind: R::Composed,
            });
        }

        m
    }
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
    /// System knows what it doesn't know — acknowledge gap.
    AcknowledgeGap,
    /// System has partial knowledge — suggest interpretation.
    SuggestInterpretation,
    /// System has no relevant knowledge — admit and guide.
    AdmitLimitation,
}

impl ResponseFrame {
    /// Map from epistemic state to response frame.
    pub fn from_epistemic(state: &crate::science::cognition::epistemics::EpistemicState) -> Self {
        use crate::science::cognition::epistemics::EpistemicState;
        match state {
            EpistemicState::KnownKnown => Self::AssertKnowledge,
            EpistemicState::KnownUnknown => Self::AcknowledgeGap,
            EpistemicState::UnknownKnown => Self::SuggestInterpretation,
            EpistemicState::UnknownUnknown => Self::AdmitLimitation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::entity::Entity;

    #[test]
    fn category_identity_law() {
        for obj in ResponseConcept::variants() {
            let id = ResponseCategory::identity(&obj);
            assert_eq!(id.from, obj);
            assert_eq!(id.to, obj);
        }
    }

    #[test]
    fn category_composition_with_identity() {
        for m in &ResponseCategory::morphisms() {
            let left = ResponseCategory::compose(&ResponseCategory::identity(&m.from), m).unwrap();
            assert_eq!(left.from, m.from);
            assert_eq!(left.to, m.to);
        }
    }

    #[test]
    fn has_six_concepts() {
        assert_eq!(ResponseConcept::variants().len(), 6);
    }

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

    #[test]
    fn context_reaches_surface_form() {
        // Full path: Context → Intent → SpeechActType → SurfaceForm
        assert!(
            ResponseCategory::morphisms()
                .iter()
                .any(|m| m.from == ResponseConcept::Context
                    && m.to == ResponseConcept::SurfaceForm
                    && m.kind == ResponseRelationKind::Composed)
        );
    }

    #[test]
    fn epistemic_frames_map_correctly() {
        use crate::science::cognition::epistemics::EpistemicState;
        assert_eq!(
            ResponseFrame::from_epistemic(&EpistemicState::KnownKnown),
            ResponseFrame::AssertKnowledge
        );
        assert_eq!(
            ResponseFrame::from_epistemic(&EpistemicState::KnownUnknown),
            ResponseFrame::AcknowledgeGap
        );
        assert_eq!(
            ResponseFrame::from_epistemic(&EpistemicState::UnknownKnown),
            ResponseFrame::SuggestInterpretation
        );
        assert_eq!(
            ResponseFrame::from_epistemic(&EpistemicState::UnknownUnknown),
            ResponseFrame::AdmitLimitation
        );
    }
}
