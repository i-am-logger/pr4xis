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

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use hashbrown::HashMap;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::definition::CANONICAL_FORM_REL;
use pr4xis_runtime::lens::archive_lens::{archived_grounded, archived_local_name};
use pr4xis_runtime::ontology::{ConceptRef, RuntimeOntology, subsumption_kind};

use crate::cognitive::linguistics::english::bridge::{
    ENGLISH_ONTOLOGY, FORM_KIND, english_synset_atoms,
};
use crate::cognitive::linguistics::english::{
    Concept, ConceptId, ConceptView, English, LexicalReasoner,
};
use crate::cognitive::linguistics::interner::{Interner, Symbol};
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
    /// The always-present embedded substrate — BORROWED, not owned. The single
    /// English instance lives once (the wasm-side `english_static()` / native
    /// `english_loaded()` `OnceLock`, or a test's `English::sample_static()`); the
    /// reasoner and the no-composed chat path reference that ONE instance rather
    /// than each holding an owned ~73 MiB copy. Sound because `English` is `Sync`
    /// and the shared instance is genuinely `'static`.
    english: &'static English,
    /// The loaded ontologies, in load order — SHARED (`Rc`), not deep-copied. The
    /// same `RuntimeOntology` instances the owner (`Pr4xis`) holds; building the
    /// reasoner clones the `Rc` handles (a refcount bump), never the ~39 MiB
    /// archive/closure buffers. A loaded `ConceptId`'s `value()` indexes
    /// `loaded_refs`; its source ontology is found here by matching
    /// `ConceptRef::ontology`.
    loaded: Vec<Rc<RuntimeOntology>>,

    // --- grounded surface (built once at construction) ---
    /// The Lemon lexicon grounding every loaded node's surface form to its
    /// typed `ConceptRef` (`F: OntologyConcepts → Lexicon`). Held so the
    /// grounding is inspectable and so `lookup` is a pure union over it.
    lexicon: Lexicon,
    /// The interner holding every surface's bytes ONCE, keyed by [`Symbol`]. It
    /// is the shared arena for English's surfaces AND every loaded ontology's —
    /// so `surface_index` keys on 4-byte handles instead of re-owning a `String`
    /// per surface (the strings the graph already owns). Held so the query path
    /// can intern a lookup word back to its handle (`interner.get`), and so a
    /// handle resolves to its surface for the `max_surface_words` bookkeeping.
    interner: Interner,
    /// interned `surface → ConceptId`s : the UNION of English's `lookup(word)`
    /// and the grounded loaded entries whose surface matches, keyed by the
    /// surface's interned [`Symbol`] (see `interner`). Returned by reference
    /// from [`LexicalReasoner::lookup`], which interns the query word first.
    surface_index: HashMap<Symbol, Vec<ConceptId>>,
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
    /// The widest surface (in whitespace-separated words) in `surface_index` —
    /// the window the chat's multi-token recognizer scans. Cached once at
    /// construction so `max_surface_words` is O(1); `1` when every surface is a
    /// single word (the recognizer then no-ops).
    max_surface_words: usize,
    /// The loaded surface→relation-kind map (today `"part of"` → the Parthood
    /// [`ConceptRef`]; Subsumption is the un-lexicalized copula default), from the
    /// committed `relation_lexicon.prx`. Held APART from `loaded` (it is reasoning
    /// vocabulary, not a queryable corpus), so `loaded_ontology_count` stays
    /// honest. Read by [`relation_for_surface`](LexicalReasoner::relation_for_surface)
    /// to lower a relational question's predicate to the kind its closure is
    /// keyed on.
    relation_surface_index: BTreeMap<String, ConceptRef>,
    /// The REFLEXIVE relation kinds — DERIVED from the typed Relations ontology's
    /// `(R, Reflexive, HasProperty)` declarations (Subsumption, Equivalence,
    /// Similarity — NOT Parthood, which is `Irreflexive`), not a hardcoded list. A
    /// relational query `reaches(c, a, kind)` answers `c == a` as `true` only when
    /// `kind` is in this set, so "is X part of X" is `false` (strict, per the data)
    /// while "is X a X" stays `true`.
    reflexive_kinds: BTreeSet<ConceptRef>,

    /// The TYPE-GROUNDING atom index, built ONCE at construction — for each loaded
    /// ontology that is a grounding TARGET (some loaded node carries a cross-
    /// ontology [`Grounded`](pr4xis_runtime::definition::EdgeTarget::Grounded) edge INTO it, e.g. `LegalSources`,
    /// the target of the USC section→`Statute` typing), a map from each of its
    /// nodes' content addresses to that node's name. A cross-ontology `reaches`
    /// resolves a grounded edge's foreign `atom` to the peer concept's NAME by an
    /// O(log n) lookup here — the resolution `AtomResolver::resolve` performed, but
    /// the index is computed once at construction, NOT rebuilt per query. Keyed by
    /// ontology name; empty when no loaded ontology grounds into a peer. Data-driven
    /// off the loaded archives' edges, never a hardcoded target list. The pin gate
    /// is trivially satisfied (the index is derived from the loaded archives
    /// themselves), so the meaningful fail-closed leg is atom PRESENCE — a typing
    /// edge into an ontology the system does not hold resolves to nothing.
    grounding_atoms: BTreeMap<OntologyName, BTreeMap<ContentAddress, String>>,

    /// The INTO-ENGLISH atom index, built ONCE at construction — for each atom some
    /// loaded node grounds into `english_wordnet` (a DECLARED into-English
    /// InstanceFunctor's typing edge, e.g. `Canine ↦ s-dog`), the synset's
    /// `original_id`. A cross-universe `reaches` from a LOADED node to an ENGLISH
    /// concept resolves the grounded atom to its synset name here, then continues in
    /// English's archived hypernym closure (see
    /// [`reaches_into_english`](Self::reaches_into_english)).
    ///
    /// GATED on some loaded edge actually grounding into `english_wordnet`
    /// (`ENGLISH_ONTOLOGY` ∈ the grounded-target set) — empty otherwise, so a load
    /// with no into-English functor pays nothing. DERIVED from
    /// [`english_synset_atoms`] (the `project_archive` addressing), retaining ONLY
    /// the edge-targeted atoms, so the resident index is BOUNDED by grounded-target
    /// count — single-MiB, never the ~107k-synset table. English is NEVER a loaded
    /// ontology: this index is the ONLY English-side state a cross-universe query
    /// consults, and it points into the borrowed `english`'s archived taxonomy (W1),
    /// adding zero new materialization.
    english_atoms: BTreeMap<ContentAddress, String>,
}

impl ComposedReasoner {
    /// Compose the embedded `english` model with the `loaded` ontologies,
    /// grounding every loaded node into the English lexicon via the Lemon
    /// functor and pre-folding the per-concept handles.
    pub fn new(english: &'static English, loaded: Vec<Rc<RuntimeOntology>>) -> Self {
        let base = english.concept_count() as u64;

        let mut lexicon = Lexicon::new("en");
        let mut loaded_refs: Vec<ConceptRef> = Vec::new();
        let mut loaded_ids: BTreeMap<ConceptRef, ConceptId> = BTreeMap::new();
        let mut loaded_concepts: Vec<Concept> = Vec::new();
        // The shared surface arena — English's surfaces and every loaded
        // ontology's are interned into ONE interner, so `surface_index` keys on
        // a 4-byte `Symbol` handle rather than a fresh owned `String` per surface.
        let mut interner = Interner::new();
        let mut surface_index: HashMap<Symbol, Vec<ConceptId>> = HashMap::new();

        // 1. Seed the surface index with the embedded English lexicon. We copy
        //    the ConceptIds (so the union slice can be returned by reference) but
        //    intern the surface — its handle keys the union, not a copied String.
        for word in english_surface_forms(english) {
            let ids = english.lookup(&word).to_vec();
            if !ids.is_empty() {
                let symbol = interner.intern(&word);
                surface_index.entry(symbol).or_default().extend(ids);
            }
        }

        // 2. Ground each loaded ontology's nodes into the lexicon and the
        //    union index. Each node becomes:
        //      - a Lemon LexicalEntry (surface → typed ConceptRef), and
        //      - a synthesized English-shaped Concept carrying its gloss,
        //        addressable by a disjoint ConceptId.
        for onto in &loaded {
            let ontology_name = onto.id().as_str().to_string();
            // The `ontolex:Form` atoms in this archive — their `writtenRep` NAMES
            // are natural-language SURFACES (a heading / label / citation), the
            // Frege *Sinn* distinct from a node's URN/IRI *Bedeutung*. A concept's
            // queryable surfaces are the Form atoms it points at (the §9
            // lexicalization channel), detected by FORM-target-ness — a data
            // property of the loaded archive, NEVER a hardcoded role allow-list.
            let form_names: BTreeSet<&str> = onto
                .archive()
                .nodes
                .iter()
                .filter(|n| n.kind == FORM_KIND)
                .map(|n| n.name.as_str())
                .collect();

            for node in onto.archive().nodes.iter() {
                // A Form atom is a SURFACE, not a concept — it gets no synthesized
                // Concept and no id; it is indexed (below) as a surface of the
                // concept that denotes it.
                if node.kind == FORM_KIND {
                    continue;
                }
                let cref = ConceptRef::new(onto.id().clone(), node.name.to_string());
                let id = ConceptId::new(base + loaded_refs.len() as u64);

                // The Lemon functor F: surface form → ConceptRef. The node's OWN
                // name is kept as a surface ADDITIVELY (a compiled ontology's node
                // name IS a natural word; the URN/IRI case is covered by its Form
                // atoms below, so this stays until every producer mints Forms).
                let surface = node.name.to_lowercase();
                lexicon.add_entry(
                    surface.clone(),
                    ontology_name.clone(),
                    node.name.to_string(),
                );

                // Union into the lookup surface (disjoint id appended), keyed by
                // the surface's interned handle rather than a copied String.
                let symbol = interner.intern(&surface);
                surface_index.entry(symbol).or_default().push(id);

                // Each Form atom this concept denotes (its `writtenRep`) is a
                // queryable surface of the concept — one *Bedeutung*, many *Sinne*.
                for edge in node.edges.iter() {
                    // Archived edges are `ArchivedTuple2(role, target)`.
                    if let Some(form) = archived_local_name(&edge.1)
                        && form_names.contains(form)
                    {
                        let form_surface = form.to_lowercase();
                        lexicon.add_entry(
                            form_surface.clone(),
                            ontology_name.clone(),
                            node.name.to_string(),
                        );
                        let form_symbol = interner.intern(&form_surface);
                        surface_index.entry(form_symbol).or_default().push(id);
                    }
                }

                // The PRINTED lemma is the concept's `canonicalForm` Form surface —
                // its ontolex:Form *writtenRep*, the natural label ("legal document",
                // "dormant fault") — NOT its Rust identifier (`node.name`, kept below
                // as the never-printed `original_id`). Frege: identity addresses
                // (`node.name`), canonicalForm generates (the lemma). Fall back to the
                // node name when the concept mints no canonicalForm (its label already
                // equals its identifier case-insensitively, e.g. "Statute" — emit skips
                // the redundant Form there, and the node name IS the natural word).
                // GENERATION-only: every Form is still indexed above for LOOKUP, so
                // this changes what prints, never what resolves.
                let canonical_lemma = node
                    .edges
                    .iter()
                    .find(|edge| {
                        // Archived edges are `ArchivedTuple2(role, target)`.
                        edge.0 == CANONICAL_FORM_REL
                            && archived_local_name(&edge.1).is_some_and(|f| form_names.contains(f))
                    })
                    .and_then(|edge| archived_local_name(&edge.1))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| node.name.to_string());

                // The synthesized Concept carries the loaded gloss as its
                // definition, read straight from the materialized ontology
                // (`RuntimeOntology::lexical`) — this is what `define_word`
                // reads back as the answer.
                let gloss = onto.lexical(&cref).map(|g| g.to_string());
                loaded_concepts.push(Concept {
                    id,
                    original_id: node.name.to_string(),
                    pos: LmfPos::Noun,
                    lemmas: alloc::vec![canonical_lemma],
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
        // The is-a kind, built ONCE — `subsumption_kind()` allocates a `String` +
        // clones the vocab `OntologyName`, so hoist it out of the per-edge filter.
        let subsumption = subsumption_kind();
        for onto in &loaded {
            for node in onto.archive().nodes.iter() {
                let cref = ConceptRef::new(onto.id().clone(), node.name.to_string());
                let Some(&child_id) = loaded_ids.get(&cref) else {
                    continue;
                };
                for edge in onto.morphisms_from(&cref) {
                    // morphisms_from now yields edges of ALL kinds; keep only the
                    // Subsumption (is-a) generators for the taxonomy build.
                    if edge.kind != subsumption {
                        continue;
                    }
                    if let Some(&parent_id) = loaded_ids.get(&edge.target) {
                        loaded_parents.entry(child_id).or_default().push(parent_id);
                        loaded_children.entry(parent_id).or_default().push(child_id);
                    }
                }
            }
        }

        // The surface→relation map (the loaded relation lexicon) — built once,
        // held apart from `loaded`. Every composed reasoner can resolve a
        // relational question's predicate ("part of" → Parthood), intrinsically,
        // the way the runtime closure intrinsically folds every transitive kind.
        let relation_surface_index =
            crate::cognitive::linguistics::relation_lexicon::relation_surface_index();

        // TYPE-GROUNDING resolver inputs. Scan every loaded node's edges for a
        // cross-ontology `Grounded` target and collect the ontologies they point
        // INTO — the peers a cross-ontology `reaches` must resolve against (e.g.
        // LegalSources, the target of the USC section→Statute typing). Data-driven:
        // the target set is read off the loaded archives, never a hardcoded list.
        let mut target_names: BTreeSet<String> = BTreeSet::new();
        // The atoms some loaded node grounds INTO english_wordnet (a declared
        // into-English InstanceFunctor's typing edges) — collected in the SAME edge
        // scan, so the into-English index is bounded by these, never the synset
        // table. Empty unless a loaded `.prx` carries an into-English functor.
        let mut english_targeted: BTreeSet<ContentAddress> = BTreeSet::new();
        for onto in &loaded {
            for node in onto.archive().nodes.iter() {
                for edge in node.edges.iter() {
                    if let Some((ont, atom)) = archived_grounded(&edge.1) {
                        target_names.insert(ont.to_string());
                        if ont == ENGLISH_ONTOLOGY {
                            english_targeted.insert(atom);
                        }
                    }
                }
            }
        }
        // For each loaded ontology that IS such a target, build its atom index ONCE
        // (each node's content address → its name) — the resolution the per-query
        // `AtomResolver` did, hoisted to construction so `cross_reaches` is a pure
        // lookup. A loaded ontology no edge grounds into (e.g. USC itself) is NOT
        // indexed — the index stays small (LegalSources is nine nodes).
        // Keyed by the typed `OntologyName` (in-memory), so `cross_reaches` looks a
        // GroundedEdge's `ontology` up directly — never `OntologyName::new(clone)`.
        let mut grounding_atoms: BTreeMap<OntologyName, BTreeMap<ContentAddress, String>> =
            BTreeMap::new();
        for onto in &loaded {
            // `target_names` holds the wire ontology-name strings read off the
            // grounded edges; compare via the id's `&str`.
            if !target_names.contains(onto.id().as_str()) {
                continue;
            }
            if let Ok(archive) = onto.to_owned_archive() {
                let mut index: BTreeMap<ContentAddress, String> = BTreeMap::new();
                for node in &archive.nodes {
                    if let Ok(addr) = node.address() {
                        index.insert(addr, node.name.clone());
                    }
                }
                grounding_atoms.insert(onto.id().clone(), index);
            }
        }

        // The INTO-ENGLISH atom index — GATED on some loaded edge grounding into
        // english_wordnet, DERIVED from the coupling-free `english_synset_atoms`
        // (project_archive addressing), retaining ONLY the edge-targeted synset
        // atoms. The full synset→address map is a transient dropped here; the
        // resident index holds one entry per grounded target (single-MiB), never the
        // ~107k-synset table. English is never a loaded ontology, so this is the only
        // English-side state a cross-universe query reads.
        let english_atoms: BTreeMap<ContentAddress, String> = if english_targeted.is_empty() {
            BTreeMap::new()
        } else {
            english_synset_atoms(english)
                .into_iter()
                .filter(|(addr, _)| english_targeted.contains(addr))
                .collect()
        };

        // `loaded_ids` (ConceptRef → id) is retained as reasoner state so the
        // loaded-side closure answers — a set of `ConceptRef`s read off each
        // ontology's MATERIALIZED Subsumption closure — can be re-keyed back to
        // the `LexicalReasoner`'s `ConceptId` surface without a linear scan.
        // The widest surface the recognizer must scan for — the max word count
        // over every key: English collocations + loaded multi-word surfaces AND
        // the relational surfaces ("part of"), so the recognizer's window reaches
        // a relation phrase. 1 when all surfaces are single words (then no-op).
        let max_surface_words = surface_index
            .keys()
            .map(|&symbol| interner.resolve(symbol))
            .chain(relation_surface_index.keys().map(String::as_str))
            .map(|k| k.split_whitespace().count())
            .max()
            .unwrap_or(1)
            .max(1);

        Self {
            english,
            loaded,
            lexicon,
            interner,
            surface_index,
            loaded_refs,
            loaded_concepts,
            loaded_parents,
            loaded_children,
            loaded_ids,
            base,
            max_surface_words,
            relation_surface_index,
            // The reflexive relation kinds, DERIVED from the typed Relations
            // ontology's `(R, Reflexive, HasProperty)` edges — so the `reaches`
            // `c == a` short-circuit consults the loaded data, not a hardcoded list.
            reflexive_kinds: crate::formal::relations::ontology::reflexive_relation_kinds(),
            grounding_atoms,
            english_atoms,
        }
    }

    /// The embedded English substrate (the pipeline's linguistic ground) — the
    /// single shared instance the reasoner borrows.
    pub fn english(&self) -> &'static English {
        self.english
    }

    /// The Lemon lexicon grounding the loaded ontologies (inspectable for tests
    /// and for the self-model catalog).
    pub fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }

    /// The loaded ontologies, in load order (the shared `Rc` handles).
    pub fn loaded(&self) -> &[Rc<RuntimeOntology>] {
        &self.loaded
    }

    /// The number of entries in the INTO-ENGLISH atom index — one per synset a
    /// loaded node grounds into `english_wordnet` (bounded by grounded-target count,
    /// never the ~107k synset table). Exposed so the resident-memory gate can report
    /// that the into-English path's resident index is single-MiB, not a projection
    /// of the whole WordNet taxonomy.
    pub fn english_atom_count(&self) -> usize {
        self.english_atoms.len()
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
        self.loaded
            .iter()
            .find(|o| o.id() == &cref.ontology)
            .map(|o| o.as_ref())
    }

    /// CROSS-ONTOLOGY reachability along `kind`: `c` and `a` live in DIFFERENT
    /// loaded ontologies, bridged by a `Grounded` TYPE edge `c` carries into `a`'s
    /// ontology (the USC section→`legal_sources:Statute` typing minted by the
    /// `usc_legal_sources_functor` grounding lens). It reads `c`'s grounded edges
    /// of the queried `kind`, resolves each foreign atom to the peer's
    /// [`ConceptRef`] via the precomputed
    /// [`grounding_atoms`](Self::grounding_atoms) index, and CONTINUES the query
    /// inside the peer ontology's own materialized closure — so
    /// `reaches(section, Statute)` is the direct typing and `reaches(section, law)`
    /// is that typing THEN the LegalSources `Statute ⊑ … ⊑ LegalSource` closure.
    ///
    /// Fail-closed: an unresolvable atom (the target ontology not loaded, a
    /// version skew, or an absent atom) contributes NO path — an honest false,
    /// never a guess. This is the resolve half of "integration via a functor": the
    /// type link exists only when BOTH the grounding functor minted the edge AND
    /// the target ontology is loaded to resolve it.
    fn cross_reaches(&self, c: &ConceptRef, a: &ConceptRef, kind: &ConceptRef) -> bool {
        let Some(onto) = self.ontology_of(c) else {
            return false;
        };
        for g in onto.grounded_edges_from(c) {
            // Only a grounded edge asserting the QUERIED relation kind bridges (a
            // `denotes` lexical edge, say, does not answer an is-a query).
            if &g.kind != kind {
                continue;
            }
            // Resolve the foreign `atom` to the peer concept's NAME via the
            // precomputed atom index (built once at construction), fail-closed to no
            // path when the target ontology is not held or the atom is absent.
            let Some(name) = self
                .grounding_atoms
                .get(&g.ontology)
                .and_then(|index| index.get(&g.atom))
            else {
                continue;
            };
            // `g.ontology` is already the typed OntologyName — no re-wrap.
            let t = ConceptRef::new(g.ontology.clone(), name.clone());
            // The typing lands directly on `a`…
            if &t == a {
                return true;
            }
            // …or `a` is a supertype `t` reaches inside the peer ontology's closure.
            if let Some(peer) = self.ontology_of(a)
                && peer.id() == &g.ontology
                && peer.closure().reaches(&t, a, kind.clone())
            {
                return true;
            }
        }
        false
    }

    /// CROSS-UNIVERSE reachability from a LOADED node INTO English: `c` is a loaded
    /// `.prx` vertex that carries a DECLARED into-English typing edge (an
    /// into-English `InstanceFunctor`'s `kind ↦ synset` grounding, minted by
    /// [`ground_declared`](crate::formal::meta::grounding::ground_declared)), and
    /// `a` is an English (WordNet) [`ConceptId`]. The loaded node inherits English's
    /// taxonomy through that one declared edge: it reads `c`'s `Grounded` edges of
    /// the queried `kind` into `english_wordnet`, resolves each atom to its synset
    /// `original_id` via the precomputed [`english_atoms`](Self::english_atoms)
    /// index, maps that to English's `ConceptId`, and answers in English's own
    /// archived hypernym closure — `s == a || english.is_a(s, a)` (the W1 archived
    /// taxonomy, zero new materialization).
    ///
    /// This is DECLARED-TYPE grounding (`node_kind ↦ synset`), NOT surface
    /// auto-matching: a node that carries no into-English typing edge — even one
    /// whose gloss is verbatim an English animal word — resolves to NO path (Policy
    /// B / WSD is §9-forbidden and declined). Fail-closed at every rung: a wrong
    /// kind, a non-English target, an atom the index does not hold, or a synset
    /// English does not know each contributes nothing — an honest false, never a
    /// guess. The `&g.kind != kind` comparison is the VERBATIM `subsumption_kind()`
    /// comparison [`cross_reaches`](Self::cross_reaches) uses, not a re-derived
    /// kind string.
    fn reaches_into_english(&self, c: &ConceptRef, a: ConceptId, kind: &ConceptRef) -> bool {
        let Some(onto) = self.ontology_of(c) else {
            return false;
        };
        for g in onto.grounded_edges_from(c) {
            // Only a grounded edge asserting the QUERIED kind into english_wordnet
            // types the node — a denotes/lexical edge, or an edge into another peer,
            // does not answer an is-a-into-English query.
            if &g.kind != kind || g.ontology.as_str() != ENGLISH_ONTOLOGY {
                continue;
            }
            // Resolve the foreign synset atom to its original_id, then to English's
            // ConceptId, fail-closed to no path when either lookup misses.
            let Some(original_id) = self.english_atoms.get(&g.atom) else {
                continue;
            };
            let Some(synset) = self.english.concept_by_synset(original_id) else {
                continue;
            };
            let s = synset.id();
            // The typing lands directly on `a`, or `a` is a hypernym `s` reaches in
            // English's archived taxonomy.
            if s == a || self.english.is_a(s, a) {
                return true;
            }
        }
        false
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
        // Resolve the query surface to its handle (non-mutating: an un-interned
        // word was never a key, so it resolves to nothing — the same answer the
        // String-keyed `get(word)` gave), then index the union by that handle.
        self.interner
            .get(word)
            .and_then(|symbol| self.surface_index.get(&symbol))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    fn max_surface_words(&self) -> usize {
        self.max_surface_words
    }

    fn concept(&self, id: ConceptId) -> Option<ConceptView<'_>> {
        match self.decode(id)? {
            GroundedConcept::English(cid) => self.english.concept(cid),
            GroundedConcept::Loaded(_) => {
                // The synthesized Concept lives at the disjoint index; it is an
                // OWNED `Concept`, viewed through the owned arm.
                self.loaded_concepts
                    .get((id.value() - self.base) as usize)
                    .map(ConceptView::Owned)
            }
        }
    }

    fn concept_by_synset(&self, synset_id: &str) -> Option<ConceptView<'_>> {
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
                if c.ontology == a.ontology {
                    // Same ontology: the closure lookup.
                    self.ontology_of(&c)
                        .map(|onto| onto.closure().reaches(&c, &a, subsumption_kind()))
                        .unwrap_or(false)
                } else {
                    // Different ontologies: follow a `Grounded` TYPE edge across the
                    // boundary (the USC section→legal_sources:Statute typing), then
                    // continue in the peer's Subsumption closure.
                    self.cross_reaches(&c, &a, &subsumption_kind())
                }
            }
            // Loaded child grounded INTO English: the loaded node carries a DECLARED
            // into-English typing edge (a `kind ↦ synset` InstanceFunctor), so it
            // inherits English's is-a chain — `is <node> an animal` answers through
            // WordNet's synset taxonomy. Directional: an English child never reaches
            // a loaded ancestor (no such edge exists), so that pairing stays false.
            (Some(GroundedConcept::Loaded(c)), Some(GroundedConcept::English(a))) => {
                self.reaches_into_english(&c, a, &subsumption_kind())
            }
            // English child / loaded ancestor, or any other mix: no cross-universe
            // subsumption edge exists in this composition (honest false, not a guess).
            _ => false,
        }
    }

    /// Relation-parametric reachability — the verbatim shape of [`is_a`] above
    /// with `subsumption_kind()` generalized to the `kind` parameter, read off
    /// the owning ontology's MATERIALIZED closure for THAT relation (the closure
    /// already folds every loaded transitive kind, Parthood included — this just
    /// reads the right one). `is_a` is now `reaches(.., subsumption_kind())`.
    ///
    /// [`is_a`]: Self::is_a
    fn reaches(&self, child: ConceptId, ancestor: ConceptId, kind: &ConceptRef) -> bool {
        match (self.decode(child), self.decode(ancestor)) {
            // Both loaded: membership in the MATERIALIZED `kind` closure of the
            // owning ontology (Subsumption, Parthood, … — whichever the question
            // names), never a BFS. Cross-ontology relations are not asserted.
            (Some(GroundedConcept::Loaded(c)), Some(GroundedConcept::Loaded(a))) => {
                // Reflexivity is per-relation, read from the loaded Relations data:
                // `c == a` holds for a reflexive kind (Subsumption — `x is_a x`) but
                // NOT an irreflexive one (Parthood — `x` is not a proper part of
                // itself). The closure is strict reachability, so reflexivity is
                // added here only for the kinds the ontology declares it for.
                if c == a {
                    return self.reflexive_kinds.contains(kind);
                }
                if c.ontology == a.ontology {
                    self.ontology_of(&c)
                        .map(|onto| onto.closure().reaches(&c, &a, kind.clone()))
                        .unwrap_or(false)
                } else {
                    // Cross-ontology: resolve `c`'s `Grounded` type edge into the
                    // peer ontology and continue the `kind` query there.
                    self.cross_reaches(&c, &a, kind)
                }
            }
            // Both English: the embedded taxonomy answers ONLY a Subsumption
            // query — it carries no other relation's closure (honest false).
            (Some(GroundedConcept::English(c)), Some(GroundedConcept::English(a)))
                if *kind == subsumption_kind() =>
            {
                self.english.is_a(c, a)
            }
            // Loaded child grounded INTO English along Subsumption (the into-English
            // functor's `denotes ↦ Subsumption` morphism): the loaded node inherits
            // English's is-a chain. Only Subsumption — the declared into-English
            // typing asserts no other kind, so a Parthood query into English is false.
            (Some(GroundedConcept::Loaded(c)), Some(GroundedConcept::English(a)))
                if *kind == subsumption_kind() =>
            {
                self.reaches_into_english(&c, a, kind)
            }
            // Mixed universes otherwise, or a non-Subsumption English query: no such
            // edge exists in this composition (honest false, not a guess).
            _ => false,
        }
    }

    /// Resolve a relational question's surface predicate to its typed relation
    /// kind through the loaded relation lexicon ("part of" → Parthood). `None`
    /// (the caller falls back to Subsumption) for an unknown surface.
    fn relation_for_surface(&self, surface: &str) -> Option<ConceptRef> {
        self.relation_surface_index.get(surface).cloned()
    }

    /// The loaded ontology a concept belongs to — `Some(name)` when the id decodes
    /// to a `Loaded` vertex (its `ConceptRef.ontology`), `None` for an English
    /// (substrate) concept. The provenance the answer path records as
    /// `reasoned_over`.
    fn ontology_of_concept(&self, id: ConceptId) -> Option<OntologyName> {
        match self.decode(id) {
            Some(GroundedConcept::Loaded(cref)) => Some(cref.ontology),
            _ => None,
        }
    }

    /// The loaded surface for a relation kind — the inverse of the relation lexicon
    /// (`"part of"` ↦ Parthood becomes Parthood ↦ `"part of"`), so a Parthood
    /// affirmation phrases as "X is part of Y". `None` for Subsumption (the is-a
    /// default) and any kind the lexicon does not carry.
    fn surface_for_relation(&self, kind: &ConceptRef) -> Option<String> {
        self.relation_surface_index
            .iter()
            .find(|(_, k)| *k == kind)
            .map(|(surface, _)| surface.clone())
    }

    /// True iff `surface` resolves to a LOADED concept (a `Loaded` vertex), not the
    /// embedded English substrate — so a single-word loaded entity can be typed NP
    /// without disturbing English function words (which never decode to `Loaded`).
    fn is_loaded_surface(&self, surface: &str) -> bool {
        self.lookup(surface)
            .iter()
            .any(|&id| matches!(self.decode(id), Some(GroundedConcept::Loaded(_))))
    }

    fn ancestors(&self, id: ConceptId) -> Vec<ConceptId> {
        match self.decode(id) {
            // English: delegate to English's materialized hypernym closure.
            Some(GroundedConcept::English(cid)) => self.english.ancestors(cid),
            // Loaded: the reflexive Subsumption image over the owning ontology's
            // MATERIALIZED closure, re-keyed back to the `ConceptId` surface. A
            // lookup over the materialized set, never a BFS.
            //
            // BEHAVIOR CHANGE (named, slice (c) of the reachability-kernel
            // unification): the image is sorted by the kernel's canonical
            // `(is-a distance, ConceptRef::Ord)` order BEFORE re-keying to
            // composed ids. It was formerly re-sorted AFTER re-keying by the
            // loaded `ConceptId.value()` — i.e. archive-POSITION order — which
            // made this surface disagree with `ancestor_chain` (which preserves
            // the closure's `(dist, ontology, name)` order) on one and the same
            // reasoner. Both now agree on the ONE kernel ordering.
            Some(GroundedConcept::Loaded(cref)) => {
                let Some(onto) = self.ontology_of(&cref) else {
                    return alloc::vec![id];
                };
                let mut image: Vec<(ConceptRef, u32)> = alloc::vec![(cref.clone(), 0)];
                image.extend(onto.closure().subsumption_image(&cref));
                image.sort_unstable_by(pr4xis::category::reach::graded_cmp);
                image
                    .into_iter()
                    .filter_map(|(anc_ref, _)| self.loaded_ids.get(&anc_ref).copied())
                    .collect()
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

    /// The ordered evidence chain along ANY relation `kind` — the relation-
    /// parametric `ancestor_chain`, reading the loaded ontology's materialized
    /// closure for that kind (Parthood's part-of chain, etc.), re-keyed to ids.
    fn relation_chain(
        &self,
        child: ConceptId,
        ancestor: ConceptId,
        kind: &ConceptRef,
    ) -> Option<Vec<ConceptId>> {
        match (self.decode(child), self.decode(ancestor)) {
            // Both loaded: the ordered chain along `kind` over the owning ontology's
            // materialized closure, re-keyed to ConceptIds.
            (Some(GroundedConcept::Loaded(c)), Some(GroundedConcept::Loaded(a))) => {
                let onto = self.ontology_of(&c)?;
                let chain_refs = onto.closure().chain(&c, &a, kind)?;
                Some(
                    chain_refs
                        .into_iter()
                        .filter_map(|r| self.loaded_ids.get(&r).copied())
                        .collect(),
                )
            }
            // Both English: only an is-a chain (one un-keyed hypernym closure).
            (Some(GroundedConcept::English(c)), Some(GroundedConcept::English(a)))
                if *kind == subsumption_kind() =>
            {
                self.english.ancestor_chain(c, a)
            }
            _ => None,
        }
    }

    fn concept_count(&self) -> usize {
        self.english.concept_count() + self.loaded_concepts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::meta::OntologyName;
    use pr4xis_runtime::archive::Archive;
    use pr4xis_runtime::definition::{Definition, EdgeTarget};
    use pr4xis_runtime::ontology::materialize;

    /// A loaded concept is queryable by the SURFACE of a Form atom it denotes (a
    /// heading / citation), not only by its URN/IRI identity — the §9
    /// lexicalization channel (one *Bedeutung*, many *Sinne*). A Form atom is a
    /// surface, never a concept of its own.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn a_form_atoms_surface_resolves_to_the_concept_that_denotes_it() {
        let archive = Archive {
            nodes: alloc::vec![
                Definition {
                    kind: "Concept".to_string(),
                    name: "/us/usc/t1/s1".to_string(),
                    edges: alloc::vec![(
                        "canonicalForm".to_string(),
                        EdgeTarget::Local("section 1".to_string()),
                    )],
                    axioms: alloc::vec![],
                    lexical: Some("Words denoting number, gender, and so forth.".to_string()),
                },
                // The Form atom — its writtenRep name IS the surface.
                Definition {
                    kind: FORM_KIND.to_string(),
                    name: "section 1".to_string(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: Some("section 1".to_string()),
                },
            ],
            connections: alloc::vec![],
        };
        let onto = materialize(archive, OntologyName::new_static("usc_test"))
            .expect("the Form-bearing archive materializes");
        let composed = ComposedReasoner::new(English::sample_static(), alloc::vec![Rc::new(onto)]);

        // The Form's writtenRep "section 1" resolves to the section concept, and
        // reading it back yields the section's gloss.
        let ids = composed.lookup("section 1");
        assert!(
            !ids.is_empty(),
            "the Form surface 'section 1' must be queryable"
        );
        let concept = composed.concept(ids[0]).expect("its concept resolves");
        assert!(
            concept
                .definitions()
                .next()
                .is_some_and(|d| d.contains("number")),
            "the surface resolves to the section's gloss; got {:?}",
            concept.definitions().collect::<alloc::vec::Vec<_>>()
        );

        // The Form atom is NOT a concept of its own: the URN still resolves to the
        // SAME concept (the node-name surface is kept additively), not a new one.
        assert_eq!(
            composed.lookup("/us/usc/t1/s1"),
            ids,
            "the URN resolves to the same concept the Form surface does"
        );
        // And the multi-word Form surface makes the recognizer active.
        assert!(composed.max_surface_words() >= 2);
    }

    /// THE GENERAL GROUNDING MECHANISM, on a NON-STATUTE fixture with zero legal
    /// vocabulary — a `menagerie` instance archive grounds into a `taxonomy` base
    /// through the SAME loader path USC→LegalSources takes: an `InstanceFunctor`
    /// connection carried as data, minted by `ground_loaded_set`, resolved by the
    /// generic `cross_reaches`. `rex` (a `Pet`) instantiates `taxonomy:Dog`, so it
    /// reaches `Dog ⊑ Mammal ⊑ Animal` — but NOT the unrelated `Rock`. This proves
    /// grounding is a source-agnostic mechanism, not a statute special case.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn a_non_statute_instance_grounds_into_its_taxonomy_by_the_general_path() {
        use pr4xis_runtime::connection::{Connection, GeneratorAction};
        use pr4xis_runtime::ontology::relations_kind;

        // The base taxonomy: Dog ⊑ Mammal ⊑ Animal, plus an unrelated Rock. Zero
        // legal vocabulary anywhere in this test.
        fn taxonomy() -> Archive {
            let concept = |name: &str, parent: Option<&str>, gloss: &str| Definition {
                kind: "Concept".to_string(),
                name: name.to_string(),
                edges: parent
                    .map(|p| {
                        alloc::vec![("Subsumption".to_string(), EdgeTarget::Local(p.to_string()))]
                    })
                    .unwrap_or_default(),
                axioms: alloc::vec![],
                lexical: Some(gloss.to_string()),
            };
            Archive {
                nodes: alloc::vec![
                    concept("Dog", Some("Mammal"), "a domesticated canine"),
                    concept("Mammal", Some("Animal"), "a warm-blooded vertebrate"),
                    concept("Animal", None, "a living organism"),
                    concept("Rock", None, "an inanimate mineral mass"),
                ],
                connections: alloc::vec![],
            }
        }

        // The instance archive: `rex` (a `Pet`) carrying a grounding functor as DATA
        // typing `Pet ↦ taxonomy:Dog`.
        let menagerie = Archive {
            nodes: alloc::vec![Definition {
                kind: "Pet".to_string(),
                name: "rex".to_string(),
                edges: alloc::vec![],
                axioms: alloc::vec![],
                lexical: Some("a good dog".to_string()),
            }],
            connections: alloc::vec![Connection {
                kind: "InstanceFunctor".to_string(),
                source: "menagerie".to_string(),
                target: "taxonomy".to_string(),
                action: GeneratorAction::Functor {
                    map_object: alloc::vec![("Pet".to_string(), "Dog".to_string())],
                    map_morphism: alloc::vec![(
                        "instantiates".to_string(),
                        "Subsumption".to_string()
                    )],
                },
                laws: alloc::vec!["PreservesTyping".to_string()],
            }],
        };

        // Both load orders: base before instance AND instance before base — the
        // general grounding pass grounds `rex` regardless of position.
        for base_first in [true, false] {
            let tax = materialize(taxonomy(), OntologyName::new_static("taxonomy"))
                .expect("taxonomy materializes");
            let men = materialize(menagerie.clone(), OntologyName::new_static("menagerie"))
                .expect("menagerie materializes");
            let mut set = if base_first {
                alloc::vec![Rc::new(tax), Rc::new(men)]
            } else {
                alloc::vec![Rc::new(men), Rc::new(tax)]
            };
            crate::formal::meta::grounding::ground_loaded_set(&mut set, English::sample_static())
                .expect("the single-level menagerie grounds");
            let composed = ComposedReasoner::new(English::sample_static(), set);
            let subsumption = subsumption_kind();

            // Select the LOADED concept for each surface — English also knows
            // "animal"/"dog"/"mammal", and English ids sort first in the union, so a
            // bare `[0]` would pick the English concept (a mixed-universe query). The
            // grounding lives between the LOADED vertices.
            let loaded_id = |surface: &str| {
                composed
                    .lookup(surface)
                    .iter()
                    .copied()
                    .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::Loaded(_))))
                    .unwrap_or_else(|| panic!("no loaded concept resolves for {surface:?}"))
            };
            let rex = loaded_id("rex");
            let animal = loaded_id("animal");
            let mammal = loaded_id("mammal");
            let rock = loaded_id("rock");

            // rex types as Dog, so it reaches Mammal and Animal in the peer closure.
            assert!(
                composed.reaches(rex, animal, &subsumption),
                "rex (a Pet grounded as Dog) reaches taxonomy:Animal (base_first={base_first})"
            );
            assert!(
                composed.reaches(rex, mammal, &subsumption),
                "rex reaches taxonomy:Mammal by the peer's Dog ⊑ Mammal closure"
            );
            // NOT a blanket yes: rex does not reach the unrelated Rock.
            assert!(
                !composed.reaches(rex, rock, &subsumption),
                "rex does NOT reach the unrelated Rock — the grounding reads the real closure"
            );
            // §9 over-generation guard: a Parthood query does not spuriously hold.
            assert!(
                !composed.reaches(rex, animal, &relations_kind("Parthood")),
                "the grounding mints only the instantiates (Subsumption) edge, not Parthood"
            );
        }
    }

    /// W2.2 — WORDS (declared types) ARE POINTERS INTO ENGLISH: a loaded `.prx`
    /// node grounds INTO `english_wordnet` through a DECLARED into-English
    /// InstanceFunctor (`Canine ↦ s-dog`, carried as data), so it INHERITS English's
    /// taxonomy and "is <node> an animal" answers through WordNet's own
    /// `s-dog ⊑ s-mammal ⊑ s-animal` chain — NOT a loaded taxonomy peer (as the
    /// USC/menagerie test above), and NEVER by installing English as a loaded
    /// ontology. The UNDECLARED control (kind `Mineral`, surface an animal word)
    /// does NOT link — DECLARED-TYPE grounding, not surface auto-matching (§9).
    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn a_declared_node_points_into_english_and_inherits_its_taxonomy() {
        use pr4xis_runtime::connection::{Connection, GeneratorAction};
        use pr4xis_runtime::ontology::relations_kind;

        // The menagerie: a DECLARED `Canine` (`rex`) and an UNDECLARED `Mineral`
        // (`salmon`, whose very surface is an English animal word), plus the
        // into-English `InstanceFunctor` typing ONLY `Canine ↦ english_wordnet:s-dog`.
        let menagerie = Archive {
            nodes: alloc::vec![
                Definition {
                    kind: "Canine".to_string(),
                    name: "rex".to_string(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: Some("a companion dog".to_string()),
                },
                Definition {
                    kind: "Mineral".to_string(),
                    name: "salmon".to_string(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: Some("typed a Mineral; its surface is an animal word".to_string()),
                },
            ],
            connections: alloc::vec![Connection {
                kind: "InstanceFunctor".to_string(),
                source: "menagerie".to_string(),
                target: "english_wordnet".to_string(),
                action: GeneratorAction::Functor {
                    map_object: alloc::vec![("Canine".to_string(), "s-dog".to_string())],
                    map_morphism: alloc::vec![("denotes".to_string(), "Subsumption".to_string())],
                },
                laws: alloc::vec!["PreservesTyping".to_string()],
            }],
        };

        let men = materialize(menagerie, OntologyName::new_static("menagerie"))
            .expect("menagerie materializes");
        let mut set = alloc::vec![Rc::new(men)];
        // The MINT-side seeds English as the transient grounding target peer.
        crate::formal::meta::grounding::ground_loaded_set(&mut set, English::sample_static())
            .expect("the single-level menagerie grounds");
        let composed = ComposedReasoner::new(English::sample_static(), set);
        let subsumption = subsumption_kind();

        // GATE (i): English is NEVER a loaded ontology.
        assert!(
            composed
                .loaded()
                .iter()
                .all(|o| o.id().as_str() != "english_wordnet"),
            "english_wordnet must never appear in the loaded set"
        );

        // The LOADED node (disjoint id) and the ENGLISH ancestor (below base).
        let loaded_id = |surface: &str| {
            composed
                .lookup(surface)
                .iter()
                .copied()
                .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::Loaded(_))))
                .unwrap_or_else(|| panic!("no loaded concept resolves for {surface:?}"))
        };
        let english_id = |surface: &str| {
            composed
                .lookup(surface)
                .iter()
                .copied()
                .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::English(_))))
                .unwrap_or_else(|| panic!("no english concept resolves for {surface:?}"))
        };

        let rex = loaded_id("rex");
        let animal = english_id("animal");
        let mammal = english_id("mammal");

        // DECLARED: rex (Canine ↦ s-dog) reaches English's `animal` via English's own
        // s-dog ⊑ s-mammal ⊑ s-animal chain (and `mammal` on the way).
        assert!(
            composed.reaches(rex, animal, &subsumption),
            "rex points into english_wordnet:s-dog, so it is an animal via English's is-a chain"
        );
        assert!(
            composed.reaches(rex, mammal, &subsumption),
            "rex reaches English's `mammal` through WordNet's s-dog ⊑ s-mammal"
        );

        // GATE (ii) §9: the UNDECLARED `salmon` (kind Mineral) carries no functor
        // entry, so it does NOT link to `animal` — surface auto-matching declined.
        let salmon = loaded_id("salmon");
        assert!(
            !composed.reaches(salmon, animal, &subsumption),
            "the undeclared Mineral 'salmon' must NOT link to animal (§9 — no declared typing)"
        );

        // GATE (iii) directional: English's `animal` does not reach the loaded rex.
        assert!(
            !composed.reaches(animal, rex, &subsumption),
            "reaches into English is directional — the English concept does not reach the loaded node"
        );

        // §9 over-generation guard: the into-English typing asserts ONLY Subsumption,
        // so a Parthood query into English is false.
        assert!(
            !composed.reaches(rex, animal, &relations_kind("Parthood")),
            "the into-English functor mints only the Subsumption typing, not Parthood"
        );
    }

    /// The relation-parametric `reaches` reads each relation's OWN materialized
    /// closure: a USC-oriented Parthood mereology (part → whole) is traversable,
    /// directionally, and is DISTINCT from Subsumption over the same edge — the
    /// Smith et al. (2005) `part_of` ≠ `is_a` distinction, enforced at the reasoner.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reaches_reads_the_parthood_closure_distinct_from_subsumption() {
        use pr4xis_runtime::ontology::relations_kind;

        // A subsection is PART OF a section — the USC orientation (part → whole),
        // edge kind "Parthood" (which materialize folds into the transitive
        // Parthood closure; Parthood ∈ relations_transitive_kinds.txt).
        let archive = Archive {
            nodes: alloc::vec![
                Definition {
                    kind: "Concept".to_string(),
                    name: "subsection".to_string(),
                    edges: alloc::vec![(
                        "Parthood".to_string(),
                        EdgeTarget::Local("section".to_string()),
                    )],
                    axioms: alloc::vec![],
                    lexical: Some("A lettered subdivision of a section.".to_string()),
                },
                Definition {
                    kind: "Concept".to_string(),
                    name: "section".to_string(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: Some("The smallest numbered unit of a statute.".to_string()),
                },
            ],
            connections: alloc::vec![],
        };
        let onto = materialize(archive, OntologyName::new_static("part_test"))
            .expect("the Parthood archive materializes");
        let composed = ComposedReasoner::new(English::sample_static(), alloc::vec![Rc::new(onto)]);
        let sub = composed.lookup("subsection")[0];
        let sec = composed.lookup("section")[0];

        let parthood = relations_kind("Parthood");
        // The subsection reaches its section along Parthood.
        assert!(
            composed.reaches(sub, sec, &parthood),
            "a subsection is part of its section"
        );
        // Antisymmetric (BFO:0000050): the section is NOT part of the subsection.
        assert!(
            !composed.reaches(sec, sub, &parthood),
            "Parthood is directional — the whole is not part of its part"
        );
        // Distinct closures: the same pair is NOT an is-a (the edge is Parthood).
        assert!(
            !composed.reaches(sub, sec, &subsumption_kind()),
            "the Parthood edge is not a Subsumption edge — is-a must be false"
        );

        // Reflexivity is per-relation, read from the loaded Relations data:
        // Parthood is declared `Irreflexive`, so a thing is NOT a proper part of
        // itself — the `c == a` short-circuit must NOT blanket-return true.
        assert!(
            !composed.reaches(sub, sub, &parthood),
            "Parthood is irreflexive — X is not part of X"
        );
        // …while Subsumption IS declared reflexive (X is-a X).
        assert!(
            composed.reaches(sub, sub, &subsumption_kind()),
            "Subsumption is reflexive — X is-a X"
        );
    }

    /// PIN — the loaded-side `ancestors` DAG-tie order after the reachability-
    /// kernel unification (slice (c)): equal-distance ancestors order by the
    /// kernel's `(is-a distance, ConceptRef::Ord)` contract, NOT by archive-
    /// position (`ConceptId.value()`) order — and `ancestors` now agrees with
    /// `ancestor_chain` on ONE ordering (they disagreed before: `ancestors`
    /// re-sorted by loaded id, `ancestor_chain` kept the closure's
    /// `(dist, ontology, name)` order).
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn loaded_ancestors_ties_order_by_concept_ref_not_archive_position() {
        // The diamond: kid → {zebra, apple} → root, with `zebra` DECLARED
        // BEFORE `apple` — so archive-position (ConceptId) order says
        // zebra-then-apple while ConceptRef name order says apple-then-zebra.
        // The distance-1 tie discriminates the two orderings.
        let concept = |name: &str, parents: &[&str], gloss: &str| Definition {
            kind: "Concept".to_string(),
            name: name.to_string(),
            edges: parents
                .iter()
                .map(|p| ("Subsumption".to_string(), EdgeTarget::Local(p.to_string())))
                .collect(),
            axioms: alloc::vec![],
            lexical: Some(gloss.to_string()),
        };
        let archive = Archive {
            nodes: alloc::vec![
                concept("kid", &["zebra", "apple"], "the diamond's bottom"),
                concept("zebra", &["root"], "the Ord-LARGER equal-distance ancestor"),
                concept(
                    "apple",
                    &["root"],
                    "the Ord-SMALLER equal-distance ancestor"
                ),
                concept("root", &[], "the diamond's top"),
            ],
            connections: alloc::vec![],
        };
        let onto = materialize(archive, OntologyName::new_static("diamond"))
            .expect("the diamond archive materializes");
        let composed = ComposedReasoner::new(English::sample_static(), alloc::vec![Rc::new(onto)]);

        // Select the LOADED concept per surface (English also knows these words).
        let loaded_id = |surface: &str| {
            composed
                .lookup(surface)
                .iter()
                .copied()
                .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::Loaded(_))))
                .unwrap_or_else(|| panic!("no loaded concept resolves for {surface:?}"))
        };
        let kid = loaded_id("kid");
        let zebra = loaded_id("zebra");
        let apple = loaded_id("apple");
        let root = loaded_id("root");
        // The declaration order really does invert the name order at the ids —
        // otherwise this test could not discriminate the two orderings.
        assert!(
            zebra.value() < apple.value(),
            "fixture invariant: zebra is archive-earlier (smaller id) than apple"
        );

        // The pinned order: (is-a distance, ConceptRef::Ord) — apple before
        // zebra at the tie. Archive-position order would say zebra first.
        let want = alloc::vec![kid, apple, zebra, root];
        assert_eq!(
            composed.ancestors(kid),
            want,
            "loaded ancestors must order ties by ConceptRef::Ord, not archive position"
        );
        // …and `ancestors` and `ancestor_chain` agree on the ONE ordering (the
        // pre-fix inconsistency: this very pair diverged at the tie).
        assert_eq!(
            composed.ancestor_chain(kid, root),
            Some(want),
            "ancestors and ancestor_chain must agree on the kernel ordering"
        );
    }
}
