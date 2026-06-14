//! OWL → praxis, the functor-as-data way — the OWL analog of the WordNet
//! [`english::bridge`](crate::cognitive::linguistics::english::bridge), built on
//! the SAME three-piece pattern so the projection is ontological, not a baked
//! converter.
//!
//! # The relabeling is data, not code
//!
//! [`owl_project_archive`] is the STRUCTURAL transcription only: it walks a
//! [`LoadedOwlVocabulary`] into a content-addressed source [`Archive`] carrying
//! the RAW OWL schema generators — node kind [`CLASS_KIND`] / [`OBJECT_PROPERTY_KIND`]
//! (the `owl:Class` / `owl:ObjectProperty` metaclass tags, W3C OWL 2 §5), edges
//! under [`SUBSUMES_REL`] (the is-a relation), name = IRI, lexical = `rdfs:comment`.
//! NO praxis kind is baked here.
//!
//! Mapping those raw generators to praxis kinds (`owl:Class → Concept`,
//! `subsumes → Subsumption`) is a separate FUNCTOR carried AS `.prx` DATA
//! ([`owl_to_praxis_functor`]) and interpreted by the ONE runtime primitive
//! [`apply`](pr4xis_runtime::apply::apply) — the finite action on generators
//! (Lawvere functorial semantics; Fong & Spivak *Seven Sketches* Ch. 3). So the
//! relation-kind table is data that re-emits to update — never a hardcoded
//! `match rel_type`. [`owl_runtime_ontology`] is the whole pipeline
//! (`project → apply(functor) → materialize`), the verbatim shape of
//! [`english_runtime_ontology`](crate::cognitive::linguistics::english::bridge::english_runtime_ontology).
//!
//! # The merged is-a relation (subClassOf ∪ subPropertyOf)
//!
//! [`LoadedOwlVocabulary`] already merges `rdfs:subClassOf` and
//! `rdfs:subPropertyOf` into one index-keyed `subsumption` graph
//! ([`from_owl_ontology`](LoadedOwlVocabulary::from_owl_ontology)), so the raw
//! syntactic relation name is gone before this projector runs. The projection
//! therefore emits ONE neutral raw relation [`SUBSUMES_REL`] (`"subsumes"`) for
//! both — faithful because the W3C RDF 1.1 Semantics give `subClassOf` and
//! `subPropertyOf` structurally identical conditions (each a reflexive+transitive
//! subset-inclusion preorder — a thin is-a category), so collapsing them onto one
//! `Subsumption` morphism generator preserves the is-a hom-structure. The OBJECT
//! sort that DOES differ (a class is not a property, OWL 2 §5.1/§5.3) is preserved
//! in `map_object` (`Class → Concept`, `ObjectProperty → Relation`).
//!
//! # Scope: the is-a closure (NL-query deferred)
//!
//! Only the subsumption (is-a) graph is projected — exactly what the catalog +
//! the reasoner's subsumption closure need. The projected node NAME is the IRI
//! (the entity's globally-unique identity, the referent edges name), so — like a
//! URN-named USC section — an IRI does not tokenize as natural language; NL
//! queries over loaded OWL await the lexical-`Form` surfacing
//! (`docs/praxis-self-aware-architecture` §9 / Step 1b), the same deferral USC has.

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_runtime::apply::apply;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::connection::{Connection, GeneratorAction};
use pr4xis_runtime::definition::{Definition, EdgeTarget};
use pr4xis_runtime::ontology::{MaterializeError, RuntimeOntology, materialize};

use super::vocabulary::{LoadedOwlVocabulary, OwlEntityKind};

/// The raw node kind of an `owl:Class` in the SOURCE archive — the W3C OWL 2 §5.1
/// metaclass tag the projector reads off the struct, before the functor relabels
/// it to [`CONCEPT_KIND`].
pub const CLASS_KIND: &str = "Class";

/// The raw node kind of an `owl:ObjectProperty` in the SOURCE archive — the W3C
/// OWL 2 §5.3 metaclass tag, before the functor relabels it to [`RELATION_KIND`].
pub const OBJECT_PROPERTY_KIND: &str = "ObjectProperty";

/// The raw is-a relation name in the SOURCE archive — the neutral name for the
/// already-merged `rdfs:subClassOf ∪ rdfs:subPropertyOf` graph (RDF Schema
/// §2.1 / §5.1.7), the schema generator the praxis functor maps to
/// [`SUBSUMPTION_REL`].
pub const SUBSUMES_REL: &str = "subsumes";

/// The praxis concept kind an `owl:Class` relabels to — appears ONLY in the
/// functor DATA, never baked into the structural projection.
pub const CONCEPT_KIND: &str = "Concept";

/// The praxis kind an `owl:ObjectProperty` relabels to — preserves the OWL object
/// sort distinction (a property is not a class). A carried kind label, not folded
/// (only the EDGE kind drives the closure); appears ONLY in the functor DATA.
pub const RELATION_KIND: &str = "Relation";

/// The praxis relation kind a `subsumes` edge relabels to — one of the canonically
/// transitive kinds [`materialize`] folds into the is-a closure.
pub const SUBSUMPTION_REL: &str = "Subsumption";

/// Project a loaded OWL vocabulary into a content-addressed SOURCE [`Archive`] —
/// the structural functor `LoadedOwlVocabulary → Archive`, carrying RAW OWL
/// generator names (no praxis kind baked).
///
/// Each [`OwlEntityRecord`](super::vocabulary::OwlEntityRecord) → a [`Definition`]
/// `{kind: raw owl metaclass tag, name: iri, lexical: rdfs:comment (None when
/// absent), edges: [(`[`SUBSUMES_REL`]`, parent_iri)]}`. Every subsumption parent
/// is a declared entity (the vocabulary dropped danglers on construction), so the
/// archive is referentially closed and [`materialize`]s.
pub fn owl_project_archive(vocab: &LoadedOwlVocabulary) -> Archive {
    let entities = vocab.entities();

    // Group the index-keyed subsumption edges by CHILD into its parents' IRIs —
    // the by-name (content-address-agreement) edge targets within this archive.
    // One edge per (child, parent) pair, so the morphism count is preserved.
    let mut parents_of: Vec<Vec<String>> = (0..entities.len()).map(|_| Vec::new()).collect();
    for &(child, parent) in vocab.subsumption_edges() {
        parents_of[child].push(entities[parent].iri.clone());
    }

    let nodes = entities
        .iter()
        .enumerate()
        .map(|(idx, entity)| {
            // Reading the source struct's OWN metaclass tag (the raw generator) —
            // the projector's irreducible transcription job, NOT a semantic map;
            // the praxis kind (Concept/Relation) lives only in the functor data.
            let kind = match entity.kind {
                OwlEntityKind::Class => CLASS_KIND,
                OwlEntityKind::ObjectProperty => OBJECT_PROPERTY_KIND,
            };
            let lexical = (!entity.definition.is_empty()).then(|| entity.definition.clone());
            let edges = parents_of[idx]
                .iter()
                .map(|parent_iri| {
                    (
                        SUBSUMES_REL.to_string(),
                        EdgeTarget::Local(parent_iri.clone()),
                    )
                })
                .collect();
            Definition {
                kind: kind.to_string(),
                name: entity.iri.clone(),
                edges,
                axioms: Vec::new(),
                lexical,
            }
        })
        .collect();

    Archive {
        nodes,
        connections: Vec::new(),
    }
}

/// The OWL → praxis projection, carried AS DATA — the [`Connection`] a `.prx`
/// ships so the relabeling re-emits to update with no recompile.
///
/// A functor's whole content is its finite action on the schema's generators, and
/// praxis serializes exactly that as [`GeneratorAction::Functor`]. So
/// `owl:Class ↦ Concept`, `owl:ObjectProperty ↦ Relation`, `subsumes ↦ Subsumption`
/// is not a compiled `match` — it is this data, interpreted by [`apply`] over an
/// [`owl_project_archive`] source. `map_morphism` is a single row because the
/// source already merged `subClassOf ∪ subPropertyOf` (both is-a) into one
/// `subsumes` generator; `map_object` keeps the class/property sort distinction.
pub fn owl_to_praxis_functor() -> Connection {
    Connection {
        kind: "Faithful".to_string(),
        source: "OwlVocabulary".to_string(),
        target: "PraxisOntology".to_string(),
        action: GeneratorAction::Functor {
            map_object: vec![
                (CLASS_KIND.to_string(), CONCEPT_KIND.to_string()),
                (OBJECT_PROPERTY_KIND.to_string(), RELATION_KIND.to_string()),
            ],
            map_morphism: vec![(SUBSUMES_REL.to_string(), SUBSUMPTION_REL.to_string())],
        },
        laws: vec![
            "PreservesIdentity".to_string(),
            "PreservesComposition".to_string(),
        ],
    }
}

/// Bridge a loaded OWL vocabulary into a generic [`RuntimeOntology`] — the whole
/// pipeline in one call: [`owl_project_archive`] → [`apply`]`(`[`owl_to_praxis_functor`]`)`
/// → [`materialize`]. The verbatim shape of
/// [`english_runtime_ontology`](crate::cognitive::linguistics::english::bridge::english_runtime_ontology).
///
/// `apply` cannot fail here: [`owl_to_praxis_functor`] is always a `Functor`
/// action (the only action `apply` interprets), so its sole error is unreachable.
/// Materialization can still fail closed (a codec error on the root); that error
/// is propagated typed.
pub fn owl_runtime_ontology(
    vocab: &LoadedOwlVocabulary,
    name: OntologyName,
) -> Result<RuntimeOntology, MaterializeError> {
    let source = owl_project_archive(vocab);
    let praxis = apply(&owl_to_praxis_functor().action, &source)
        .expect("owl_to_praxis_functor is a Functor action, which apply always interprets");
    materialize(praxis, name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::owl::reader::read_owl;
    use pr4xis_runtime::ontology::subsumption_kind;

    // Animal ← Mammal ← {Dog}, and Dog ALSO ⊑ Pet — a multi-parent class, so the
    // edge-grouping (one edge per (child,parent)) and the morphism count are
    // exercised, not just a chain.
    const SAMPLE_OWL: &str = r#"<?xml version="1.0"?>
<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:rdfs="http://www.w3.org/2000/01/rdf-schema#"
         xmlns:owl="http://www.w3.org/2002/07/owl#">
  <owl:Class rdf:about="http://example.org/test#Animal">
    <rdfs:comment>A living organism.</rdfs:comment>
  </owl:Class>
  <owl:Class rdf:about="http://example.org/test#Pet"/>
  <owl:Class rdf:about="http://example.org/test#Mammal">
    <rdfs:subClassOf rdf:resource="http://example.org/test#Animal"/>
  </owl:Class>
  <owl:Class rdf:about="http://example.org/test#Dog">
    <rdfs:subClassOf rdf:resource="http://example.org/test#Mammal"/>
    <rdfs:subClassOf rdf:resource="http://example.org/test#Pet"/>
  </owl:Class>
</rdf:RDF>"#;

    fn sample_vocab() -> LoadedOwlVocabulary {
        let ont = read_owl(SAMPLE_OWL).expect("parse sample OWL");
        LoadedOwlVocabulary::from_owl_ontology(&ont)
    }

    #[test]
    fn project_emits_raw_owl_generators_not_praxis_kinds() {
        // The structural projection carries the RAW OWL names — never a baked
        // praxis kind. The praxis relabel is the functor's job (next test).
        let archive = owl_project_archive(&sample_vocab());
        let dog = archive
            .nodes
            .iter()
            .find(|n| n.name == "http://example.org/test#Dog")
            .expect("Dog node");
        assert_eq!(dog.kind, CLASS_KIND, "raw owl:Class tag, not 'Concept'");
        assert!(
            dog.edges.iter().all(|(rel, _)| rel == SUBSUMES_REL),
            "raw is-a relation 'subsumes', not 'Subsumption'; got {:?}",
            dog.edges
        );
        assert_eq!(dog.edges.len(), 2, "Dog's two superclasses are both edges");
    }

    #[test]
    fn projection_preserves_the_catalog_stats() {
        // One node per entity, one edge per subsumption pair — the counts wasm's
        // catalog reports must survive the projection (a multi-parent grouping bug
        // would drop or duplicate an edge and fail here).
        let vocab = sample_vocab();
        let archive = owl_project_archive(&vocab);
        assert_eq!(archive.nodes.len(), vocab.entity_count());
        let edges: usize = archive.nodes.iter().map(|n| n.edges.len()).sum();
        assert_eq!(edges, vocab.subsumption_edge_count());
    }

    #[test]
    fn functor_carries_the_relabeling_as_data() {
        // The semantic map is DATA on the Connection, re-aimable without recompile.
        let GeneratorAction::Functor {
            map_object,
            map_morphism,
        } = owl_to_praxis_functor().action
        else {
            panic!("owl_to_praxis_functor must be a Functor action");
        };
        assert!(map_object.contains(&(CLASS_KIND.to_string(), CONCEPT_KIND.to_string())));
        assert!(
            map_object.contains(&(OBJECT_PROPERTY_KIND.to_string(), RELATION_KIND.to_string()))
        );
        assert_eq!(
            map_morphism,
            vec![(SUBSUMES_REL.to_string(), SUBSUMPTION_REL.to_string())]
        );
    }

    #[test]
    fn pipeline_materializes_a_real_subsumption_closure() {
        // The full functor-as-data pipeline: raw 'subsumes' → apply → 'Subsumption'
        // → materialize folds the is-a closure. Dog ⊑ Mammal ⊑ Animal collapses to
        // Dog → Animal (Subsumption is a loaded transitive kind — Phase A).
        let onto = owl_runtime_ontology(&sample_vocab(), OntologyName::new_static("owl_test"))
            .expect("the projected OWL ontology materializes");
        let dog = onto.concept("http://example.org/test#Dog");
        let animal = onto.concept("http://example.org/test#Animal");
        assert!(
            onto.reachable_from(&dog, subsumption_kind())
                .contains(&animal),
            "Dog must transitively subsume under Animal through the projected closure"
        );
        // The rdfs:comment rode the projection (unchanged by apply) as the gloss.
        assert_eq!(onto.lexical(&animal), Some("A living organism."));
    }
}
