//! The WordNet → `.prx` bridge — project the loaded [`English`] domain struct
//! into a content-addressed runtime [`Archive`](pr4xis_runtime::archive::Archive),
//! then relabel it into the praxis schema with a projection carried AS DATA.
//!
//! This is the domains half of the B1 engine bridge (#87): it dissolves the
//! SUBSTRATE SPLIT. `english_loaded()` hands back an [`English`] — a closed
//! domain struct whose synsets are positional `Reference<4>` indices and whose
//! relations are typed `HashMap`s. That is NOT a runtime
//! [`Archive`](pr4xis_runtime::archive::Archive), so a generic engine has no
//! addressable atom to point at and no traverser to follow.
//! [`project_archive`](crate::cognitive::linguistics::english::bridge::project_archive) is the functor `English → Archive`
//! that gives each synset a definition-bearing
//! [`ContentAddress`](pr4xis_runtime::address) and re-expresses its hypernym
//! links as runtime edges.
//!
//! # The relabeling is data, not code
//!
//! The projection emits each synset edge under its RAW WordNet relation name
//! (`hypernym`), NOT a praxis kind. Mapping `hypernym → Subsumption`,
//! `Synset → Concept` is a separate FUNCTOR carried as `.prx` data — the
//! committed `data/projections/english_functor.prx`, loaded fail-closed against
//! its baked root and interpreted by the one runtime primitive
//! [`apply`](pr4xis_runtime::apply::apply). So the relation-kind table is data
//! that re-emits to update — never the hardcoded `match rel_type`
//! (`pr4xis::codegen::wordnet`) the old codegen path baked in. The projection no
//! longer even lives in Rust: it is `.prx` on disk.
//!
//! # Scope (B1): the is-a closure
//!
//! Only `hypernym` (the Subsumption-closure-bearing relation) is projected — it
//! is exactly what the grounding gate ("is a dog an animal") needs. Meronymy is
//! a DECLARED follow-up, not a stub: the [`English`] struct's `mereology_parts`
//! map conflates the six holo-/mero- sub-types into one whole→part map (its
//! `is_mereology()` filter treats `synset.id` as the whole even for the `mero_*`
//! relations, whose true direction is part→whole), so projecting Parthood from
//! it would bake a direction-mixed edge into the runtime closure. Faithful
//! directional meronymy must project from the typed `holo_*` / `mero_*` sub-type
//! maps, which is its own slice.
//!
//! Literature:
//! - Fellbaum (1998) *WordNet* — the synset / hypernym lexical database the
//!   `English` struct loads.
//! - Lawvere functorial semantics; Fong & Spivak (2019) *Seven Sketches* Ch. 3 —
//!   a functor is determined by its finite action on generators, which is why
//!   the relation→kind table is carried as data and applied by a table lookup.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::apply::apply;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::connection::Connection;
use pr4xis_runtime::definition::{CANONICAL_FORM_REL, Definition, EdgeTarget};
use pr4xis_runtime::ontology::{MaterializeError, RuntimeOntology, materialize};

use super::ontology::English;
use crate::social::software::markup::xml::lmf::reader::LmfReadError;

/// The ontology name the projected English `.prx` materializes under — the same
/// `OntologyName` the existing English-grounding path (`english_adjunction`)
/// stamps onto its `ConceptRef`s, so a synset's runtime identity is stable
/// across the bridge.
pub const ENGLISH_ONTOLOGY: &str = "english_wordnet";

/// The praxis concept kind a relabeled `Synset` becomes — the SAME node kind
/// [`emit`](pr4xis_runtime::emit) writes for a compiled `ontology!` concept, so
/// the functor's image is structurally a praxis-shaped archive.
pub const CONCEPT_KIND: &str = "Concept";

/// The praxis relation kind a relabeled `hypernym` edge becomes — one of the
/// canonically transitive kinds [`materialize`](pr4xis_runtime::ontology) folds
/// into its is-a closure.
pub const SUBSUMPTION_REL: &str = "Subsumption";

/// The node kind every projected synset carries in the SOURCE archive — the raw
/// WordNet schema generator, before the praxis functor relabels it. Held as one
/// constant so the projection and the committed `english_functor.prx` name the
/// same source generator.
pub const SYNSET_KIND: &str = "Synset";

/// The raw WordNet relation name a hypernym (is-a) edge carries in the SOURCE
/// archive — the schema generator the praxis functor maps to `Subsumption`.
pub const HYPERNYM_REL: &str = "hypernym";

/// The praxis relation kind a relabeled `antonym` edge becomes — the
/// NON-transitive kind (Saussure 1916 / Cruse 1986 antonymy; see
/// [`opposition_relation_kind`](crate::formal::relations::ontology::opposition_relation_kind))
/// [`materialize`](pr4xis_runtime::ontology) leaves OUT of its transitive-fold
/// set, so a single direct edge is the whole answer — the same semantics
/// [`English::opposes`](super::ontology::English::opposes) already gives the
/// embedded lexicon's own antonym pairs, now available for a LOADED WN-LMF
/// lexicon's concepts too.
pub const OPPOSITION_REL: &str = "Opposition";

/// The raw WordNet relation name an antonym (opposition) edge carries in the
/// SOURCE archive — the schema generator the praxis functor maps to
/// [`OPPOSITION_REL`]. Matches the wire string
/// [`SynsetRelationType::parse`](crate::social::software::markup::xml::lmf::ontology::SynsetRelationType::parse)
/// recognizes for `antonym` (WN-LMF `<SynsetRelation relType="antonym">`).
pub const ANTONYM_REL: &str = "antonym";

// The `ontolex:Form` wire primitives (`FORM_KIND`, `form_atom`) live in the
// runtime crate beside [`Definition`], so the compiled-ontology emitter
// (`pr4xis_runtime::emit`) mints Form atoms under the EXACT kind string this
// bridge — and the composed reasoner — filter on. Re-exported here to keep the
// `english::bridge::{FORM_KIND, form_atom}` path every downstream grounding site
// already imports.
pub use pr4xis_runtime::definition::{FORM_KIND, form_atom};

/// Project the loaded [`English`] struct into a content-addressed source
/// [`Archive`] — the functor `English → Archive`.
///
/// Each [`Concept`](super::ontology::Concept) (synset) becomes a [`Definition`]:
/// - `kind` = [`SYNSET_KIND`] (the raw schema generator);
/// - `name` = the synset's `original_id` (its stable WordNet identity, the
///   referent every edge target names — so the archive is referentially closed);
/// - `edges` = one `(`[`HYPERNYM_REL`]`, parent_original_id)` per direct
///   hypernym, raw WordNet names carried through for the functor to relabel;
/// - `lexical` = the synset's first gloss (its Lemon grounding);
/// - `axioms` = none (synsets carry no governing axioms at this layer).
///
/// Every hypernym target is itself a declared synset (the `English` taxonomy is
/// built only from resolved synset ids), so the resulting archive satisfies the
/// referential closure [`materialize`](pr4xis_runtime::ontology) requires.
pub fn project_archive(english: &English) -> Archive {
    let nodes = english
        .concepts()
        .map(|concept| synset_definition(english, &concept))
        .collect();

    Archive {
        nodes,
        connections: Vec::new(),
    }
}

/// ONE synset's projected [`Definition`] — the node shape [`project_archive`]
/// mints, factored out so a PER-NAME resolver ([`english_atom_address`]) builds
/// exactly the same node (hence exactly the same content address) as the full
/// projection, BY CONSTRUCTION rather than by a parallel re-implementation.
/// The shape depends only on the synset itself and its direct parents'
/// `original_id`s, so a single-synset build is byte-identical to the full
/// projection's node for that synset.
pub fn synset_definition(
    english: &English,
    concept: &super::concept_store::ConceptView<'_>,
) -> Definition {
    let mut edges: Vec<(String, EdgeTarget)> = english
        .parents(concept.id())
        .iter()
        .filter_map(|&parent| {
            english.concept(parent).map(|p| {
                (
                    HYPERNYM_REL.to_string(),
                    EdgeTarget::Local(p.original_id().to_string()),
                )
            })
        })
        .collect();
    // Antonym (Opposition) edges, additive — a concept with no antonyms (the
    // overwhelming majority; WordNet's antonym relation is chiefly an
    // adjective/adverb phenomenon) gets no edges appended here, so its
    // Definition — and hence its content address — is BYTE-IDENTICAL to
    // before this was added. WordNet's antonym relation is SENSE-keyed
    // ("big#1" opposes "small#1", not the synset wholesale — see
    // `English::opposes`), so this bridges concept → its senses → each
    // sense's direct opposition targets → back to the target's OWNING
    // concept, deduplicating since two synonymous senses of this concept can
    // independently point at the same opposite concept.
    for &sense in english.senses_of(concept.id()) {
        for &opp_sense in english.opposites(sense) {
            if let Some(opp_concept) = english.concept_of_sense(opp_sense)
                && let Some(p) = english.concept(opp_concept)
            {
                let edge = (
                    ANTONYM_REL.to_string(),
                    EdgeTarget::Local(p.original_id().to_string()),
                );
                if !edges.contains(&edge) {
                    edges.push(edge);
                }
            }
        }
    }
    Definition {
        kind: SYNSET_KIND.to_string(),
        name: concept.original_id().to_string(),
        edges,
        axioms: Vec::new(),
        lexical: concept.definitions().next().map(|d| d.to_string()),
    }
}

/// Resolve ONE English grounding-target name to its content address — the
/// PER-NAME resolver the grounding pass uses instead of projecting the whole
/// `english_wordnet` archive per install ([`project_archive_with_forms`], the
/// measured +82.79 MiB per-install transient this replaces).
///
/// Resolution order mirrors the full projection's node order (the order
/// `type_lens`'s first-match `find` over [`project_archive_with_forms`]
/// observed): SYNSETS first — a name that is a synset `original_id` resolves
/// to that synset's [`synset_definition`] address — then FORM atoms — a name
/// present in the word index resolves to its [`form_atom`] address. `Ok(None)`
/// when English holds neither (the caller's fail-closed
/// `GroundTargetAbsent`).
///
/// Address agreement with the full projection is BY CONSTRUCTION for synsets
/// ([`synset_definition`] is the same code path) and documented for forms (a
/// [`form_atom`] is position-independent; the with-forms projection appends
/// the identical atom).
pub fn english_atom_address(
    english: &English,
    name: &str,
) -> Result<Option<ContentAddress>, pr4xis_runtime::codec::CodecError> {
    if let Some(concept) = english.concept_by_synset(name) {
        return synset_definition(english, &concept).address().map(Some);
    }
    if !english.lookup(name).is_empty() {
        return form_atom(name).address().map(Some);
    }
    Ok(None)
}

/// The `english_wordnet` archive a statute grounds INTO: the synset nodes (the
/// is-a reasoning graph, [`project_archive`]) PLUS one [`form_atom`] per written
/// form. ONE archive carries both — the reasoning graph AND the lexical surface a
/// foreign `denotes` pointer resolves against (a Title-1 word → an `ontolex:Form`
/// atom by content address).
///
/// The Form atoms are inert in the is-a closure (no edges), so reasoning over the
/// synsets is unaffected; they exist purely as addressable written-form targets.
/// (`project_archive` stays the lean synset-only reasoning projection; this is
/// the grounding-target superset.)
pub fn project_archive_with_forms(english: &English) -> Archive {
    let mut archive = project_archive(english);
    archive
        .nodes
        .extend(english.word_index.words().map(form_atom));
    archive
}

/// Project a loaded [`English`] struct into a LEXICALIZED source [`Archive`] —
/// [`project_archive_with_forms`] PLUS the concept→surface edges: for every
/// written form `w` in the word index, each concept `english.lookup(w)` resolves
/// to gains a ([`CANONICAL_FORM_REL`], `Local(w)`) edge aimed at `w`'s appended
/// [`form_atom`]. This is the OntoLex-Lemon lexicalization channel (McCrae,
/// Bosque-Gil, Gracia, Buitelaar & Cimiano 2017, *The OntoLex-Lemon Model*,
/// Proc. eLex 2017: a `LexicalEntry`'s `Form` carries the surface, its `Sense`
/// points at the concept) — the SAME channel the compiled-ontology emitter
/// (`pr4xis_runtime::emit`) mints for `ontology!` labels, so the projection's
/// image is chat-composable BY CONSTRUCTION: the composed reasoner indexes
/// exactly these Form-targeted edges as queryable surfaces.
///
/// This is the projection a REGISTERED WN-LMF LEXICON source (a bounded
/// closed-class lexicon like `us_legal_lexicon@2026`) rides into the runtime
/// substrate ([`lexicon_runtime_ontology`]). The full-corpus `english_wordnet`
/// deliberately does NOT take this path (see the NOTE below
/// [`english_functor`]): its grounding uses the lean
/// [`project_archive_with_forms`] transient, whose Form atoms stay edge-inert.
///
/// Shape guarantees, mirroring `emit`:
/// - one Form atom per DISTINCT written form (the word index is keyed by
///   surface), appended AFTER the concept edges are aimed, so the archive stays
///   referentially closed with no duplicate;
/// - a form that (case-insensitively) equals its concept's identifier adds no
///   redundant edge — the node-name grounding already covers that surface;
/// - a touched node's edge rows are canonicalized (sorted, deduped) so its
///   content address does not depend on lexicalization order.
pub fn project_lexicon_archive(english: &English) -> Archive {
    let mut archive = project_archive(english);
    // name → node position: `project_archive` emits one node per concept, named
    // by its `original_id`, so each written form's concepts are found by id.
    let position: BTreeMap<String, usize> = archive
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.name.clone(), i))
        .collect();
    // Nodes that gained lexicalization edges — canonicalized once after the
    // walk (the `emit` discipline), not per push.
    let mut touched: BTreeSet<usize> = BTreeSet::new();
    for word in english.word_index.words() {
        for &cid in english.lookup(word) {
            let Some(concept) = english.concept(cid) else {
                continue;
            };
            let Some(&pos) = position.get(concept.original_id()) else {
                continue;
            };
            // Skip the redundant: a form that equals the identifier adds no
            // surface the node-name grounding lacks (the `emit` rule).
            if word.eq_ignore_ascii_case(&archive.nodes[pos].name) {
                continue;
            }
            archive.nodes[pos].edges.push((
                CANONICAL_FORM_REL.to_string(),
                EdgeTarget::Local(word.to_string()),
            ));
            touched.insert(pos);
        }
    }
    for pos in touched {
        archive.nodes[pos].edges.sort();
        archive.nodes[pos].edges.dedup();
    }
    // One `ontolex:Form` atom per distinct written form — the referents every
    // canonicalForm edge above names, appended last (same order as
    // `project_archive_with_forms`, whose atom set this is exactly).
    archive
        .nodes
        .extend(english.word_index.words().map(form_atom));
    archive
}

/// The committed WordNet→praxis projection functor — the CANONICAL home of the
/// projection, the directive "projections live in `.prx`, not code" realized
/// (Track C #203). The map tables (`Synset ↦ Concept`, `hypernym ↦ Subsumption`)
/// live ONLY as a content-addressed [`Connection`] inside these bytes, never in a
/// Rust literal. Re-aiming the projection (say `hypernym ↦ Parthood`) means
/// re-emitting this file and updating [`ENGLISH_FUNCTOR_ROOT_HEX`] — NO recompile
/// of any projection logic. There is deliberately NO committed regenerator: the
/// pin IS the integrity, and a permanent Rust emitter would smuggle the relabel
/// table back into code (the #203 design rule). The projection's reviewable
/// provenance is the prior `wordnet_to_praxis_functor()` `Connection`, deleted in
/// `745f38e` (recoverable from git history); a re-aim emits a fresh `.prx` from a
/// one-off `Connection` and re-pins.
const ENGLISH_FUNCTOR_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/projections/english_functor.prx"
));

/// The trusted Merkle root of [`ENGLISH_FUNCTOR_PRX`] — the integrity pin the
/// fail-closed load checks against (file ⇔ pin coherence is asserted in tests).
const ENGLISH_FUNCTOR_ROOT_HEX: &str =
    "5bfb24ec2092b0fc72d7ab6286922f278be6d9ed9d09aaf2542e64286e40d7b2";

/// Load the WordNet→praxis functor from its committed `.prx`
/// (`ENGLISH_FUNCTOR_PRX`) — FAIL-CLOSED: the embedded bytes are admitted only
/// if they re-derive to `ENGLISH_FUNCTOR_ROOT_HEX`, so a tampered or stale
/// projection is refused, never silently mis-applied. Reuses the kernel
/// [`load`](pr4xis_runtime::load::load); no new runtime API. The functor is a
/// finite action on generators (the finite-presentation theorem; Fong & Spivak
/// *Seven Sketches* Ch. 3), interpreted by [`apply`] over a [`project_archive`]
/// source. A load failure here is a build-time invariant violation (the bytes
/// ship embedded in the binary), exactly like the `english.xml` parse `expect`.
///
/// Public because its corpus-scale consumer lives in `praxis-corpus-tests`
/// (`english_functor_projects_the_csr_edge_set`): the ARCHIVE-LEVEL gate that
/// applies this functor over the full loaded corpus and compares the relabeled
/// edge set against the `TaxonomyStore` CSR — a transient `Vec<Definition>`
/// comparison, never a materialized `RuntimeOntology` (the former full-corpus
/// bridge held a +216 MiB transient this data-level theorem does not need).
pub fn english_functor() -> Connection {
    let root = ContentAddress::from_hex(ENGLISH_FUNCTOR_ROOT_HEX)
        .expect("ENGLISH_FUNCTOR_ROOT_HEX is valid 64-hex");
    let archive = pr4xis_runtime::load::load(ENGLISH_FUNCTOR_PRX, root)
        .expect("committed english_functor.prx must load against its baked root");
    archive
        .connections
        .into_iter()
        .next()
        .expect("english_functor.prx carries exactly one Connection")
}

// NOTE: there is deliberately NO `english_runtime_ontology` here anymore — the
// fat bridge (`project_archive → apply → materialize`, an owned generic
// `RuntimeOntology` re-serializing all 107,519 synsets, ~+216 MiB resident over
// the full corpus) had NO production caller: grounding uses the LEAN transient
// [`project_archive_with_forms`] peer, and the engine-level theorem ("the
// generic engine reasons is-a over English") is now true BY CONSTRUCTION — the
// runtime's `MaterializedClosure` and English's `TaxonomyStore` instantiate the
// ONE graded-reach engine (`pr4xis::category::reach`). The remaining DATA-level
// theorem (English's schema projects via the committed functor) is proven
// archive-level, without materializing, by the corpus gate
// `english_functor_projects_the_csr_edge_set` in `praxis-corpus-tests`.
//
// `lexicon_runtime_ontology` below is NOT that fat bridge coming back: it
// materializes a REGISTERED WN-LMF LEXICON source — a bounded closed-class
// lexicon (hundreds of entries, e.g. `us_legal_lexicon@2026`), never the
// 107k-synset English corpus — into a chat-composable `RuntimeOntology`.

/// Why a registered WN-LMF lexicon source failed to become a
/// [`RuntimeOntology`] — the typed sum of the two fallible pipeline stages
/// ([`lexicon_runtime_ontology`]). The embedded-`.prx` gate stage is NOT here:
/// a pin/content-gate failure on an EMBEDDED source is a build-time invariant
/// violation (the bytes ship in the binary) and panics inside
/// `raw_source_text_embedded`, exactly like the `english.xml` parse `expect`.
#[derive(Debug)]
pub enum LexiconBridgeError {
    /// The source text is not well-formed WN-LMF.
    Lmf(LmfReadError),
    /// The projected archive failed to materialize (codec failure or a
    /// referential-closure violation, carried typed).
    Materialize(MaterializeError),
}

impl core::fmt::Display for LexiconBridgeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Lmf(e) => write!(f, "lexicon source failed to parse as WN-LMF: {e}"),
            Self::Materialize(e) => {
                write!(f, "projected lexicon archive failed to materialize: {e:?}")
            }
        }
    }
}

/// Bridge WN-LMF lexicon TEXT into a chat-composable [`RuntimeOntology`] — the
/// generic pipeline core, source-agnostic:
///
/// ```text
/// read_wordnet → English::from_wordnet → project_lexicon_archive
///              → apply(english_functor) → materialize(name)
/// ```
///
/// The verbatim shape of `usc_runtime_ontology` (`project → apply →
/// materialize`), with the ONE committed `english_functor.prx` relabeling the
/// raw WN-LMF generators (`Synset ↦ Concept`, `hypernym ↦ Subsumption`) — a
/// lexicon IS WordNet-shaped data, so it rides the SAME functor, no per-source
/// relabel table. The lexicalization edges ([`CANONICAL_FORM_REL`]) and Form
/// atoms are already praxis wire data and pass through [`apply`] unchanged
/// (an unmapped generator is carried, never dropped).
///
/// `apply` cannot fail here (the loaded `english_functor` is always a
/// `Functor` action); parse and materialization failures propagate typed.
pub fn lexicon_runtime_ontology_from_lmf(
    lmf_xml: &str,
    name: OntologyName,
) -> Result<RuntimeOntology, LexiconBridgeError> {
    let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(lmf_xml)
        .map_err(LexiconBridgeError::Lmf)?;
    let english = English::from_wordnet(&wn);
    let source = project_lexicon_archive(&english);
    let praxis = apply(&english_functor().action, &source)
        .expect("the loaded english_functor is always a Functor action, which apply interprets");
    materialize(praxis, name).map_err(LexiconBridgeError::Materialize)
}

/// Bridge a REGISTERED, EMBEDDED WN-LMF lexicon source into a chat-composable
/// [`RuntimeOntology`] named after the source — the loader any registered
/// lexicon (`us_legal_lexicon` today; further domain lexicons as they are
/// registered) calls with its own `(name, version, committed .prx bytes)`.
/// Zero per-domain code: the source entry IS the parameterization.
///
/// The source text is admitted through the fail-closed
/// [`raw_source_text_embedded`](crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded)
/// gate (praxis.lock `[compact_archive_signatures]` pin), then takes
/// [`lexicon_runtime_ontology_from_lmf`]. The materialized ontology's
/// [`OntologyName`] is the registered source name, so every concept's
/// [`ConceptRef`](pr4xis_runtime::ontology::ConceptRef) cites its source.
pub fn lexicon_runtime_ontology(
    name: &str,
    version: &str,
    embedded_prx: &[u8],
) -> Result<RuntimeOntology, LexiconBridgeError> {
    let xml = crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded(
        name,
        version,
        embedded_prx,
    );
    lexicon_runtime_ontology_from_lmf(&xml, OntologyName::new(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeSet;
    use pr4xis_runtime::connection::GeneratorAction;

    fn node<'a>(archive: &'a Archive, name: &str) -> &'a Definition {
        archive
            .nodes
            .iter()
            .find(|n| n.name == name)
            .unwrap_or_else(|| panic!("archive must declare synset {name:?}"))
    }

    /// A local (same-ontology) edge target — what every projected hypernym edge
    /// is (a synset id within `english_wordnet`).
    fn local(name: &str) -> EdgeTarget {
        EdgeTarget::Local(name.to_string())
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn projects_every_synset_as_a_node() {
        // English::sample() = dog, cat, mammal, animal (nouns) + run, see (verbs)
        // + big (adj) = 7 synsets.
        let archive = project_archive(&English::sample());
        assert_eq!(archive.nodes.len(), 7, "one node per synset");
        assert!(
            archive.connections.is_empty(),
            "B1 projects only nodes/edges"
        );
        for n in &archive.nodes {
            assert_eq!(n.kind, SYNSET_KIND, "every node is a raw Synset generator");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_synset_carries_its_hypernym_edge_and_gloss() {
        let archive = project_archive(&English::sample());
        let dog = node(&archive, "s-dog");
        assert_eq!(
            dog.edges,
            alloc::vec![(HYPERNYM_REL.to_string(), local("s-mammal"))],
            "dog's only generating edge is the raw hypernym → its parent synset id"
        );
        assert_eq!(
            dog.lexical.as_deref(),
            Some("a domesticated canine"),
            "the synset's gloss rides into the node's lexical grounding"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_taxonomy_root_has_no_outgoing_edges() {
        // `animal` is the top of the sample taxonomy — no hypernym.
        let archive = project_archive(&English::sample());
        let animal = node(&archive, "s-animal");
        assert!(
            animal.edges.is_empty(),
            "a root synset projects no hypernym edge; got {:?}",
            animal.edges
        );
        assert_eq!(animal.lexical.as_deref(), Some("a living organism"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_archive_is_referentially_closed() {
        // Every hypernym edge target must be a declared node — the precondition
        // `materialize` enforces. Projecting from the resolved `English` taxonomy
        // guarantees it; assert it directly so a regression is caught here, not
        // only downstream in `materialize`.
        let archive = project_archive(&English::sample());
        let declared: BTreeSet<&str> = archive.nodes.iter().map(|n| n.name.as_str()).collect();
        for n in &archive.nodes {
            for (kind, target) in &n.edges {
                let name = target
                    .local_name()
                    .expect("B1 projects only local hypernym edges");
                assert!(
                    declared.contains(name),
                    "edge {}--{kind}-->{name} names an undeclared synset",
                    n.name
                );
            }
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_is_a_chain_is_present_as_generating_edges() {
        // dog ⊑ mammal ⊑ animal must appear as two raw hypernym generators that
        // the functor + materialize will later fold into the is-a closure.
        let archive = project_archive(&English::sample());
        assert_eq!(
            node(&archive, "s-dog").edges,
            alloc::vec![(HYPERNYM_REL.to_string(), local("s-mammal"))]
        );
        assert_eq!(
            node(&archive, "s-mammal").edges,
            alloc::vec![(HYPERNYM_REL.to_string(), local("s-animal"))]
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_synset_with_no_antonym_projects_byte_identical_edges() {
        // The antonym walk is additive: a concept with no declared antonym
        // (the overwhelming majority — WordNet's antonym relation is chiefly
        // adjective/adverb) must project the EXACT SAME edge list as before
        // this capability existed, so every content address depending on
        // `synset_definition` for a non-antonym concept is unchanged.
        let archive = project_archive(&English::sample());
        assert_eq!(
            node(&archive, "s-dog").edges,
            alloc::vec![(HYPERNYM_REL.to_string(), local("s-mammal"))],
            "no antonym for 'dog' means no ANTONYM_REL edge is appended"
        );
    }

    /// A tiny WN-LMF fixture declaring a sense-keyed antonym pair (hot#1 /
    /// cold#1), the SAME shape [`English::opposes`]'s own test fixture uses —
    /// two single-sense adjective synsets whose senses carry a mutual
    /// `SynsetRelation relType="antonym"` edge.
    const ANTONYM_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-hot-a">
      <Lemma writtenForm="hot" partOfSpeech="a"/>
      <Sense id="hot-a-1" synset="s-hot">
        <SenseRelation relType="antonym" target="cold-a-1"/>
      </Sense>
    </LexicalEntry>
    <LexicalEntry id="e-cold-a">
      <Lemma writtenForm="cold" partOfSpeech="a"/>
      <Sense id="cold-a-1" synset="s-cold">
        <SenseRelation relType="antonym" target="hot-a-1"/>
      </Sense>
    </LexicalEntry>
    <Synset id="s-hot" ili="i1" partOfSpeech="a"><Definition>having a high temperature</Definition></Synset>
    <Synset id="s-cold" ili="i2" partOfSpeech="a"><Definition>having a low temperature</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_declared_antonym_projects_as_a_raw_antonym_edge() {
        let english = English::from_wordnet(
            &crate::social::software::markup::xml::lmf::reader::read_wordnet(ANTONYM_LMF)
                .expect("the antonym fixture is well-formed WN-LMF"),
        );
        let archive = project_archive(&english);
        assert_eq!(
            node(&archive, "s-hot").edges,
            alloc::vec![(ANTONYM_REL.to_string(), local("s-cold"))],
            "hot's sense-keyed antonym edge bridges to cold's OWNING synset, \
             carried under the raw ANTONYM_REL generator for the functor to relabel"
        );
    }

    // --- piece 3: the functor carried as data, applied ---

    #[pr4xis::praxis_value(Honest, Verifiable)]
    #[test]
    fn the_functor_loads_from_its_committed_prx_fail_closed() {
        // The projection LIVES in `english_functor.prx` (Track C #203): the loader
        // admits the committed bytes ONLY against the baked root and yields a
        // Functor action with non-empty relabel tables. The exact rows are NOT
        // re-asserted here — that would re-smuggle the map back into code; the
        // relabel BEHAVIOR is proven by `applying_the_functor_relabels...` below.
        let functor = english_functor();
        let GeneratorAction::Functor {
            map_object,
            map_morphism,
        } = &functor.action
        else {
            panic!("the loaded projection is a Functor action");
        };
        assert!(
            !map_object.is_empty() && !map_morphism.is_empty(),
            "the loaded functor carries non-empty relabel tables"
        );
        // File ⇔ pin coherence + fail-closed: the committed bytes re-derive to the
        // baked root, and a WRONG root is refused (no drift test needed — the pin
        // IS the integrity, there is no Rust source to drift from).
        let pin = ContentAddress::from_hex(ENGLISH_FUNCTOR_ROOT_HEX).unwrap();
        assert_eq!(
            pr4xis_runtime::load::load(ENGLISH_FUNCTOR_PRX, pin)
                .unwrap()
                .root()
                .unwrap(),
            pin,
            "the committed .prx re-derives to its baked root"
        );
        assert!(
            pr4xis_runtime::load::load(ENGLISH_FUNCTOR_PRX, ContentAddress::of(b"wrong")).is_err(),
            "a wrong trusted root is refused (fail-closed)"
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn applying_the_functor_relabels_synset_kinds_into_praxis_kinds() {
        let source = project_archive(&English::sample());
        let target = apply(&english_functor().action, &source).expect("a Functor action applies");

        // Same cardinality — the functor relabels, never drops.
        assert_eq!(target.nodes.len(), source.nodes.len());

        let dog = node(&target, "s-dog");
        assert_eq!(
            dog.kind, CONCEPT_KIND,
            "Synset relabeled to the praxis Concept kind"
        );
        assert_eq!(
            dog.edges,
            alloc::vec![(SUBSUMPTION_REL.to_string(), local("s-mammal"))],
            "the raw hypernym edge is relabeled to Subsumption; its target (identity) is carried"
        );
        assert_eq!(
            dog.lexical.as_deref(),
            Some("a domesticated canine"),
            "the gloss — identity-bearing content — is carried unchanged"
        );
        assert_eq!(
            dog.name, "s-dog",
            "the synset id (identity) is carried unchanged"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_relabeled_archive_is_still_referentially_closed() {
        // apply carries edge targets unchanged, so the functor's image is closed
        // exactly when its source is — the precondition materialize needs holds
        // after relabeling, not only before.
        let source = project_archive(&English::sample());
        let target = apply(&english_functor().action, &source).unwrap();
        let declared: BTreeSet<&str> = target.nodes.iter().map(|n| n.name.as_str()).collect();
        for n in &target.nodes {
            for (_, t) in &n.edges {
                let name = t.local_name().expect("only local edges projected");
                assert!(
                    declared.contains(name),
                    "relabeled edge target {name} undeclared"
                );
            }
        }
    }

    // --- G3b: the honest `denotes` floor — a span grounds into an ontolex:Form ---

    use alloc::collections::BTreeMap;
    use pr4xis_runtime::grounding::{AtomResolver, ConnectedOntologies, ConnectedOntology};

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_grounding_archive_carries_form_atoms_beside_the_synsets() {
        let english = English::sample();
        let reasoning = project_archive(&english);
        let grounding = project_archive_with_forms(&english);
        assert!(
            grounding.nodes.len() > reasoning.nodes.len(),
            "the grounding archive adds Form atoms to the synset reasoning graph"
        );
        // The Form atoms are inert: they carry no edges (no sense), so the is-a
        // closure is unchanged by their presence.
        for n in grounding.nodes.iter().filter(|n| n.kind == FORM_KIND) {
            assert!(
                n.edges.is_empty(),
                "a Form has no senses — it carries no edges"
            );
        }
    }

    /// THE HONEST DENOTES FLOOR: a statute-like span grounds into the written-form
    /// atom "dog" by content address, resolves through the connected
    /// `english_wordnet` archive (G3a), and the resolved target IS an `ontolex:Form`
    /// — never a sense. The sense-deferral is structural and machine-checked.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_denotes_floor_edge_resolves_to_a_form_atom_never_a_sense() {
        let english = English::sample();
        let archive = project_archive_with_forms(&english);
        let english_root = archive.root().unwrap();

        // The Form atom for the written form "dog" — the honest floor target.
        let dog_form_addr = form_atom("dog").address().unwrap();

        // A statute-like provision that DENOTES the written form "dog" (in the full
        // slice this is a USC subdivision; here a bare content node). The target is
        // a Grounded foreign atom — a content address into english_wordnet.
        let provision = Definition {
            kind: "Provision".into(),
            name: "title-1-§1-word".into(),
            edges: alloc::vec![(
                "denotes".to_string(),
                EdgeTarget::Grounded {
                    ontology: ENGLISH_ONTOLOGY.to_string(),
                    atom: dog_form_addr,
                },
            )],
            axioms: alloc::vec![],
            lexical: None,
        };

        // Resolve the floor edge against the connected english_wordnet archive,
        // pinned by its root (fail-closed if it drifts — G3a).
        let mut peers = BTreeMap::new();
        peers.insert(ENGLISH_ONTOLOGY.to_string(), archive);
        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: ENGLISH_ONTOLOGY.to_string(),
            root: english_root,
            role: "denotes".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).expect("the english pin agrees");

        let (_, target) = &provision.edges[0];
        let resolved = resolver
            .resolve(target)
            .expect("the floor edge resolves to its Form atom by content address");

        // THE HONESTY TRIPWIRE (structural): the floor target IS a Form, and a Form
        // has no senses — so the pointer asserts "this written form occurred",
        // never a meaning. A synset (kind "Synset") would be the over-claim.
        assert_eq!(
            resolved.kind, FORM_KIND,
            "a denotes floor edge must resolve to an ontolex:Form, never a sense"
        );
        assert_eq!(resolved.name, "dog");
        assert!(
            resolved.edges.is_empty(),
            "the resolved Form carries no sense edge — sense-deferral is structural"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_synset_atom_is_not_a_written_form_floor_target() {
        // Grounding a denotes floor into a SYNSET (a concept) would be a sense-level
        // over-claim. The structural tripwire catches it: the synset's kind is not
        // Form, so the floor's "resolved.kind == FORM_KIND" check rejects it.
        let archive = project_archive_with_forms(&English::sample());
        let dog_synset = archive
            .nodes
            .iter()
            .find(|n| n.name == "s-dog")
            .expect("the synset is present");
        assert_ne!(
            dog_synset.kind, FORM_KIND,
            "a synset is a sense-bearing concept, not a written-form floor target"
        );
    }

    // --- the generic lexicon projection (project_lexicon_archive) ---

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn the_lexicon_projection_is_referentially_closed() {
        // Every edge — the raw hypernym mereology AND the §9 canonicalForm
        // lexicalization — is a same-archive local edge to a DECLARED node
        // (the closure `materialize` enforces). The Form atoms are appended
        // for exactly the written forms the canonicalForm edges name.
        let archive = project_lexicon_archive(&English::sample());
        let declared: BTreeSet<&str> = archive.nodes.iter().map(|n| n.name.as_str()).collect();
        for n in &archive.nodes {
            for (kind, target) in &n.edges {
                let to = target
                    .local_name()
                    .expect("a projected edge is a same-archive local edge");
                assert!(
                    declared.contains(to),
                    "edge {}--{kind}-->{to} names an undeclared node",
                    n.name
                );
                assert!(
                    kind == HYPERNYM_REL || kind == CANONICAL_FORM_REL,
                    "projected edge kinds are hypernym / canonicalForm; got {kind}"
                );
            }
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_written_form_lexicalizes_the_concepts_it_expresses() {
        // "dog" is a written form of the s-dog synset: the projection aims a
        // canonicalForm edge from the concept at the "dog" Form atom, KEEPING
        // the raw hypernym reasoning edge beside it.
        let archive = project_lexicon_archive(&English::sample());
        let dog = node(&archive, "s-dog");
        assert!(
            dog.edges
                .contains(&(CANONICAL_FORM_REL.to_string(), local("dog"))),
            "s-dog lexicalizes its written form; edges: {:?}",
            dog.edges
        );
        assert!(
            dog.edges
                .contains(&(HYPERNYM_REL.to_string(), local("s-mammal"))),
            "the reasoning edge is kept beside the lexicalization"
        );
        assert!(
            archive
                .nodes
                .iter()
                .any(|n| n.kind == FORM_KIND && n.name == "dog"),
            "the written form's Form atom is appended"
        );
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn the_lexicon_projection_is_deterministic() {
        // Two projections of the same loaded lexicon derive the same Merkle
        // root — every consumer (materialize, grounding pins) sees ONE stable
        // content identity. The touched-node edge canonicalization (sort +
        // dedup) is what makes this hold independent of lexicalization order.
        let english = English::sample();
        let a = project_lexicon_archive(&english).root().unwrap();
        let b = project_lexicon_archive(&english).root().unwrap();
        assert_eq!(a, b, "the lexicon projection is content-deterministic");
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn the_lexicon_pipeline_materializes_and_composes() {
        // The whole generic pipeline over the sample lexicon: project →
        // apply(english_functor) → materialize → compose under the
        // ComposedReasoner — and a define-style lookup ("dog") resolves a
        // LOADED concept whose definition (the synset gloss) is reachable.
        use crate::cognitive::linguistics::composed::{ComposedReasoner, GroundedConcept};
        use crate::cognitive::linguistics::english::LexicalReasoner;
        use alloc::rc::Rc;

        let onto = lexicon_runtime_ontology_from_lmf(
            SAMPLE_LMF,
            OntologyName::new_static("sample_lexicon"),
        )
        .expect("the sample lexicon materializes");
        let composed = ComposedReasoner::new(English::sample_static(), alloc::vec![Rc::new(onto)]);

        let ids = composed.lookup("dog");
        let loaded = ids
            .iter()
            .copied()
            .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::Loaded(_))))
            .expect("the lexicon's written form resolves to a LOADED concept");
        let concept = composed.concept(loaded).expect("its concept view resolves");
        assert_eq!(
            concept.definitions().next(),
            Some("a domesticated canine"),
            "the define-style lookup reaches the loaded concept's definition"
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn a_loaded_lexicons_antonym_pair_answers_a_confident_opposition_reaches() {
        // The capability this whole antonym/Opposition addition exists for:
        // a FRESHLY-AUTHORED WN-LMF lexicon (not the embedded English) declares
        // an antonym pair, and the generic materialized-closure `reaches()`
        // path (`ComposedReasoner`'s `(Loaded, Loaded)` branch — the SAME path
        // Parthood already proves for `LegalSources`) answers a confident
        // Opposition query for it, through the REAL committed
        // `english_functor()`, not a one-off test Connection.
        use crate::cognitive::linguistics::composed::{ComposedReasoner, GroundedConcept};
        use crate::cognitive::linguistics::english::LexicalReasoner;
        use crate::formal::relations::ontology::opposition_relation_kind;
        use alloc::rc::Rc;

        let onto = lexicon_runtime_ontology_from_lmf(
            ANTONYM_LMF,
            OntologyName::new_static("antonym_lexicon"),
        )
        .expect("the antonym lexicon materializes");
        let composed = ComposedReasoner::new(English::sample_static(), alloc::vec![Rc::new(onto)]);

        let hot = composed
            .lookup("hot")
            .iter()
            .copied()
            .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::Loaded(_))))
            .expect("'hot' resolves to a LOADED concept");
        let cold = composed
            .lookup("cold")
            .iter()
            .copied()
            .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::Loaded(_))))
            .expect("'cold' resolves to a LOADED concept");

        assert!(
            composed.reaches(hot, cold, &opposition_relation_kind()),
            "hot opposes cold through the materialized Opposition closure"
        );
        assert!(
            composed.reaches(cold, hot, &opposition_relation_kind()),
            "Opposition is checked both directions at query time"
        );
        assert!(
            !composed.reaches(
                hot,
                cold,
                &crate::cognitive::linguistics::relation_lexicon::subsumption_kind()
            ),
            "an Opposition edge is not a Subsumption edge — the two closures are distinct"
        );
    }

    /// The inline WN-LMF fixture the pipeline test loads — the same shape as the
    /// one `English::sample()` embeds, cut to the dog ⊑ mammal witness pair.
    const SAMPLE_LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="test" label="Test" language="en" email="" license="" version="1.0" url="">
    <LexicalEntry id="e-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="dog-n-01" synset="s-dog"/></LexicalEntry>
    <LexicalEntry id="e-mammal-n"><Lemma writtenForm="mammal" partOfSpeech="n"/><Sense id="mammal-n-01" synset="s-mammal"/></LexicalEntry>
    <Synset id="s-dog" ili="i1" partOfSpeech="n"><Definition>a domesticated canine</Definition><SynsetRelation relType="hypernym" target="s-mammal"/></Synset>
    <Synset id="s-mammal" ili="i3" partOfSpeech="n"><Definition>warm-blooded vertebrate</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
}
