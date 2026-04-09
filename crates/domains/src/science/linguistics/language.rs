use super::morphology::MorphologicalRule;
use super::orthography::WritingSystem;

/// A natural language — the complete ontological binding of all linguistic layers.
///
/// A Language is a SocialObject (DOLCE): it exists by social agreement,
/// persists through time, and has rich internal structure.
///
/// Each layer connects to praxis concepts:
/// - Orthography uses the symbols ontology (characters, scripts, punctuation)
/// - Morphology uses reasoning ontology (negation → OppositionDef, derivation → TaxonomyDef)
/// - Grammar uses the engine (ParseState, preconditions enforce structure)
/// - Semantics uses logic (propositions, truth conditions)
/// - Pragmatics uses systems thinking (discourse IS a feedback loop)
pub trait Language {
    /// Human-readable name of this language.
    fn name(&self) -> &str;

    /// ISO 639-1 code (e.g., "en", "he", "ar").
    fn code(&self) -> &str;

    /// The writing system this language uses.
    fn writing_system(&self) -> &WritingSystem;

    /// Morphological rules for word formation.
    fn morphological_rules(&self) -> &[MorphologicalRule];

    /// Number of concepts (meanings) in this language's lexicon.
    fn concept_count(&self) -> usize;

    /// Number of unique words.
    fn word_count(&self) -> usize;
}

/// English language — implements Language using WordNet-backed ontology.
pub struct EnglishLanguage {
    pub ontology: super::english::English,
    pub writing: WritingSystem,
    pub morphology: Vec<MorphologicalRule>,
}

impl EnglishLanguage {
    /// Create English from a WordNet instance.
    pub fn from_wordnet(
        wn: &crate::technology::software::markup::xml::lmf::ontology::WordNet,
    ) -> Self {
        Self {
            ontology: super::english::English::from_wordnet(wn),
            writing: super::orthography::english_writing_system(),
            morphology: super::morphology::english_rules(),
        }
    }
}

impl Language for EnglishLanguage {
    fn name(&self) -> &str {
        "English"
    }

    fn code(&self) -> &str {
        "en"
    }

    fn writing_system(&self) -> &WritingSystem {
        &self.writing
    }

    fn morphological_rules(&self) -> &[MorphologicalRule] {
        &self.morphology
    }

    fn concept_count(&self) -> usize {
        self.ontology.concept_count()
    }

    fn word_count(&self) -> usize {
        self.ontology.word_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::science::linguistics::symbols::character::Direction;

    #[test]
    fn english_language_trait() {
        // Use sample WordNet for testing
        let sample = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dog-n">
      <Lemma writtenForm="dog" partOfSpeech="n"/>
      <Sense id="dog-n-01" synset="s-dog"/>
    </LexicalEntry>
    <Synset id="s-dog" ili="i1" partOfSpeech="n" members="e-dog-n">
      <Definition>a domesticated carnivore</Definition>
    </Synset>
  </Lexicon>
</LexicalResource>"#;

        let wn =
            crate::technology::software::markup::xml::lmf::reader::read_wordnet(sample).unwrap();
        let en = EnglishLanguage::from_wordnet(&wn);

        assert_eq!(en.name(), "English");
        assert_eq!(en.code(), "en");
        assert_eq!(en.writing_system().direction, Direction::LeftToRight);
        assert_eq!(en.concept_count(), 1);
        assert_eq!(en.word_count(), 1);
        assert!(!en.morphological_rules().is_empty());
    }

    #[test]
    fn english_writing_system_complete() {
        let ws = super::super::orthography::english_writing_system();
        // Has Latin script
        assert!(ws.recognizes('a'));
        assert!(ws.recognizes('Z'));
        // Has Arabic numerals
        assert!(ws.recognizes('5'));
        // Has punctuation
        assert!(ws.recognizes('.'));
        assert!(ws.recognizes('?'));
    }

    #[test]
    fn english_morphology_rules() {
        let rules = super::super::morphology::english_rules();
        // Has prefix and suffix rules
        assert!(
            rules
                .iter()
                .any(|r| matches!(r.affix, super::super::morphology::Affix::Prefix(_)))
        );
        assert!(
            rules
                .iter()
                .any(|r| matches!(r.affix, super::super::morphology::Affix::Suffix(_)))
        );
    }
}
