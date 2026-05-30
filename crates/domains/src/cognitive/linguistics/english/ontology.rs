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
    /// All other WordNet relations (derivation, pertainym,
    /// domain_topic, attribute, causes, entails, …). Bundled
    /// to keep the [`English::new`] constructor manageable.
    relations: WordnetRelations,
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

/// All non-taxonomy / non-opposition / non-mereology WordNet
/// relations, loaded into typed maps. Per the LMF schema each
/// relation kind has its own field so callers can query by name
/// without conditional `match` on a generic relType.
///
/// Literature:
/// - **Fellbaum (1998)** *WordNet: An Electronic Lexical Database*
///   — Ch. 3 (verb relations: causes, entails), Ch. 5 (adjective
///   relations: similar, attribute, pertainym), Ch. 1 (antonym).
/// - **Fellbaum, Osherson & Clark (2009)** "Putting Semantics into
///   WordNet's Morphosemantic Links" *LNCS* 5603 — `derivation`.
/// - **Bentivogli & Pianta (2004)** "Extending WordNet with
///   Syntagmatic Information" *Proc. GWC 2004* — domain_topic /
///   domain_region pointers.
/// - **Global WordNet Association schema** —
///   <https://globalwordnet.github.io/schemas/>.
#[derive(Debug, Default, Clone)]
pub struct WordnetRelations {
    // ── Sense-level (SenseRelation in LMF) ──────────────────────
    /// Derivation: sense ↔ morphologically-related sense
    /// (compensate ↔ compensation). Fellbaum-Osherson-Clark (2009).
    pub derivation: HashMap<SenseId, Vec<SenseId>>,
    /// Pertainym: relational adjective → noun base
    /// (e.g. "legal" pertains-to "law"). Fellbaum (1998) §5.2.
    pub pertainym: HashMap<SenseId, Vec<SenseId>>,
    /// Sense-level similar.
    pub similar_sense: HashMap<SenseId, Vec<SenseId>>,
    /// Sense-level see-also.
    pub also_sense: HashMap<SenseId, Vec<SenseId>>,
    /// Sense-level exemplifies (instance-of).
    pub exemplifies_sense: HashMap<SenseId, Vec<SenseId>>,
    /// Sense-level is_exemplified_by (inverse of exemplifies).
    pub is_exemplified_by_sense: HashMap<SenseId, Vec<SenseId>>,
    /// Sense-level participle of.
    pub participle_sense: HashMap<SenseId, Vec<SenseId>>,

    // ── Synset-level (SynsetRelation in LMF) ────────────────────
    /// Synset-level similar (adjective satellites).
    pub similar_synset: HashMap<ConceptId, Vec<ConceptId>>,
    /// Synset-level see-also.
    pub also_synset: HashMap<ConceptId, Vec<ConceptId>>,
    /// Verb causation: kill → die.
    pub causes: HashMap<ConceptId, Vec<ConceptId>>,
    /// Inverse: die ← kill.
    pub is_caused_by: HashMap<ConceptId, Vec<ConceptId>>,
    /// Verb entailment: walk → move.
    pub entails: HashMap<ConceptId, Vec<ConceptId>>,
    /// Inverse: move ← walk.
    pub is_entailed_by: HashMap<ConceptId, Vec<ConceptId>>,
    /// Attribute relation: adjective ↔ noun-attribute (hot ↔ heat).
    pub attribute: HashMap<ConceptId, Vec<ConceptId>>,
    /// Synset-level exemplifies / is_exemplified_by (instance of).
    pub exemplifies: HashMap<ConceptId, Vec<ConceptId>>,
    pub is_exemplified_by: HashMap<ConceptId, Vec<ConceptId>>,
    /// Topic-domain labels (term → domain, e.g. "patent" → "law").
    pub has_domain_topic: HashMap<ConceptId, Vec<ConceptId>>,
    /// Inverse: domain → terms in that domain.
    pub domain_topic: HashMap<ConceptId, Vec<ConceptId>>,
    /// Region-domain labels (term → region, e.g. "kangaroo" → "Australia").
    pub has_domain_region: HashMap<ConceptId, Vec<ConceptId>>,
    /// Inverse: region → terms.
    pub domain_region: HashMap<ConceptId, Vec<ConceptId>>,
    /// Synset-level participle.
    pub participle_synset: HashMap<ConceptId, Vec<ConceptId>>,

    // ── Meronym sub-types (refines mereology_parts) ─────────────
    /// HoloMember: collective → member (forest ← tree).
    pub holo_member: HashMap<ConceptId, Vec<ConceptId>>,
    /// HoloSubstance: substance-whole → constituent (cake ← flour).
    pub holo_substance: HashMap<ConceptId, Vec<ConceptId>>,
    /// MeroMember: member → collective (tree → forest).
    pub mero_member: HashMap<ConceptId, Vec<ConceptId>>,
    /// MeroSubstance: constituent → substance-whole.
    pub mero_substance: HashMap<ConceptId, Vec<ConceptId>>,
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
            relations: WordnetRelations::default(),
            synset_to_concept,
            sense_to_id,
            function_words,
            function_word_list,
            verb_transitivity,
            writing,
            morphology,
        }
    }

    /// Replace the SKOS-style cross-reference map (synset-level
    /// `also_synset`) with the supplied edges. Used by `from_codegen`
    /// to wire the static `RAW_REFERENCES` array into the runtime
    /// `WordnetRelations::also_synset` slot.
    pub fn set_also_synset_references(
        &mut self,
        edges: impl IntoIterator<Item = (ConceptId, ConceptId)>,
    ) {
        let mut map: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        for (from, to) in edges {
            map.entry(from).or_default().push(to);
        }
        self.relations.also_synset = map;
    }

    /// Access to the full bundle of non-taxonomy / non-opposition /
    /// non-mereology relations loaded from WordNet.
    pub fn relations(&self) -> &WordnetRelations {
        &self.relations
    }

    /// All derivation links for a sense (sense ↔ morphologically-
    /// related sense per Fellbaum-Osherson-Clark 2009).
    pub fn derivations(&self, sense: SenseId) -> &[SenseId] {
        self.relations
            .derivation
            .get(&sense)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Pertainym targets for a sense (relational-adjective → noun
    /// base per Fellbaum 1998 §5.2).
    pub fn pertainyms(&self, sense: SenseId) -> &[SenseId] {
        self.relations
            .pertainym
            .get(&sense)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Domain-topic labels assigned to a concept (term → domain,
    /// per Bentivogli & Pianta 2004).
    pub fn has_domain_topic(&self, concept: ConceptId) -> &[ConceptId] {
        self.relations
            .has_domain_topic
            .get(&concept)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
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
        // The sample XML is a compile-time const; parsing it is a
        // structural invariant — if it fails, the test fixture is
        // broken, not user input. expect() with a diagnostic message
        // is appropriate here per the compliance "no silent failures"
        // rule: this is a hard fail, not a swallowed error.
        let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(xml)
            .expect("English::sample() inline LMF fixture must parse");
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

            // A synset with no LexicalEntries pointing to it (no
            // word expresses this concept) is semantically valid in
            // WordNet — e.g. abstract intermediate synsets in the
            // taxonomy. unwrap_or_default() correctly models "no
            // lemmas" as an empty vec, not a silent failure. This is
            // NOT swallowing an error: synset_lemmas is exhaustively
            // populated above from wn.entries; absence here is real
            // information.
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

        // Phase 1.5: Index inflected forms (Form elements in LMF) into
        // the same word_index slots as the entry's lemma. This makes
        // surface inflections — `ran` for `run`, `was` for `be`,
        // `children` for `child` — directly resolvable by the
        // statute-to-WordNet adjunction without invoking the
        // morphology lemmatizer. The data lives in the loaded WordNet
        // XML's `<Form writtenForm="...">` children (~4,400 forms in
        // English WordNet 2025); see
        // crate::social::software::markup::xml::lmf::reader.
        for entry in &wn.entries {
            if entry.forms.is_empty() {
                continue;
            }
            let lemma_concepts: Vec<ConceptId> = entry
                .senses
                .iter()
                .filter_map(|s| synset_to_concept.get(&s.synset).copied())
                .collect();
            if lemma_concepts.is_empty() {
                continue;
            }
            for form in &entry.forms {
                let key = form.written_form.clone();
                let existing = word_index.entry(key).or_default();
                for cid in &lemma_concepts {
                    if !existing.contains(cid) {
                        existing.push(*cid);
                    }
                }
            }
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

        // Phase 5b: Build the rest of the WordNet relation web —
        // derivation, pertainym, domain-topic, similar, causes,
        // entails, attribute, exemplifies, participle, meronym sub-
        // types. One pass over synset_relations + one pass over
        // sense_relations.
        let mut relations = WordnetRelations::default();
        for synset in &wn.synsets {
            let Some(&src_id) = synset_to_concept.get(&synset.id) else {
                continue;
            };
            for rel in &synset.relations {
                let Some(&tgt_id) = synset_to_concept.get(&rel.target) else {
                    continue;
                };
                use lmf::SynsetRelationType as SR;
                let bucket = match rel.rel_type {
                    SR::Similar => &mut relations.similar_synset,
                    SR::Also => &mut relations.also_synset,
                    SR::Causes => &mut relations.causes,
                    SR::IsCausedBy => &mut relations.is_caused_by,
                    SR::Entails => &mut relations.entails,
                    SR::IsEntailedBy => &mut relations.is_entailed_by,
                    SR::Attribute => &mut relations.attribute,
                    SR::Exemplifies => &mut relations.exemplifies,
                    SR::IsExemplifiedBy => &mut relations.is_exemplified_by,
                    SR::HasDomainTopic => &mut relations.has_domain_topic,
                    SR::DomainTopic => &mut relations.domain_topic,
                    SR::HasDomainRegion => &mut relations.has_domain_region,
                    SR::DomainRegion => &mut relations.domain_region,
                    SR::Participle => &mut relations.participle_synset,
                    SR::HoloMember => &mut relations.holo_member,
                    SR::HoloSubstance => &mut relations.holo_substance,
                    SR::MeroMember => &mut relations.mero_member,
                    SR::MeroSubstance => &mut relations.mero_substance,
                    // Already handled above:
                    SR::Hypernym
                    | SR::InstanceHypernym
                    | SR::Hyponym
                    | SR::InstanceHyponym
                    | SR::HoloPart
                    | SR::MeroPart
                    | SR::Other(_) => continue,
                };
                bucket.entry(src_id).or_default().push(tgt_id);
            }
        }
        for entry in &wn.entries {
            for sense in &entry.senses {
                let Some(&src_id) = sense_to_id.get(&sense.id) else {
                    continue;
                };
                for rel in &sense.relations {
                    let Some(&tgt_id) = sense_to_id.get(&rel.target) else {
                        continue;
                    };
                    use lmf::SenseRelationType as SnR;
                    let bucket = match rel.rel_type {
                        SnR::Derivation => &mut relations.derivation,
                        SnR::Pertainym => &mut relations.pertainym,
                        SnR::Similar => &mut relations.similar_sense,
                        SnR::Also => &mut relations.also_sense,
                        SnR::Exemplifies => &mut relations.exemplifies_sense,
                        SnR::IsExemplifiedBy => &mut relations.is_exemplified_by_sense,
                        SnR::Participle => &mut relations.participle_sense,
                        // Already handled above:
                        SnR::Antonym | SnR::Other(_) => continue,
                    };
                    bucket.entry(src_id).or_default().push(tgt_id);
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
        let morphology = crate::cognitive::linguistics::morphology::english::english_rules();

        English {
            concepts,
            word_index,
            taxonomy_children,
            taxonomy_parents,
            opposition,
            mereology_parts,
            relations,
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

#[cfg(test)]
mod inflection_index_tests {
    use super::*;
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;

    /// Sample LMF with inflected Form elements — verifies the
    /// from_wordnet inflection index wires forms to lemma concepts.
    const INFLECTED_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" version="1.0">
    <LexicalEntry id="e-run-v">
      <Lemma writtenForm="run" partOfSpeech="v"/>
      <Form writtenForm="runs"/>
      <Form writtenForm="ran"/>
      <Form writtenForm="running"/>
      <Sense id="run-v-01" synset="s-run"/>
    </LexicalEntry>
    <LexicalEntry id="e-child-n">
      <Lemma writtenForm="child" partOfSpeech="n"/>
      <Form writtenForm="children"/>
      <Sense id="child-n-01" synset="s-child"/>
    </LexicalEntry>
    <Synset id="s-run" ili="i1" partOfSpeech="v"><Definition>move fast on foot</Definition></Synset>
    <Synset id="s-child" ili="i2" partOfSpeech="n"><Definition>young person</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

    #[test]
    fn inflected_forms_index_to_lemma_concepts() {
        let wn = read_wordnet(INFLECTED_LMF).unwrap();
        let en = English::from_wordnet(&wn);

        // `run` (the lemma) and `ran` / `runs` / `running` (its forms)
        // all resolve to the same s-run concept.
        let run_ids = en.lookup("run");
        let ran_ids = en.lookup("ran");
        let runs_ids = en.lookup("runs");
        let running_ids = en.lookup("running");

        assert!(!run_ids.is_empty(), "run should resolve");
        assert_eq!(ran_ids, run_ids, "ran should map to same concepts as run");
        assert_eq!(runs_ids, run_ids, "runs should map to same concepts");
        assert_eq!(running_ids, run_ids, "running should map to same concepts");
    }

    #[test]
    fn irregular_plural_forms_resolve() {
        let wn = read_wordnet(INFLECTED_LMF).unwrap();
        let en = English::from_wordnet(&wn);
        let child_ids = en.lookup("child");
        let children_ids = en.lookup("children");
        assert!(!child_ids.is_empty(), "child should resolve");
        assert_eq!(
            children_ids, child_ids,
            "children should map to child's concepts"
        );
    }

    #[test]
    fn wordnet_relations_loaded_for_derivation_pertainym_domain() {
        // Inline LMF exercising derivation (sense-level),
        // pertainym (sense-level), and has_domain_topic (synset-
        // level) — three of the previously-unused relation types
        // now loaded into English::relations.
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-compensate-v">
      <Lemma writtenForm="compensate" partOfSpeech="v"/>
      <Sense id="s-compensate-v-1" synset="s-compensate">
        <SenseRelation relType="derivation" target="s-compensation-n-1"/>
      </Sense>
    </LexicalEntry>
    <LexicalEntry id="e-compensation-n">
      <Lemma writtenForm="compensation" partOfSpeech="n"/>
      <Sense id="s-compensation-n-1" synset="s-compensation">
        <SenseRelation relType="derivation" target="s-compensate-v-1"/>
      </Sense>
    </LexicalEntry>
    <LexicalEntry id="e-legal-a">
      <Lemma writtenForm="legal" partOfSpeech="a"/>
      <Sense id="s-legal-a-1" synset="s-legal">
        <SenseRelation relType="pertainym" target="s-law-n-1"/>
      </Sense>
    </LexicalEntry>
    <LexicalEntry id="e-law-n">
      <Lemma writtenForm="law" partOfSpeech="n"/>
      <Sense id="s-law-n-1" synset="s-law"/>
    </LexicalEntry>
    <Synset id="s-compensate" ili="i1" partOfSpeech="v"><Definition>pay back</Definition></Synset>
    <Synset id="s-compensation" ili="i2" partOfSpeech="n">
      <Definition>payment</Definition>
      <SynsetRelation relType="has_domain_topic" target="s-law"/>
    </Synset>
    <Synset id="s-legal" ili="i3" partOfSpeech="a"><Definition>of the law</Definition></Synset>
    <Synset id="s-law" ili="i4" partOfSpeech="n"><Definition>system of rules</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(LMF).expect("LMF");
        let en = English::from_wordnet(&wn);
        let rels = en.relations();

        // Derivation: compensate ↔ compensation, both directions.
        assert!(
            !rels.derivation.is_empty(),
            "derivation relation should be populated"
        );

        // Pertainym: "legal" → "law".
        assert!(
            !rels.pertainym.is_empty(),
            "pertainym relation should be populated"
        );

        // Domain-topic: compensation has_domain_topic LAW.
        assert!(
            !rels.has_domain_topic.is_empty(),
            "has_domain_topic should be populated"
        );
    }

    #[test]
    fn entries_without_form_elements_still_resolve() {
        const NO_FORMS_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" version="1.0">
    <LexicalEntry id="e-dog-n">
      <Lemma writtenForm="dog" partOfSpeech="n"/>
      <Sense id="dog-n-01" synset="s-dog"/>
    </LexicalEntry>
    <Synset id="s-dog" ili="i1" partOfSpeech="n"><Definition>canine</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        let wn = read_wordnet(NO_FORMS_LMF).unwrap();
        let en = English::from_wordnet(&wn);
        let dog_ids = en.lookup("dog");
        assert!(!dog_ids.is_empty(), "lemma without forms still resolves");
    }
}
