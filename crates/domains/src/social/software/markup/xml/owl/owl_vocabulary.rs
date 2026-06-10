//! OWL vocabulary → praxis [`CodegenData`](pr4xis::codegen_data::CodegenData) codegen.
//!
//! Turns a loaded [`OwlOntology`] (the output of
//! [`super::reader::read_owl`]) into a populated
//! [`pr4xis::codegen::OntologyBuilder`], then hands that builder to the
//! existing [`pr4xis::codegen::generate_rust`] emitter — the same
//! build-time → runtime transport every other praxis source uses
//! (WordNet → English, USLM → UsCode, statutes → Statute). No new
//! emitter, no new IR: this module is the thin adapter from the OWL
//! reader's loaded ontology to the established `OntologyBuilder` shape.
//!
//! ## Where this lives, and why
//!
//! The OWL reader ([`super::reader::read_owl`]) and its loaded types
//! ([`OwlOntology`], [`OwlClass`](super::ontology::OwlClass), [`OwlObjectProperty`](super::ontology::OwlObjectProperty)) live in this
//! crate (`pr4xis-domains`). The codegen machinery
//! ([`pr4xis::codegen::OntologyBuilder`] / `generate_rust` /
//! `GenerateConfig`) lives in `pr4xis` core, behind its `codegen`
//! feature. `pr4xis-domains` depends on `pr4xis` (never the reverse),
//! so the adapter that needs *both* the loaded `OwlOntology` and the
//! core emitter must live here, on the domains side — mirroring the
//! crate boundary the USLM / WordNet codegens already respect (core
//! parses formats it owns a parser for; domains owns the OWL parser,
//! so the OWL → builder step is a domains-side build/test helper).
//!
//! It is gated on `any(test, feature = "codegen")` because
//! `pr4xis::codegen` is only present when `pr4xis`'s `codegen` feature
//! is enabled — which happens in this crate's `[build-dependencies]`
//! and `[dev-dependencies]`, not the normal (WASM-facing) dep.
//!
//! ## Mapping (`owl_to_builder`)
//!
//! - **Entities.** One [`EntityDef`] per OWL class AND one per object
//!   property. The entity `id` is the IRI verbatim; `kind` (carried in
//!   the builder's `pos` field, emitted as `entity_kind`) is `"Class"`
//!   for classes and `"ObjectProperty"` for properties; `label` is
//!   `rdfs:label`, falling back to the IRI's local name (the substring
//!   after the last `#` or `/`) when absent; `definition` is
//!   `rdfs:comment`, empty when absent.
//! - **Taxonomy.** The union of the class hierarchy
//!   ([`OwlOntology::taxonomy`], `rdfs:subClassOf`) and the property
//!   hierarchy ([`OwlOntology::property_taxonomy`],
//!   `rdfs:subPropertyOf`). Both are `(child, parent)` edges that stay
//!   within a single kind (a class is never a subclass of a property,
//!   per W3C OWL 2 §5 / RDF Schema §2.1), so the union does not mix
//!   kinds. Each edge maps to the entity ids of its endpoints; an edge
//!   whose endpoint is not itself a declared entity (e.g. a subclass of
//!   the external `owl:Thing`) is dropped by `generate_rust`'s id
//!   resolution.
//!
//! ## Deferred (not mapped in this first pass)
//!
//! `owl:inverseOf`, `rdfs:domain`, and `rdfs:range` are **not** mapped
//! here. Mereology / opposition / equivalence / causation / references
//! are left empty. There is no clean, single-source mapping from these
//! OWL constructs onto the builder's relation slots yet — inventing one
//! would be approximation, not loading. They are tracked as follow-on
//! work (#253.b / #253.c, the runtime Category + `from_codegen` functor)
//! and added once their target relation kind is grounded in literature.
//!
//! ## Citations
//!
//! - **W3C OWL 2 Web Ontology Language: Structural Specification and
//!   Functional-Style Syntax (2nd ed.)**, Motik, Patel-Schneider &
//!   Parsia (eds.), W3C Recommendation 2012-12-11. §5 (Entities),
//!   §9.2.1 (object-property hierarchy). <https://www.w3.org/TR/owl2-syntax/>.
//! - **RDF Schema 1.1**, Brickley & Guha (eds.), W3C Recommendation
//!   2014-02-25. §2.1 (`rdfs:subClassOf`), §5.1.7 (`rdfs:subPropertyOf`),
//!   §2.4 (`rdfs:label`), §2.5 (`rdfs:comment`).
//!   <https://www.w3.org/TR/rdf-schema/>.
//! - **Shotton, D. & Peroni, S.** *CiTO, the Citation Typing
//!   Ontology* (v2.8.1). Semantic Publishing and Referencing (SPAR)
//!   Ontologies. <http://purl.org/spar/cito>.

#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use pr4xis::codegen::{EntityDef, GenerateConfig, OntologyBuilder};

use super::ontology::OwlOntology;

/// The `entity_kind` tag for an OWL class entity — the W3C OWL 2
/// metaclass name (`owl:Class`, §5.1).
const KIND_CLASS: &str = "Class";
/// The `entity_kind` tag for an OWL object-property entity — the W3C
/// OWL 2 metaclass name (`owl:ObjectProperty`, §5.4).
const KIND_OBJECT_PROPERTY: &str = "ObjectProperty";

/// Local name of an IRI: the substring after the last `#` or `/`.
///
/// W3C OWL 2 §5.5 (IRIs) and RDF 1.1 Concepts §3.2 identify a term by
/// its full IRI; the local name is a human-facing convenience used as a
/// label fallback when no `rdfs:label` is supplied. CiTO terms are
/// slash-delimited (`http://purl.org/spar/cito/citesAsEvidence`); the
/// OWL/RDFS vocabulary is hash-delimited
/// (`http://www.w3.org/2002/07/owl#Class`). Handle both.
fn local_name(iri: &str) -> &str {
    let after_hash = iri.rsplit_once('#').map(|(_, l)| l);
    match after_hash {
        Some(local) if !local.is_empty() => local,
        _ => match iri.rsplit_once('/') {
            Some((_, local)) if !local.is_empty() => local,
            _ => iri,
        },
    }
}

/// Map a loaded [`OwlOntology`] to a populated
/// [`OntologyBuilder`] — one entity per OWL class and one per object
/// property, plus the union of the class- and property-subsumption
/// hierarchies as the builder's `taxonomy` relation.
///
/// `read_owl` already deduplicates by IRI; the secondary dedupe here
/// (a seen-IRI set) is belt-and-suspenders so an entity is never
/// emitted twice even if the input was assembled by hand. Entities
/// whose IRI is empty are skipped (the OWL reader never produces those,
/// but a synthesised input might).
///
/// See the module docs for the field-by-field mapping and the deferred
/// constructs (`owl:inverseOf`, `rdfs:domain`, `rdfs:range`).
pub fn owl_to_builder(ont: &OwlOntology) -> OntologyBuilder {
    let mut builder = OntologyBuilder::new();
    // Track which IRIs have been emitted, so the taxonomy union below
    // only references declared entities and no IRI is added twice.
    let mut seen: hashbrown::HashSet<String> = hashbrown::HashSet::new();

    // One entity per OWL class (W3C OWL 2 §5.1).
    for class in &ont.classes {
        if class.iri.is_empty() || seen.contains(&class.iri) {
            continue;
        }
        seen.insert(class.iri.clone());
        let label = class
            .label
            .clone()
            .unwrap_or_else(|| local_name(&class.iri).to_string());
        let mut entity = EntityDef::new(&class.iri, &label).pos(KIND_CLASS);
        if let Some(comment) = &class.comment {
            entity = entity.definition(comment);
        }
        builder.add_entity(entity);
    }

    // One entity per OWL object property (W3C OWL 2 §5.4).
    for prop in &ont.properties {
        if prop.iri.is_empty() || seen.contains(&prop.iri) {
            continue;
        }
        seen.insert(prop.iri.clone());
        let label = prop
            .label
            .clone()
            .unwrap_or_else(|| local_name(&prop.iri).to_string());
        let mut entity = EntityDef::new(&prop.iri, &label).pos(KIND_OBJECT_PROPERTY);
        if let Some(comment) = &prop.comment {
            entity = entity.definition(comment);
        }
        builder.add_entity(entity);
    }

    // Taxonomy = union of class subsumption (rdfs:subClassOf) and
    // property subsumption (rdfs:subPropertyOf). Both are (child,
    // parent) edges that stay within a single kind, so the union does
    // not conflate the two hierarchies. `generate_rust` resolves each
    // endpoint to its entity index and silently drops edges whose
    // endpoint is not a declared entity (e.g. a subclass of the
    // external owl:Thing), so no guard is needed here.
    for (child, parent) in ont.taxonomy.iter().chain(ont.property_taxonomy.iter()) {
        builder.add_taxonomy(child, parent);
    }

    // Deferred (see module docs): owl:inverseOf, rdfs:domain,
    // rdfs:range, and the mereology / opposition / equivalence /
    // causation / references relation slots. Left empty in this pass —
    // no literature-grounded mapping yet.

    builder
}

/// Generate Rust source for a loaded OWL vocabulary: the standard
/// `pub static CODEGEN_DATA: CodegenData<Marker>` module, emitted by
/// the existing [`pr4xis::codegen::generate_rust`] from the builder
/// [`owl_to_builder`] produces.
///
/// `ont` is already parsed by [`super::reader::read_owl`], so this
/// takes `&OwlOntology` rather than re-parsing XML (unlike the
/// path-taking `generate_uslm_schema_source`, whose XSD parser lives
/// inside `pr4xis` core). `config` supplies the marker type name and
/// module path for the emitted `CodegenData<Marker>`.
pub fn generate_owl_vocabulary_source(ont: &OwlOntology, config: &GenerateConfig) -> String {
    pr4xis::codegen::generate_rust(&owl_to_builder(ont), config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::owl::ontology::{OwlClass, OwlObjectProperty};
    use crate::social::software::markup::xml::owl::reader::read_owl;
    use proptest::prelude::*;

    /// The bundled CiTO 2.8.1 OWL vocabulary (SPAR Ontologies). Embedded
    /// at build time via `include_str!` so the test is hermetic — the
    /// same `concat!(env!("CARGO_MANIFEST_DIR"), …)` convention the
    /// USLM vocabulary loader uses.
    const CITO_2_8_1_OWL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ontologies/cito-2.8.1.owl"
    ));

    /// CiTO's `citesAsEvidence` object-property IRI (SPAR uses
    /// slash-delimited IRIs, no `#` fragment).
    const CITES_AS_EVIDENCE_IRI: &str = "http://purl.org/spar/cito/citesAsEvidence";

    fn cito() -> OwlOntology {
        read_owl(CITO_2_8_1_OWL).expect("bundled CiTO 2.8.1 must parse")
    }

    // ── Unit: build the builder from the real bundled CiTO ───────────

    #[test]
    fn cito_builder_has_entities_and_taxonomy() {
        let ont = cito();
        let builder = owl_to_builder(&ont);

        // CiTO declares dozens of citation-typing object properties
        // plus a handful of classes; the builder must carry one entity
        // per class + per property.
        assert!(
            builder.entity_count() > 30,
            "expected >30 CiTO entities, got {}",
            builder.entity_count()
        );
        assert_eq!(
            builder.entity_count(),
            ont.classes.len() + ont.properties.len(),
            "entity count must equal classes + properties"
        );

        // citesAsEvidence is a CiTO object property — it must appear as
        // an ObjectProperty entity keyed by its IRI.
        let cae = builder
            .entities
            .iter()
            .find(|e| e.id == CITES_AS_EVIDENCE_IRI)
            .unwrap_or_else(|| {
                panic!("citesAsEvidence ({CITES_AS_EVIDENCE_IRI}) not found among CiTO entities")
            });
        assert_eq!(
            cae.pos.as_deref(),
            Some(KIND_OBJECT_PROPERTY),
            "citesAsEvidence must be tagged as an ObjectProperty"
        );

        // CiTO's property hierarchy (rdfs:subPropertyOf) is rich, so the
        // unioned taxonomy must be non-empty.
        assert!(
            !builder.taxonomy.is_empty(),
            "CiTO taxonomy (subClassOf ∪ subPropertyOf) must be non-empty"
        );
    }

    #[test]
    fn cito_kinds_are_class_or_object_property_only() {
        let builder = owl_to_builder(&cito());
        for e in &builder.entities {
            let kind = e.pos.as_deref().unwrap_or("");
            assert!(
                kind == KIND_CLASS || kind == KIND_OBJECT_PROPERTY,
                "entity {} has unexpected kind {kind:?}",
                e.id
            );
        }
    }

    // ── Unit: generated source is a real CodegenData module ──────────

    #[test]
    fn generated_source_is_codegen_data_with_cito_iri() {
        let ont = cito();
        let config = GenerateConfig::new("cito_codegen", "Cito");
        let src = generate_owl_vocabulary_source(&ont, &config);

        assert!(!src.is_empty(), "generated source must be non-empty");
        assert!(
            src.contains("CodegenData"),
            "generated source must reference CodegenData"
        );
        assert!(
            src.contains("CODEGEN_DATA"),
            "generated source must declare the CODEGEN_DATA static"
        );
        // At least one CiTO IRI must survive into the emitted ENTITY_IDS.
        assert!(
            src.contains(CITES_AS_EVIDENCE_IRI),
            "generated source must embed the citesAsEvidence CiTO IRI"
        );
    }

    // ── local_name helper ────────────────────────────────────────────

    #[test]
    fn local_name_handles_hash_slash_and_bare() {
        assert_eq!(local_name("http://www.w3.org/2002/07/owl#Class"), "Class");
        assert_eq!(
            local_name("http://purl.org/spar/cito/citesAsEvidence"),
            "citesAsEvidence"
        );
        assert_eq!(local_name("bareword"), "bareword");
        // A trailing slash leaves no local part after the slash; fall
        // back to the whole IRI rather than the empty string.
        assert_eq!(local_name("http://example.org/"), "http://example.org/");
    }

    #[test]
    fn empty_iri_entities_are_skipped() {
        let ont = OwlOntology {
            iri: "http://example.org/x".to_string(),
            classes: alloc::vec![
                OwlClass {
                    iri: String::new(),
                    label: Some("ghost".to_string()),
                    ..Default::default()
                },
                OwlClass {
                    iri: "http://example.org/x#A".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let builder = owl_to_builder(&ont);
        assert_eq!(builder.entity_count(), 1, "empty-IRI class must be skipped");
        assert_eq!(builder.entities[0].id, "http://example.org/x#A");
        // Label falls back to the local name when rdfs:label is absent.
        assert_eq!(builder.entities[0].label, "A");
    }

    // ── proptest: mapping properties over synthesised ontologies ─────

    /// A small synthesised OWL ontology: a few classes, a few object
    /// properties, and a child→parent hierarchy edge within each kind.
    #[derive(Debug, Clone)]
    struct SynthOnt {
        classes: Vec<String>,
        properties: Vec<String>,
        class_edges: Vec<(usize, usize)>,
        prop_edges: Vec<(usize, usize)>,
    }

    /// A child→parent edge generator that never produces a self-loop by
    /// *construction* (not by rejection): a `parent` offset is chosen in
    /// `0..n-1` and mapped to `(child + 1 + offset) % n`, which is always
    /// distinct from `child`. Yields no edges when `n < 2` (a single node
    /// has no non-self edge). De-duplicated so the union-cardinality
    /// assertion in `prop_every_edge_in_taxonomy` holds.
    fn arb_edges(n: usize) -> BoxedStrategy<Vec<(usize, usize)>> {
        if n < 2 {
            return Just(Vec::new()).boxed();
        }
        proptest::collection::vec((0..n, 0..n - 1), 0..6)
            .prop_map(move |raw| {
                let mut edges: Vec<(usize, usize)> = raw
                    .into_iter()
                    .map(|(child, off)| (child, (child + 1 + off) % n))
                    .collect();
                edges.sort_unstable();
                edges.dedup();
                edges
            })
            .boxed()
    }

    fn arb_synth() -> impl Strategy<Value = SynthOnt> {
        // 1..=6 distinct class names, 1..=6 distinct property names,
        // then non-self-loop hierarchy edges within each kind.
        (1usize..=6, 1usize..=6).prop_flat_map(|(n_cls, n_prop)| {
            let classes: Vec<String> = (0..n_cls)
                .map(|i| format!("http://ex.org/o#C{i}"))
                .collect();
            let properties: Vec<String> = (0..n_prop)
                .map(|i| format!("http://ex.org/o#p{i}"))
                .collect();
            (
                Just(classes),
                Just(properties),
                arb_edges(n_cls),
                arb_edges(n_prop),
            )
                .prop_map(|(classes, properties, class_edges, prop_edges)| SynthOnt {
                    classes,
                    properties,
                    class_edges,
                    prop_edges,
                })
        })
    }

    fn build_ontology(s: &SynthOnt) -> OwlOntology {
        let classes = s
            .classes
            .iter()
            .map(|iri| OwlClass {
                iri: iri.clone(),
                ..Default::default()
            })
            .collect();
        let properties = s
            .properties
            .iter()
            .map(|iri| OwlObjectProperty {
                iri: iri.clone(),
                ..Default::default()
            })
            .collect();
        let taxonomy = s
            .class_edges
            .iter()
            .map(|(c, p)| (s.classes[*c].clone(), s.classes[*p].clone()))
            .collect();
        let property_taxonomy = s
            .prop_edges
            .iter()
            .map(|(c, p)| (s.properties[*c].clone(), s.properties[*p].clone()))
            .collect();
        OwlOntology {
            iri: "http://ex.org/o".to_string(),
            classes,
            properties,
            taxonomy,
            property_taxonomy,
            ..Default::default()
        }
    }

    proptest! {
        /// (a) Entity count equals classes + properties.
        #[test]
        fn prop_entity_count_is_classes_plus_properties(s in arb_synth()) {
            let ont = build_ontology(&s);
            let builder = owl_to_builder(&ont);
            prop_assert_eq!(
                builder.entity_count(),
                ont.classes.len() + ont.properties.len()
            );
        }

        /// (b) Every subClassOf and subPropertyOf edge appears in the
        /// builder's taxonomy relation.
        #[test]
        fn prop_every_edge_in_taxonomy(s in arb_synth()) {
            let ont = build_ontology(&s);
            let builder = owl_to_builder(&ont);
            for edge in ont.taxonomy.iter().chain(ont.property_taxonomy.iter()) {
                prop_assert!(
                    builder.taxonomy.contains(edge),
                    "edge {:?} missing from builder taxonomy",
                    edge
                );
            }
            // And the union has exactly the combined cardinality.
            prop_assert_eq!(
                builder.taxonomy.len(),
                ont.taxonomy.len() + ont.property_taxonomy.len()
            );
        }

        /// (c) The mapping is deterministic: same input → same builder
        /// (entities + taxonomy identical, in order).
        #[test]
        fn prop_mapping_is_deterministic(s in arb_synth()) {
            let ont = build_ontology(&s);
            let a = owl_to_builder(&ont);
            let b = owl_to_builder(&ont);
            prop_assert_eq!(a.taxonomy.clone(), b.taxonomy.clone());
            let a_ids: Vec<&str> = a.entities.iter().map(|e| e.id.as_str()).collect();
            let b_ids: Vec<&str> = b.entities.iter().map(|e| e.id.as_str()).collect();
            prop_assert_eq!(a_ids, b_ids);
            // Determinism extends to the emitted source string.
            let cfg = GenerateConfig::new("synth_codegen", "Synth");
            prop_assert_eq!(
                generate_owl_vocabulary_source(&ont, &cfg),
                generate_owl_vocabulary_source(&ont, &cfg)
            );
        }
    }
}
