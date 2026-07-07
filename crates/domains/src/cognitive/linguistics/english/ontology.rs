#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use hashbrown::HashMap;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::ontology::{ConceptRef, subsumption_kind};

use crate::cognitive::linguistics::english::concept_store::{ConceptStore, ConceptView};
use crate::cognitive::linguistics::english::taxonomy_store::TaxonomyStore;
use crate::cognitive::linguistics::english::word_index::WordIndex;
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
    /// All concepts (synsets) indexed by [`ConceptId`]. Held as a compact,
    /// zero-copy [`ConceptStore`] — under `prx` a single `rkyv`-archived buffer
    /// (the largest reclaim of the loaded reasoner's footprint after the
    /// [`word_index`](super::word_index) reclaim), an owned `Vec<Concept>`
    /// otherwise. Read through [`ConceptView`] via [`concept`](Self::concept);
    /// see [`concept_store`](super::concept_store).
    concepts: ConceptStore,
    /// Word text → concept IDs (one word can mean multiple things). Held as a
    /// compact, zero-copy [`WordIndex`] — under `prx` a single packed archive
    /// (the largest single reclaim of the loaded reasoner's footprint), an owned
    /// map otherwise. See [`word_index`](super::word_index).
    pub word_index: WordIndex,
    /// The hypernym (Subsumption) taxonomy — both adjacency directions
    /// (child → parents, parent → children) held as a compact, zero-copy
    /// [`TaxonomyStore`] CSR under `prx` (an owned pair of `HashMap`s otherwise).
    ///
    /// `is_a` / `ancestors` / `common_ancestor` / `ancestor_chain` are answered by
    /// a bounded, `Sync`, per-query breadth-first ascent over these edges — no
    /// pre-folded reflexive-transitive closure and no interior mutability. WordNet's
    /// hypernym DAG is shallow (max is-a depth 16, max reflexive ancestor set 33),
    /// so a query visits only a few tens of nodes and reproduces the closure's
    /// answer exactly. See [`taxonomy_store`](super::taxonomy_store).
    taxonomy: TaxonomyStore,
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

/// The lexical-reasoning surface the chat pipeline queries — the closed set of
/// English operations it actually uses. Re-typing the pipeline onto this trait
/// (instead of the concrete `English`) is what lets a COMPOSED ontology
/// (English ⊕ a loaded corpus) later satisfy the same interface. English is the
/// canonical implementor; the methods mirror its inherent query API 1:1.
pub trait LexicalReasoner {
    fn lookup(&self, word: &str) -> &[ConceptId];
    fn concept(&self, id: ConceptId) -> Option<ConceptView<'_>>;
    fn concept_by_synset(&self, synset_id: &str) -> Option<ConceptView<'_>>;
    fn parents(&self, id: ConceptId) -> &[ConceptId];
    fn children(&self, id: ConceptId) -> &[ConceptId];
    fn is_a(&self, child: ConceptId, ancestor: ConceptId) -> bool;
    /// The reflexive-transitive is-a image of `id` — every ancestor (hypernym)
    /// reachable up the taxonomy, including `id` itself, ordered nearest-first.
    ///
    /// This is a REQUIRED method, not a defaulted breadth-first ascent: every
    /// implementor must answer it from its OWN materialized reachability closure
    /// (the reasoner owns the closure; reachability is never re-derived per
    /// query). The previous per-query BFS default has been eliminated — there is
    /// no hand-walk fallback anywhere on this surface.
    fn ancestors(&self, id: ConceptId) -> Vec<ConceptId>;
    /// The lowest common ancestor of `a` and `b` — the nearest shared hypernym,
    /// or `None` if they share none. The LATTICE MEET over the implementor's
    /// materialized closure (`strict_ancestors(b) ∩ ancestors(a)`, nearest-
    /// first), a categorical op over the materialized set — REQUIRED, never a
    /// hand-BFS.
    fn common_ancestor(&self, a: ConceptId, b: ConceptId) -> Option<ConceptId>;
    /// The ORDERED hypernym chain from `child` up to and including `ancestor` —
    /// `[child, …intermediate hypernyms…, ancestor]`, nearest-first — when
    /// `child` is-a `ancestor`, else `None`. This is the is-a EVIDENCE/
    /// justification path, owned by the reasoner's closure rather than
    /// hand-walked by a consumer. Read off the materialized closure (the chain is
    /// the ancestors of `child` that are themselves at-or-below `ancestor`,
    /// ordered by is-a distance), never a `0..N` parent loop.
    fn ancestor_chain(&self, child: ConceptId, ancestor: ConceptId) -> Option<Vec<ConceptId>>;
    fn concept_count(&self) -> usize;

    /// The maximum number of whitespace-separated words in any surface this
    /// reasoner can resolve — the window the chat's multi-token recognizer scans
    /// to collapse a multi-word surface (a loaded citation/label, a WordNet
    /// collocation) into one lookup unit. Defaults to `1` (a single-word lexicon
    /// like embedded English's primary path → the recognizer is a no-op, so the
    /// single-token pipeline is unchanged); a composed reasoner that holds
    /// multi-word surfaces overrides it with its real maximum.
    fn max_surface_words(&self) -> usize {
        1
    }

    /// Does `child` reach `ancestor` along the loaded relation `kind`? — the
    /// RELATION-PARAMETRIC generalization of [`is_a`](Self::is_a) (which is the
    /// `kind = `[`subsumption_kind`] case). ONE query parameterized by a typed
    /// relation [`ConceptRef`], never a family of per-relation `part_of()` /
    /// `has_part()` methods (that would re-bake the relation set into Rust — the
    /// Subsumption-OR-Parthood anti-pattern). The relation identity is loaded
    /// data; this method only interprets it against the implementor's closures.
    ///
    /// The standard shape is a single relation-parametrized reachability query
    /// (SPARQL property paths; OWL-RL): a transitive closure read keyed by which
    /// relation, not N specialized predicates.
    ///
    /// Default: a Subsumption query delegates to [`is_a`](Self::is_a); any other
    /// kind is an honest `false` — an implementor that carries only a taxonomy
    /// cannot witness a non-Subsumption relation, and must not guess. A reasoner
    /// holding loaded ontologies overrides this to read each kind's MATERIALIZED
    /// closure (Parthood, etc.).
    fn reaches(&self, child: ConceptId, ancestor: ConceptId, kind: &ConceptRef) -> bool {
        if *kind == subsumption_kind() {
            self.is_a(child, ancestor)
        } else {
            false
        }
    }

    /// The relation a natural-language surface asserts, resolved through the
    /// loaded relation lexicon — `"is a"` ↦ [`subsumption_kind`], `"part of"` ↦
    /// the Parthood kind, etc. The ONE blessed surface→kind crossing for a
    /// relational question (the lexicon is `.prx` data; this is its lookup).
    ///
    /// Default: `None` — a reasoner with no loaded relation lexicon (embedded
    /// English) cannot name a relation from a surface, and the caller falls back
    /// to Subsumption. A composed reasoner that loaded the lexicon overrides it.
    fn relation_for_surface(&self, _surface: &str) -> Option<ConceptRef> {
        None
    }

    /// The LOADED ontology a concept belongs to, by [`OntologyName`] — `Some` only
    /// for a concept materialized from a loaded `.prx` (its provenance); `None` for
    /// an embedded-English concept (which has no loaded ontology). The answer path
    /// reads this to record WHICH loaded ontology a turn reasoned over (doc §2.3 —
    /// the trace names a loaded ontology, not just the compiled pipeline ones).
    ///
    /// Default: `None` — embedded English's concepts are the substrate, not a
    /// loaded source. A composed reasoner overrides it (decoding the id's universe).
    fn ontology_of_concept(&self, _id: ConceptId) -> Option<OntologyName> {
        None
    }

    /// The natural-language SURFACE for a relation kind — the inverse of
    /// [`relation_for_surface`](Self::relation_for_surface), used to PHRASE an
    /// affirmation ("part of" for Parthood, so the answer reads "X is part of Y",
    /// not "X is a Y"). Reads the SAME loaded relation lexicon, so the phrasing
    /// connective is loaded data, never a hardcoded "is part of".
    ///
    /// Default: `None` — the is-a default (the copula "is a") and any kind not in
    /// the loaded lexicon have no relational connective. A composed reasoner that
    /// loaded the lexicon overrides it.
    fn surface_for_relation(&self, _kind: &ConceptRef) -> Option<String> {
        None
    }

    /// Does this surface resolve to a LOADED-corpus concept (not the embedded
    /// substrate)? — distinct from [`lookup`](Self::lookup), which unions English
    /// AND loaded. The chat uses this to type a single-word LOADED entity as a
    /// proper noun (NP) so "is X part of Y" parses with a one-word X, WITHOUT
    /// touching English function words (which resolve to the substrate, never a
    /// loaded ontology — so they are not upgraded, the parse-breaking trap a naive
    /// union-lookup gate falls into).
    ///
    /// Default: `false` — embedded English carries no loaded corpus. A composed
    /// reasoner overrides it (true iff a lookup hits a `Loaded` vertex).
    fn is_loaded_surface(&self, _surface: &str) -> bool {
        false
    }

    /// The ordered EVIDENCE chain `[child, …, ancestor]` along the relation `kind` —
    /// the relation-parametric generalization of [`ancestor_chain`](Self::ancestor_chain)
    /// (which is the `kind = `[`subsumption_kind`] case). For Parthood it is the
    /// part-of chain (`subsection → section → title`), so a "is X part of Y" answer
    /// can show its mereological evidence, not just the endpoints.
    ///
    /// Default: a Subsumption chain delegates to [`ancestor_chain`](Self::ancestor_chain);
    /// any other kind is `None` — the embedded substrate has ONE un-keyed hypernym
    /// closure and cannot chain a non-is-a relation (the audit's constraint: the
    /// relation-parametric chain lives on the closure + composed reasoner, never the
    /// substrate). A composed reasoner reads each kind's materialized closure.
    fn relation_chain(
        &self,
        child: ConceptId,
        ancestor: ConceptId,
        kind: &ConceptRef,
    ) -> Option<Vec<ConceptId>> {
        if *kind == subsumption_kind() {
            self.ancestor_chain(child, ancestor)
        } else {
            None
        }
    }
}

impl LexicalReasoner for English {
    fn lookup(&self, word: &str) -> &[ConceptId] {
        English::lookup(self, word)
    }
    fn concept(&self, id: ConceptId) -> Option<ConceptView<'_>> {
        English::concept(self, id)
    }
    fn concept_by_synset(&self, s: &str) -> Option<ConceptView<'_>> {
        English::concept_by_synset(self, s)
    }
    fn parents(&self, id: ConceptId) -> &[ConceptId] {
        English::parents(self, id)
    }
    fn children(&self, id: ConceptId) -> &[ConceptId] {
        English::children(self, id)
    }
    fn is_a(&self, c: ConceptId, a: ConceptId) -> bool {
        English::is_a(self, c, a)
    }
    fn ancestors(&self, id: ConceptId) -> Vec<ConceptId> {
        English::ancestors(self, id)
    }
    fn common_ancestor(&self, a: ConceptId, b: ConceptId) -> Option<ConceptId> {
        English::common_ancestor(self, a, b)
    }
    fn ancestor_chain(&self, child: ConceptId, ancestor: ConceptId) -> Option<Vec<ConceptId>> {
        English::ancestor_chain(self, child, ancestor)
    }
    fn concept_count(&self) -> usize {
        English::concept_count(self)
    }
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
        function_words: HashMap<String, Vec<LexicalEntry>>,
        function_word_list: Vec<String>,
        verb_transitivity: HashMap<String, Vec<Transitivity>>,
        writing: WritingSystem,
        morphology: Vec<MorphologicalRule>,
    ) -> Self {
        let concept_count = concepts.len();
        Self {
            // Transcode the owned concept build into the compact store ONCE; the
            // source `Vec<Concept>` is consumed and freed, only the archived form
            // survives (§`concept_store`).
            concepts: ConceptStore::build(concepts),
            // Transcode the owned build into the compact index ONCE; the source
            // map is consumed and freed, only the packed form survives.
            word_index: WordIndex::build(word_index),
            // Transcode the owned adjacency maps into the compact taxonomy CSR
            // ONCE; the source maps are consumed and freed (§`taxonomy_store`).
            taxonomy: TaxonomyStore::build(taxonomy_parents, taxonomy_children, concept_count),
            opposition,
            mereology_parts,
            relations: WordnetRelations::default(),
            synset_to_concept,
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

    /// [`English::sample`] behind a process-wide `OnceLock`, for callers that need
    /// a shared `&'static English` — the [`ComposedReasoner`] fixtures, which now
    /// BORROW their English (single-substrate-instance ownership) rather than own
    /// it. `sample()` is a deterministic const-fixture parse, so one shared static
    /// instance is observationally identical to a fresh `sample()` per call. Sound
    /// because `English` is `Sync` (see the `assert_sync` witness below).
    ///
    /// [`ComposedReasoner`]: crate::cognitive::linguistics::composed::ComposedReasoner
    pub fn sample_static() -> &'static English {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<English> = OnceLock::new();
        INSTANCE.get_or_init(English::sample)
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
                let bucket = match &rel.rel_type {
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
                    let bucket = match &rel.rel_type {
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

        // `sense_to_id` is a BUILD-TIME index only: it keys the sense-level
        // relation folds above (opposition — Phase 4 — and the derivation /
        // pertainym / similar-sense / participle / exemplifies passes in
        // Phase 5b). Nothing reads a sense's numeric id after construction —
        // there is no forward-facing accessor — so it is intentionally NOT
        // stored on `English`; the ~185k-entry `String`-keyed map is dropped
        // here with the rest of the load transients. The codegen path
        // (`language::from_codegen`) never builds it at all.
        drop(sense_to_id);

        let concept_count = concepts.len();
        English {
            // Transcode the owned concept build into the compact store ONCE; the
            // source `Vec<Concept>` is consumed and freed, only the archived form
            // survives (§`concept_store`).
            concepts: ConceptStore::build(concepts),
            // Transcode the owned build into the compact index ONCE; the source
            // map is consumed and freed, only the packed form survives.
            word_index: WordIndex::build(word_index),
            // Transcode the owned adjacency maps (Phase 3) into the compact
            // taxonomy CSR ONCE; the source maps are consumed and freed. The
            // reflexive-transitive is-a reachability is computed per query by a
            // bounded, `Sync` breadth-first ascent over these edges — no eager
            // closure fold (§`taxonomy_store`).
            taxonomy: TaxonomyStore::build(taxonomy_parents, taxonomy_children, concept_count),
            opposition,
            mereology_parts,
            relations,
            synset_to_concept,
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
        self.word_index.lookup(word)
    }

    /// Get a concept by its [`ConceptId`], as a [`ConceptView`] over the compact
    /// store.
    pub fn concept(&self, id: ConceptId) -> Option<ConceptView<'_>> {
        self.concepts.get(id)
    }

    /// Get a concept by its original WordNet synset ID string.
    pub fn concept_by_synset(&self, synset_id: &str) -> Option<ConceptView<'_>> {
        self.synset_to_concept
            .get(synset_id)
            .and_then(|id| self.concept(*id))
    }

    /// Every concept, as a [`ConceptView`], in [`ConceptId`] order — the
    /// representation-agnostic replacement for reading the (now compact) store's
    /// records directly.
    pub fn concepts(&self) -> impl Iterator<Item = ConceptView<'_>> {
        self.concepts.iter()
    }

    /// Direct parents (hypernyms) of a concept — is-a targets.
    pub fn parents(&self, id: ConceptId) -> &[ConceptId] {
        self.taxonomy.parents(id)
    }

    /// Direct children (hyponyms) of a concept — is-a sources.
    pub fn children(&self, id: ConceptId) -> &[ConceptId] {
        self.taxonomy.children(id)
    }

    /// Check if `child` is-a `ancestor` (reflexive-transitively) — a bounded,
    /// `Sync` breadth-first ascent over the archived parent edges. WordNet's
    /// hypernym DAG is shallow (max reflexive ancestor set 33), so this visits a
    /// few tens of nodes and reproduces the eager closure's answer exactly.
    /// `ancestor` is in `child`'s reflexive Subsumption image iff the is-a relation
    /// holds (reflexive: every concept is-a itself). See
    /// [`taxonomy_store`](super::taxonomy_store).
    pub fn is_a(&self, child: ConceptId, ancestor: ConceptId) -> bool {
        self.taxonomy.is_a(child, ancestor)
    }

    /// The reflexive-transitive hypernym image of `id` — `id` itself plus every
    /// ancestor reachable up the taxonomy, ordered nearest-first by
    /// `(minimal is-a distance, ConceptId.value())`. A bounded per-query ascent.
    pub fn ancestors(&self, id: ConceptId) -> Vec<ConceptId> {
        self.taxonomy.ancestors(id)
    }

    /// The lowest common ancestor of `a` and `b` — the nearest shared hypernym,
    /// computed as the LATTICE MEET over the hypernym relation (the argmin, by
    /// distance from `b`, of `strict_ancestors(b) ∩ reflexive_ancestors(a)`).
    /// Ties are broken by the smaller `ConceptId.value()` for a deterministic
    /// result, exactly as the eager closure's `meet_by`.
    pub fn common_ancestor(&self, a: ConceptId, b: ConceptId) -> Option<ConceptId> {
        self.taxonomy.common_ancestor(a, b)
    }

    /// The ordered hypernym chain `[child, …, ancestor]` (nearest-first) when
    /// `child` is-a `ancestor`, else `None`. The is-a evidence path: exactly those
    /// reflexive ancestors `x` of `child` from which `ancestor` is itself reachable
    /// (so `x` lies on a `child ⇝ ancestor` path), ordered by
    /// `(is-a distance from child, ConceptId.value())`. On a tree taxonomy this is
    /// the unique path; on a DAG it is the distance-ordered set of intermediate
    /// hypernyms on a shortest witnessing path.
    pub fn ancestor_chain(&self, child: ConceptId, ancestor: ConceptId) -> Option<Vec<ConceptId>> {
        self.taxonomy.ancestor_chain(child, ancestor)
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
        self.taxonomy.parent_edge_count()
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

/// The canonical full English (Open English WordNet) ontology, loaded ONCE per
/// process behind a `OnceLock`.
///
/// Reads the content-addressed compact `.prx` archive
/// ([`english_compact_prx_cache_dir`][cd]) when one is present — gunzip +
/// fail-closed content gate + succinct decode + `from_wordnet` materialization,
/// far cheaper than the 89 MB WN-LMF XML parse it does otherwise. The English
/// analogue of [`uslm::corpus::loaded`][usc]: the shared fast path for every
/// full-English consumer (the `pr4xis chat` CLI, the lambek/adjunction test
/// fixtures), so each `OnceLock` re-init under nextest's process-per-test model
/// loads the compact archive instead of re-parsing the giant.
///
/// [cd]: crate::social::software::markup::xml::lmf::prx::english_compact_prx_cache_dir
/// [usc]: crate::social::software::markup::xml::uslm::corpus::loaded
pub fn english_load_owned() -> English {
    let english = english_load_owned_inner();
    // Return the freed load transient to the OS. Building `English` materializes
    // a large, short-lived word→concept map that is transcoded into the packed
    // `WordIndex` buffer and dropped (§`word_index`); glibc keeps those freed
    // small allocations in its arena rather than returning the pages, which would
    // otherwise mask the buffer's reclaim in RSS. This one-shot page-return makes
    // the reclaim resident. It is an OS-accounting call, not a correctness device
    // — the live-heap reduction holds on every platform; this is a no-op off
    // glibc.
    return_freed_pages();
    english
}

/// Return freed pages held in the glibc arena back to the OS (`malloc_trim`).
/// glibc-only; a no-op on every other target (musl, wasm32, non-Linux), where
/// the packed-buffer live-heap reduction still holds — this only affects when the
/// freed load transient stops counting toward RSS.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn return_freed_pages() {
    // SAFETY: `malloc_trim` is a glibc libc entry point with no preconditions; it
    // returns freed top-of-arena pages and reports whether any were released.
    unsafe {
        unsafe extern "C" {
            fn malloc_trim(pad: usize) -> core::ffi::c_int;
        }
        let _ = malloc_trim(0);
    }
}

/// No-op page-return on non-glibc targets (see the glibc variant).
#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
fn return_freed_pages() {}

/// The load body — reads the compact `.prx` fast path, else parses the WN-LMF
/// XML. Wrapped by [`english_load_owned`], which returns the freed load transient
/// to the OS afterwards.
fn english_load_owned_inner() -> English {
    use crate::applied::data_provisioning::registry::data_sources;
    use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;

    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    // THE canonical English: the single registered `Language`-kind source
    // (English is the sole Language implementor today). Selected by kind —
    // exactly as every emit/anchor path filters Language — not by a name
    // literal, so a registry rename can never silently desync the loader from
    // the emitter that produced its archive.
    let entry = data_sources()
        .iter()
        .find(|e| e.kind == SourceTaxonomyConcept::Language)
        .expect("english_load_owned(): no Language-kind source registered");

    // Fastest path: the content-addressed COMPACT archive, admitted through
    // the fail-closed `[compact_archive_signatures]` gate — gunzip +
    // hash-check + succinct decode, with NO XML re-parse. Tried first; an
    // absent or unpinned compact archive falls through to the XML parse.
    #[cfg(feature = "prx")]
    {
        use crate::applied::data_provisioning::registry::lock_compact_archive_signature;
        use crate::social::software::markup::xml::lmf::prx;
        let cprx_path = prx::english_compact_prx_cache_dir(&workspace_root)
            .join(format!("{}-{}.cprx.gz", entry.name, entry.version));
        if let Ok(cprx_gz) = std::fs::read(&cprx_path)
            && let Some(pin) = lock_compact_archive_signature(&entry.name, &entry.version)
        {
            let key = format!("{}@{}", entry.name, entry.version);
            match prx::load_compact_english_prx_gz_gated(&cprx_gz, pin, &key) {
                Ok(en) => return en,
                // Pinned but the compact archive failed the content gate — the
                // committed pin and emitted bytes disagree. Fail LOUD.
                Err(e) => panic!(
                    "english_load_owned(): compact archive {} is pinned but failed the \
                     content gate: {e}",
                    cprx_path.display()
                ),
            }
        }
    }

    // Fallback: parse the WN-LMF XML (a fresh checkout with no compiled
    // archive pays the ~89 MB parse, the same graceful path the emitters use).
    let src_path = workspace_root.join(entry.local_path());
    let xml = std::fs::read_to_string(&src_path)
        .expect("english_load_owned(): WordNet XML not on disk — run `pr4xis update`");
    let wn = read_wordnet(&xml).expect("english_load_owned(): parse WordNet");
    English::from_wordnet(&wn)
}

/// [`english_load_owned`] behind a process-wide `OnceLock`, for callers that want
/// a shared `&'static` (the test/anchor fixtures that observe `english()` many
/// times per process). NOT load-bearing: the compact `.prx` load is ms-cheap, so
/// the cache is a convenience, not a perf necessity. A consumer that needs an
/// OWNED English — the [`ComposedReasoner`](crate::cognitive::linguistics::composed)
/// the chat builds from a loaded corpus — calls [`english_load_owned`] directly.
pub fn english_loaded() -> &'static English {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<English> = OnceLock::new();
    INSTANCE.get_or_init(english_load_owned)
}

/// `English` is `Sync` — required for the `OnceLock<English>` static above. Every
/// field is `Sync` with NO interior mutability: the taxonomy reachability is a
/// per-query breadth-first ascent over immutable edges (no `RefCell` memo), which
/// is precisely what lets us drop the eager closure yet keep the process-wide
/// static valid. This compile-time assertion fails the moment a `!Sync` field
/// (e.g. a memoizing closure) is reintroduced.
const _: fn() = || {
    fn assert_sync<T: Sync>() {}
    assert_sync::<English>();
};

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
                concept.pos(),
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
                && seen_pos.insert(concept.pos())
            {
                let transitivities = self.verb_transitivities(word);
                results.extend(
                    crate::cognitive::linguistics::language::lmf_pos_to_lexical_entries(
                        word,
                        concept.pos(),
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
                        && seen_pos.insert(concept.pos())
                    {
                        let transitivities = self.verb_transitivities(stem);
                        results.extend(
                            crate::cognitive::linguistics::language::lmf_pos_to_lexical_entries(
                                stem,
                                concept.pos(),
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
        words.extend(self.word_index.words());
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

    #[pr4xis::praxis_value(Verifiable)]
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

    #[pr4xis::praxis_value(Verifiable)]
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

    #[pr4xis::praxis_value(Verifiable)]
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

    #[pr4xis::praxis_value(Verifiable)]
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
