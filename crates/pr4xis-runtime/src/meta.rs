//! The meta-ontology — the `.prx` format defined IN a `.prx`.
//!
//! This is the bootstrap kernel: the vocabulary that defines what a `.prx` IS —
//! `Concept`, `Connection`, the connection families, `Archive`, `ContentAddress`,
//! `MerkleRoot`, `Version`, `Axiom` — emitted AS an [`Archive`]. The format is
//! data, not Rust: a peer learns the schema by loading THIS, and the runtime
//! grounds only the hash. It is **self-describing** — it contains the concepts
//! that describe itself (`Concept` is a Concept; `Archive` is the node carrying
//! it; `ContentAddress` is how every node here is addressed) — and it
//! round-trips through the very runtime it defines.

use crate::archive::Archive;
use crate::connection::{Connection, GeneratorAction};
use crate::definition::{Definition, EdgeTarget};

/// One meta-concept node: `name` with its `(relation, target)` edges (all
/// targets other meta-concepts), lexically grounded (in praxis "everything is
/// Lemon").
fn concept(name: &str, edges: &[(&str, &str)]) -> Definition {
    Definition {
        kind: "Concept".into(),
        name: name.into(),
        edges: edges
            .iter()
            .map(|(r, t)| ((*r).to_string(), EdgeTarget::Local((*t).to_string())))
            .collect(),
        axioms: vec![],
        lexical: Some(name.to_lowercase()),
    }
}

/// The meta-ontology as a `.prx` archive: the format defining itself. Its
/// [`Archive::root`] is the bootstrap content address — the identity the rest of
/// the system is interpreted against.
pub fn ontology() -> Archive {
    Archive {
        nodes: vec![
            // The atoms.
            concept("Concept", &[]),
            concept("Source", &[]),
            concept(
                "Connection",
                &[("HasSource", "Concept"), ("HasTarget", "Concept")],
            ),
            // The connection families — each IS-A Connection.
            concept("Functor", &[("Subsumption", "Connection")]),
            concept("Adjunction", &[("Subsumption", "Connection")]),
            concept("Lens", &[("Subsumption", "Connection")]),
            concept("NaturalTransformation", &[("Subsumption", "Connection")]),
            // Grounding — a Connection refined to the ATOM level: a typed,
            // content-addressed morphism from a node in one archive INTO an
            // addressable atom of ANOTHER (a span A ←π— G —π→ B, partial +
            // many-to-one; Spivak ologs). It is the same content-addressed
            // connection wire, narrowed so endpoints are atoms (a `Definition`'s
            // address), not ontology names — the substrate for "which ontologies
            // our content is connected to, and how" (the lexical `denotes` floor
            // being the first, most universal kind).
            concept("Grounding", &[("Subsumption", "Connection")]),
            // The container + the open-world presentation.
            concept(
                "Archive",
                &[("Contains", "Concept"), ("Contains", "Connection")],
            ),
            concept(
                "Category",
                &[("Contains", "Concept"), ("Contains", "Connection")],
            ),
            concept("Generator", &[("Presents", "Category")]),
            // Identity / addressing — the ground.
            concept(
                "ContentAddress",
                &[("Addresses", "Concept"), ("Addresses", "Connection")],
            ),
            concept(
                "MerkleRoot",
                &[("Subsumption", "ContentAddress"), ("Roots", "Archive")],
            ),
            concept("HashAlgorithm", &[("Grounds", "ContentAddress")]),
            // Versioning + constraints.
            concept("Version", &[("Versions", "Connection")]),
            concept(
                "Axiom",
                &[("Constrains", "Concept"), ("Constrains", "Connection")],
            ),
            // The morphism-kind vocabulary (A3) — the relation/edge KINDS the
            // format itself uses, promoted from bare edge-labels to addressable
            // meta-concepts. Every kind a node's edges name (`Subsumption`,
            // `Contains`, …) now EXISTS as a concept to content-address, so a
            // serialized morphism can carry the kind's ADDRESS, not its spelling.
            // This is the hand-authored bootstrap FLOOR — the self-describing
            // kernel (praxis-way rule 11: the meta-`.prx` is the honest string
            // floor). The DOMAIN relation kinds (Parthood/Causation/Opposition/…)
            // are a separate, LOADED tier (the Relations ontology projection); an
            // edge whose kind is in neither stays a graceful free leaf, exactly as
            // an unmapped kind is the IDENTITY image in `apply`.
            concept("MorphismKind", &[("Subsumption", "Concept")]),
            // A structural property a kind may carry (OWL property characteristic —
            // Hitzler et al., OWL 2 Primer, W3C 2012).
            concept("Property", &[("Subsumption", "Concept")]),
            // The OWL `TransitiveProperty` marker (OWL-RL `prp-trp`).
            concept("Transitive", &[("Subsumption", "Property")]),
            // `Subsumption` IS-A MorphismKind and is Transitive — `rdfs:subClassOf`
            // is a transitive property (Brickley & Guha, RDF Schema 1.1, W3C 2014).
            // Its content address therefore folds in `Transitive`, so a Subsumption
            // defined WITHOUT that property addresses differently.
            concept(
                "Subsumption",
                &[
                    ("Subsumption", "MorphismKind"),
                    ("HasProperty", "Transitive"),
                ],
            ),
            // The kind that attaches a `Property` to a `MorphismKind` (the
            // `(R, Property, HasProperty)` morphism the Relations ontology declares).
            concept("HasProperty", &[("Subsumption", "MorphismKind")]),
            // The remaining structural kinds the meta-ontology's own edges use,
            // each IS-A MorphismKind. No property is asserted of them (none is
            // claimed beyond what is cited).
            concept("HasSource", &[("Subsumption", "MorphismKind")]),
            concept("HasTarget", &[("Subsumption", "MorphismKind")]),
            concept("Contains", &[("Subsumption", "MorphismKind")]),
            concept("Presents", &[("Subsumption", "MorphismKind")]),
            concept("Addresses", &[("Subsumption", "MorphismKind")]),
            concept("Roots", &[("Subsumption", "MorphismKind")]),
            concept("Grounds", &[("Subsumption", "MorphismKind")]),
            concept("Versions", &[("Subsumption", "MorphismKind")]),
            concept("Constrains", &[("Subsumption", "MorphismKind")]),
        ],
        connections: vec![
            // The decompile lens itself, as a Connection in the meta-ontology:
            // Parse ⊣ Generate — the source ⇄ ontology round-trip (issue #93).
            Connection {
                kind: "WellBehaved".into(),
                source: "Source".into(),
                target: "Concept".into(),
                action: GeneratorAction::Lens {
                    view: "Source".into(),
                    get: "Parse".into(),
                    put: "Generate".into(),
                },
                laws: vec!["GetPut".into(), "PutGet".into()],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::ContentAddress;
    use crate::load;
    use crate::rebind::{RebindTarget, rebind_nodes};
    use std::collections::{BTreeSet, HashMap};

    #[test]
    fn meta_ontology_round_trips() {
        let meta = ontology();
        let bytes = load::emit(&meta).unwrap();
        let loaded = load::load(&bytes, meta.root().unwrap()).unwrap();
        assert_eq!(loaded, meta);
    }

    #[test]
    fn meta_root_is_stable() {
        assert_eq!(ontology().root().unwrap(), ontology().root().unwrap());
    }

    #[test]
    fn is_self_describing() {
        let meta = ontology();
        let names: BTreeSet<&str> = meta.nodes.iter().map(|n| n.name.as_str()).collect();
        for required in [
            "Concept",
            "Connection",
            "Archive",
            "ContentAddress",
            "MerkleRoot",
            "Lens",
            "Version",
            "Functor",
            // The morphism-kind vocabulary describes itself too: `MorphismKind`
            // is a Concept, and `Subsumption` — the kind every IS-A edge here
            // uses — is among the meta-ontology's own nodes.
            "MorphismKind",
            "Subsumption",
        ] {
            assert!(
                names.contains(required),
                "meta-ontology must define {required}"
            );
        }
    }

    #[test]
    fn the_format_kinds_are_addressable_meta_concepts() {
        // A3: every relation KIND the meta-ontology's own edges use is itself a
        // defined concept (a `MorphismKind`), so a morphism can carry the kind's
        // ADDRESS, not its bare name. The kinds are no longer just edge labels.
        let meta = ontology();
        let names: BTreeSet<&str> = meta.nodes.iter().map(|n| n.name.as_str()).collect();
        let used_kinds: BTreeSet<&str> = meta
            .nodes
            .iter()
            .flat_map(|n| n.edges.iter().map(|(kind, _)| kind.as_str()))
            .collect();
        for kind in used_kinds {
            assert!(
                names.contains(kind),
                "edge-kind {kind:?} must exist as an addressable meta-concept"
            );
        }
        // And each such kind IS-A MorphismKind.
        for node in &meta.nodes {
            for (kind, _) in &node.edges {
                let def = meta
                    .nodes
                    .iter()
                    .find(|n| &n.name == kind)
                    .expect("kind is defined");
                assert!(
                    def.edges
                        .iter()
                        .any(|(r, t)| r == "Subsumption" && t.local_name() == Some("MorphismKind")),
                    "kind {kind:?} must IS-A MorphismKind"
                );
            }
        }
    }

    #[test]
    fn is_referentially_closed() {
        let meta = ontology();
        let names: BTreeSet<&str> = meta.nodes.iter().map(|n| n.name.as_str()).collect();
        for n in &meta.nodes {
            for (_, target) in &n.edges {
                // The meta-ontology is entirely local; a Grounded target would be
                // a foreign atom (resolved elsewhere), so only local names are
                // checked for referential closure here.
                if let Some(name) = target.local_name() {
                    assert!(names.contains(name), "edge target {name} undefined");
                }
            }
        }
        for c in &meta.connections {
            assert!(
                names.contains(c.source.as_str()),
                "source {} undefined",
                c.source
            );
            assert!(
                names.contains(c.target.as_str()),
                "target {} undefined",
                c.target
            );
        }
    }

    #[test]
    fn grounding_is_a_connection_in_the_meta() {
        // The grounding vocabulary enters the self-describing format as a refined
        // Connection — so a peer that loads the meta-`.prx` learns what a
        // grounding IS (a kind of connection) the same way it learns Functor /
        // Lens, by reference, never by compiled-in code.
        let meta = ontology();
        let grounding = meta
            .nodes
            .iter()
            .find(|n| n.name == "Grounding")
            .expect("the meta-ontology defines Grounding");
        assert_eq!(grounding.kind, "Concept");
        assert!(
            grounding
                .edges
                .iter()
                .any(|(rel, target)| rel == "Subsumption"
                    && target.local_name() == Some("Connection")),
            "Grounding must IS-A Connection; got {:?}",
            grounding.edges
        );
    }

    #[test]
    fn the_runtime_rebinds_against_its_own_meta() {
        // The format describing itself, recognized by the runtime: a target that
        // holds each meta-concept at its definition-bearing address rebinds the
        // whole meta-ontology — every node binds, none stays free.
        struct SelfKnowledge(HashMap<String, ContentAddress>);
        impl RebindTarget for SelfKnowledge {
            fn address_of(&self, name: &str) -> Option<ContentAddress> {
                self.0.get(name).copied()
            }
        }

        let meta = ontology();
        let known: HashMap<String, ContentAddress> = meta
            .nodes
            .iter()
            .map(|n| (n.name.clone(), n.address().unwrap()))
            .collect();
        let rebound = rebind_nodes(&meta, &SelfKnowledge(known)).unwrap();
        assert!(
            rebound.iter().all(|r| r.is_bound()),
            "every meta-concept must rebind to itself"
        );
    }
}
