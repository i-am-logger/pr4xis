#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use hashbrown::HashMap;

use super::lexicon::pos::*;
use super::morphology::MorphologicalRule;
use super::orthography::WritingSystem;
use crate::cognitive::linguistics::lambek::pregroup::{self, PregroupType};
use crate::social::software::markup::xml::lmf::ontology as lmf;

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
    pub fn from_wordnet(wn: &crate::social::software::markup::xml::lmf::ontology::WordNet) -> Self {
        let function_words = build_english_function_words();
        let function_word_list: Vec<String> = function_words.keys().cloned().collect();
        let verb_transitivity = build_verb_transitivity(wn);
        Self {
            ontology: super::english::English::from_wordnet(wn),
            writing: super::orthography::english_writing_system(),
            morphology: super::morphology::english::english_rules(),
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
        let mut seen_pos = hashbrown::HashSet::new();
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
        // A satellite adjective (WN-LMF `s`) is, grammatically, an
        // adjective — it differs from a head adjective only in its
        // cluster role (Fellbaum 1998 §1.5), which the pregroup grammar
        // does not distinguish. So it maps to the same lexical entry.
        lmf::LmfPos::Adjective | lmf::LmfPos::SatelliteAdjective => {
            vec![LexicalEntry::Adjective(Adjective { text: word.into() })]
        }
        lmf::LmfPos::Adverb => vec![LexicalEntry::Adverb(Adverb {
            text: word.into(),
            olia_class: None,
        })],
        lmf::LmfPos::Determiner | lmf::LmfPos::Numeral => {
            vec![LexicalEntry::Determiner(Determiner {
                text: word.into(),
                kind: DeterminerKind::Indefinite,
                number: None,
                olia_class: None,
            })]
        }
        lmf::LmfPos::Pronoun => vec![LexicalEntry::Pronoun(Pronoun {
            text: word.into(),
            kind: PronounKind::Personal,
            number: Number::Singular,
            person: Person::Third,
            olia_class: None,
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
///
/// TRACKED-LEGACY (Batch K): pregroup is a SECOND grammar formalism (Lambek
/// adjoints, not CCG slashes); the live parse path is `chart_reduce` over the
/// loaded-functor Lambek categories (`pregroup_types` has no non-test callers).
/// This match is NOT migrated into the OLiA→CCG functor because that functor's
/// values are CCG notation, not pregroup types — feeding pregroup from it would
/// need a separate `notation → PregroupType` interpreter, not built for dead
/// code. If pregroup parity is ever wanted, add a `pregroup-notation` column +
/// that interpreter; until then this stays as the formalism's own map.
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
    wn: &crate::social::software::markup::xml::lmf::ontology::WordNet,
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

/// Build the English function-word lexicon — the closed-class `ClosedClassLexicon`
/// stratum, the disjoint complement of the open-class english_wordnet
/// (Quirk et al. 1985 §2.34).
///
/// The OLiA-`reference_model()` twin: under `feature = "prx"` it fast-loads the
/// committed `english-function-words-2026.prx.gz` (a graph-faithful rkyv
/// `WordNetPrxEnvelope`) via [`function_words_wordnet_from_prx`], reconstructing
/// the parsed `WordNet` with its human-meaningful `fw-*` synset ids intact, then
/// projects it through the unchanged [`function_words_from_lmf`]. The committed
/// `.prx.gz` is content-addressed and (with the registered `WordNetLmfLens`)
/// pinned in `praxis.lock` — the loaded-not-`include_str!` path that closes the
/// asymmetry with OLiA.
///
/// A corrupt/failed artifact falls through to the authoritative
/// `include_str!(english.xml)` parse — exactly OLiA's fail-soft fallback. Without
/// `prx`, the XML path is the only path. The bundle ships with praxis, so a
/// parse failure is a build-time invariant. Function words are DATA; the
/// interrogative class rides each `Sense.subcat`, the determiner/interjection
/// features ride each `Sense.synset`.
///
/// [`function_words_wordnet_from_prx`]: crate::social::software::markup::xml::lmf::prx::function_words_wordnet_from_prx
pub fn build_english_function_words() -> HashMap<String, Vec<LexicalEntry>> {
    #[cfg(feature = "prx")]
    {
        const PRX_GZ: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/function-words/english-function-words-2026.prx.gz"
        ));
        if let Ok(wn) =
            crate::social::software::markup::xml::lmf::prx::function_words_wordnet_from_prx(PRX_GZ)
        {
            return function_words_from_lmf(&wn);
        }
        // Corrupt or failed-gate artifact → fall through to the always-correct
        // XML parse, exactly as `olia::reference_model()` falls back to the OWL.
    }
    const XML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/function-words/english.xml"
    ));
    let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(XML).expect(
        "bundled crates/domains/data/function-words/english.xml failed to parse — \
         build-time invariant violated",
    );
    function_words_from_lmf(&wn)
}

/// Parse function words from an LMF WordNet instance.
/// Maps synset categories (from OLiA) to rich LexicalEntry types.
#[cfg_attr(not(feature = "std"), allow(dead_code))]
fn function_words_from_lmf(
    wn: &crate::social::software::markup::xml::lmf::ontology::WordNet,
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

        // The loaded OLiA class fragment, decoded ONCE from the Sense `subcat`
        // (the universal grammatical-class identity); the OLiA→CCG functor
        // projects it to a category. The interrogative dispatch is this typed
        // value, not a `synset_id.contains("interrogative")` substring test.
        let olia_class = entry.senses.first().and_then(|s| s.subcat.first()).cloned();

        let lexical_entry = match entry.lemma.pos {
            lmf::LmfPos::Determiner => {
                // DeterminerKind decoded ONCE from the loaded synset (the wire form
                // of the feature), not a scattered `synset_id.contains(...)`
                // dispatch — the codec lowering, like the OLiA fragment resolver.
                let kind = determiner_kind_from_synset(synset_id);
                LexicalEntry::Determiner(Determiner {
                    text: word.clone(),
                    kind,
                    number: None,
                    olia_class: olia_class.clone(),
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
                // Interrogative-ness is the loaded OLiA class, not a synset
                // substring test.
                let kind = if olia_class.as_deref() == Some("InterrogativePronoun") {
                    PronounKind::Interrogative
                } else {
                    PronounKind::Personal
                };
                LexicalEntry::Pronoun(Pronoun {
                    text: word.clone(),
                    number: Number::Singular,
                    person: Person::Third,
                    kind,
                    olia_class: olia_class.clone(),
                })
            }
            // Interrogative adverbs (where/when/why/how) are closed-class
            // function words carrying an OLiA class — retained, not dropped by
            // the `_ => continue` below (the silent-drop the redesign removes).
            lmf::LmfPos::Adverb => LexicalEntry::Adverb(Adverb {
                text: word.clone(),
                olia_class: olia_class.clone(),
            }),
            lmf::LmfPos::Preposition => {
                LexicalEntry::Preposition(Preposition { text: word.clone() })
            }
            lmf::LmfPos::Conjunction => {
                LexicalEntry::Conjunction(Conjunction { text: word.clone() })
            }
            lmf::LmfPos::Particle => LexicalEntry::Particle(Particle { text: word.clone() }),
            lmf::LmfPos::Interjection => {
                // Interjection kind decoded ONCE from the loaded synset (the
                // codec lowering), not scattered `synset_id.contains(...)`.
                let kind = interjection_kind_from_synset(synset_id);
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

/// Decode a determiner's [`DeterminerKind`] from its loaded synset id — the ONE
/// codec lowering (wire → typed feature) on the determiner-feature axis, the
/// sibling of [`interjection_kind_from_synset`]. Replaces the scattered
/// `synset_id.contains(...)` dispatch. The synset is the loaded feature encoding
/// (from `function-words/english.xml`).
///
/// Definite/Indefinite is the core definiteness contrast (Lyons 1999,
/// DOI 10.1017/CBO9780511605789); Demonstrative/Quantifier are determiner
/// subclasses on a different axis (Abbott 2010) — see the PRAXIS-HONESTY FLAG on
/// [`DeterminerKind`]. An unknown synset fails to the unmarked default
/// (Indefinite), per Huddleston & Pullum 2002 Ch.5.
fn determiner_kind_from_synset(synset_id: &str) -> DeterminerKind {
    match synset_id {
        "fw-definite-det" => DeterminerKind::Definite,
        "fw-demonstrative-det" => DeterminerKind::Demonstrative,
        "fw-universal-det" | "fw-negative-det" => DeterminerKind::Quantifier,
        // fw-indefinite-det, fw-interrogative-det, and any unknown → Indefinite.
        _ => DeterminerKind::Indefinite,
    }
}

/// Decode an interjection's [`InterjectionKind`] from its loaded synset id — the
/// ONE codec lowering, replacing the scattered `synset_id.contains(...)`
/// dispatch. OLiA has only a single `Interjection` class (no functional
/// subclasses), so the communicative-function kind is a praxis feature decoded
/// from the loaded synset, grounded in Ameka 1992's expressive/conative/phatic
/// typology (DOI 10.1016/0378-2166(92)90048-G), not an OLiA fragment. An unknown
/// synset fails to the prototypical default (Expressive).
fn interjection_kind_from_synset(synset_id: &str) -> InterjectionKind {
    match synset_id {
        "fw-greeting" => InterjectionKind::Greeting,
        "fw-farewell" => InterjectionKind::Farewell,
        "fw-politeness" => InterjectionKind::Politeness,
        "fw-response" => InterjectionKind::Response,
        "fw-conative" => InterjectionKind::Conative,
        // fw-expressive and any unknown → Expressive.
        _ => InterjectionKind::Expressive,
    }
}

#[cfg(test)]
mod feature_decoders {
    use super::*;

    #[test]
    fn definiteness_decodes_from_the_loaded_synset() {
        assert_eq!(
            determiner_kind_from_synset("fw-definite-det"),
            DeterminerKind::Definite
        );
        assert_eq!(
            determiner_kind_from_synset("fw-demonstrative-det"),
            DeterminerKind::Demonstrative
        );
        assert_eq!(
            determiner_kind_from_synset("fw-universal-det"),
            DeterminerKind::Quantifier
        );
        assert_eq!(
            determiner_kind_from_synset("fw-negative-det"),
            DeterminerKind::Quantifier
        );
        assert_eq!(
            determiner_kind_from_synset("fw-indefinite-det"),
            DeterminerKind::Indefinite
        );
        // Unknown / interrogative-det → the open-class default.
        assert_eq!(
            determiner_kind_from_synset("fw-interrogative-det"),
            DeterminerKind::Indefinite
        );
    }

    #[test]
    fn interjection_kind_decodes_from_the_loaded_synset() {
        assert_eq!(
            interjection_kind_from_synset("fw-greeting"),
            InterjectionKind::Greeting
        );
        assert_eq!(
            interjection_kind_from_synset("fw-farewell"),
            InterjectionKind::Farewell
        );
        assert_eq!(
            interjection_kind_from_synset("fw-politeness"),
            InterjectionKind::Politeness
        );
        assert_eq!(
            interjection_kind_from_synset("fw-response"),
            InterjectionKind::Response
        );
        assert_eq!(
            interjection_kind_from_synset("fw-conative"),
            InterjectionKind::Conative
        );
        assert_eq!(
            interjection_kind_from_synset("fw-expressive"),
            InterjectionKind::Expressive
        );
    }
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
pub fn from_codegen(
    data: &pr4xis::codegen_data::CodegenData<super::english::English>,
) -> super::english::English {
    use super::english::{Concept, ConceptId, SenseId};

    // Delegate to the canonical WN-LMF tag parser so the codegen→Language
    // path agrees with the from_wordnet path on every tag — including the
    // satellite adjective `s`, which now round-trips as its own
    // [`LmfPos::SatelliteAdjective`] variant rather than collapsing to
    // `Adjective` (the byte-loss this slice closes).
    let pos_from_str = lmf::LmfPos::parse;

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
            pos: pos_from_str(data.entity_kind[idx]),
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
        let cids: Vec<ConceptId> = ids.iter().map(|h| ConceptId::new(h.value())).collect();
        for h in ids {
            if let Some(c) = concepts.get_mut(h.value() as usize) {
                c.lemmas.push(word.to_string());
            }
        }
        word_index.insert(word.to_string(), cids);
    }

    // Phase 3: Taxonomy adjacency
    let mut taxonomy_parents: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
    let mut taxonomy_children: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
    for &(child, parent) in data.taxonomy {
        let c = ConceptId::new(child.value());
        let p = ConceptId::new(parent.value());
        taxonomy_parents.entry(c).or_default().push(p);
        taxonomy_children.entry(p).or_default().push(c);
    }

    // Phase 4: Mereology
    let mut mereology_parts: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
    for &(whole, part) in data.mereology {
        let w = ConceptId::new(whole.value());
        let p = ConceptId::new(part.value());
        mereology_parts.entry(w).or_default().push(p);
    }

    // Language-specific data (function words, writing system, morphology)
    let function_words = build_english_function_words();
    let function_word_list: Vec<String> = function_words.keys().cloned().collect();
    let writing = super::orthography::english_writing_system();
    let morphology = super::morphology::english::english_rules();

    let mut english = super::english::English::new(
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
    );

    // SKOS seeAlso (WordNet `also`) wired into the existing
    // `also_synset` slot in WordnetRelations. Miles & Bechhofer (2009)
    // W3C SKOS §8.
    english.set_also_synset_references(
        data.references
            .iter()
            .map(|&(from, to)| (ConceptId::new(from.value()), ConceptId::new(to.value()))),
    );

    english
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::symbols::character::Direction;

    fn sample_wn() -> crate::social::software::markup::xml::lmf::ontology::WordNet {
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
        crate::social::software::markup::xml::lmf::reader::read_wordnet(sample).unwrap()
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

    use crate::cognitive::linguistics::lambek::pregroup;

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

/// The function-words `.prx` fast-load (audit 2026-06-12 FW-A) — the
/// `ClosedClassLexicon` analogue of OLiA's `reference_model()` artifact tests.
#[cfg(all(test, feature = "prx"))]
mod function_words_prx {
    use super::*;
    use crate::social::software::markup::xml::lmf::prx::{
        emit_wordnet_prx_gz, function_words_wordnet_from_prx,
    };

    const SOURCE_XML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/function-words/english.xml"
    ));
    const COMMITTED_PRX_GZ: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/function-words/english-function-words-2026.prx.gz"
    ));

    fn emit_from_source() -> Vec<u8> {
        emit_wordnet_prx_gz(
            SOURCE_XML.as_bytes(),
            "english_function_words",
            "2026",
            "https://aclanthology.org/J93-2004/",
        )
        .expect("emit english_function_words .prx.gz")
    }

    /// Regenerate the committed `english-function-words-2026.prx.gz`. `#[ignore]`d
    /// (it WRITES, asserting nothing) — the function-words analogue of
    /// `regenerate_olia_compact_prx`. Run by hand when `english.xml` changes:
    /// `cargo test -p pr4xis-domains --features prx -- --ignored regenerate_english_function_words_prx`.
    #[test]
    #[ignore]
    fn regenerate_english_function_words_prx() {
        let prx_gz = emit_from_source();
        let out = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/function-words/english-function-words-2026.prx.gz"
        );
        std::fs::write(out, &prx_gz).expect("write bundled english_function_words .prx.gz");
        // The source content address — the value to pin in praxis.lock
        // `[hashes]` and `[byte_exact_signatures]` as
        // `english_function_words@2026` (the WordNetLmfLens round-trips
        // byte-exact, so the byte-exact signature equals the source hash).
        let src_addr = pr4xis_runtime::address::ContentAddress::of(SOURCE_XML.as_bytes()).to_hex();
        eprintln!("english_function_words@2026 source blake3 = {src_addr}");
    }

    /// STRUCTURAL staleness guard (normal suite) — strictly stronger than OLiA's
    /// entity-count-only guard: the FULL function-word map loaded from the
    /// committed `.prx.gz` must equal the map parsed straight from `english.xml`.
    /// Any drift (a changed lemma / POS / subcat / definiteness / interjection
    /// kind) fails, not just a changed count. If `english.xml` was edited without
    /// regenerating, this is the test that catches it.
    #[test]
    fn bundled_function_words_prx_matches_the_xml() {
        let from_prx = function_words_wordnet_from_prx(COMMITTED_PRX_GZ)
            .map(|wn| function_words_from_lmf(&wn))
            .expect("load committed function-words .prx.gz");
        let from_xml = function_words_from_lmf(
            &crate::social::software::markup::xml::lmf::reader::read_wordnet(SOURCE_XML)
                .expect("parse english.xml"),
        );
        assert_eq!(
            from_prx, from_xml,
            "committed english-function-words-2026.prx.gz is STALE — regenerate with \
             `--ignored regenerate_english_function_words_prx`"
        );
    }

    /// The guardrail documenting WHY the rkyv envelope (not the lossy compact
    /// codec) was chosen: these are the exact decoder outputs that COLLAPSE if a
    /// future change swaps in the compact codec (synthetic `s{i}` ids → every
    /// determiner `Indefinite`, every interjection `Expressive`). Loaded through
    /// `build_english_function_words`, which engages the `.prx` fast path.
    #[test]
    fn decoders_survive_the_prx_roundtrip() {
        let map = build_english_function_words();
        let det_kind = |w: &str| match map.get(w).and_then(|v| v.first()) {
            Some(LexicalEntry::Determiner(d)) => Some(d.kind),
            _ => None,
        };
        assert_eq!(det_kind("the"), Some(DeterminerKind::Definite));
        assert_eq!(det_kind("this"), Some(DeterminerKind::Demonstrative));
        assert_eq!(det_kind("every"), Some(DeterminerKind::Quantifier));
        assert_eq!(det_kind("no"), Some(DeterminerKind::Quantifier));
        assert_eq!(det_kind("a"), Some(DeterminerKind::Indefinite));

        let kind = |w: &str| match map.get(w).and_then(|v| v.first()) {
            Some(LexicalEntry::Interjection(i)) => Some(i.kind),
            _ => None,
        };
        assert_eq!(kind("hello"), Some(InterjectionKind::Greeting));
        assert_eq!(kind("goodbye"), Some(InterjectionKind::Farewell));
    }

    /// The lossless-source-recovery proof for the chosen tier: emitting from
    /// `english.xml` then reloading yields a `WordNet` whose
    /// `(lemma, pos, synset_id, subcat)` tuples equal the directly-parsed
    /// source's — proving the `fw-*` synset ids survive the round-trip and the
    /// new loader composes with the emitter (the function-words analogue of
    /// `wordnet_graph_faithful_reconstructs_source_byte_exact`).
    #[test]
    fn round_trip_recovers_source_features() {
        let reparsed = function_words_wordnet_from_prx(&emit_from_source())
            .expect("round-trip emit→load english_function_words");
        let direct = crate::social::software::markup::xml::lmf::reader::read_wordnet(SOURCE_XML)
            .expect("parse english.xml");
        let project = |wn: &crate::social::software::markup::xml::lmf::ontology::WordNet| {
            wn.entries
                .iter()
                .map(|e| {
                    let s = e.senses.first();
                    (
                        e.lemma.written_form.clone(),
                        e.lemma.pos,
                        s.map(|s| s.synset.clone()).unwrap_or_default(),
                        s.map(|s| s.subcat.clone()).unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(project(&reparsed), project(&direct));
    }
}
