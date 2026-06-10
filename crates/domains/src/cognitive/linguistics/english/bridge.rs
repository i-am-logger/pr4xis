//! The WordNet → `.prx` bridge — project the loaded [`English`] domain struct
//! into a content-addressed runtime [`Archive`], then relabel it into the
//! praxis schema with a projection carried AS DATA.
//!
//! This is the domains half of the B1 engine bridge (#87): it dissolves the
//! SUBSTRATE SPLIT. `english_loaded()` hands back an [`English`] — a closed
//! domain struct whose synsets are positional `Reference<4>` indices and whose
//! relations are typed `HashMap`s. That is NOT a runtime [`Archive`], so a
//! generic engine has no addressable atom to point at and no traverser to
//! follow. [`project_archive`] is the functor `English → Archive` that gives
//! each synset a definition-bearing [`ContentAddress`](pr4xis_runtime::address)
//! and re-expresses its hypernym links as runtime edges.
//!
//! # The relabeling is data, not code
//!
//! The projection emits each synset edge under its RAW WordNet relation name
//! (`hypernym`), NOT a praxis kind. Mapping `hypernym → Subsumption`,
//! `Synset → Concept` is a separate FUNCTOR carried as `.prx` data
//! ([`wordnet_to_praxis_functor`]) and interpreted by the one runtime primitive
//! [`apply`](pr4xis_runtime::apply::apply). So the relation-kind table is data
//! that re-emits to update — never the hardcoded `match rel_type`
//! (`pr4xis::codegen::wordnet`) the old codegen path baked in.
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

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::connection::{Connection, GeneratorAction};
use pr4xis_runtime::definition::Definition;

use super::ontology::English;

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
/// constant so the projection and [`wordnet_to_praxis_functor`] name the same
/// source generator.
pub const SYNSET_KIND: &str = "Synset";

/// The raw WordNet relation name a hypernym (is-a) edge carries in the SOURCE
/// archive — the schema generator the praxis functor maps to `Subsumption`.
pub const HYPERNYM_REL: &str = "hypernym";

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
        .concepts
        .iter()
        .map(|concept| Definition {
            kind: SYNSET_KIND.to_string(),
            name: concept.original_id.clone(),
            edges: english
                .parents(concept.id)
                .iter()
                .filter_map(|&parent| {
                    english
                        .concept(parent)
                        .map(|p| (HYPERNYM_REL.to_string(), p.original_id.clone()))
                })
                .collect(),
            axioms: Vec::new(),
            lexical: concept.definitions.first().cloned(),
        })
        .collect();

    Archive {
        nodes,
        connections: Vec::new(),
    }
}

/// The WordNet → praxis projection, carried AS DATA — the [`Connection`] node a
/// `.prx` ships so the relabeling re-emits to update with no recompile.
///
/// The whole content of a functor is its finite action on the schema's
/// generators (the finite-presentation theorem), and praxis already serializes
/// exactly that as [`GeneratorAction::Functor`]. So the map
/// `Synset ↦ Concept`, `hypernym ↦ Subsumption` is not a compiled `match` — it
/// is this data, interpreted by [`apply`](pr4xis_runtime::apply::apply) over a
/// [`project_archive`] source. Re-emitting this node with a different table
/// (say `hypernym ↦ Parthood`) re-aims the projection without touching code —
/// the directive "projections live in `.prx`, not code" realized.
///
/// It is faithful by construction: distinct source generators map to distinct
/// praxis generators (an injective relabeling — an inclusion of the WordNet
/// is-a schema into the praxis schema), so it preserves the hom-set structure.
/// The [`laws`](Connection::laws) it must satisfy are carried as NAMES (data);
/// resolving those to runnable axioms at materialize time is the documented
/// deferral in [`materialize`](pr4xis_runtime::ontology), not done here.
///
/// Scope tracks [`project_archive`]: the is-a generator only. As the projection
/// grows the full GWN relation vocabulary (the meronymy / antonymy follow-up),
/// this table grows the corresponding rows — additively, still as data.
pub fn wordnet_to_praxis_functor() -> Connection {
    Connection {
        kind: "Faithful".to_string(),
        source: "EnglishWordNet".to_string(),
        target: "PraxisOntology".to_string(),
        action: GeneratorAction::Functor {
            map_object: vec![(SYNSET_KIND.to_string(), CONCEPT_KIND.to_string())],
            map_morphism: vec![(HYPERNYM_REL.to_string(), SUBSUMPTION_REL.to_string())],
        },
        laws: vec![
            "PreservesIdentity".to_string(),
            "PreservesComposition".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeSet;

    fn node<'a>(archive: &'a Archive, name: &str) -> &'a Definition {
        archive
            .nodes
            .iter()
            .find(|n| n.name == name)
            .unwrap_or_else(|| panic!("archive must declare synset {name:?}"))
    }

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

    #[test]
    fn a_synset_carries_its_hypernym_edge_and_gloss() {
        let archive = project_archive(&English::sample());
        let dog = node(&archive, "s-dog");
        assert_eq!(
            dog.edges,
            alloc::vec![(HYPERNYM_REL.to_string(), "s-mammal".to_string())],
            "dog's only generating edge is the raw hypernym → its parent synset id"
        );
        assert_eq!(
            dog.lexical.as_deref(),
            Some("a domesticated canine"),
            "the synset's gloss rides into the node's lexical grounding"
        );
    }

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
                assert!(
                    declared.contains(target.as_str()),
                    "edge {}--{kind}-->{target} names an undeclared synset",
                    n.name
                );
            }
        }
    }

    #[test]
    fn the_is_a_chain_is_present_as_generating_edges() {
        // dog ⊑ mammal ⊑ animal must appear as two raw hypernym generators that
        // the functor + materialize will later fold into the is-a closure.
        let archive = project_archive(&English::sample());
        assert_eq!(
            node(&archive, "s-dog").edges,
            alloc::vec![(HYPERNYM_REL.to_string(), "s-mammal".to_string())]
        );
        assert_eq!(
            node(&archive, "s-mammal").edges,
            alloc::vec![(HYPERNYM_REL.to_string(), "s-animal".to_string())]
        );
    }

    // --- piece 3: the functor carried as data, applied ---

    use pr4xis_runtime::apply::apply;

    #[test]
    fn the_functor_is_the_relabeling_table_as_data() {
        let functor = wordnet_to_praxis_functor();
        match &functor.action {
            GeneratorAction::Functor {
                map_object,
                map_morphism,
            } => {
                assert_eq!(
                    map_object,
                    &alloc::vec![(SYNSET_KIND.to_string(), CONCEPT_KIND.to_string())],
                    "the object generator Synset maps to the praxis Concept kind"
                );
                assert_eq!(
                    map_morphism,
                    &alloc::vec![(HYPERNYM_REL.to_string(), SUBSUMPTION_REL.to_string())],
                    "the morphism generator hypernym maps to Subsumption"
                );
            }
            other => panic!("the WordNet projection is a Functor action; got {other:?}"),
        }
        // It is content-addressable data — a `.prx` node, not code.
        assert!(functor.address().is_ok());
    }

    #[test]
    fn applying_the_functor_relabels_synset_kinds_into_praxis_kinds() {
        let source = project_archive(&English::sample());
        let target =
            apply(&wordnet_to_praxis_functor().action, &source).expect("a Functor action applies");

        // Same cardinality — the functor relabels, never drops.
        assert_eq!(target.nodes.len(), source.nodes.len());

        let dog = node(&target, "s-dog");
        assert_eq!(
            dog.kind, CONCEPT_KIND,
            "Synset relabeled to the praxis Concept kind"
        );
        assert_eq!(
            dog.edges,
            alloc::vec![(SUBSUMPTION_REL.to_string(), "s-mammal".to_string())],
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

    #[test]
    fn the_relabeled_archive_is_still_referentially_closed() {
        // apply carries edge targets unchanged, so the functor's image is closed
        // exactly when its source is — the precondition materialize needs holds
        // after relabeling, not only before.
        let source = project_archive(&English::sample());
        let target = apply(&wordnet_to_praxis_functor().action, &source).unwrap();
        let declared: BTreeSet<&str> = target.nodes.iter().map(|n| n.name.as_str()).collect();
        for n in &target.nodes {
            for (_, t) in &n.edges {
                assert!(
                    declared.contains(t.as_str()),
                    "relabeled edge target {t} undeclared"
                );
            }
        }
    }
}
