#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use hashbrown::HashMap;

use crate::cognitive::linguistics::lambek::pregroup::PregroupType;
use crate::cognitive::linguistics::lexicon::pos::*;
use crate::cognitive::linguistics::morphology::MorphologicalRule;
use crate::cognitive::linguistics::orthography::WritingSystem;
use crate::formal::information::ontology::Reference;
use crate::social::software::markup::xml::lmf::ontology as lmf;

// English language ontology — built from Open English WordNet 2025.
//
// English is a natural language (SocialObject in DOLCE).
// This ontology represents what English IS — its concepts (synsets),
// their relationships (taxonomy, mereology, opposition), and
// its vocabulary (words mapped to concepts).
//
// The ontology is loaded through the LMF functor:
// XML → XmlOntology → LmfFunctor → WordNet → EnglishOntology

/// A concept identifier — a Ref32 pointing to a synset.
/// Ontologically: a Reference to a meaning in the English language.
pub type ConceptId = Reference<4>;

/// A sense identifier — a Ref32 pointing to a specific word-meaning pair.
pub type SenseId = Reference<4>;

/// The English language ontology — pre-computed, frozen, fast to query.
///
/// This is the OUTPUT of the loading functor. All adjacency maps are
/// built once during initialization. Queries return references, not
/// freshly allocated collections.
/// The English language — a complete ontology implementing the Language trait.
///
/// Built from WordNet via the `from_wordnet` functor. Contains:
/// - Concepts (synsets) with taxonomy, mereology, opposition
/// - Function words (closed class, from OLiA categories)
/// - Verb transitivity (from WordNet subcategorization frames)
/// - Writing system, morphological rules
/// - Pregroup type assignments
///
/// This is ONE type, not two. The WordNet data and the Language interface
/// are the same ontology — the functor from WordNet produces this.
#[derive(Debug)]
pub struct English {
    // === WordNet concept data ===
    /// All concepts (synsets) indexed by ConceptId.
    pub concepts: Vec<Concept>,
    /// Word text → concept IDs (one word can mean multiple things).
    pub word_index: HashMap<String, Vec<ConceptId>>,
    /// Pre-computed taxonomy: parent → children.
    taxonomy_children: HashMap<ConceptId, Vec<ConceptId>>,
    /// Pre-computed taxonomy: child → parents.
    taxonomy_parents: HashMap<ConceptId, Vec<ConceptId>>,
    /// Pre-computed opposition: sense → opposite senses.
    opposition: HashMap<SenseId, Vec<SenseId>>,
    /// Pre-computed mereology: whole → parts.
    mereology_parts: HashMap<ConceptId, Vec<ConceptId>>,
    /// Synset ID string → ConceptId mapping.
    synset_to_concept: HashMap<String, ConceptId>,
    /// Sense ID string → SenseId mapping.
    pub sense_to_id: HashMap<String, SenseId>,

    // === Language trait data ===
    /// Function words (closed class, OLiA-classified).
    function_words: HashMap<String, Vec<LexicalEntry>>,
    /// All function word texts (for spelling correction).
    function_word_list: Vec<String>,
    /// Verb transitivity from WordNet subcategorization frames.
    verb_transitivity: HashMap<String, Vec<Transitivity>>,
    /// Writing system.
    writing: WritingSystem,
    /// Morphological rules.
    morphology: Vec<MorphologicalRule>,
}

/// A concept — a meaning in the English language.
/// Multiple words can express the same concept (synonyms share a concept).
#[derive(Debug, Clone)]
pub struct Concept {
    pub id: ConceptId,
    pub original_id: String,
    pub pos: lmf::LmfPos,
    pub lemmas: Vec<String>,
    pub definitions: Vec<String>,
    pub examples: Vec<String>,
}

impl English {
    /// Construct an English ontology from pre-computed parts.
    /// Used by the Language module's deployment functors (codegen, mmap, async).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        concepts: Vec<Concept>,
        word_index: HashMap<String, Vec<ConceptId>>,
        taxonomy_children: HashMap<ConceptId, Vec<ConceptId>>,
        taxonomy_parents: HashMap<ConceptId, Vec<ConceptId>>,
        opposition: HashMap<SenseId, Vec<SenseId>>,
        mereology_parts: HashMap<ConceptId, Vec<ConceptId>>,
        synset_to_concept: HashMap<String, ConceptId>,
        sense_to_id: HashMap<String, SenseId>,
        function_words: HashMap<String, Vec<LexicalEntry>>,
        function_word_list: Vec<String>,
        verb_transitivity: HashMap<String, Vec<Transitivity>>,
        writing: WritingSystem,
        morphology: Vec<MorphologicalRule>,
    ) -> Self {
        Self {
            concepts,
            word_index,
            taxonomy_children,
            taxonomy_parents,
            opposition,
            mereology_parts,
            synset_to_concept,
            sense_to_id,
            function_words,
            function_word_list,
            verb_transitivity,
            writing,
            morphology,
        }
    }

    /// Minimal sample English for testing — no full WordNet needed.
    pub fn sample() -> Self {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="dog-n-01" synset="s-dog"/></LexicalEntry>
    <LexicalEntry id="e-cat-n"><Lemma writtenForm="cat" partOfSpeech="n"/><Sense id="cat-n-01" synset="s-cat"/></LexicalEntry>
    <LexicalEntry id="e-mammal-n"><Lemma writtenForm="mammal" partOfSpeech="n"/><Sense id="mammal-n-01" synset="s-mammal"/></LexicalEntry>
    <LexicalEntry id="e-animal-n"><Lemma writtenForm="animal" partOfSpeech="n"/><Sense id="animal-n-01" synset="s-animal"/></LexicalEntry>
    <LexicalEntry id="e-run-v"><Lemma writtenForm="run" partOfSpeech="v"/><Sense id="run-v-01" synset="s-run" subcat="vtai"/></LexicalEntry>
    <LexicalEntry id="e-see-v"><Lemma writtenForm="see" partOfSpeech="v"/><Sense id="see-v-01" synset="s-see" subcat="vtaa"/></LexicalEntry>
    <LexicalEntry id="e-big-a"><Lemma writtenForm="big" partOfSpeech="a"/><Sense id="big-a-01" synset="s-big"/></LexicalEntry>
    <Synset id="s-dog" ili="i1" partOfSpeech="n"><Definition>a domesticated canine</Definition><SynsetRelation relType="hypernym" target="s-mammal"/></Synset>
    <Synset id="s-cat" ili="i2" partOfSpeech="n"><Definition>a small feline</Definition><SynsetRelation relType="hypernym" target="s-mammal"/></Synset>
    <Synset id="s-mammal" ili="i3" partOfSpeech="n"><Definition>warm-blooded vertebrate</Definition><SynsetRelation relType="hypernym" target="s-animal"/></Synset>
    <Synset id="s-animal" ili="i4" partOfSpeech="n"><Definition>a living organism</Definition></Synset>
    <Synset id="s-run" ili="i5" partOfSpeech="v"><Definition>move fast on foot</Definition></Synset>
    <Synset id="s-see" ili="i6" partOfSpeech="v"><Definition>perceive with the eyes</Definition></Synset>
    <Synset id="s-big" ili="i7" partOfSpeech="a"><Definition>of considerable size</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(xml).unwrap();
        Self::from_wordnet(&wn)
    }

    /// Build the English ontology from a WordNet instance.
    /// This is the functor: WordNet → English.
    /// Computes all adjacency maps ONCE (the initialization phase).
    pub fn from_wordnet(wn: &lmf::WordNet) -> Self {
        let mut concepts = Vec::with_capacity(wn.synsets.len());
        let mut word_index: HashMap<String, Vec<ConceptId>> = HashMap::new();
        let mut synset_to_concept: HashMap<String, ConceptId> = HashMap::new();
        let mut sense_to_id: HashMap<String, SenseId> = HashMap::new();

        // Phase 0: Build synset → lemmas reverse index (O(entries), not O(synsets × entries))
        let mut synset_lemmas: HashMap<String, Vec<String>> = HashMap::new();
        for entry in &wn.entries {
            for sense in &entry.senses {
                synset_lemmas
                    .entry(sense.synset.clone())
                    .or_default()
                    .push(entry.lemma.written_form.clone());
            }
        }

        // Phase 1: Create concepts from synsets, assign ConceptIds
        for (idx, synset) in wn.synsets.iter().enumerate() {
            let concept_id = ConceptId::new(idx as u64);
            synset_to_concept.insert(synset.id.clone(), concept_id);

            let lemmas = synset_lemmas.remove(&synset.id).unwrap_or_default();

            for lemma in &lemmas {
                word_index
                    .entry(lemma.clone())
                    .or_default()
                    .push(concept_id);
            }

            concepts.push(Concept {
                id: concept_id,
                original_id: synset.id.clone(),
                pos: synset.pos,
                lemmas,
                definitions: synset.definitions.clone(),
                examples: synset.examples.clone(),
            });
        }

        // Phase 2: Assign SenseIds
        let mut sense_counter = 0u64;
        for entry in &wn.entries {
            for sense in &entry.senses {
                let sense_id = SenseId::new(sense_counter);
                sense_to_id.insert(sense.id.clone(), sense_id);
                sense_counter += 1;
            }
        }

        // Phase 3: Build taxonomy adjacency maps (pre-computed, query many)
        let mut taxonomy_parents: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        let mut taxonomy_children: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();

        for synset in &wn.synsets {
            if let Some(&child_id) = synset_to_concept.get(&synset.id) {
                for rel in &synset.relations {
                    if rel.rel_type.is_taxonomy()
                        && let Some(&parent_id) = synset_to_concept.get(&rel.target)
                    {
                        taxonomy_parents
                            .entry(child_id)
                            .or_default()
                            .push(parent_id);
                        taxonomy_children
                            .entry(parent_id)
                            .or_default()
                            .push(child_id);
                    }
                }
            }
        }

        // Phase 4: Build opposition map
        let mut opposition: HashMap<SenseId, Vec<SenseId>> = HashMap::new();
        for entry in &wn.entries {
            for sense in &entry.senses {
                if let Some(&sense_id) = sense_to_id.get(&sense.id) {
                    for rel in &sense.relations {
                        if rel.rel_type.is_opposition()
                            && let Some(&target_id) = sense_to_id.get(&rel.target)
                        {
                            opposition.entry(sense_id).or_default().push(target_id);
                        }
                    }
                }
            }
        }

        // Phase 5: Build mereology maps
        let mut mereology_parts: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        for synset in &wn.synsets {
            if let Some(&whole_id) = synset_to_concept.get(&synset.id) {
                for rel in &synset.relations {
                    if rel.rel_type.is_mereology()
                        && let Some(&part_id) = synset_to_concept.get(&rel.target)
                    {
                        mereology_parts.entry(whole_id).or_default().push(part_id);
                    }
                }
            }
        }

        // Build Language data: function words, verb transitivity, writing, morphology
        let function_words =
            crate::cognitive::linguistics::language::build_english_function_words();
        let function_word_list: Vec<String> = function_words.keys().cloned().collect();
        let verb_transitivity =
            crate::cognitive::linguistics::language::build_verb_transitivity(wn);
        let writing = crate::cognitive::linguistics::orthography::english_writing_system();
        let morphology = crate::cognitive::linguistics::morphology::english_rules();

        English {
            concepts,
            word_index,
            taxonomy_children,
            taxonomy_parents,
            opposition,
            mereology_parts,
            synset_to_concept,
            sense_to_id,
            function_words,
            function_word_list,
            verb_transitivity,
            writing,
            morphology,
        }
    }

    // ---- Query methods (zero allocation — return references) ----

    /// Look up a word → all concepts (meanings) it can express.
    pub fn lookup(&self, word: &str) -> &[ConceptId] {
        self.word_index
            .get(word)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get a concept by its ConceptId.
    pub fn concept(&self, id: ConceptId) -> Option<&Concept> {
        self.concepts.get(id.value() as usize)
    }

    /// Get a concept by its original WordNet synset ID string.
    pub fn concept_by_synset(&self, synset_id: &str) -> Option<&Concept> {
        self.synset_to_concept
            .get(synset_id)
            .and_then(|id| self.concept(*id))
    }

    /// Direct parents (hypernyms) of a concept — is-a targets.
    pub fn parents(&self, id: ConceptId) -> &[ConceptId] {
        self.taxonomy_parents
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Direct children (hyponyms) of a concept — is-a sources.
    pub fn children(&self, id: ConceptId) -> &[ConceptId] {
        self.taxonomy_children
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Check if child is-a ancestor (transitively).
    pub fn is_a(&self, child: ConceptId, ancestor: ConceptId) -> bool {
        if child == ancestor {
            return true;
        }
        // BFS up the taxonomy
        let mut visited = hashbrown::HashSet::new();
        let mut queue = alloc::collections::VecDeque::new();
        for &parent in self.parents(child) {
            if visited.insert(parent) {
                queue.push_back(parent);
            }
        }
        while let Some(current) = queue.pop_front() {
            if current == ancestor {
                return true;
            }
            for &parent in self.parents(current) {
                if visited.insert(parent) {
                    queue.push_back(parent);
                }
            }
        }
        false
    }

    /// Direct parts (meronyms) of a concept.
    pub fn parts(&self, id: ConceptId) -> &[ConceptId] {
        self.mereology_parts
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Opposites (antonyms) of a sense.
    pub fn opposites(&self, sense_id: SenseId) -> &[SenseId] {
        self.opposition
            .get(&sense_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Total number of concepts.
    pub fn concept_count(&self) -> usize {
        self.concepts.len()
    }

    /// Total number of unique words.
    pub fn word_count(&self) -> usize {
        self.word_index.len()
    }

    /// Total taxonomy relations.
    pub fn taxonomy_count(&self) -> usize {
        self.taxonomy_parents.values().map(|v| v.len()).sum()
    }

    /// Total opposition relations.
    pub fn opposition_count(&self) -> usize {
        self.opposition.values().map(|v| v.len()).sum()
    }

    /// Get verb transitivity options from pre-computed frames.
    fn verb_transitivities(&self, word: &str) -> &[Transitivity] {
        self.verb_transitivity
            .get(word)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

impl crate::cognitive::linguistics::language::Language for English {
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
        if let Some(entries) = self.function_words.get(word) {
            return entries.first().cloned();
        }
        let concept_ids = self.lookup(word);
        if let Some(&cid) = concept_ids.first()
            && let Some(concept) = self.concept(cid)
        {
            let transitivities = self.verb_transitivities(word);
            return crate::cognitive::linguistics::language::lmf_pos_to_lexical_entries(
                word,
                concept.pos,
                transitivities,
            )
            .into_iter()
            .next();
        }
        None
    }

    fn lexical_lookup_all(&self, word: &str) -> Vec<LexicalEntry> {
        let mut results = Vec::new();
        if let Some(entries) = self.function_words.get(word) {
            results.extend(entries.iter().cloned());
        }
        let mut seen_pos = hashbrown::HashSet::new();
        for &cid in self.lookup(word) {
            if let Some(concept) = self.concept(cid)
                && seen_pos.insert(concept.pos)
            {
                let transitivities = self.verb_transitivities(word);
                results.extend(
                    crate::cognitive::linguistics::language::lmf_pos_to_lexical_entries(
                        word,
                        concept.pos,
                        transitivities,
                    ),
                );
            }
        }

        // Morphological stemming: if the word has a known suffix, try the stem.
        // "runs" → strip "s" → "run" → lookup "run" → get verb entries.
        // This IS the morphology functor: InflectedForm → Stem → LexicalEntry.
        for rule in &self.morphology {
            if let crate::cognitive::linguistics::morphology::Affix::Suffix(suffix) = &rule.affix
                && let Some(stem) = word.strip_suffix(suffix.text.as_str())
                && !stem.is_empty()
            {
                for &cid in self.lookup(stem) {
                    if let Some(concept) = self.concept(cid)
                        && seen_pos.insert(concept.pos)
                    {
                        let transitivities = self.verb_transitivities(stem);
                        results.extend(
                            crate::cognitive::linguistics::language::lmf_pos_to_lexical_entries(
                                stem,
                                concept.pos,
                                transitivities,
                            ),
                        );
                    }
                }
            }
        }

        results
    }

    fn pregroup_types(&self, word: &str) -> Vec<PregroupType> {
        self.lexical_lookup_all(word)
            .iter()
            .map(crate::cognitive::linguistics::language::lexical_entry_to_pregroup)
            .collect()
    }

    fn known_words(&self) -> Vec<&str> {
        let mut words: Vec<&str> = self.function_word_list.iter().map(|s| s.as_str()).collect();
        words.extend(self.word_index.keys().map(|s| s.as_str()));
        words
    }

    fn concept_count(&self) -> usize {
        self.concepts.len()
    }

    fn word_count(&self) -> usize {
        self.word_index.len() + self.function_word_list.len()
    }
}
