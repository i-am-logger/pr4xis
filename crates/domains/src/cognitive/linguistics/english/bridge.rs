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

use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec::Vec;

use pr4xis_runtime::address::ContentAddress;
// `apply` is used only by the test module (the corpus-scale consumer is the
// archive-level functor gate in `praxis-corpus-tests`); gate it so non-test
// builds don't see an unused import.
#[cfg(test)]
use pr4xis_runtime::apply::apply;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::connection::Connection;
use pr4xis_runtime::definition::{Definition, EdgeTarget};

use super::ontology::English;

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
        .map(|concept| Definition {
            kind: SYNSET_KIND.to_string(),
            name: concept.original_id().to_string(),
            edges: english
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
                .collect(),
            axioms: Vec::new(),
            lexical: concept.definitions().next().map(|d| d.to_string()),
        })
        .collect();

    Archive {
        nodes,
        connections: Vec::new(),
    }
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

/// The reverse index a cross-ontology `reaches` needs to resolve an into-English
/// grounded edge: each synset node's content address → its `original_id` (the
/// synset name, e.g. `s-dog`) — the inverse of the addressing
/// [`project_archive`] mints.
///
/// DERIVED from [`project_archive`] (the same synset nodes an into-English
/// grounding functor targets by content address via
/// [`type_lens`](pr4xis_runtime::grounding::type_lens); the Form atoms
/// [`project_archive_with_forms`] appends do not change a synset node's address,
/// so the synset-only projection and the with-forms grounding target agree on
/// every synset address). COUPLING-FREE: it knows only `&English`, never the
/// loaded set or which atoms are grounded into — the CALLER
/// ([`ComposedReasoner`](crate::cognitive::linguistics::composed::ComposedReasoner))
/// retains only the edge-targeted subset, so the resident index is bounded by
/// grounded-target count, not synset count.
pub fn english_synset_atoms(english: &English) -> BTreeMap<ContentAddress, alloc::string::String> {
    project_archive(english)
        .nodes
        .iter()
        .filter_map(|n| n.address().ok().map(|addr| (addr, n.name.clone())))
        .collect()
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
    "55c3a8cabc2f52cb42e0e10d6a232c881f50e91a24ec35e06dadda6550c3cc1f";

/// Load the WordNet→praxis functor from its committed `.prx`
/// ([`ENGLISH_FUNCTOR_PRX`]) — FAIL-CLOSED: the embedded bytes are admitted only
/// if they re-derive to [`ENGLISH_FUNCTOR_ROOT_HEX`], so a tampered or stale
/// projection is refused, never silently mis-applied. Reuses the kernel
/// [`load`](pr4xis_runtime::load::load); no new runtime API. The functor is a
/// finite action on generators (the finite-presentation theorem; Fong & Spivak
/// *Seven Sketches* Ch. 3), interpreted by [`apply`](pr4xis_runtime::apply::apply) over a [`project_archive`]
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
}
