//! `ComposedReasoner` — the embedded English model COMPOSED with zero or more
//! loaded `.prx` ontologies, presented as ONE [`LexicalReasoner`].
//!
//! This is the runtime convergence point for the demo: a chat that "consults a
//! loaded corpus, understood through English". The embedded [`English`] model is
//! the always-present substrate; each [`RuntimeOntology`] loaded from a `.prx`
//! is GROUNDED into the same lexical surface via the Lemon functor
//! `F: OntologyConcepts → Lexicon(English)` (`lemon::lexicon`): every loaded
//! node's surface form becomes a lexical entry whose `reference` is the typed
//! [`ConceptRef`]`{ontology, name}`. A word then resolves through the UNION of
//! the English lexicon and the grounded loaded entries — so "what is X" answers
//! from the loaded gloss when X is loaded, and abstains exactly as the embedded
//! model already does when it is not.
//!
//! # The typed join key — never `String ==`
//!
//! English's concept world is keyed by [`ConceptId`] (a `Reference<4>` index
//! into WordNet synsets); a loaded concept's identity is the typed
//! [`ConceptRef`]`{ontology: OntologyName, name}`. These are two different
//! identity universes. They are bridged by a typed sum, [`GroundedConcept`]:
//!
//! ```text
//! GroundedConcept = English(ConceptId) | Loaded(ConceptRef)
//! ```
//!
//! The [`LexicalReasoner`] surface is keyed on `ConceptId`, so each loaded
//! `ConceptRef` is assigned a `ConceptId` in a range DISJOINT from English's
//! (offset above `english.concept_count()`). That `ConceptId` is an opaque
//! handle; its MEANING is recovered by decoding it back into a
//! [`GroundedConcept`] through a typed table ([`ComposedReasoner::decode`]), never by
//! comparing names as strings. Taxonomy over loaded concepts is answered from
//! the loaded ontology's MATERIALIZED closure
//! ([`RuntimeOntology::reachable_from`] / [`RuntimeOntology::is_a`]) — never a
//! query-time BFS, never `String ==`.
//!
//! Literature:
//! - McCrae et al. (2017) *The OntoLex-Lemon Model* — the lexicon-ontology
//!   interface: a `LexicalEntry`'s `Form` carries the surface, its `Sense`'s
//!   `reference` points at the ontology concept. The grounding here IS that
//!   functor applied to a loaded `.prx`.
//! - Reiter (1978) *On Closed World Data Bases* — the loaded vertex is
//!   open-world (`ConceptRef`, not a closed enum), which is why it cannot share
//!   English's finite `ConceptId` space without an explicit disjoint offset.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use hashbrown::HashMap;

use pr4xis_runtime::ontology::{ConceptRef, RuntimeOntology, subsumption_kind};

use crate::cognitive::linguistics::english::{Concept, ConceptId, English, LexicalReasoner};
use crate::cognitive::linguistics::lemon::lexicon::Lexicon;
use crate::social::software::markup::xml::lmf::ontology::LmfPos;

/// The typed join key bridging the two identity universes the composed reasoner
/// spans: English's WordNet [`ConceptId`] and a loaded `.prx`'s open-world
/// [`ConceptRef`]. A `ConceptId` handed back across the [`LexicalReasoner`]
/// surface decodes to exactly one of these (never a `String ==` on the name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundedConcept {
    /// A concept in the embedded English (WordNet) model.
    English(ConceptId),
    /// A concept materialized from a loaded `.prx` ontology — the typed
    /// `(ontology, name)` vertex.
    Loaded(ConceptRef),
}

/// The embedded English model composed with the loaded `.prx` ontologies,
/// presented as one [`LexicalReasoner`].
///
/// Construction GROUNDS every loaded node into the English lexicon (the Lemon
/// functor) and pre-folds the per-concept handles, so every query is a lookup —
/// the taxonomy answers are read from each [`RuntimeOntology`]'s materialized
/// closure.
#[derive(Debug)]
pub struct ComposedReasoner {
    /// The always-present embedded substrate.
    english: English,
    /// The loaded ontologies, in load order. A loaded `ConceptId`'s
    /// `value()` indexes `loaded_refs`; its source ontology is found here by
    /// matching `ConceptRef::ontology`.
    loaded: Vec<RuntimeOntology>,

    // --- grounded surface (built once at construction) ---
    /// The Lemon lexicon grounding every loaded node's surface form to its
    /// typed `ConceptRef` (`F: OntologyConcepts → Lexicon`). Held so the
    /// grounding is inspectable and so `lookup` is a pure union over it.
    lexicon: Lexicon,
    /// `surface → ConceptId`s : the UNION of English's `lookup(word)` and the
    /// grounded loaded entries whose surface matches. Returned by reference
    /// from [`LexicalReasoner::lookup`].
    surface_index: HashMap<String, Vec<ConceptId>>,
    /// The loaded concepts, indexed by `ConceptId::value() - base`. `base` is
    /// `english.concept_count()`; this keeps loaded ids disjoint from English.
    loaded_refs: Vec<ConceptRef>,
    /// Synthesized [`Concept`]s for the loaded vertices, so `concept(id)` can
    /// hand back a `&Concept` whose single definition IS the loaded gloss.
    /// Parallel to `loaded_refs`.
    loaded_concepts: Vec<Concept>,
    /// Pre-folded direct Subsumption parents per loaded `ConceptId` (read from
    /// each ontology's generating edges), so `parents`/`children` return a
    /// reference without per-call allocation.
    loaded_parents: HashMap<ConceptId, Vec<ConceptId>>,
    /// Pre-folded direct Subsumption children per loaded `ConceptId`.
    loaded_children: HashMap<ConceptId, Vec<ConceptId>>,
    /// Reverse of `loaded_refs`: typed `ConceptRef` → its disjoint `ConceptId`.
    /// Held so the materialized-closure answers (a `ConceptRef` set) can be
    /// mapped back to the `LexicalReasoner`'s `ConceptId` surface without a
    /// linear scan — the loaded-side `ancestors`/`common_ancestor`/
    /// `ancestor_chain` read each ontology's closure and re-key through this map.
    loaded_ids: BTreeMap<ConceptRef, ConceptId>,
    /// The id base above which all ids are loaded (== `english.concept_count()`).
    base: u64,
}

impl ComposedReasoner {
    /// Compose the embedded `english` model with the `loaded` ontologies,
    /// grounding every loaded node into the English lexicon via the Lemon
    /// functor and pre-folding the per-concept handles.
    pub fn new(english: English, loaded: Vec<RuntimeOntology>) -> Self {
        let base = english.concept_count() as u64;

        let mut lexicon = Lexicon::new("en");
        let mut loaded_refs: Vec<ConceptRef> = Vec::new();
        let mut loaded_ids: BTreeMap<ConceptRef, ConceptId> = BTreeMap::new();
        let mut loaded_concepts: Vec<Concept> = Vec::new();
        let mut surface_index: HashMap<String, Vec<ConceptId>> = HashMap::new();

        // 1. Seed the surface index with the embedded English lexicon. We copy
        //    rather than borrow so the union slice can be returned by reference.
        for word in english_surface_forms(&english) {
            let ids = english.lookup(&word).to_vec();
            if !ids.is_empty() {
                surface_index.entry(word).or_default().extend(ids);
            }
        }

        // 2. Ground each loaded ontology's nodes into the lexicon and the
        //    union index. Each node becomes:
        //      - a Lemon LexicalEntry (surface → typed ConceptRef), and
        //      - a synthesized English-shaped Concept carrying its gloss,
        //        addressable by a disjoint ConceptId.
        for onto in &loaded {
            let ontology_name = onto.id().as_str().to_string();
            for node in &onto.archive().nodes {
                let cref = ConceptRef::new(onto.id().clone(), node.name.clone());
                let id = ConceptId::new(base + loaded_refs.len() as u64);

                // The Lemon functor F: surface form → ConceptRef. The surface
                // people type is the lowercased node name; the reference keeps
                // the ontology's canonical name.
                let surface = node.name.to_lowercase();
                lexicon.add_entry(surface.clone(), ontology_name.clone(), node.name.clone());

                // Union into the lookup surface (disjoint id appended).
                surface_index.entry(surface.clone()).or_default().push(id);

                // The synthesized Concept carries the loaded gloss as its
                // definition, read straight from the materialized ontology
                // (`RuntimeOntology::lexical`) — this is what `define_word`
                // reads back as the answer.
                let gloss = onto.lexical(&cref).map(|g| g.to_string());
                loaded_concepts.push(Concept {
                    id,
                    original_id: node.name.clone(),
                    pos: LmfPos::Noun,
                    lemmas: alloc::vec![node.name.clone()],
                    definitions: gloss.into_iter().collect(),
                    examples: Vec::new(),
                });

                loaded_ids.insert(cref.clone(), id);
                loaded_refs.push(cref);
            }
        }

        // 3. Pre-fold the direct Subsumption parents/children per loaded id,
        //    from each ontology's GENERATING edges (not the closure — these are
        //    the direct is-a links the English `parents`/`children` mirror).
        let mut loaded_parents: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        let mut loaded_children: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        for onto in &loaded {
            for node in &onto.archive().nodes {
                let cref = ConceptRef::new(onto.id().clone(), node.name.clone());
                let Some(&child_id) = loaded_ids.get(&cref) else {
                    continue;
                };
                for edge in onto.morphisms_from(&cref) {
                    // morphisms_from now yields edges of ALL kinds; keep only the
                    // Subsumption (is-a) generators for the taxonomy build.
                    if edge.kind != subsumption_kind() {
                        continue;
                    }
                    if let Some(&parent_id) = loaded_ids.get(&edge.target) {
                        loaded_parents.entry(child_id).or_default().push(parent_id);
                        loaded_children.entry(parent_id).or_default().push(child_id);
                    }
                }
            }
        }

        // `loaded_ids` (ConceptRef → id) is retained as reasoner state so the
        // loaded-side closure answers — a set of `ConceptRef`s read off each
        // ontology's MATERIALIZED Subsumption closure — can be re-keyed back to
        // the `LexicalReasoner`'s `ConceptId` surface without a linear scan.
        Self {
            english,
            loaded,
            lexicon,
            surface_index,
            loaded_refs,
            loaded_concepts,
            loaded_parents,
            loaded_children,
            loaded_ids,
            base,
        }
    }

    /// The embedded English substrate (the pipeline's linguistic ground).
    pub fn english(&self) -> &English {
        &self.english
    }

    /// The Lemon lexicon grounding the loaded ontologies (inspectable for tests
    /// and for the self-model catalog).
    pub fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }

    /// The loaded ontologies, in load order.
    pub fn loaded(&self) -> &[RuntimeOntology] {
        &self.loaded
    }

    /// Decode a `ConceptId` back into the typed [`GroundedConcept`] join key —
    /// the structural recovery of which universe the id belongs to. An id below
    /// the base is English; otherwise it indexes `loaded_refs`. Returns `None`
    /// for an id that is neither (out of range).
    pub fn decode(&self, id: ConceptId) -> Option<GroundedConcept> {
        let v = id.value();
        if v < self.base {
            Some(GroundedConcept::English(id))
        } else {
            self.loaded_refs
                .get((v - self.base) as usize)
                .cloned()
                .map(GroundedConcept::Loaded)
        }
    }

    /// The loaded ontology that owns `cref`, by `OntologyName` identity.
    fn ontology_of(&self, cref: &ConceptRef) -> Option<&RuntimeOntology> {
        self.loaded.iter().find(|o| o.id() == &cref.ontology)
    }
}

/// Every surface form English can resolve — its WordNet words plus its function
/// words. Used to seed the union lookup index.
fn english_surface_forms(english: &English) -> Vec<String> {
    use crate::cognitive::linguistics::language::Language;
    english
        .known_words()
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

impl LexicalReasoner for ComposedReasoner {
    fn lookup(&self, word: &str) -> &[ConceptId] {
        self.surface_index
            .get(word)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    fn concept(&self, id: ConceptId) -> Option<&Concept> {
        match self.decode(id)? {
            GroundedConcept::English(cid) => self.english.concept(cid),
            GroundedConcept::Loaded(_) => {
                // The synthesized Concept lives at the disjoint index.
                self.loaded_concepts.get((id.value() - self.base) as usize)
            }
        }
    }

    fn concept_by_synset(&self, synset_id: &str) -> Option<&Concept> {
        // Synset ids are an English-only addressing scheme; loaded concepts are
        // addressed by ConceptRef, not synset id. Delegate to English.
        self.english.concept_by_synset(synset_id)
    }

    fn parents(&self, id: ConceptId) -> &[ConceptId] {
        match self.decode(id) {
            Some(GroundedConcept::English(cid)) => self.english.parents(cid),
            Some(GroundedConcept::Loaded(_)) => self
                .loaded_parents
                .get(&id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]),
            None => &[],
        }
    }

    fn children(&self, id: ConceptId) -> &[ConceptId] {
        match self.decode(id) {
            Some(GroundedConcept::English(cid)) => self.english.children(cid),
            Some(GroundedConcept::Loaded(_)) => self
                .loaded_children
                .get(&id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]),
            None => &[],
        }
    }

    fn is_a(&self, child: ConceptId, ancestor: ConceptId) -> bool {
        match (self.decode(child), self.decode(ancestor)) {
            // Both English: the embedded taxonomy answers.
            (Some(GroundedConcept::English(c)), Some(GroundedConcept::English(a))) => {
                self.english.is_a(c, a)
            }
            // Both loaded: the answer is membership in the MATERIALIZED
            // Subsumption closure of the owning ontology — never a BFS here.
            (Some(GroundedConcept::Loaded(c)), Some(GroundedConcept::Loaded(a))) => {
                if c == a {
                    return true;
                }
                // Cross-ontology subsumption is not asserted in the
                // single-ontology demo; same-ontology is the closure lookup.
                self.ontology_of(&c)
                    .map(|onto| onto.closure().reaches(&c, &a, subsumption_kind()))
                    .unwrap_or(false)
            }
            // Mixed English/loaded: no cross-universe subsumption edges exist in
            // this composition, so the relation does not hold (honest false, not
            // a guess).
            _ => false,
        }
    }

    fn ancestors(&self, id: ConceptId) -> Vec<ConceptId> {
        match self.decode(id) {
            // English: delegate to English's materialized hypernym closure.
            Some(GroundedConcept::English(cid)) => self.english.ancestors(cid),
            // Loaded: the reflexive Subsumption image over the owning ontology's
            // MATERIALIZED closure, nearest-first by is-a distance, re-keyed back
            // to the `ConceptId` surface. A lookup over the materialized set,
            // never a BFS.
            Some(GroundedConcept::Loaded(cref)) => {
                let Some(onto) = self.ontology_of(&cref) else {
                    return alloc::vec![id];
                };
                let mut image: Vec<(ConceptId, u32)> = alloc::vec![(id, 0)];
                for (anc_ref, dist) in onto.closure().subsumption_image(&cref) {
                    if let Some(&anc_id) = self.loaded_ids.get(&anc_ref) {
                        image.push((anc_id, dist));
                    }
                }
                image.sort_unstable_by(|(a, da), (b, db)| {
                    da.cmp(db).then_with(|| a.value().cmp(&b.value()))
                });
                image.into_iter().map(|(v, _)| v).collect()
            }
            None => Vec::new(),
        }
    }

    fn common_ancestor(&self, a: ConceptId, b: ConceptId) -> Option<ConceptId> {
        match (self.decode(a), self.decode(b)) {
            // Both English: English's closure lattice-meet.
            (Some(GroundedConcept::English(ea)), Some(GroundedConcept::English(eb))) => {
                self.english.common_ancestor(ea, eb)
            }
            // Both loaded in the same ontology: the lattice-meet over that
            // ontology's MATERIALIZED Subsumption closure, re-keyed to a
            // `ConceptId`. Never a hand-BFS.
            (Some(GroundedConcept::Loaded(ra)), Some(GroundedConcept::Loaded(rb))) => {
                let onto = self.ontology_of(&ra)?;
                if self.ontology_of(&rb)?.id() != onto.id() {
                    return None;
                }
                let meet = onto.closure().subsumption_meet(&ra, &rb)?;
                self.loaded_ids.get(&meet).copied()
            }
            // Mixed: no shared hypernym across the two disjoint universes.
            _ => None,
        }
    }

    fn ancestor_chain(&self, child: ConceptId, ancestor: ConceptId) -> Option<Vec<ConceptId>> {
        match (self.decode(child), self.decode(ancestor)) {
            (Some(GroundedConcept::English(c)), Some(GroundedConcept::English(a))) => {
                self.english.ancestor_chain(c, a)
            }
            (Some(GroundedConcept::Loaded(c)), Some(GroundedConcept::Loaded(a))) => {
                let onto = self.ontology_of(&c)?;
                let chain_refs = onto.closure().subsumption_chain(&c, &a)?;
                // Re-key the ordered ConceptRef chain to ConceptIds.
                let chain: Vec<ConceptId> = chain_refs
                    .into_iter()
                    .filter_map(|r| self.loaded_ids.get(&r).copied())
                    .collect();
                Some(chain)
            }
            _ => None,
        }
    }

    fn concept_count(&self) -> usize {
        self.english.concept_count() + self.loaded_concepts.len()
    }
}
