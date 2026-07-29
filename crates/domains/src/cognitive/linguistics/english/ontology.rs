#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use hashbrown::HashMap;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::ontology::{ConceptRef, subsumption_kind};

use crate::cognitive::linguistics::english::concept_senses_index::{self, ConceptSensesIndex};
use crate::cognitive::linguistics::english::concept_store::{ConceptStore, ConceptView};
use crate::cognitive::linguistics::english::function_word_store::FunctionWordStore;
use crate::cognitive::linguistics::english::morphology_store::MorphologyStore;
use crate::cognitive::linguistics::english::relation_store::{RelationKind, RelationStore};
use crate::cognitive::linguistics::english::sense_concept_index::SenseConceptIndex;
use crate::cognitive::linguistics::english::synset_index::SynsetIndex;
use crate::cognitive::linguistics::english::taxonomy_store::TaxonomyStore;
use crate::cognitive::linguistics::english::verb_transitivity_index::VerbTransitivityIndex;
use crate::cognitive::linguistics::english::word_index::WordIndex;
use crate::cognitive::linguistics::english::writing_system_store::WritingSystemStore;
use crate::cognitive::linguistics::lambek::pregroup::PregroupType;
use crate::cognitive::linguistics::lexicon::pos::*;
use crate::cognitive::linguistics::morphology::MorphologicalRule;
use crate::cognitive::linguistics::orthography::WritingSystem;
use crate::formal::information::ontology::Reference;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
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
    /// The largest whitespace-separated word count among ALL of
    /// [`word_index`](Self::word_index)'s own keys — e.g. `4` if the loaded
    /// WordNet carries a 4-word multi-word lemma like "old-age insurance
    /// benefits" as one of its longest entries. DERIVED from `word_index`
    /// at construction (a single O(word-count) scan over `word_index.words()`,
    /// the same "compute once at construction" convention
    /// [`fold_index`](Self::fold_index)/[`concept_senses`](Self::concept_senses)
    /// already establish), never recomputed per query. Backs
    /// [`Language::max_known_surface_words`](crate::cognitive::linguistics::language::Language::max_known_surface_words) —
    /// the bound `tokenize::multiword_surface_spans` searches up to when
    /// protecting an already-known WordNet multi-word surface from
    /// `correct_unknown_word_surfaces`'s per-word noisy-channel correction. A
    /// real, DATA-DERIVED bound, not a hand-picked constant: before this
    /// field existed, that search tried every window length from 2 up to the
    /// FULL remaining sentence for every start position — confirmed via
    /// direct instrumentation to be the dominant O(n²)-to-O(n³) cost behind
    /// `defines_pointers` timing out on real, long USC Title 42 candidates
    /// (`crates/domains/src/cognitive/linguistics/lambek/tokenize.rs`'s own
    /// `multiword_surface_spans` doc has the full measurement).
    max_multiword_surface_words: usize,
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
    /// Every non-taxonomy WordNet relation — opposition (antonym), mereology
    /// (whole → parts), and the ~25 `WordnetRelations` sub-maps (derivation,
    /// pertainym, domain_topic, attribute, causes, entails, …) — held as ONE
    /// compact, zero-copy [`RelationStore`] family of labelled CSRs under `prx`
    /// (owned `HashMap`s otherwise). Read through [`RelationKind`]-keyed accessors
    /// ([`opposites`](Self::opposites) / [`parts`](Self::parts) /
    /// [`derivations`](Self::derivations) / …). See
    /// [`relation_store`](super::relation_store).
    relations: RelationStore,
    /// Synset ID string → [`ConceptId`], held as a compact, zero-copy
    /// [`SynsetIndex`] sorted-key dictionary under `prx` (an owned `HashMap`
    /// otherwise). Backs [`concept_by_synset`](Self::concept_by_synset). See
    /// [`synset_index`](super::synset_index).
    synset_index: SynsetIndex,

    // === Language trait data ===
    /// Function words (closed class, OLiA-classified) — held as a compact,
    /// zero-copy [`FunctionWordStore`] under `prx` (a single sorted-key `rkyv`
    /// archive that ALSO subsumes the old `function_word_list`: the sorted key set
    /// IS the word list), an owned `HashMap` otherwise. Read through
    /// [`first`](FunctionWordStore::first) / [`all`](FunctionWordStore::all) /
    /// [`words`](FunctionWordStore::words). See
    /// [`function_word_store`](super::function_word_store).
    function_words: FunctionWordStore,
    /// Verb transitivity from WordNet subcategorization frames — held as a compact,
    /// zero-copy [`VerbTransitivityIndex`] (a sorted-key dictionary with a one-byte
    /// discriminant run cast zero-copy to `&[Transitivity]`) under `prx`, an owned
    /// `HashMap` otherwise. See
    /// [`verb_transitivity_index`](super::verb_transitivity_index).
    verb_transitivity: VerbTransitivityIndex,
    /// Writing system — held as a compact, zero-copy [`WritingSystemStore`] `rkyv`
    /// archive under `prx`, an owned [`WritingSystem`] otherwise. See
    /// [`writing_system_store`](super::writing_system_store).
    writing: WritingSystemStore,
    /// Morphological rules — held as a compact, zero-copy [`MorphologyStore`] `rkyv`
    /// archive under `prx`, an owned `Vec` otherwise. The warm stemming loop reads
    /// the suffix texts zero-copy; the cold trait reader deserializes. See
    /// [`morphology_store`](super::morphology_store).
    morphology: MorphologyStore,
    /// The fold-on-miss secondary index (Slice D,
    /// `.notes/chat-fix-c-build-state.md`) — fold(original) → the union of
    /// concept ids across every original-cased surface in THIS instance's
    /// [`word_index`](Self::word_index) that folds to it, via the loaded
    /// Unicode simple case-folding table
    /// ([`case_folding`](crate::cognitive::linguistics::orthography::case_folding)).
    /// Computed EAGERLY at construction (mirroring `word_index` itself),
    /// not lazily behind a global cache — an `English` value is queried
    /// through [`lookup_case_folded`](Self::lookup_case_folded) on its OWN
    /// data, correctly scoped to whichever instance is `self` (a small test
    /// fixture and the process-wide `english_loaded()` singleton each get
    /// their own). See [`fold_index`](super::fold_index).
    fold_index: WordIndex,
    /// The sense→concept bridge (`SenseId → ConceptId`) — the forward leg
    /// bridging WordNet's sense-keyed [`RelationKind::Opposition`] onto
    /// concept-keyed queries ([`opposes`](Self::opposes)). Held as a compact,
    /// zero-copy [`SenseConceptIndex`] under `prx` (an owned map otherwise).
    /// See [`sense_concept_index`](super::sense_concept_index).
    sense_concept: SenseConceptIndex,
    /// The concept→senses inverse (`ConceptId → [SenseId]`) — DERIVED from
    /// [`sense_concept`](Self::sense_concept) at construction (never
    /// persisted, mirroring [`fold_index`](Self::fold_index)'s derive-at-
    /// construction pattern). See
    /// [`concept_senses_index`](super::concept_senses_index).
    concept_senses: ConceptSensesIndex,
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
    /// Topic-domain membership, keyed by the DOMAIN synset -> its member
    /// terms (e.g. "law" -> "patent"). CORRECTED direction (2026-07-21):
    /// despite the field's name reading as "a term has a domain topic",
    /// OEWN 2025 stores this `relType` on the DOMAIN synset pointing AT
    /// its members -- verified against the loaded corpus:
    /// `oewn-08458195-n` "law" carries `has_domain_topic` edges to ~30
    /// member synsets (including the "letters patent" sense
    /// `oewn-06563618-n`), and `oewn-06376048-n` "literature" carries 19.
    /// See [`domain_topic`](Self::domain_topic) for the inverse
    /// (member -> domain) direction. Bentivogli & Pianta (2004).
    pub has_domain_topic: HashMap<ConceptId, Vec<ConceptId>>,
    /// The inverse of `has_domain_topic`: topic-domain membership keyed
    /// by the MEMBER synset -> the domain(s) it belongs to (e.g.
    /// "patent" -> "law"). Verified against the loaded corpus:
    /// `oewn-06563618-n` ("letters patent") carries a `domain_topic`
    /// edge to `oewn-08458195-n` ("law"). Bentivogli & Pianta (2004).
    pub domain_topic: HashMap<ConceptId, Vec<ConceptId>>,
    /// Region-domain membership, keyed by the REGION synset -> its
    /// member terms (e.g. "Australia" -> "kangaroo") -- by the same
    /// container-on-the-`has_`-side convention verified for
    /// `has_domain_topic` above (not independently re-verified against a
    /// real `domain_region` edge; OEWN 2025's bundled data has none under
    /// "kangaroo").
    pub has_domain_region: HashMap<ConceptId, Vec<ConceptId>>,
    /// Inverse: region member -> the region(s) it belongs to.
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    fn concept_count(&self) -> Quantity;

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

    /// The [`ConditionalRule`](crate::social::judicial::conditional_rule::ConditionalRule)
    /// governing `predicate` when asked about `object`, if a loaded
    /// registry has grounded one — e.g. `predicate` "eligible for" and
    /// `object` naming an asset transfer → the Medicaid asset-transfer rule
    /// extracted from 42 U.S.C. § 1396p(c)(1)(A). `object` is REQUIRED, not
    /// optional: a real registry entry is topically narrow (this one is
    /// specifically about asset transfers, not general program eligibility),
    /// so matching on `predicate` alone would confidently attach a rule to
    /// unrelated questions — see `conditional_rule::registry::
    /// rule_for_predicate_and_object`'s module doc. Defaults to `None`
    /// (embedded English carries no rule corpus, making every existing
    /// question shape byte-identical); `English`'s own impl delegates to the
    /// registry's matcher, and a composed reasoner delegates to `English`.
    fn conditional_rule_for_predicate(
        &self,
        predicate: &str,
        object: &str,
    ) -> Option<crate::social::judicial::conditional_rule::ConditionalRule> {
        let _ = (predicate, object);
        None
    }

    /// Case-FOLDED fallback lookup — tried only after `lookup(word)` (exact
    /// case) misses. Recovers a loaded surface whose ORIGINAL casing differs
    /// from `word` (the WordNet lemma "Section Eight" for a query surface
    /// "section eight"; "O.K."; "Turkish bath") via the loaded Unicode
    /// simple case-folding table
    /// ([`case_folding`](crate::cognitive::linguistics::orthography::case_folding)),
    /// never `str::to_lowercase` (Slice D,
    /// `.notes/chat-fix-c-build-state.md`: 17,272 of 131,798 loaded WordNet
    /// surfaces carry an uppercase letter, and the tokenizer's lowercasing
    /// discards that case before `lookup`'s exact-case index ever runs).
    ///
    /// Returns an OWNED `Vec` (unlike `lookup`'s zero-copy `&[ConceptId]`) —
    /// this is a rarely-hit fallback tier, not the hot exact-match path, so
    /// unioning across every differently-cased variant that shares a fold is
    /// not worth a second zero-copy index for every implementor.
    ///
    /// Default: no fold support (an empty result) — matching
    /// [`max_surface_words`](Self::max_surface_words)'s own default-no-op
    /// pattern. Embedded English overrides it (the concrete loaded lexicon
    /// case-folding targets); a composed reasoner overrides it too,
    /// delegating to the wrapped English substrate for the English-vertex
    /// population.
    fn lookup_case_folded(&self, _word: &str) -> Vec<ConceptId> {
        Vec::new()
    }

    /// Is `word` a loaded function word (closed-class: determiner,
    /// preposition, auxiliary, …)? Used to strip non-content words from the
    /// gloss-overlap word-sense scorer ([`word_sense::best_reaching_pair`](super::word_sense::best_reaching_pair) —
    /// standard Lesk-family stopword removal, sourced from this reasoner's
    /// OWN loaded closed-class lexicon, never a hand-authored stopword list.
    ///
    /// Default: `false` — a reasoner with no loaded function-word lexicon
    /// treats every word as content (conservative: no false stripping).
    fn is_function_word(&self, _word: &str) -> bool {
        false
    }

    /// Is `word`'s loaded closed-class reading SPECIFICALLY a non-possessive
    /// pronoun ("I", "he", "who", …)? A narrower query than
    /// [`is_function_word`](Self::is_function_word): several closed-class
    /// words ("above", "so") are ALSO legitimate open-class content words in
    /// a different sense ("the above information", "deadly serious"), so
    /// gating entity-hood on `is_function_word` over-excludes those
    /// polysemous readings. A personal/interrogative/demonstrative/relative/
    /// reflexive/indefinite pronoun essentially never IS the queried content
    /// word in a caregiver-style question (Bobrow, Kaplan, Norman, Thompson &
    /// Winograd 1977, GUS: refuse rather than guess), so excluding those
    /// kinds is exact rather than a coarse stopword-style filter.
    ///
    /// [`PronounKind::Possessive`]
    /// is DELIBERATELY excluded from this check even though it is a pronoun
    /// reading: Huddleston & Pullum 2002 Ch. 5 §10's genitive/independent-
    /// possessive class ("mine", "yours", "his", …) is exactly the kind that
    /// collides with an unrelated open-class common noun ("mine" = "a gold
    /// mine") — the same over-exclusion failure mode `is_function_word`
    /// has for "above"/"so", now known to recur inside the pronoun class
    /// itself. A caller that also needs to catch a genuine possessive-
    /// pronoun content word must query the loaded [`PronounKind`] directly,
    /// not this coarse gate.
    ///
    /// Default: `false` — a reasoner with no loaded closed-class lexicon
    /// treats no word as a pronoun (conservative: no false exclusion).
    fn is_pronoun(&self, _word: &str) -> bool {
        false
    }

    /// Is `word` an interrogative pronoun/determiner whose expected-answer
    /// type is a THING or a SELECTION-from-a-set ("what"/"which") — as
    /// opposed to a PERSON-asking wh-word ("who"/"whom"/"whose")? The
    /// cross-linguistic THING/SELECTION vs PERSON split among wh-words
    /// (Cysouw 2004, "Interrogative words: an exercise in lexical typology",
    /// handout, session on question formation in Bantu, ZAS Berlin, 13 Feb
    /// 2004, §3.2 table (9)) — the loaded [`WhReferentRole`]
    /// (`crate::cognitive::linguistics::lexicon::pos`) feature this reads,
    /// grouping `Thing`+`Selection` together (not `Selection` alone) since
    /// this gate's only use so far ("is this a 'what/which is X' definitional
    /// query, not a person-identifying one") treats both the same way a
    /// caller needing the finer 3-way split must query [`WhReferentRole`]
    /// directly, the same "coarse gate vs. loaded feature" split
    /// [`is_pronoun`](Self::is_pronoun)'s own doc draws for
    /// `PronounKind::Possessive`.
    ///
    /// Default: `false` — a reasoner with no loaded closed-class lexicon
    /// treats no word as any interrogative kind (conservative: no false
    /// inclusion).
    fn is_nonpersonal_interrogative(&self, _word: &str) -> bool {
        false
    }

    /// Every real statutory definition of `word` reachable through a loaded
    /// USC-style corpus's `defines` grounding edges — `(provision URN, the
    /// defining provision's own prose)` pairs, most-specific-first as the
    /// loaded ontology orders them. Distinct from
    /// [`concept`](Self::concept)'s WordNet/LKIF gloss: this reads the
    /// separate statutory-definition channel a
    /// `social::judicial::statute_structure::grounding::defines_pointers`
    /// chart-parse extraction (never regex) grounds onto a provision node,
    /// letting a "what is X" answer cite the actual controlling statutory
    /// text when one exists, rather than (or alongside) a dictionary gloss.
    ///
    /// Default: empty — a reasoner with no loaded statute corpus (or no
    /// `defines` edges within it) has no statutory definitions to offer.
    fn statute_definitions(&self, _word: &str) -> alloc::vec::Vec<(&str, &str)> {
        alloc::vec::Vec::new()
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

    /// The comparison-relation kind a DERIVED relational-noun HEAD word
    /// asserts, resolved through the loaded comparison-relation lexicon —
    /// `"difference"` ↦ the Relations vocabulary's `Association` kind, etc.
    /// (`comparison_relation_lexicon::comparison_relation_surface_index`).
    /// Barker (2011) "Possessives and Relational Nouns" §1.4: a DERIVED
    /// relational noun ("difference", deverbal from "differ (from)") can
    /// overtly express MULTIPLE participants via its own PP complement
    /// ("the difference BETWEEN X and Y"), unlike an underived relational
    /// noun's single-participant ceiling ("the Secretary OF Commerce" —
    /// "secretary" is never in this lexicon). A SEPARATE method/index from
    /// [`relation_for_surface`](Self::relation_for_surface) — deliberately:
    /// a "difference between X and Y" question is not a fact to verify
    /// against a materialized closure (there is no Contrast edge between
    /// two arbitrary defined terms), it is a request to recite each named
    /// term's own gloss, so folding it into the closure-verification
    /// surface `relation_for_surface` feeds would silently misroute it into
    /// relation-verification instead (see
    /// `comparison_relation_lexicon`'s own module doc for the full
    /// rationale).
    ///
    /// Default: `None` — a reasoner with no loaded comparison-relation
    /// lexicon (embedded English) cannot name a comparison relation from a
    /// surface. A composed reasoner that loaded the lexicon overrides it.
    fn comparison_relation_for_surface(&self, _head_word: &str) -> Option<ConceptRef> {
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
    fn lookup_case_folded(&self, word: &str) -> Vec<ConceptId> {
        use crate::cognitive::linguistics::orthography::case_folding;
        let folded = case_folding::table().fold(word);
        // The common case first: a lemma that is ALREADY all-lowercase
        // (the overwhelming majority) needs no fold-index entry at all —
        // an all-caps/title-case query folds straight back to its ordinary
        // exact key.
        let exact = English::lookup(self, &folded);
        if !exact.is_empty() {
            return exact.to_vec();
        }
        // The genuinely case-marked population ("Section Eight", "O.K.",
        // "Turkish bath") whose OWN WordIndex key does not fold to itself —
        // THIS instance's own fold index (Self::fold_index), never a global.
        self.fold_index.lookup(&folded).to_vec()
    }
    fn is_function_word(&self, word: &str) -> bool {
        self.function_words.first(word).is_some()
    }
    fn is_pronoun(&self, word: &str) -> bool {
        use crate::cognitive::linguistics::lexicon::pos::{LexicalEntry, PronounKind};
        matches!(
            self.function_words.first(word),
            Some(LexicalEntry::Pronoun(p)) if p.kind != PronounKind::Possessive
        )
    }
    fn is_nonpersonal_interrogative(&self, word: &str) -> bool {
        use crate::cognitive::linguistics::lexicon::pos::WhReferentRole;
        matches!(
            self.function_words
                .first(word)
                .and_then(|e| e.wh_referent_role()),
            Some(WhReferentRole::Thing | WhReferentRole::Selection)
        )
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
    fn concept_count(&self) -> Quantity {
        English::concept_count(self)
    }
    fn conditional_rule_for_predicate(
        &self,
        predicate: &str,
        object: &str,
    ) -> Option<crate::social::judicial::conditional_rule::ConditionalRule> {
        crate::social::judicial::conditional_rule::registry::rule_for_predicate_and_object(
            self, predicate, object,
        )
    }
}

impl English {
    /// Construct an English ontology from pre-computed parts.
    /// Used by the Language module's deployment functors (codegen, mmap, async).
    ///
    /// `sense_count` is the dense sense-id key space (`0..sense_count`) for the
    /// sense-level relations (opposition, derivation, …); a deployment path with
    /// no assigned senses (codegen) passes `0` and an empty `sense_concept` map
    /// — codegen's flat generated arrays carry no sense-level data at all (every
    /// other sense-level map is likewise empty on that path), so a codegen-built
    /// `English` has no opposition reachability, consistent with its existing
    /// zero-sense-level-relations posture. The synset-level relations
    /// (mereology, `also_synset`, …) are keyed over `concepts.len()`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        concepts: Vec<Concept>,
        word_index: HashMap<String, Vec<ConceptId>>,
        taxonomy_children: HashMap<ConceptId, Vec<ConceptId>>,
        taxonomy_parents: HashMap<ConceptId, Vec<ConceptId>>,
        opposition: HashMap<SenseId, Vec<SenseId>>,
        mereology_parts: HashMap<ConceptId, Vec<ConceptId>>,
        relations: WordnetRelations,
        sense_count: usize,
        sense_concept: HashMap<SenseId, ConceptId>,
        synset_to_concept: HashMap<String, ConceptId>,
        function_words: HashMap<String, Vec<LexicalEntry>>,
        verb_transitivity: HashMap<String, Vec<Transitivity>>,
        writing: WritingSystem,
        morphology: Vec<MorphologicalRule>,
    ) -> Self {
        let concept_count = concepts.len();
        // Scanned BEFORE `word_index` moves into the packed `WordIndex` build
        // below — see `English`'s own `max_multiword_surface_words` field doc
        // for why this is a real, DATA-DERIVED bound rather than a hand-picked
        // constant. One-time O(word-count) pass, the same cost class every
        // other `*_index::build`/`*Store::build` call here already pays.
        let max_multiword_surface_words = word_index
            .keys()
            .map(|w| w.split_whitespace().count())
            .max()
            .unwrap_or(1)
            .max(1);
        // Transcode the owned build into the compact index ONCE; the source map
        // is consumed and freed, only the packed form survives. The fold index
        // is DERIVED from the packed form (never persisted, never re-parsed) —
        // see `fold_index::build`.
        let word_index = WordIndex::build(word_index);
        let fold_index = super::fold_index::build(&word_index);
        // Transcode the owned sense→concept map into the compact index ONCE; the
        // source map is consumed and freed. `concept_senses` is DERIVED from the
        // packed form (never persisted, never re-parsed) — see
        // `concept_senses_index::build` (§`sense_concept_index`).
        let sense_concept = SenseConceptIndex::build(sense_concept, sense_count);
        let concept_senses = concept_senses_index::build(&sense_concept, concept_count);
        Self {
            // Transcode the owned concept build into the compact store ONCE; the
            // source `Vec<Concept>` is consumed and freed, only the archived form
            // survives (§`concept_store`).
            concepts: ConceptStore::build(concepts),
            word_index,
            max_multiword_surface_words,
            fold_index,
            // Transcode the owned adjacency maps into the compact taxonomy CSR
            // ONCE; the source maps are consumed and freed (§`taxonomy_store`).
            taxonomy: TaxonomyStore::build(taxonomy_parents, taxonomy_children, concept_count),
            // Transcode every relation map into the compact labelled CSR family
            // ONCE; the source maps are consumed and freed (§`relation_store`).
            relations: RelationStore::build(
                opposition,
                mereology_parts,
                relations,
                sense_count,
                concept_count,
            ),
            // Transcode the synset-id dictionary into the compact index ONCE; the
            // source map is consumed and freed (§`synset_index`).
            synset_index: SynsetIndex::build(synset_to_concept),
            // Transcode the Language-trait data into its compact stores ONCE; each
            // source collection is consumed and freed, only the packed form
            // survives. `function_word_list` is GONE — the sorted key set of
            // `function_words` IS the word list.
            function_words: FunctionWordStore::build(function_words),
            verb_transitivity: VerbTransitivityIndex::build(verb_transitivity),
            writing: WritingSystemStore::build(writing),
            morphology: MorphologyStore::build(morphology),
            sense_concept,
            concept_senses,
        }
    }

    /// All derivation links for a sense (sense ↔ morphologically-
    /// related sense per Fellbaum-Osherson-Clark 2009).
    pub fn derivations(&self, sense: SenseId) -> &[SenseId] {
        self.relations.rel(RelationKind::Derivation, sense)
    }

    /// Pertainym targets for a sense (relational-adjective → noun
    /// base per Fellbaum 1998 §5.2).
    pub fn pertainyms(&self, sense: SenseId) -> &[SenseId] {
        self.relations.rel(RelationKind::Pertainym, sense)
    }

    /// Every member concept assigned to `domain`'s domain-topic (domain →
    /// member, per Bentivogli & Pianta 2004). CORRECTED direction
    /// (2026-07-21): despite the name reading "concept has a domain
    /// topic", the loaded `has_domain_topic` edge is carried by the
    /// DOMAIN synset — verified against the loaded OEWN 2025 corpus (see
    /// [`WordnetRelations::has_domain_topic`] doc). See
    /// [`domain_topic`](Self::domain_topic) for the inverse.
    pub fn has_domain_topic(&self, domain: ConceptId) -> &[ConceptId] {
        self.relations.rel(RelationKind::HasDomainTopic, domain)
    }

    /// The inverse of [`has_domain_topic`](Self::has_domain_topic): the
    /// domain-topic(s) `member` itself belongs to (member → domain, per
    /// Bentivogli & Pianta 2004) — e.g. `domain_topic(patent) ==
    /// [law]`, verified against the loaded OEWN 2025 corpus.
    pub fn domain_topic(&self, member: ConceptId) -> &[ConceptId] {
        self.relations.rel(RelationKind::DomainTopic, member)
    }

    /// The class(es) `instance` exemplifies (synset-level `exemplifies`,
    /// WN-LMF instance-of: the classic FRBR/IFLA-relevant "Homer exemplifies
    /// poet" edge — `exemplifies(homer) == [poet]`, empirically confirmed;
    /// the edge is keyed by the INSTANCE synset, not the class, so a prior
    /// doc revision naming the parameter `class` had the direction backwards).
    pub fn exemplifies(&self, instance: ConceptId) -> &[ConceptId] {
        self.relations.rel(RelationKind::Exemplifies, instance)
    }

    /// The inverse of [`exemplifies`](Self::exemplifies): the instance(s)
    /// that exemplify `class` — `is_exemplified_by(poet) == [homer]`.
    pub fn is_exemplified_by(&self, class: ConceptId) -> &[ConceptId] {
        self.relations.rel(RelationKind::IsExemplifiedBy, class)
    }

    /// The number of edges of a given relation `kind` — the discoverable count
    /// accessor over the labelled [`RelationStore`], replacing the removed
    /// `relations()` struct exposure (the sole reader was a self-test asserting a
    /// relation is populated).
    pub fn relation_edge_count(&self, kind: RelationKind) -> Quantity {
        self.relations.edge_count(kind)
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

        // Phase 2: Assign SenseIds, and record each sense's owning concept
        // (synset). `synset_to_concept` is already fully populated (Phase 1
        // ran first), so this is a direct lookup, not a second parse.
        let mut sense_counter = 0u64;
        let mut sense_concept: HashMap<SenseId, ConceptId> = HashMap::new();
        for entry in &wn.entries {
            for sense in &entry.senses {
                let sense_id = SenseId::new(sense_counter);
                sense_to_id.insert(sense.id.clone(), sense_id);
                if let Some(&concept_id) = synset_to_concept.get(&sense.synset) {
                    sense_concept.insert(sense_id, concept_id);
                }
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
        // Transcode the owned build into the compact index ONCE; the source map
        // is consumed and freed, only the packed form survives. The fold index
        // is DERIVED from the packed form — see `fold_index::build`.
        // Scanned BEFORE `word_index` moves into the packed `WordIndex` build
        // below — see `English`'s own `max_multiword_surface_words` field doc.
        let max_multiword_surface_words = word_index
            .keys()
            .map(|w| w.split_whitespace().count())
            .max()
            .unwrap_or(1)
            .max(1);
        let word_index = WordIndex::build(word_index);
        let fold_index = super::fold_index::build(&word_index);
        // Transcode the owned sense→concept map (Phase 2) into the compact index
        // ONCE; the source map is consumed and freed. `concept_senses` is
        // DERIVED from the packed form — see `concept_senses_index::build`
        // (§`sense_concept_index`).
        let sense_concept = SenseConceptIndex::build(sense_concept, sense_counter as usize);
        let concept_senses = concept_senses_index::build(&sense_concept, concept_count);
        English {
            // Transcode the owned concept build into the compact store ONCE; the
            // source `Vec<Concept>` is consumed and freed, only the archived form
            // survives (§`concept_store`).
            concepts: ConceptStore::build(concepts),
            word_index,
            max_multiword_surface_words,
            fold_index,
            // Transcode the owned adjacency maps (Phase 3) into the compact
            // taxonomy CSR ONCE; the source maps are consumed and freed. The
            // reflexive-transitive is-a reachability is computed per query by a
            // bounded, `Sync` breadth-first ascent over these edges — no eager
            // closure fold (§`taxonomy_store`).
            taxonomy: TaxonomyStore::build(taxonomy_parents, taxonomy_children, concept_count),
            // Transcode every relation map (opposition — Phase 4, mereology —
            // Phase 5, and the WordnetRelations web — Phase 5b) into the compact
            // labelled CSR family ONCE; the source maps are consumed and freed
            // (§`relation_store`). Sense-level relations are keyed over the
            // `sense_counter` dense space assigned in Phase 2.
            relations: RelationStore::build(
                opposition,
                mereology_parts,
                relations,
                sense_counter as usize,
                concept_count,
            ),
            // Transcode the synset-id → ConceptId dictionary (Phase 1) into the
            // compact index ONCE; the source map is consumed and freed
            // (§`synset_index`).
            synset_index: SynsetIndex::build(synset_to_concept),
            sense_concept,
            concept_senses,
            // Transcode the Language-trait data into its compact stores ONCE; each
            // source collection is consumed and freed. `function_word_list` is GONE
            // — the sorted key set of `function_words` IS the word list
            // (§`function_word_store`, §`verb_transitivity_index`,
            // §`writing_system_store`, §`morphology_store`).
            function_words: FunctionWordStore::build(function_words),
            verb_transitivity: VerbTransitivityIndex::build(verb_transitivity),
            writing: WritingSystemStore::build(writing),
            morphology: MorphologyStore::build(morphology),
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
        self.synset_index
            .lookup(synset_id)
            .and_then(|id| self.concept(id))
    }

    /// The concept (synset) `sense` belongs to — the forward leg of the
    /// sense↔concept bridge (`None` if `sense` is absent from the source
    /// WordNet dump or out of range). See [`senses_of`](Self::senses_of) for
    /// the inverse.
    pub fn concept_of_sense(&self, sense: SenseId) -> Option<ConceptId> {
        self.sense_concept.concept_of(sense)
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
        self.relations.rel(RelationKind::MereologyParts, id)
    }

    /// Opposites (antonyms) of a sense.
    pub fn opposites(&self, sense_id: SenseId) -> &[SenseId] {
        self.relations.rel(RelationKind::Opposition, sense_id)
    }

    /// Does concept `a` oppose concept `b` — Saussure (1916) / Cruse (1986)
    /// antonymy, at the CONCEPT level? WordNet's `Opposition` edges are
    /// sense-keyed (`big#1` opposes `small#1`, not the concept "big" wholesale)
    /// and — unlike [`is_a`](Self::is_a)/[`parts_reach`](Self::parts_reach) —
    /// non-transitive (a single direct edge is the whole answer; see
    /// [`opposition_relation_kind`](crate::formal::relations::ontology::opposition_relation_kind)),
    /// so this bridges concept → its senses ([`senses_of`](Self::senses_of))
    /// → each sense's direct opposition targets → back to `b`'s sense set: `a`
    /// opposes `b` iff SOME sense of `a` has a direct edge to SOME sense of `b`.
    pub fn opposes(&self, a: ConceptId, b: ConceptId) -> bool {
        let b_senses = self.senses_of(b);
        self.senses_of(a).iter().any(|&sa| {
            self.relations
                .rel(RelationKind::Opposition, sa)
                .iter()
                .any(|target| b_senses.contains(target))
        })
    }

    /// Every sense naming `concept` (its synonym set), in ascending
    /// [`SenseId`] order — the [`ConceptSensesIndex`] read backing
    /// [`opposes`](Self::opposes).
    pub fn senses_of(&self, concept: ConceptId) -> &[SenseId] {
        self.concept_senses.senses_of(concept)
    }

    /// Does `whole` transitively have `part` as a part — Casati & Varzi (1999)
    /// mereology, part-of is transitive (unlike [`opposes`](Self::opposes)'s
    /// direct-edge-only check; see
    /// [`parthood_relation_kind`](crate::formal::relations::ontology::parthood_relation_kind)).
    /// A bounded, `Sync` per-query breadth-first descent over the direct
    /// `MereologyParts` edges — the same shared graded-reach engine
    /// [`is_a`](Self::is_a) mints over the taxonomy. Parthood is `Irreflexive`,
    /// so — unlike `is_a`'s reflexive short-circuit — `whole == part` never
    /// itself witnesses `true`.
    pub fn parts_reach(&self, whole: ConceptId, part: ConceptId) -> bool {
        self.relations.parts_reach(whole, part)
    }

    /// Does concept `a` relate to concept `b` via WordNet's Derivation
    /// relation (morphological relatedness — Fellbaum-Osherson-Clark 2009,
    /// `compensate ↔ compensation`) at the CONCEPT level? Sense-keyed and
    /// non-transitive (a single direct edge is the whole answer, like
    /// [`opposes`](Self::opposes)), bridged the same way: `a` derivation-
    /// relates to `b` iff some sense of `a` has a direct Derivation edge to
    /// some sense of `b`.
    pub fn derivation_relates(&self, a: ConceptId, b: ConceptId) -> bool {
        let b_senses = self.senses_of(b);
        self.senses_of(a).iter().any(|&sa| {
            self.relations
                .rel(RelationKind::Derivation, sa)
                .iter()
                .any(|target| b_senses.contains(target))
        })
    }

    /// Does concept `a` (a relational adjective, e.g. "dental") pertain to
    /// concept `b` (its noun base, e.g. "tooth") — Fellbaum 1998 §5.2?
    /// Sense-keyed, DIRECTIONAL (adjective → noun; WordNet declares no
    /// inverse Pertainym pointer) and non-transitive, bridged the same way
    /// [`opposes`](Self::opposes) bridges Opposition.
    pub fn pertains_to(&self, a: ConceptId, b: ConceptId) -> bool {
        let b_senses = self.senses_of(b);
        self.senses_of(a).iter().any(|&sa| {
            self.relations
                .rel(RelationKind::Pertainym, sa)
                .iter()
                .any(|target| b_senses.contains(target))
        })
    }

    /// Total number of concepts.
    pub fn concept_count(&self) -> Quantity {
        self.concepts.len()
    }

    /// Total number of unique words.
    pub fn word_count(&self) -> Quantity {
        Quantity::from_unit(self.word_index.len() as f64, &unit::UNITLESS)
    }

    /// Total taxonomy relations.
    pub fn taxonomy_count(&self) -> Quantity {
        self.taxonomy.parent_edge_count()
    }

    /// Total opposition relations.
    pub fn opposition_count(&self) -> Quantity {
        self.relations.edge_count(RelationKind::Opposition)
    }

    /// Get verb transitivity options from pre-computed frames — a zero-copy slice
    /// borrowed from the compact [`VerbTransitivityIndex`].
    fn verb_transitivities(&self, word: &str) -> &[Transitivity] {
        self.verb_transitivity.lookup(word)
    }
}

/// The store-bundle surface — the per-store access the bundle codec
/// ([`store_bundle`](super::store_bundle)) frames and the direct-from-stores
/// constructor its decode leg assembles. Archived (`prx` + little-endian) only:
/// the bundle serializes the packed/archived store representations verbatim.
#[cfg(all(feature = "prx", target_endian = "little"))]
impl English {
    /// Assemble an `English` DIRECTLY from its ten already-validated stores —
    /// the decode leg of the store bundle. No WordNet decode, no
    /// [`from_wordnet`](Self::from_wordnet), no owned intermediate maps: each
    /// store was validated by its own fail-closed entry
    /// (`from_untrusted_buf` for the six packed CSR stores, the `bytecheck`
    /// `from_validated_buf` pass for the four rich `rkyv` stores) before it
    /// reaches here.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn from_stores(
        concepts: ConceptStore,
        word_index: WordIndex,
        taxonomy: TaxonomyStore,
        relations: RelationStore,
        synset_index: SynsetIndex,
        sense_concept: SenseConceptIndex,
        function_words: FunctionWordStore,
        verb_transitivity: VerbTransitivityIndex,
        writing: WritingSystemStore,
        morphology: MorphologyStore,
    ) -> Self {
        // Not bundled stores — DERIVED (never persisted, no bundle-schema
        // change): `fold_index` from `word_index` (`fold_index::build`),
        // `concept_senses` from `sense_concept` (`concept_senses_index::build`),
        // `max_multiword_surface_words` from `word_index` (see `English`'s
        // own field doc — the SAME derivation the two HashMap-based
        // constructors run, just over the already-packed store's own
        // `words()` reader instead of an owned map's `keys()`).
        let fold_index = super::fold_index::build(&word_index);
        let concept_senses =
            concept_senses_index::build(&sense_concept, concepts.len().value as usize);
        let max_multiword_surface_words = word_index
            .words()
            .map(|w| w.split_whitespace().count())
            .max()
            .unwrap_or(1)
            .max(1);
        Self {
            concepts,
            word_index,
            max_multiword_surface_words,
            fold_index,
            taxonomy,
            relations,
            synset_index,
            sense_concept,
            concept_senses,
            function_words,
            verb_transitivity,
            writing,
            morphology,
        }
    }

    /// The concept store — bundle-emit access.
    pub(super) fn concepts_store(&self) -> &ConceptStore {
        &self.concepts
    }

    /// The taxonomy store — bundle-emit access.
    pub(super) fn taxonomy_store(&self) -> &TaxonomyStore {
        &self.taxonomy
    }

    /// The relation store — bundle-emit access.
    pub(super) fn relations_store(&self) -> &RelationStore {
        &self.relations
    }

    /// The synset-id index — bundle-emit access.
    pub(super) fn synset_index_store(&self) -> &SynsetIndex {
        &self.synset_index
    }

    /// The sense→concept index — bundle-emit access. `concept_senses` (its
    /// derived inverse) is NOT emitted — it is rebuilt at
    /// [`from_stores`](Self::from_stores) time from this store, same as
    /// `fold_index` from `word_index`.
    pub(super) fn sense_concept_store(&self) -> &SenseConceptIndex {
        &self.sense_concept
    }

    /// The function-word store — bundle-emit access.
    pub(super) fn function_words_store(&self) -> &FunctionWordStore {
        &self.function_words
    }

    /// The verb-transitivity index — bundle-emit access.
    pub(super) fn verb_transitivity_store(&self) -> &VerbTransitivityIndex {
        &self.verb_transitivity
    }

    /// The writing-system store — bundle-emit access.
    pub(super) fn writing_store(&self) -> &WritingSystemStore {
        &self.writing
    }

    /// The morphology store — bundle-emit access.
    pub(super) fn morphology_store(&self) -> &MorphologyStore {
        &self.morphology
    }
}

/// The `Derivation` relation kind (Fellbaum-Osherson-Clark 2009) as a
/// [`ConceptRef`] — the typed handle `ComposedReasoner::reaches` (`crate::
/// cognitive::linguistics::composed`) compares against to route a query to
/// [`English::derivation_relates`]. Named directly via `relations_kind`
/// (`pr4xis_runtime::ontology`), the same minting pattern [`subsumption_kind`]
/// itself uses — not one of the ten canonical cross-ontology
/// `RelationsConcept` types (those live in `formal::relations::ontology`), a
/// WordNet-specific lexical-semantic relation kind instead.
pub fn derivation_relation_kind() -> ConceptRef {
    pr4xis_runtime::ontology::relations_kind("Derivation")
}

/// The `Pertainym` relation kind (Fellbaum 1998 §5.2) as a [`ConceptRef`] —
/// see [`derivation_relation_kind`] for the minting pattern.
pub fn pertainym_relation_kind() -> ConceptRef {
    pr4xis_runtime::ontology::relations_kind("Pertainym")
}

/// The `HasDomainTopic` relation kind (term → domain; Bentivogli & Pianta
/// 2004) as a [`ConceptRef`] — see [`derivation_relation_kind`] for the
/// minting pattern.
pub fn has_domain_topic_relation_kind() -> ConceptRef {
    pr4xis_runtime::ontology::relations_kind("HasDomainTopic")
}

/// The inverse of [`has_domain_topic_relation_kind`]: `DomainTopic` (domain
/// → term).
pub fn domain_topic_relation_kind() -> ConceptRef {
    pr4xis_runtime::ontology::relations_kind("DomainTopic")
}

/// The `Exemplifies` relation kind (synset-level instance-of; the FRBR/IFLA
/// "Homer exemplifies poet" edge) as a [`ConceptRef`] — see
/// [`derivation_relation_kind`] for the minting pattern.
pub fn exemplifies_relation_kind() -> ConceptRef {
    pr4xis_runtime::ontology::relations_kind("Exemplifies")
}

/// The inverse of [`exemplifies_relation_kind`]: `IsExemplifiedBy`.
pub fn is_exemplified_by_relation_kind() -> ConceptRef {
    pr4xis_runtime::ontology::relations_kind("IsExemplifiedBy")
}

/// The canonical full English (Open English WordNet) ontology, loaded ONCE per
/// process behind a `OnceLock`.
///
/// Three tiers, fastest first: (1) the content-addressed STORE BUNDLE
/// (`english_store_bundle_cache_dir`) — the nine BUILT store buffers behind
/// the fail-closed `[store_bundle_signatures]` gate, assembled with NO WordNet
/// decode and NO `from_wordnet` (the load transient collapses to ~the resident
/// cost); (2) the content-addressed compact `.prx` archive
/// (`english_compact_prx_cache_dir`) — gunzip + fail-closed content gate +
/// succinct decode + `from_wordnet` materialization; (3) the 89 MB WN-LMF XML
/// parse. The `from_wordnet` tiers MUST remain: they are the path that emits
/// tier (1) in the first place (`pr4xis compile`). The English analogue of
/// [`uslm::corpus::loaded`][usc]: the shared fast path for every full-English
/// consumer (the `pr4xis chat` CLI, the lambek/adjunction test fixtures), so
/// each `OnceLock` re-init under nextest's process-per-test model loads a
/// compiled archive instead of re-parsing the giant.
///
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

    // FASTEST tier: the STORE BUNDLE — the nine BUILT store buffers, admitted
    // through the fail-closed `[store_bundle_signatures]` gate (gunzip +
    // hash-check + per-store validation), assembled with NO WordNet decode
    // and NO `from_wordnet` (the load transient collapses to ~the resident
    // cost). Same-toolchain by construction: the bundle in `.prx-cache` was
    // emitted by this workspace's own `pr4xis compile`. An absent or unpinned
    // bundle falls through to the compact succinct tier below.
    #[cfg(all(feature = "prx", target_endian = "little"))]
    {
        use crate::applied::data_provisioning::registry::lock_store_bundle_signature;
        use crate::social::software::markup::xml::lmf::prx;
        let bundle_path = prx::english_store_bundle_cache_dir(&workspace_root)
            .join(format!("{}-{}.stores.gz", entry.name, entry.version));
        if let Ok(bundle_gz) = std::fs::read(&bundle_path)
            && let Some(pin) = lock_store_bundle_signature(&entry.name, &entry.version)
        {
            let key = format!("{}@{}", entry.name, entry.version);
            match prx::load_english_store_bundle_gz_gated(&bundle_gz, pin, &key) {
                Ok(en) => return en,
                // Pinned but the bundle failed the content gate — the committed
                // pin and emitted bytes disagree (a toolchain bump without a
                // re-pin, or tampering). Fail LOUD, exactly as the compact tier
                // does: a pinned fast path must never silently degrade.
                Err(e) => panic!(
                    "english_load_owned(): store bundle {} is pinned but failed the \
                     content gate: {e} — re-run `pr4xis compile --lock` after a \
                     deliberate toolchain/codec change",
                    bundle_path.display()
                ),
            }
        }
    }

    // Fast path: the content-addressed COMPACT archive, admitted through
    // the fail-closed `[compact_archive_signatures]` gate — gunzip +
    // hash-check + succinct decode, with NO XML re-parse. An absent or
    // unpinned compact archive falls through to the XML parse.
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

    fn writing_system(&self) -> WritingSystem {
        self.writing.writing_system()
    }

    fn morphological_rules(&self) -> Vec<MorphologicalRule> {
        self.morphology.rules()
    }

    fn lexical_lookup(&self, word: &str) -> Option<LexicalEntry> {
        if let Some(entry) = self.function_words.first(word) {
            return Some(entry);
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
        results.extend(self.function_words.all(word));
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

        // Morphological analysis: resolve inflected surfaces through the cited
        // dual-route lemmatizer (identity → loaded AGID irregulars → rule
        // inversion + Spencer-§5.2 allomorphy) instead of a naive suffix
        // strip — "coughing"/"running"/"exhaling" all reach their verb lemma
        // where bare stripping left "runn"/"exhal" unresolvable. For each
        // candidate stem the lexicon knows, emit the stem's entries (the
        // morphology functor InflectedForm → Stem → LexicalEntry, as before).
        //
        // When the surface IS the stem's -ing form — decided by the same
        // dual route in the GENERATING direction (`ing_form(stem) == word`:
        // loaded AGID exceptions blocking the Quirk-§3 rule) — ALSO emit the
        // verb entries MARKED with the form-level OLiA class (`ing`, the
        // EAGLES gerund-participle merger class; CGEL pp. 1220–1222), so the
        // loaded OLiA→CCG functor projects the gerundial-nominal reading
        // (CCGbank Manual App. B.4.1: gerund subjects are treated like NPs)
        // with zero tokenizer logic.
        use crate::cognitive::linguistics::lexicon::olia::form_level_class;
        use crate::cognitive::linguistics::morphology::SemanticEffect;
        use crate::cognitive::linguistics::morphology::english::generation::{
            ing_form, is_past_participle_form_of, is_plural_form_of,
        };
        use crate::cognitive::linguistics::morphology::lemmatizer::{
            Language as MorphLanguage, lemmatize,
        };
        let mut seen_gerund = hashbrown::HashSet::new();
        let mut seen_participle = hashbrown::HashSet::new();
        let mut seen_plural = hashbrown::HashSet::new();
        for form in lemmatize(word, MorphLanguage::English) {
            let stem = form.written_rep;
            if stem == word {
                continue;
            }
            for &cid in self.lookup(&stem) {
                let Some(concept) = self.concept(cid) else {
                    continue;
                };
                if seen_pos.insert(concept.pos()) {
                    let transitivities = self.verb_transitivities(&stem);
                    results.extend(
                        crate::cognitive::linguistics::language::lmf_pos_to_lexical_entries(
                            &stem,
                            concept.pos(),
                            transitivities,
                        ),
                    );
                }
                // The gerundial reading is keyed by stem (one marked entry
                // set per verb lemma), NOT by bare POS — a plain verb entry
                // for the stem must not swallow it, nor vice versa.
                if concept.pos() == crate::social::software::markup::xml::lmf::LmfPos::Verb
                    && ing_form(&stem) == word
                    && seen_gerund.insert(stem.clone())
                    && let Some(class) = form_level_class(SemanticEffect::Progressive)
                {
                    let transitivities = self.verb_transitivities(&stem);
                    results.extend(
                        crate::cognitive::linguistics::language::lmf_pos_to_lexical_entries(
                            word,
                            concept.pos(),
                            transitivities,
                        )
                        .into_iter()
                        .filter_map(|e| match e {
                            LexicalEntry::Verb(mut v) => {
                                v.lemma = stem.clone();
                                v.olia_class = Some(class.to_string());
                                Some(LexicalEntry::Verb(v))
                            }
                            _ => None,
                        }),
                    );
                }
                // The PAST/PASSIVE-PARTICIPLE reading — decided by the same
                // dual route in the GENERATING direction
                // (`is_past_participle_form_of(stem, word)`: the loaded AGID
                // participle/preterite slots blocking the Quirk-§3 `-ed`
                // rule) — ALSO emits the verb entries MARKED with the
                // form-level OLiA class `PastParticiple`
                // (`rdfs:subClassOf olia.owl#Participle`,
                // `owl:equivalentClass Participle and (hasTense some Past)`),
                // so a consumer can tell the participial reading of an `-ed`
                // surface apart from the FINITE reading `lmf_pos_to_lexical_
                // entries` mints above — which is all WordNet's own frame
                // data can give, since WordNet indexes lemmas, not word
                // forms. Without this mark the two readings are literally the
                // same `Verb` value and no grammar downstream can select the
                // participial one. Keyed by stem (one marked entry set per
                // verb lemma), mirroring the gerund block above exactly.
                //
                // The mark is INERT unless a consumer asks for it: the
                // loaded OLiA→CCG functor
                // (`data/grammar/olia-ccg-categories.tsv`) carries no
                // `PastParticiple` row, so `categories_for_class` returns
                // empty for it and the tokenizer's OLiA-class projection adds
                // no category — by design (see
                // `statute_structure::grounding::participle_alternatives`,
                // which scopes the CCGbank reduced-relative analysis to the
                // defines lens the way `bare_noun_phrase_unary_rule` is
                // already scoped there).
                if concept.pos() == crate::social::software::markup::xml::lmf::LmfPos::Verb
                    && is_past_participle_form_of(&stem, word)
                    && seen_participle.insert(stem.clone())
                    && let Some(class) = form_level_class(SemanticEffect::PastParticiple)
                {
                    let transitivities = self.verb_transitivities(&stem);
                    results.extend(
                        crate::cognitive::linguistics::language::lmf_pos_to_lexical_entries(
                            word,
                            concept.pos(),
                            transitivities,
                        )
                        .into_iter()
                        .filter_map(|e| match e {
                            LexicalEntry::Verb(mut v) => {
                                v.lemma = stem.clone();
                                v.olia_class = Some(class.to_string());
                                Some(LexicalEntry::Verb(v))
                            }
                            _ => None,
                        }),
                    );
                }
                // The plural reading — decided by the same dual route in the
                // GENERATING direction (`is_plural_form_of(stem, word)`:
                // loaded AGID exceptions blocking the Quirk et al. 1985
                // §3.21 rule) — ALSO emits a Noun entry keyed to the SURFACE
                // `word` (not the stem) and marked `Number::Plural`, so a
                // downstream bare-plural-NP promotion (Carlson 1977, "A
                // unified analysis of the English bare plural") can tell a
                // genuine plural surface ("dogs", "children") from the
                // stem's own (singular) entry above, without re-deriving
                // morphology of its own. Keyed by stem (one marked entry per
                // noun lemma), mirroring the gerund block immediately above.
                if concept.pos() == crate::social::software::markup::xml::lmf::LmfPos::Noun
                    && is_plural_form_of(&stem, word)
                    && seen_plural.insert(stem.clone())
                {
                    results.push(LexicalEntry::Noun(
                        crate::cognitive::linguistics::lexicon::pos::Noun {
                            text: word.to_string(),
                            number: crate::cognitive::linguistics::lexicon::pos::Number::Plural,
                            person: crate::cognitive::linguistics::lexicon::pos::Person::Third,
                            countability:
                                crate::cognitive::linguistics::lexicon::pos::Countability::Countable,
                            kind: crate::cognitive::linguistics::lexicon::pos::NounKind::Common,
                        },
                    ));
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
        let mut words: Vec<&str> = self.function_words.words().collect();
        words.extend(self.word_index.words());
        words
    }

    fn concept_count(&self) -> Quantity {
        self.concepts.len()
    }

    fn word_count(&self) -> Quantity {
        Quantity::from_unit(
            self.word_index.len() as f64 + self.function_words.len().value,
            &unit::UNITLESS,
        )
    }

    /// Exact-case lookup first (WordNet spells `"O.K."` with its defining
    /// capitals and periods verbatim in `word_index`), then case-folded —
    /// the SAME exact-then-folded order [`LexicalReasoner::lookup_case_folded`]
    /// already applies, reused rather than re-implemented.
    fn is_known_surface(&self, word: &str) -> bool {
        !self.lookup(word).is_empty() || !self.lookup_case_folded(word).is_empty()
    }

    /// Reads the field computed ONCE at construction — see
    /// `max_multiword_surface_words`'s own doc for the real O(word-count)
    /// derivation and why it lives there rather than being rescanned per
    /// query.
    fn max_known_surface_words(&self) -> usize {
        self.max_multiword_surface_words
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

        // Derivation: compensate ↔ compensation, both directions.
        assert!(
            en.relation_edge_count(RelationKind::Derivation).value > 0.0,
            "derivation relation should be populated"
        );

        // Pertainym: "legal" → "law".
        assert!(
            en.relation_edge_count(RelationKind::Pertainym).value > 0.0,
            "pertainym relation should be populated"
        );

        // Domain-topic: this fixture's `has_domain_topic` edge is carried
        // by "compensation" pointing at "law" -- an inline-fixture
        // convenience only, not a claim about which side is the domain in
        // real WN-LMF data (the loaded corpus carries `has_domain_topic`
        // on the DOMAIN synset; see `WordnetRelations::has_domain_topic`).
        // This assertion just checks the relType loads at all.
        assert!(
            en.relation_edge_count(RelationKind::HasDomainTopic).value > 0.0,
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

/// FIX-A: concept-level Opposition (`opposes`) and Parthood (`parts_reach`)
/// reachability, bridged through the new sense↔concept stores
/// ([`sense_concept_index`](super::sense_concept_index),
/// [`concept_senses_index`](super::concept_senses_index)). WordNet's
/// `Opposition` edges are sense-keyed (a single direct edge is the whole
/// answer — non-transitive); `MereologyParts` edges are concept-keyed and
/// genuinely transitive (Casati & Varzi 1999), so `parts_reach` must find a
/// link `parts()` (direct-only) does not.
#[cfg(test)]
mod reachability_bridge_tests {
    use super::*;
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;

    /// `big`/`small` each have TWO senses (a synonym pair per concept); only
    /// ONE sense-pair (`big-a-1` ↔ `small-a-1`) carries the antonym edge. A
    /// three-link mereology chain `car -> engine -> piston -> ring` exercises
    /// multi-hop Parthood; `car -> wheel` is a direct (one-hop) edge as a
    /// control.
    const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-big-a">
      <Lemma writtenForm="big" partOfSpeech="a"/>
      <Sense id="big-a-1" synset="s-big">
        <SenseRelation relType="antonym" target="small-a-1"/>
      </Sense>
    </LexicalEntry>
    <LexicalEntry id="e-large-a">
      <Lemma writtenForm="large" partOfSpeech="a"/>
      <Sense id="large-a-1" synset="s-big"/>
    </LexicalEntry>
    <LexicalEntry id="e-small-a">
      <Lemma writtenForm="small" partOfSpeech="a"/>
      <Sense id="small-a-1" synset="s-small">
        <SenseRelation relType="antonym" target="big-a-1"/>
      </Sense>
    </LexicalEntry>
    <LexicalEntry id="e-little-a">
      <Lemma writtenForm="little" partOfSpeech="a"/>
      <Sense id="little-a-1" synset="s-small"/>
    </LexicalEntry>
    <LexicalEntry id="e-red-a">
      <Lemma writtenForm="red" partOfSpeech="a"/>
      <Sense id="red-a-1" synset="s-red"/>
    </LexicalEntry>
    <LexicalEntry id="e-car-n">
      <Lemma writtenForm="car" partOfSpeech="n"/>
      <Sense id="car-n-1" synset="s-car"/>
    </LexicalEntry>
    <LexicalEntry id="e-engine-n">
      <Lemma writtenForm="engine" partOfSpeech="n"/>
      <Sense id="engine-n-1" synset="s-engine"/>
    </LexicalEntry>
    <LexicalEntry id="e-piston-n">
      <Lemma writtenForm="piston" partOfSpeech="n"/>
      <Sense id="piston-n-1" synset="s-piston"/>
    </LexicalEntry>
    <LexicalEntry id="e-ring-n">
      <Lemma writtenForm="ring" partOfSpeech="n"/>
      <Sense id="ring-n-1" synset="s-ring"/>
    </LexicalEntry>
    <LexicalEntry id="e-wheel-n">
      <Lemma writtenForm="wheel" partOfSpeech="n"/>
      <Sense id="wheel-n-1" synset="s-wheel"/>
    </LexicalEntry>
    <Synset id="s-big" ili="i1" partOfSpeech="a"><Definition>of considerable size</Definition></Synset>
    <Synset id="s-small" ili="i2" partOfSpeech="a"><Definition>of little size</Definition></Synset>
    <Synset id="s-red" ili="i3" partOfSpeech="a"><Definition>of the color red</Definition></Synset>
    <Synset id="s-car" ili="i4" partOfSpeech="n">
      <Definition>a motor vehicle</Definition>
      <SynsetRelation relType="mero_part" target="s-engine"/>
      <SynsetRelation relType="mero_part" target="s-wheel"/>
    </Synset>
    <Synset id="s-engine" ili="i5" partOfSpeech="n">
      <Definition>a machine that converts energy to motion</Definition>
      <SynsetRelation relType="mero_part" target="s-piston"/>
    </Synset>
    <Synset id="s-piston" ili="i6" partOfSpeech="n">
      <Definition>a sliding engine component</Definition>
      <SynsetRelation relType="mero_part" target="s-ring"/>
    </Synset>
    <Synset id="s-ring" ili="i7" partOfSpeech="n"><Definition>a piston seal</Definition></Synset>
    <Synset id="s-wheel" ili="i8" partOfSpeech="n"><Definition>a circular frame</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

    fn fixture() -> English {
        English::from_wordnet(&read_wordnet(LMF).unwrap())
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn opposes_holds_for_a_concept_pair_bridged_through_either_synonym() {
        let en = fixture();
        let big = en.lookup("big")[0];
        let large = en.lookup("large")[0];
        let small = en.lookup("small")[0];
        let little = en.lookup("little")[0];
        assert_eq!(big, large, "big/large share one concept (s-big)");
        assert_eq!(small, little, "small/little share one concept (s-small)");

        assert!(
            en.opposes(big, small),
            "the concept 'big' opposes 'small' via the big-a-1/small-a-1 sense edge"
        );
        // Symmetric read via the SAME concept pair through its other name —
        // still resolves, since `large`/`little` are the same concepts.
        assert!(en.opposes(large, little));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn opposes_is_false_for_an_unrelated_concept_pair() {
        let en = fixture();
        let big = en.lookup("big")[0];
        let red = en.lookup("red")[0];
        assert!(
            !en.opposes(big, red),
            "no antonym edge exists between 'big' and 'red' — honest false, not a guess"
        );
        // Irreflexive in practice: no self-antonym edge was declared.
        assert!(!en.opposes(big, big));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn senses_of_and_concept_of_sense_are_mutual_inverses() {
        let en = fixture();
        let big = en.lookup("big")[0];
        let senses = en.senses_of(big);
        assert_eq!(senses.len(), 2, "big-a-1 and large-a-1 both name s-big");
        for &s in senses {
            assert_eq!(en.concept_of_sense(s), Some(big));
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn parts_reach_finds_a_multi_hop_link_direct_parts_does_not() {
        let en = fixture();
        let car = en.lookup("car")[0];
        let engine = en.lookup("engine")[0];
        let piston = en.lookup("piston")[0];
        let ring = en.lookup("ring")[0];
        let wheel = en.lookup("wheel")[0];

        // Direct edges: `parts()` sees only the one-hop targets.
        assert!(en.parts(car).contains(&engine));
        assert!(en.parts(car).contains(&wheel));
        assert!(
            !en.parts(car).contains(&ring),
            "ring is 3 hops from car — not a direct edge"
        );

        // `parts_reach` follows the full transitive chain car -> engine ->
        // piston -> ring (Casati & Varzi 1999: a part of a part is a part).
        assert!(en.parts_reach(car, engine), "1 hop");
        assert!(en.parts_reach(car, piston), "2 hops");
        assert!(
            en.parts_reach(car, ring),
            "3 hops — the link parts() misses"
        );
        assert!(
            en.parts_reach(engine, ring),
            "2 hops from the intermediate node"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn parts_reach_is_irreflexive_and_directional() {
        let en = fixture();
        let car = en.lookup("car")[0];
        let ring = en.lookup("ring")[0];
        assert!(
            !en.parts_reach(car, car),
            "Parthood is Irreflexive — no self-part witness"
        );
        assert!(
            !en.parts_reach(ring, car),
            "part-of is directional — the ring does not have the car as a part"
        );
    }
}
