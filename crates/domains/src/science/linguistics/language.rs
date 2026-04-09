use std::collections::HashMap;

use super::lexicon::pos::*;
use super::morphology::MorphologicalRule;
use super::orthography::WritingSystem;
use crate::technology::software::markup::xml::lmf::ontology as lmf;

// The Language trait — the SINGLE interface for all lexical access.
//
// Grounded in:
// - LMF (ISO 24613): Language → Lexicon → LexicalEntry → Form + Sense
// - OntoLex-Lemon (W3C 2016): lexicon-ontology bridge
// - Pustejovsky, The Generative Lexicon (1991): structured lexical entries
//
// The tokenizer calls language.lexical_lookup(word) — it doesn't know
// which language it's processing. English, Hebrew, whatever implements it.
// No static word lists. No hardcoded files. The language IS the ontology.

/// A natural language — the complete ontological binding of all linguistic layers.
pub trait Language {
    /// Human-readable name of this language.
    fn name(&self) -> &str;

    /// ISO 639-1 code (e.g., "en", "he", "ar").
    fn code(&self) -> &str;

    /// The writing system this language uses.
    fn writing_system(&self) -> &WritingSystem;

    /// Morphological rules for word formation.
    fn morphological_rules(&self) -> &[MorphologicalRule];

    /// Look up a word in the language's lexicon.
    /// Returns the lexical entry with full POS and features.
    /// This is the ONLY way to look up words — no static lists, no hardcoding.
    /// Function words and content words both come through here.
    fn lexical_lookup(&self, word: &str) -> Option<LexicalEntry>;

    /// Look up all entries for a word (handles homographs).
    fn lexical_lookup_all(&self, word: &str) -> Vec<LexicalEntry>;

    /// Get all known words (for spelling correction candidate generation).
    fn known_words(&self) -> Vec<&str>;

    /// Number of concepts (meanings) in this language's lexicon.
    fn concept_count(&self) -> usize;

    /// Number of unique words.
    fn word_count(&self) -> usize;
}

/// English language — implements Language using WordNet + function words.
///
/// Function words (closed class) are constructed during language initialization,
/// classified by OLiA categories. Content words come from WordNet's POS.
/// Both are accessed through the same `lexical_lookup` interface.
pub struct EnglishLanguage {
    pub ontology: super::english::English,
    pub writing: WritingSystem,
    pub morphology: Vec<MorphologicalRule>,
    /// Function words — the closed class, built at construction time.
    /// Classified by OLiA: Determiner, Copula, Auxiliary, Pronoun, etc.
    function_words: HashMap<String, Vec<LexicalEntry>>,
    /// All function word texts, for spelling correction.
    function_word_list: Vec<String>,
}

impl EnglishLanguage {
    /// Create English from a WordNet instance.
    /// Function words are constructed here — part of the language, not a separate file.
    pub fn from_wordnet(
        wn: &crate::technology::software::markup::xml::lmf::ontology::WordNet,
    ) -> Self {
        let function_words = build_english_function_words();
        let function_word_list: Vec<String> = function_words.keys().cloned().collect();
        Self {
            ontology: super::english::English::from_wordnet(wn),
            writing: super::orthography::english_writing_system(),
            morphology: super::morphology::english_rules(),
            function_words,
            function_word_list,
        }
    }

    /// Access the underlying English ontology (for concept/taxonomy queries).
    pub fn english(&self) -> &super::english::English {
        &self.ontology
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

    fn lexical_lookup(&self, word: &str) -> Option<LexicalEntry> {
        // Function words first (closed class — finite, checked first)
        if let Some(entries) = self.function_words.get(word) {
            return entries.first().cloned();
        }

        // Content words from WordNet (open class)
        let concept_ids = self.ontology.lookup(word);
        if let Some(&cid) = concept_ids.first()
            && let Some(concept) = self.ontology.concept(cid)
        {
            return lmf_pos_to_lexical_entries(word, concept.pos)
                .into_iter()
                .next();
        }

        None
    }

    fn lexical_lookup_all(&self, word: &str) -> Vec<LexicalEntry> {
        let mut results = Vec::new();

        // Function word entries
        if let Some(entries) = self.function_words.get(word) {
            results.extend(entries.iter().cloned());
        }

        // Content word entries from WordNet (all senses, all types)
        let mut seen_pos = std::collections::HashSet::new();
        for &cid in self.ontology.lookup(word) {
            if let Some(concept) = self.ontology.concept(cid) {
                if seen_pos.insert(concept.pos) {
                    results.extend(lmf_pos_to_lexical_entries(word, concept.pos));
                }
            }
        }

        results
    }

    fn known_words(&self) -> Vec<&str> {
        let mut words: Vec<&str> = self.function_word_list.iter().map(|s| s.as_str()).collect();
        words.extend(self.ontology.word_index.keys().map(|s| s.as_str()));
        words
    }

    fn concept_count(&self) -> usize {
        self.ontology.concept_count()
    }

    fn word_count(&self) -> usize {
        self.ontology.word_count() + self.function_word_list.len()
    }
}

/// Map WordNet's LmfPos to ALL possible lexical entries.
/// Verbs return BOTH transitive and intransitive — the grammar resolves
/// which interpretation succeeds in context. This is how CCG works:
/// lexical ambiguity is resolved by successful derivation.
///
/// "She runs" uses intransitive. "She runs the company" uses transitive.
/// Same word, both valid — the ontology carries both.
fn lmf_pos_to_lexical_entries(word: &str, pos: lmf::LmfPos) -> Vec<LexicalEntry> {
    match pos {
        lmf::LmfPos::Noun => vec![LexicalEntry::Noun(Noun {
            text: word.into(),
            number: Number::Singular,
            person: Person::Third,
            countability: Countability::Countable,
            kind: NounKind::Common,
        })],
        lmf::LmfPos::Verb => vec![
            // Both transitive and intransitive until verb frames disambiguate
            LexicalEntry::Verb(Verb {
                text: word.into(),
                lemma: word.into(),
                number: Number::Singular,
                person: Person::Third,
                tense: Tense::Present,
                transitivity: Transitivity::Intransitive,
            }),
            LexicalEntry::Verb(Verb {
                text: word.into(),
                lemma: word.into(),
                number: Number::Singular,
                person: Person::Third,
                tense: Tense::Present,
                transitivity: Transitivity::Transitive,
            }),
        ],
        lmf::LmfPos::Adjective => vec![LexicalEntry::Adjective(Adjective { text: word.into() })],
        lmf::LmfPos::Adverb => vec![LexicalEntry::Adverb(Adverb { text: word.into() })],
        lmf::LmfPos::Other => vec![LexicalEntry::Noun(Noun {
            text: word.into(),
            number: Number::Singular,
            person: Person::Third,
            countability: Countability::Countable,
            kind: NounKind::Common,
        })],
    }
}

/// Build the English closed-class function words.
/// Classified by OLiA categories. Constructed once during language initialization.
fn build_english_function_words() -> HashMap<String, Vec<LexicalEntry>> {
    let mut map: HashMap<String, Vec<LexicalEntry>> = HashMap::new();

    let mut add = |entry: LexicalEntry| {
        let text = entry.text().to_string();
        map.entry(text).or_default().push(entry);
    };

    // ---- Determiners (OLiA: Determiner) ----
    for (text, def, num) in [
        ("the", Definiteness::Definite, None),
        ("a", Definiteness::Indefinite, Some(Number::Singular)),
        ("an", Definiteness::Indefinite, Some(Number::Singular)),
        ("this", Definiteness::Demonstrative, Some(Number::Singular)),
        ("that", Definiteness::Demonstrative, Some(Number::Singular)),
        ("these", Definiteness::Demonstrative, Some(Number::Plural)),
        ("those", Definiteness::Demonstrative, Some(Number::Plural)),
        ("every", Definiteness::Quantifier, Some(Number::Singular)),
        ("some", Definiteness::Quantifier, None),
        ("no", Definiteness::Quantifier, None),
        ("all", Definiteness::Quantifier, Some(Number::Plural)),
        ("any", Definiteness::Quantifier, None),
        ("each", Definiteness::Quantifier, Some(Number::Singular)),
    ] {
        add(LexicalEntry::Determiner(Determiner {
            text: text.into(),
            definiteness: def,
            number: num,
        }));
    }

    // ---- Copulas (OLiA: Copula) ----
    for (text, num, per, tense) in [
        ("is", Number::Singular, Person::Third, Tense::Present),
        ("are", Number::Plural, Person::Third, Tense::Present),
        ("am", Number::Singular, Person::First, Tense::Present),
        ("was", Number::Singular, Person::Third, Tense::Past),
        ("were", Number::Plural, Person::Third, Tense::Past),
    ] {
        add(LexicalEntry::Copula(Copula {
            text: text.into(),
            number: num,
            person: per,
            tense,
        }));
    }

    // ---- Auxiliaries (OLiA: AuxiliaryVerb) ----
    for text in [
        "has", "have", "had", "do", "does", "did", "will", "would", "can", "could", "shall",
        "should", "may", "might", "must",
    ] {
        add(LexicalEntry::Auxiliary(Auxiliary {
            text: text.into(),
            number: None,
            tense: None,
        }));
    }

    // ---- Personal Pronouns (OLiA: PersonalPronoun) ----
    for (text, num, per) in [
        ("i", Number::Singular, Person::First),
        ("you", Number::Singular, Person::Second),
        ("he", Number::Singular, Person::Third),
        ("she", Number::Singular, Person::Third),
        ("it", Number::Singular, Person::Third),
        ("we", Number::Plural, Person::First),
        ("they", Number::Plural, Person::Third),
        ("me", Number::Singular, Person::First),
        ("him", Number::Singular, Person::Third),
        ("her", Number::Singular, Person::Third),
        ("us", Number::Plural, Person::First),
        ("them", Number::Plural, Person::Third),
    ] {
        add(LexicalEntry::Pronoun(Pronoun {
            text: text.into(),
            number: num,
            person: per,
            kind: PronounKind::Personal,
        }));
    }

    // ---- Interrogative Pronouns (OLiA: InterrogativePronoun) ----
    for text in ["what", "who", "which"] {
        add(LexicalEntry::Pronoun(Pronoun {
            text: text.into(),
            number: Number::Singular,
            person: Person::Third,
            kind: PronounKind::Interrogative,
        }));
    }

    // ---- Prepositions (OLiA: Preposition) ----
    for text in [
        "in", "on", "at", "with", "to", "from", "by", "for", "of", "about", "into", "through",
        "during", "before", "after", "above", "below", "between", "under", "over",
    ] {
        add(LexicalEntry::Preposition(Preposition { text: text.into() }));
    }

    // ---- Conjunctions (OLiA: Conjunction) ----
    for text in [
        "and", "but", "or", "so", "yet", "nor", "because", "although", "if", "when",
    ] {
        add(LexicalEntry::Conjunction(Conjunction { text: text.into() }));
    }

    // ---- Particles (OLiA: Particle) ----
    for text in ["not", "to"] {
        add(LexicalEntry::Particle(Particle { text: text.into() }));
    }

    // ---- Interjections (OLiA: Interjection) ----
    for text in [
        "hello", "hi", "hey", "oh", "wow", "yes", "no", "please", "thanks", "goodbye", "bye",
    ] {
        add(LexicalEntry::Interjection(Interjection {
            text: text.into(),
        }));
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::science::linguistics::symbols::character::Direction;

    fn sample_wn() -> crate::technology::software::markup::xml::lmf::ontology::WordNet {
        let sample = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dog-n">
      <Lemma writtenForm="dog" partOfSpeech="n"/>
      <Sense id="dog-n-01" synset="s-dog"/>
    </LexicalEntry>
    <LexicalEntry id="e-run-v">
      <Lemma writtenForm="run" partOfSpeech="v"/>
      <Sense id="run-v-01" synset="s-run"/>
    </LexicalEntry>
    <Synset id="s-dog" ili="i1" partOfSpeech="n" members="e-dog-n">
      <Definition>a domesticated carnivore</Definition>
    </Synset>
    <Synset id="s-run" ili="i2" partOfSpeech="v" members="e-run-v">
      <Definition>move fast</Definition>
    </Synset>
  </Lexicon>
</LexicalResource>"#;
        crate::technology::software::markup::xml::lmf::reader::read_wordnet(sample).unwrap()
    }

    #[test]
    fn english_language_trait() {
        let wn = sample_wn();
        let en = EnglishLanguage::from_wordnet(&wn);
        assert_eq!(en.name(), "English");
        assert_eq!(en.code(), "en");
        assert_eq!(en.writing_system().direction, Direction::LeftToRight);
        assert!(!en.morphological_rules().is_empty());
    }

    #[test]
    fn lexical_lookup_function_word() {
        let wn = sample_wn();
        let en = EnglishLanguage::from_wordnet(&wn);
        let the = en.lexical_lookup("the").unwrap();
        assert_eq!(the.pos_tag(), PosTag::Determiner);
    }

    #[test]
    fn lexical_lookup_content_word() {
        let wn = sample_wn();
        let en = EnglishLanguage::from_wordnet(&wn);
        let dog = en.lexical_lookup("dog").unwrap();
        assert_eq!(dog.pos_tag(), PosTag::Noun);
    }

    #[test]
    fn lexical_lookup_copula() {
        let wn = sample_wn();
        let en = EnglishLanguage::from_wordnet(&wn);
        let is = en.lexical_lookup("is").unwrap();
        assert_eq!(is.pos_tag(), PosTag::Copula);
    }

    #[test]
    fn lexical_lookup_interrogative_pronoun() {
        let wn = sample_wn();
        let en = EnglishLanguage::from_wordnet(&wn);
        let what = en.lexical_lookup("what").unwrap();
        assert!(what.is_interrogative());
        assert!(!what.is_anaphoric());
    }

    #[test]
    fn lexical_lookup_personal_pronoun() {
        let wn = sample_wn();
        let en = EnglishLanguage::from_wordnet(&wn);
        let it = en.lexical_lookup("it").unwrap();
        assert!(it.is_anaphoric());
        assert!(!it.is_interrogative());
    }

    #[test]
    fn lexical_lookup_unknown() {
        let wn = sample_wn();
        let en = EnglishLanguage::from_wordnet(&wn);
        assert!(en.lexical_lookup("xyzzy").is_none());
    }

    #[test]
    fn known_words_includes_both() {
        let wn = sample_wn();
        let en = EnglishLanguage::from_wordnet(&wn);
        let words = en.known_words();
        assert!(words.contains(&"the")); // function word
        assert!(words.contains(&"dog")); // content word
    }

    #[test]
    fn writing_system_complete() {
        let ws = super::super::orthography::english_writing_system();
        assert!(ws.recognizes('a'));
        assert!(ws.recognizes('Z'));
        assert!(ws.recognizes('5'));
        assert!(ws.recognizes('.'));
    }
}
