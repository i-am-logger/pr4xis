use std::collections::HashMap;

use crate::science::information::ontology::Reference;
use crate::science::linguistics::lambek::pregroup::{self, PregroupType};
use crate::science::linguistics::lexicon::pos::*;
use crate::science::linguistics::morphology::MorphologicalRule;
use crate::science::linguistics::orthography::WritingSystem;
use crate::technology::software::markup::xml::lmf::ontology as lmf;

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
        let function_words = crate::science::linguistics::language::build_english_function_words();
        let function_word_list: Vec<String> = function_words.keys().cloned().collect();
        let verb_transitivity = crate::science::linguistics::language::build_verb_transitivity(wn);
        let writing = crate::science::linguistics::orthography::english_writing_system();
        let morphology = crate::science::linguistics::morphology::english_rules();

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
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
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

impl crate::science::linguistics::language::Language for English {
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
            return crate::science::linguistics::language::lmf_pos_to_lexical_entries(
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
        let mut seen_pos = std::collections::HashSet::new();
        for &cid in self.lookup(word) {
            if let Some(concept) = self.concept(cid)
                && seen_pos.insert(concept.pos)
            {
                let transitivities = self.verb_transitivities(word);
                results.extend(
                    crate::science::linguistics::language::lmf_pos_to_lexical_entries(
                        word,
                        concept.pos,
                        transitivities,
                    ),
                );
            }
        }
        results
    }

    fn pregroup_types(&self, word: &str) -> Vec<PregroupType> {
        self.lexical_lookup_all(word)
            .iter()
            .map(|entry| {
                crate::science::linguistics::language::lexical_entry_to_pregroup(entry)
            })
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
