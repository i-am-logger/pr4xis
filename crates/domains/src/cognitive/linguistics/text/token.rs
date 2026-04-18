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
}

impl Token {
    /// Whether this token has a resolved ontology reference.
    pub fn has_sense(&self) -> bool {
        self.sense.is_some()
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::lambek::types::LambekType;

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
        };
        assert!(token.has_sense());
        assert_eq!(token.sense.as_ref().unwrap().concept, "Canine");
    }

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
        };
        let legacy: crate::cognitive::linguistics::lambek::reduce::TypedToken = token.into();
        assert_eq!(legacy.word, "runs");
    }
}
