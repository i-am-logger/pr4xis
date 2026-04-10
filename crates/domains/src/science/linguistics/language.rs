use std::collections::HashMap;

use super::lexicon::pos::*;
use super::morphology::MorphologicalRule;
use super::orthography::WritingSystem;
use crate::science::linguistics::lambek::pregroup::{self, PregroupType};
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

    /// Get the pregroup type(s) for a word.
    /// The pregroup type determines how the word composes grammatically.
    /// Returns all possible types (verbs may be both transitive and intransitive).
    fn pregroup_types(&self, word: &str) -> Vec<PregroupType>;

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
    function_words: HashMap<String, Vec<LexicalEntry>>,
    /// All function word texts, for spelling correction.
    function_word_list: Vec<String>,
    /// Verb transitivity from WordNet subcategorization frames.
    /// Pre-computed at construction time from LMF Sense.subcat.
    verb_transitivity: HashMap<String, Vec<Transitivity>>,
}

impl EnglishLanguage {
    /// Create English from a WordNet instance.
    /// Function words are constructed here — part of the language, not a separate file.
    /// Verb transitivity is pre-computed from WordNet subcategorization frames.
    pub fn from_wordnet(
        wn: &crate::technology::software::markup::xml::lmf::ontology::WordNet,
    ) -> Self {
        let function_words = build_english_function_words();
        let function_word_list: Vec<String> = function_words.keys().cloned().collect();
        let verb_transitivity = build_verb_transitivity(wn);
        Self {
            ontology: super::english::English::from_wordnet(wn),
            writing: super::orthography::english_writing_system(),
            morphology: super::morphology::english_rules(),
            function_words,
            function_word_list,
            verb_transitivity,
        }
    }

    /// Access the underlying English ontology (for concept/taxonomy queries).
    pub fn english(&self) -> &super::english::English {
        &self.ontology
    }

    /// Get verb transitivity options for a word from pre-computed frames.
    fn verb_transitivities(&self, word: &str) -> &[Transitivity] {
        self.verb_transitivity
            .get(word)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
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
            let transitivities = self.verb_transitivities(word);
            return lmf_pos_to_lexical_entries(word, concept.pos, transitivities)
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

        // Content word entries from WordNet — use verb frames for transitivity
        let mut seen_pos = std::collections::HashSet::new();
        for &cid in self.ontology.lookup(word) {
            if let Some(concept) = self.ontology.concept(cid)
                && seen_pos.insert(concept.pos)
            {
                let transitivities = self.verb_transitivities(word);
                results.extend(lmf_pos_to_lexical_entries(
                    word,
                    concept.pos,
                    transitivities,
                ));
            }
        }

        results
    }

    fn pregroup_types(&self, word: &str) -> Vec<PregroupType> {
        self.lexical_lookup_all(word)
            .iter()
            .map(lexical_entry_to_pregroup)
            .collect()
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
/// For verbs, uses transitivity from WordNet subcategorization frames.
/// If no frames are available, returns both transitive and intransitive.
pub fn lmf_pos_to_lexical_entries(
    word: &str,
    pos: lmf::LmfPos,
    verb_transitivities: &[Transitivity],
) -> Vec<LexicalEntry> {
    match pos {
        lmf::LmfPos::Noun => vec![LexicalEntry::Noun(Noun {
            text: word.into(),
            number: Number::Singular,
            person: Person::Third,
            countability: Countability::Countable,
            kind: NounKind::Common,
        })],
        lmf::LmfPos::Verb => {
            if verb_transitivities.is_empty() {
                // No frame data — return both (grammar resolves in context)
                vec![
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
                ]
            } else {
                // Frame data available — return only the known transitivities
                verb_transitivities
                    .iter()
                    .map(|&t| {
                        LexicalEntry::Verb(Verb {
                            text: word.into(),
                            lemma: word.into(),
                            number: Number::Singular,
                            person: Person::Third,
                            tense: Tense::Present,
                            transitivity: t,
                        })
                    })
                    .collect()
            }
        }
        lmf::LmfPos::Adjective => vec![LexicalEntry::Adjective(Adjective { text: word.into() })],
        lmf::LmfPos::Adverb => vec![LexicalEntry::Adverb(Adverb { text: word.into() })],
        lmf::LmfPos::Determiner | lmf::LmfPos::Numeral => {
            vec![LexicalEntry::Determiner(Determiner {
                text: word.into(),
                definiteness: Definiteness::Indefinite,
                number: None,
            })]
        }
        lmf::LmfPos::Pronoun => vec![LexicalEntry::Pronoun(Pronoun {
            text: word.into(),
            kind: PronounKind::Personal,
            number: Number::Singular,
            person: Person::Third,
        })],
        lmf::LmfPos::Preposition => {
            vec![LexicalEntry::Preposition(Preposition { text: word.into() })]
        }
        lmf::LmfPos::Conjunction => {
            vec![LexicalEntry::Conjunction(Conjunction { text: word.into() })]
        }
        lmf::LmfPos::Particle => vec![LexicalEntry::Particle(Particle { text: word.into() })],
        lmf::LmfPos::Copula => vec![LexicalEntry::Copula(Copula {
            text: word.into(),
            number: Number::Singular,
            person: Person::Third,
            tense: Tense::Present,
        })],
        lmf::LmfPos::Auxiliary => {
            vec![LexicalEntry::Auxiliary(Auxiliary {
                text: word.into(),
                number: Some(Number::Singular),
                tense: Some(Tense::Present),
            })]
        }
        lmf::LmfPos::Interjection => vec![LexicalEntry::Interjection(Interjection {
            text: word.into(),
            kind: InterjectionKind::Expressive,
        })],
        lmf::LmfPos::Other => vec![LexicalEntry::Noun(Noun {
            text: word.into(),
            number: Number::Singular,
            person: Person::Third,
            countability: Countability::Countable,
            kind: NounKind::Common,
        })],
    }
}

/// Map a lexical entry to its pregroup type.
/// This is the bridge between the lexicon ontology and the grammar ontology.
pub fn lexical_entry_to_pregroup(entry: &LexicalEntry) -> PregroupType {
    use pregroup::{BasicType, PregroupElement};

    match entry {
        LexicalEntry::Noun(_) => pregroup::svo::noun(),
        LexicalEntry::Verb(v) => match v.transitivity {
            Transitivity::Intransitive => pregroup::svo::intransitive_verb(),
            Transitivity::Transitive => pregroup::svo::transitive_verb(),
            Transitivity::Ditransitive => {
                // np^r · s · np^l · np^l (subject + two objects)
                PregroupType::new(vec![
                    PregroupElement::right_adj(BasicType::NP),
                    PregroupElement::basic(BasicType::S),
                    PregroupElement::left_adj(BasicType::NP),
                    PregroupElement::left_adj(BasicType::NP),
                ])
            }
        },
        LexicalEntry::Determiner(_) => pregroup::svo::determiner(),
        LexicalEntry::Adjective(_) => pregroup::svo::adjective(),
        LexicalEntry::Adverb(_) => {
            // Adverb modifies verb: (np^r · s)^r · np^r · s
            // Simplified: s^r · np · np^r · s = modifier of VP
            // For now, use simple s · s^l (sentence modifier)
            PregroupType::new(vec![
                PregroupElement::basic(BasicType::S),
                PregroupElement::left_adj(BasicType::S),
            ])
        }
        LexicalEntry::Preposition(_) => {
            // pp · np^l (takes NP on right, produces PP)
            PregroupType::new(vec![
                PregroupElement::basic(BasicType::PP),
                PregroupElement::left_adj(BasicType::NP),
            ])
        }
        LexicalEntry::Pronoun(_) => pregroup::svo::proper_noun(),
        LexicalEntry::Conjunction(_) => {
            // Simplified: s · s^l · s^l (joins two sentences)
            PregroupType::new(vec![
                PregroupElement::basic(BasicType::S),
                PregroupElement::left_adj(BasicType::S),
                PregroupElement::left_adj(BasicType::S),
            ])
        }
        LexicalEntry::Copula(_) => {
            // Copula with NP predicate: np^r · s · np^l (like transitive)
            pregroup::svo::transitive_verb()
        }
        LexicalEntry::Auxiliary(_) => {
            // Auxiliary modifies VP: (np^r · s)^r · np^r · s
            // Simplified: s · s^l (sentence-level modifier)
            PregroupType::new(vec![
                PregroupElement::basic(BasicType::S),
                PregroupElement::left_adj(BasicType::S),
            ])
        }
        LexicalEntry::Interjection(_) => {
            // Standalone: s
            PregroupType::single(BasicType::S)
        }
        LexicalEntry::Particle(_) => {
            // Modifier: s · s^l
            PregroupType::new(vec![
                PregroupElement::basic(BasicType::S),
                PregroupElement::left_adj(BasicType::S),
            ])
        }
        LexicalEntry::Numeral(_) => pregroup::svo::determiner(),
    }
}

/// Pre-compute verb transitivity from WordNet subcategorization frames.
pub fn build_verb_transitivity(
    wn: &crate::technology::software::markup::xml::lmf::ontology::WordNet,
) -> HashMap<String, Vec<Transitivity>> {
    let mut result: HashMap<String, Vec<Transitivity>> = HashMap::new();

    for entry in &wn.entries {
        if entry.lemma.pos != lmf::LmfPos::Verb {
            continue;
        }
        let word = &entry.lemma.written_form;

        for sense in &entry.senses {
            for frame_id in &sense.subcat {
                if let Some(vt) = lmf::VerbTransitivity::from_frame_id(frame_id) {
                    let transitivity = match vt {
                        lmf::VerbTransitivity::Intransitive => Transitivity::Intransitive,
                        lmf::VerbTransitivity::Transitive => Transitivity::Transitive,
                        lmf::VerbTransitivity::Ditransitive => Transitivity::Ditransitive,
                    };
                    let entry = result.entry(word.to_lowercase()).or_default();
                    if !entry.contains(&transitivity) {
                        entry.push(transitivity);
                    }
                }
            }
        }
    }

    result
}

/// Build function words from LMF data file.
///
/// Parses the same LMF format as WordNet — function words are DATA,
/// not hardcoded Rust. The synset IDs encode linguistic features
/// (OLiA categories: definite-det, personal-pron, greeting, etc.).
///
/// Falls back to embedded XML if the data file is not found.
pub fn build_english_function_words() -> HashMap<String, Vec<LexicalEntry>> {
    // Try loading from data file first
    let data_paths = [
        "crates/domains/data/function-words/english.xml",
        "data/function-words/english.xml",
        "../domains/data/function-words/english.xml",
    ];

    for path in &data_paths {
        if let Ok(xml) = std::fs::read_to_string(path)
            && let Ok(wn) =
                crate::technology::software::markup::xml::lmf::reader::read_wordnet(&xml)
        {
            return function_words_from_lmf(&wn);
        }
    }

    // Fallback: embedded minimal LMF for when data file isn't available (tests, WASM)
    build_english_function_words_embedded()
}

/// Parse function words from an LMF WordNet instance.
/// Maps synset categories (from OLiA) to rich LexicalEntry types.
fn function_words_from_lmf(
    wn: &crate::technology::software::markup::xml::lmf::ontology::WordNet,
) -> HashMap<String, Vec<LexicalEntry>> {
    let mut map: HashMap<String, Vec<LexicalEntry>> = HashMap::new();

    // Build synset → synset_id lookup
    let synset_ids: HashMap<String, &str> = wn
        .synsets
        .iter()
        .map(|s| (s.id.clone(), s.id.as_str()))
        .collect();

    for entry in &wn.entries {
        let word = entry.lemma.written_form.to_lowercase();
        let synset_id = entry
            .senses
            .first()
            .map(|s| s.synset.as_str())
            .unwrap_or("");

        let lexical_entry = match entry.lemma.pos {
            lmf::LmfPos::Determiner => {
                let definiteness =
                    if synset_id.contains("definite") && !synset_id.contains("indefinite") {
                        Definiteness::Definite
                    } else if synset_id.contains("demonstrative") {
                        Definiteness::Demonstrative
                    } else if synset_id.contains("universal") || synset_id.contains("negative") {
                        Definiteness::Quantifier
                    } else {
                        Definiteness::Indefinite
                    };
                LexicalEntry::Determiner(Determiner {
                    text: word.clone(),
                    definiteness,
                    number: None,
                })
            }
            lmf::LmfPos::Copula => LexicalEntry::Copula(Copula {
                text: word.clone(),
                number: Number::Singular,
                person: Person::Third,
                tense: Tense::Present,
            }),
            lmf::LmfPos::Auxiliary => LexicalEntry::Auxiliary(Auxiliary {
                text: word.clone(),
                number: None,
                tense: None,
            }),
            lmf::LmfPos::Pronoun => {
                let kind = if synset_id.contains("interrogative") {
                    PronounKind::Interrogative
                } else {
                    PronounKind::Personal
                };
                LexicalEntry::Pronoun(Pronoun {
                    text: word.clone(),
                    number: Number::Singular,
                    person: Person::Third,
                    kind,
                })
            }
            lmf::LmfPos::Preposition => {
                LexicalEntry::Preposition(Preposition { text: word.clone() })
            }
            lmf::LmfPos::Conjunction => {
                LexicalEntry::Conjunction(Conjunction { text: word.clone() })
            }
            lmf::LmfPos::Particle => LexicalEntry::Particle(Particle { text: word.clone() }),
            lmf::LmfPos::Interjection => {
                let kind = if synset_id.contains("greeting") {
                    InterjectionKind::Greeting
                } else if synset_id.contains("farewell") {
                    InterjectionKind::Farewell
                } else if synset_id.contains("politeness") {
                    InterjectionKind::Politeness
                } else if synset_id.contains("response") {
                    InterjectionKind::Response
                } else {
                    InterjectionKind::Expressive
                };
                LexicalEntry::Interjection(Interjection {
                    text: word.clone(),
                    kind,
                })
            }
            _ => continue, // Skip non-function-word POS
        };

        let _ = synset_ids; // used for future feature expansion
        map.entry(word).or_default().push(lexical_entry);
    }

    map
}

/// Embedded function words — used when data file is not available.
/// Same content as data/function-words/english.xml, but inline.
fn build_english_function_words_embedded() -> HashMap<String, Vec<LexicalEntry>> {
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

    // ---- Interjections (OLiA: Interjection) — classified by function ----
    for (text, kind) in [
        ("hello", InterjectionKind::Greeting),
        ("hi", InterjectionKind::Greeting),
        ("hey", InterjectionKind::Greeting),
        ("goodbye", InterjectionKind::Farewell),
        ("bye", InterjectionKind::Farewell),
        ("quit", InterjectionKind::Farewell),
        ("exit", InterjectionKind::Farewell),
        ("oh", InterjectionKind::Expressive),
        ("wow", InterjectionKind::Expressive),
        ("yes", InterjectionKind::Response),
        ("no", InterjectionKind::Response),
        ("please", InterjectionKind::Politeness),
        ("thanks", InterjectionKind::Politeness),
    ] {
        add(LexicalEntry::Interjection(Interjection {
            text: text.into(),
            kind,
        }));
    }

    map
}

// =========================================================================
// Codegen → Language functor
// =========================================================================
//
// Three deployment functors (roadmap.md):
//   Codegen (0s, static), Mmap (2ms, file), Async (1.25s, heap)
//   All produce the same Language. Equivalence proven.
//
// This is the codegen functor: CodegenData → Language.
// Language-agnostic: maps static arrays to runtime ontology structures.
// No knowledge of any specific language — only the Language interface.

/// Codegen → Language functor.
///
/// Maps language-agnostic static arrays (produced at build time)
/// to a live Language instance. Zero XML parsing.
pub fn from_codegen(data: &praxis::codegen_data::CodegenData) -> super::english::English {
    use super::english::{Concept, ConceptId, SenseId};

    let pos_from_str = |s: &str| match s {
        "n" => lmf::LmfPos::Noun,
        "v" => lmf::LmfPos::Verb,
        "a" | "s" => lmf::LmfPos::Adjective,
        "r" => lmf::LmfPos::Adverb,
        _ => lmf::LmfPos::Other,
    };

    // Phase 1: Concepts from static arrays
    let mut concepts = Vec::with_capacity(data.entity_count);
    let mut synset_to_concept = HashMap::new();
    for idx in 0..data.entity_count {
        let concept_id = ConceptId::new(idx as u64);
        let original_id = data.entity_ids[idx].to_string();
        synset_to_concept.insert(original_id.clone(), concept_id);
        let def = data.entity_defs[idx];
        concepts.push(Concept {
            id: concept_id,
            original_id,
            pos: pos_from_str(data.entity_pos[idx]),
            lemmas: Vec::new(),
            definitions: if def.is_empty() {
                vec![]
            } else {
                vec![def.into()]
            },
            examples: vec![],
        });
    }

    // Phase 2: Word index + fill concept lemmas
    let mut word_index: HashMap<String, Vec<ConceptId>> = HashMap::new();
    for &(word, ids) in data.word_index {
        let cids: Vec<ConceptId> = ids.iter().map(|&i| ConceptId::new(u64::from(i))).collect();
        for &i in ids {
            if let Some(c) = concepts.get_mut(i as usize) {
                c.lemmas.push(word.to_string());
            }
        }
        word_index.insert(word.to_string(), cids);
    }

    // Phase 3: Taxonomy adjacency
    let mut taxonomy_parents: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
    let mut taxonomy_children: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
    for &(child, parent) in data.taxonomy {
        let c = ConceptId::new(u64::from(child));
        let p = ConceptId::new(u64::from(parent));
        taxonomy_parents.entry(c).or_default().push(p);
        taxonomy_children.entry(p).or_default().push(c);
    }

    // Phase 4: Mereology
    let mut mereology_parts: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
    for &(whole, part) in data.mereology {
        let w = ConceptId::new(u64::from(whole));
        let p = ConceptId::new(u64::from(part));
        mereology_parts.entry(w).or_default().push(p);
    }

    // Language-specific data (function words, writing system, morphology)
    let function_words = build_english_function_words();
    let function_word_list: Vec<String> = function_words.keys().cloned().collect();
    let writing = super::orthography::english_writing_system();
    let morphology = super::morphology::english_rules();

    super::english::English::new(
        concepts,
        word_index,
        taxonomy_children,
        taxonomy_parents,
        HashMap::<SenseId, Vec<SenseId>>::new(), // opposition (sense-level needs full LMF)
        mereology_parts,
        synset_to_concept,
        HashMap::new(), // sense_to_id
        function_words,
        function_word_list,
        HashMap::new(), // verb_transitivity (chart parser resolves in context)
        writing,
        morphology,
    )
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
    <LexicalEntry id="e-runs-v">
      <Lemma writtenForm="runs" partOfSpeech="v"/>
      <Sense id="runs-v-01" subcat="via vtai" synset="s-run"/>
    </LexicalEntry>
    <LexicalEntry id="e-sees-v">
      <Lemma writtenForm="sees" partOfSpeech="v"/>
      <Sense id="sees-v-01" subcat="vtai vtaa" synset="s-see"/>
    </LexicalEntry>
    <Synset id="s-dog" ili="i1" partOfSpeech="n" members="e-dog-n">
      <Definition>a domesticated carnivore</Definition>
    </Synset>
    <Synset id="s-run" ili="i2" partOfSpeech="v" members="e-run-v e-runs-v">
      <Definition>move fast</Definition>
    </Synset>
    <Synset id="s-see" ili="i3" partOfSpeech="v" members="e-sees-v">
      <Definition>perceive with eyes</Definition>
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

    // =========================================================================
    // Pregroup pipeline tests — end-to-end through Language trait
    // =========================================================================

    use crate::science::linguistics::lambek::pregroup;

    #[test]
    fn pregroup_the_dog_runs() {
        let wn = sample_wn();
        let lang = EnglishLanguage::from_wordnet(&wn);

        let words = ["the", "dog", "runs"];
        let types: Vec<pregroup::PregroupType> = words
            .iter()
            .map(|w| {
                let pts = lang.pregroup_types(w);
                assert!(!pts.is_empty(), "'{}' should have pregroup types", w);
                pts.into_iter().next().unwrap()
            })
            .collect();

        assert!(
            pregroup::parse(&types),
            "the dog runs should parse: {}",
            types
                .iter()
                .map(|t| t.notation())
                .collect::<Vec<_>>()
                .join(" | ")
        );
    }

    #[test]
    fn pregroup_she_sees_the_dog() {
        let wn = sample_wn();
        let lang = EnglishLanguage::from_wordnet(&wn);

        let words = ["she", "sees", "the", "dog"];
        let types: Vec<pregroup::PregroupType> = words
            .iter()
            .map(|w| {
                let pts = lang.pregroup_types(w);
                assert!(!pts.is_empty(), "'{}' should have pregroup types", w);
                // For verbs with multiple types, prefer transitive (3 elements)
                pts.iter()
                    .find(|t| t.elements.len() == 3)
                    .cloned()
                    .unwrap_or_else(|| pts.into_iter().next().unwrap())
            })
            .collect();

        assert!(
            pregroup::parse(&types),
            "she sees the dog should parse: {}",
            types
                .iter()
                .map(|t| t.notation())
                .collect::<Vec<_>>()
                .join(" | ")
        );
    }

    #[test]
    fn every_function_word_has_pregroup_type() {
        let wn = sample_wn();
        let lang = EnglishLanguage::from_wordnet(&wn);
        for word in ["the", "a", "is", "she", "it", "what", "and", "in", "not"] {
            let pts = lang.pregroup_types(word);
            assert!(
                !pts.is_empty(),
                "function word '{}' should have pregroup types",
                word
            );
        }
    }
}
