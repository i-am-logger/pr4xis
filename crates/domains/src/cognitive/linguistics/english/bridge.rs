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
//! `Synset → Concept` is a separate FUNCTOR carried as `.prx` data
//! ([`wordnet_to_praxis_functor`](crate::cognitive::linguistics::english::bridge::wordnet_to_praxis_functor)) and
//! interpreted by the one runtime primitive
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

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::apply::apply;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::connection::{Connection, GeneratorAction};
use pr4xis_runtime::definition::{Definition, EdgeTarget};
use pr4xis_runtime::ontology::{ConceptRef, MaterializeError, RuntimeOntology, materialize};

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
/// constant so the projection and [`wordnet_to_praxis_functor`] name the same
/// source generator.
pub const SYNSET_KIND: &str = "Synset";

/// The raw WordNet relation name a hypernym (is-a) edge carries in the SOURCE
/// archive — the schema generator the praxis functor maps to `Subsumption`.
pub const HYPERNYM_REL: &str = "hypernym";

/// The praxis kind of a written-form atom — an `ontolex:Form` (its `writtenRep`),
/// carrying NO sense. The honest target of the lexical `denotes` floor: a span
/// grounds into "this written form occurred", never into a meaning.
pub const FORM_KIND: &str = "Form";

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
                    english.concept(parent).map(|p| {
                        (
                            HYPERNYM_REL.to_string(),
                            EdgeTarget::Local(p.original_id.clone()),
                        )
                    })
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

/// The `ontolex:Form` atom for a written representation — a bare surface-form
/// node carrying its `writtenRep` as both `name` and `lexical`, and NOTHING
/// else: no sense, no synset edge. Its content [`address`](Definition::address)
/// is what a lexical `denotes` floor edge points AT.
///
/// Sense-deferral is STRUCTURAL: a Form has no senses, so a pointer that resolves
/// to one cannot have over-committed to a meaning (the written-form floor's
/// honesty tripwire — a `denotes` edge must land on a `Form`, never a synset).
pub fn form_atom(written_rep: &str) -> Definition {
    Definition {
        kind: FORM_KIND.to_string(),
        name: written_rep.to_string(),
        edges: Vec::new(),
        axioms: Vec::new(),
        lexical: Some(written_rep.to_string()),
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
        .extend(english.word_index.keys().map(|word| form_atom(word)));
    archive
}

/// The WordNet → praxis projection, carried AS DATA — the [`Connection`] node a
/// `.prx` ships so the relabeling re-emits to update with no recompile.
///
/// The whole content of a functor is its finite action on the schema's
/// generators (the finite-presentation theorem), and praxis already serializes
/// exactly that as [`GeneratorAction::Functor`]. So the map
/// `Synset ↦ Concept`, `hypernym ↦ Subsumption` is not a compiled `match` — it
/// is this data, interpreted by [`apply`] over a
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

/// Bridge the loaded [`English`] struct into a generic [`RuntimeOntology`] — the
/// whole B1 pipeline in one call: `English` → [`project_archive`] →
/// [`apply`]`(`[`wordnet_to_praxis_functor`]`)` → [`materialize`].
///
/// The result is a source-agnostic runtime ontology a generic engine reasons
/// over (`is_a` → `Verdict`, `reachable_from`, `lexical`), exactly as it would
/// over a `.prx` loaded from disk — the SUBSTRATE SPLIT dissolved. English's
/// hypernym taxonomy is now an addressable, traversable graph of content-
/// addressed atoms, not a closed domain struct.
///
/// `apply` cannot fail here: [`wordnet_to_praxis_functor`] is always a
/// `Functor` action (the only action `apply` interprets), so the sole
/// [`ApplyError`](pr4xis_runtime::apply::ApplyError) is unreachable — treated as
/// a structural invariant. Materialization can still fail closed (a codec error
/// on the root); that error is propagated typed.
pub fn english_runtime_ontology(english: &English) -> Result<RuntimeOntology, MaterializeError> {
    let source = project_archive(english);
    let praxis = apply(&wordnet_to_praxis_functor().action, &source)
        .expect("wordnet_to_praxis_functor is a Functor action, which apply always interprets");
    materialize(praxis, OntologyName::new_static(ENGLISH_ONTOLOGY))
}

/// The runtime [`ConceptRef`]s the English senses of `word` denote in `onto` —
/// the Lemon ground (word → synset, via English's lexicon) composed with the
/// runtime identity (a synset's `original_id` IS its node name in `onto`).
///
/// This is the thin lexical VIEW onto the generic ontology: the lexicon stays
/// English (the only thing that knows "dog" denotes synset `s-dog`), while the
/// REASONING substrate is the source-agnostic `onto`. One word yields many refs
/// (polysemy); the caller decides over them (e.g. "is a dog an animal" holds if
/// SOME sense pair does).
pub fn concept_refs_for_word(
    onto: &RuntimeOntology,
    english: &English,
    word: &str,
) -> Vec<ConceptRef> {
    english
        .lookup(word)
        .iter()
        .filter_map(|&id| english.concept(id))
        .map(|concept| onto.concept(concept.original_id.clone()))
        .collect()
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

    /// A local (same-ontology) edge target — what every projected hypernym edge
    /// is (a synset id within `english_wordnet`).
    fn local(name: &str) -> EdgeTarget {
        EdgeTarget::Local(name.to_string())
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
            alloc::vec![(HYPERNYM_REL.to_string(), local("s-mammal"))],
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
                let name = t.local_name().expect("only local edges projected");
                assert!(
                    declared.contains(name),
                    "relabeled edge target {name} undeclared"
                );
            }
        }
    }

    // --- piece 4: the bridge to a RuntimeOntology + the grounding gate ---

    #[test]
    fn the_bridge_materializes_a_runtime_ontology() {
        let onto = english_runtime_ontology(&English::sample()).expect("English materializes");
        assert_eq!(onto.id().as_str(), ENGLISH_ONTOLOGY);
        // The is-a generators folded into the Subsumption closure: dog ⊑ mammal
        // ⊑ animal collapses, so dog reaches animal.
        let dog = onto.concept("s-dog");
        let animal = onto.concept("s-animal");
        assert!(
            onto.reachable_from(&dog, pr4xis_runtime::ontology::subsumption_kind())
                .contains(&animal),
            "the transitive is-a closure must put animal in dog's Subsumption image"
        );
        // The gloss rode the whole pipeline (projection → apply → materialize).
        assert_eq!(onto.lexical(&dog), Some("a domesticated canine"));
    }

    /// THE GATE (fast lane, miniature corpus): "is a dog an animal" answered over
    /// the GENERIC RuntimeOntology — its materialized Subsumption closure, via
    /// typed `ConceptRef`s resolved through English's lexicon. The claim IS the
    /// Verdict (pattern-matched, never `.is_ok()`).
    #[test]
    fn gate_is_a_dog_an_animal_over_the_runtime_ontology() {
        let english = English::sample();
        let onto = english_runtime_ontology(&english).expect("English materializes");

        let dog = concept_refs_for_word(&onto, &english, "dog");
        let animal = concept_refs_for_word(&onto, &english, "animal");
        assert!(
            !dog.is_empty() && !animal.is_empty(),
            "lexicon resolves both words"
        );

        // Some (dog sense, animal sense) pair must witness the is-a relation.
        let witness = dog
            .iter()
            .flat_map(|d| animal.iter().map(move |a| (d, a)))
            .find_map(|(d, a)| onto.is_a(d, a).ok().map(|proof| (a.clone(), proof)));
        match witness {
            Some((animal_ref, proof)) => {
                let claim = proof.meta().name;
                assert!(
                    claim.as_str().contains("s-dog") && claim.as_str().contains(&animal_ref.name),
                    "the proof must name the witnessed dog ⊑ animal claim; got {claim}"
                );
            }
            None => panic!("the engine must witness 'a dog is an animal' over the loaded ontology"),
        }

        // And it does NOT over-claim the converse: an animal is not a dog. The
        // generic engine refutes (honest counterexample), like English itself.
        let any_animal_is_a_dog = animal
            .iter()
            .flat_map(|a| dog.iter().map(move |d| (a, d)))
            .any(|(a, d)| onto.is_a(a, d).is_ok());
        assert!(!any_animal_is_a_dog, "animal is-a dog must refute");
    }

    #[test]
    fn the_runtime_answer_agrees_with_english_is_a() {
        // The bridge is FAITHFUL: the generic engine's verdict matches English's
        // own bespoke hypernym closure for the same pair — same answer, different
        // (source-agnostic) substrate.
        use crate::cognitive::linguistics::english::LexicalReasoner;
        let english = English::sample();
        let onto = english_runtime_ontology(&english).expect("materializes");

        let dog_id = english.lookup("dog")[0];
        let animal_id = english.lookup("animal")[0];
        let english_says = LexicalReasoner::is_a(&english, dog_id, animal_id);

        let dog_ref = onto.concept(english.concept(dog_id).unwrap().original_id.clone());
        let animal_ref = onto.concept(english.concept(animal_id).unwrap().original_id.clone());
        let engine_says = onto.is_a(&dog_ref, &animal_ref).is_ok();

        assert_eq!(
            english_says, engine_says,
            "the bridge must agree with English's is_a"
        );
        assert!(engine_says, "and both say dog is-a animal");
    }

    // --- G3b: the honest `denotes` floor — a span grounds into an ontolex:Form ---

    use alloc::collections::BTreeMap;
    use pr4xis_runtime::grounding::{AtomResolver, ConnectedOntologies, ConnectedOntology};

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
