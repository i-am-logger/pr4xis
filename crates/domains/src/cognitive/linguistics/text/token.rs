#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

// Typed token — a Word occurrence connected through the ontologies.
//
// A token IS a path through Text × Lemon × Lambek × Pipeline:
//
//   Text::Word (occurrence at position)
//       → Lemon::LexicalEntry (lexicon unit)
//       → Lemon::LexicalSense (meaning bridge)
//       → Pipeline::SyntacticStructure (grammatical type)
//
// This replaces the mechanical TypedToken { word: String, lambek_type }.
// Every field is a reference to an ontology concept, not a primitive.
//
// Source: NIF (Hellmann 2013); Ontolex-Lemon (McCrae 2017);
//         Lambek (1958); DisCoCat (Coecke 2010)

use crate::cognitive::linguistics::lambek::types::LambekType;
use crate::cognitive::linguistics::lemon::lexicon::ConceptRef;

/// A pr4xis token — an occurrence of a lexical unit in text.
///
/// Instance of TextConcept::Word. Connected to Lemon (lexicon),
/// Lambek (grammar), and OLiA (annotation) via the bridging
/// concepts in the Text ontology.
///
/// The word field is the surface form (ontolex:writtenRep).
/// The lambek_type is the grammatical type (Lambek pregroup).
/// The sense is the ontology concept referenced (ontolex:reference).
/// The pos is the part-of-speech annotation (OLiA).
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// ontolex:Form writtenRep — the surface realization.
    pub word: String,

    /// Lambek pregroup type — governs grammatical composition.
    pub lambek_type: LambekType,

    /// ontolex:LexicalSense reference — which ontology concept.
    /// None if the word is unknown or not in any ontology.
    pub sense: Option<ConceptRef>,

    /// Part-of-speech from the lexicon (OLiA annotation).
    /// Derived from the lexical entry's POS tag.
    pub pos: Option<crate::cognitive::linguistics::lexicon::pos::PosTag>,

    /// Tense/aspect (Reichenbach 1947; Comrie 1976, 1985) — `None` until a
    /// producer populates it. No current call site sets this to `Some`; the
    /// field exists so a claimed tense/aspect capability
    /// (`crate::cognitive::linguistics::morphology::tense`) has a real,
    /// compileable slot to check for reachability
    /// (`crate::cognitive::linguistics::nlp_task`) rather than the
    /// capability being checkable only by whether the accessor exists at
    /// all (Rust has no runtime reflection for that) — see
    /// [`Self::tense`].
    pub tense: Option<crate::cognitive::linguistics::morphology::tense::TenseAspect>,

    /// Whether this occurrence USES its surface or MENTIONS it — Quine's
    /// use/mention distinction, see
    /// [`ExpressionUse`](crate::cognitive::linguistics::lambek::reduce::ExpressionUse)
    /// for the citation and for why the tokenizer is the only stage that can
    /// still establish it. Carried here so the ontological token is not
    /// STRICTLY POORER than the `TypedToken` it exists to replace: a
    /// conversion that dropped it would silently re-introduce the erasure.
    pub expression_use: crate::cognitive::linguistics::lambek::reduce::ExpressionUse,
}

impl Token {
    /// Whether this token has a resolved ontology reference.
    pub fn has_sense(&self) -> bool {
        self.sense.is_some()
    }

    /// This token's tense/aspect, if resolved. `None` today for every
    /// token — no producer populates [`Self::tense`] yet — an honest,
    /// checkable gap rather than a silently-absent accessor.
    pub fn tense(&self) -> Option<crate::cognitive::linguistics::morphology::tense::TenseAspect> {
        self.tense
    }
}

/// Convert an ontological `Token` to the lower-level `TypedToken` the
/// Lambek reducer currently consumes (drops `sense` / `pos`).
///
/// Used by `chat::process_with_metadata` to feed tokenised input into
/// the reducer. When the reducer is migrated to consume `Token`
/// directly, this conversion disappears.
impl From<Token> for crate::cognitive::linguistics::lambek::reduce::TypedToken {
    fn from(t: Token) -> Self {
        Self {
            word: t.word,
            lambek_type: t.lambek_type,
            expression_use: t.expression_use,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::lambek::types::LambekType;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn token_with_sense() {
        let token = Token {
            word: "dog".into(),
            lambek_type: LambekType::n(),
            sense: Some(ConceptRef {
                ontology: "biology".to_string(),
                concept: "Canine".to_string(),
            }),
            pos: None,
            tense: None,
            expression_use: crate::cognitive::linguistics::lambek::reduce::ExpressionUse::Used,
        };
        assert!(token.has_sense());
        assert_eq!(token.sense.as_ref().unwrap().concept, "Canine");
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn roundtrip_to_legacy() {
        let token = Token {
            word: "runs".into(),
            lambek_type: LambekType::n(),
            sense: Some(ConceptRef {
                ontology: "test".to_string(),
                concept: "Run".to_string(),
            }),
            pos: None,
            tense: None,
            expression_use: crate::cognitive::linguistics::lambek::reduce::ExpressionUse::Used,
        };
        let legacy: crate::cognitive::linguistics::lambek::reduce::TypedToken = token.into();
        assert_eq!(legacy.word, "runs");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn tense_is_none_until_a_producer_populates_it() {
        // No current call site sets `tense` to `Some` -- an honest,
        // checkable gap (a real accessor exists; nothing feeds it yet),
        // not a claimed capability with no accessor to check at all.
        let token = Token {
            word: "ran".into(),
            lambek_type: LambekType::n(),
            sense: None,
            pos: None,
            tense: None,
            expression_use: crate::cognitive::linguistics::lambek::reduce::ExpressionUse::Used,
        };
        assert_eq!(token.tense(), None);
    }
}
