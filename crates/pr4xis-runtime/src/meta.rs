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
use crate::definition::Definition;

/// One meta-concept node: `name` with its `(relation, target)` edges (all
/// targets other meta-concepts), lexically grounded (in praxis "everything is
/// Lemon").
fn concept(name: &str, edges: &[(&str, &str)]) -> Definition {
    Definition {
        kind: "Concept".into(),
        name: name.into(),
        edges: edges
            .iter()
            .map(|(r, t)| ((*r).to_string(), (*t).to_string()))
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
        ] {
            assert!(
                names.contains(required),
                "meta-ontology must define {required}"
            );
        }
    }

    #[test]
    fn is_referentially_closed() {
        let meta = ontology();
        let names: BTreeSet<&str> = meta.nodes.iter().map(|n| n.name.as_str()).collect();
        for n in &meta.nodes {
            for (_, target) in &n.edges {
                assert!(
                    names.contains(target.as_str()),
                    "edge target {target} undefined"
                );
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
