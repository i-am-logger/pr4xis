//! Integration — a REAL domain ontology emitted through `pr4xis-runtime`.
//!
//! Where the runtime's own emit test uses a 4-concept toy fixture, this feeds it
//! the actual Dependability ontology (Avizienis et al. 2004's Fault/Error/Failure
//! taxonomy, ~44 concepts) straight from its macro-generated `Category`, and
//! shows it round-trips through the runtime and rebinds against itself — the
//! chrysalis ingesting one of praxis's real ontologies, not a toy.

use std::collections::HashMap;

use pr4xis_domains::applied::dependability::ontology::DependabilityCategory;
use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::emit::emit;
use pr4xis_runtime::load;
use pr4xis_runtime::rebind::{RebindTarget, rebind_nodes};

#[test]
fn real_dependability_ontology_round_trips_through_the_runtime() {
    let archive = emit::<DependabilityCategory>();

    // The full taxonomy — dozens of concepts, carrying real structure.
    assert!(
        archive.nodes.len() >= 40,
        "expected the whole Dependability ontology, got {} nodes",
        archive.nodes.len()
    );
    let total_edges: usize = archive.nodes.iter().map(|n| n.edges.len()).sum();
    assert!(total_edges > 0, "a real ontology must carry relations");

    // It round-trips through the runtime, fail-closed against its own root.
    let bytes = load::emit(&archive).unwrap();
    let loaded = load::load(&bytes, archive.root().unwrap()).unwrap();
    assert_eq!(
        loaded, archive,
        "the real ontology must round-trip faithfully"
    );

    // A wrong root is refused — the fail-closed gate, on a real ontology.
    assert!(load::load(&bytes, ContentAddress::of(b"wrong root")).is_err());
}

#[test]
fn real_dependability_ontology_rebinds_against_itself() {
    struct Selfish(HashMap<String, ContentAddress>);
    impl RebindTarget for Selfish {
        fn address_of(&self, name: &str) -> Option<ContentAddress> {
            self.0.get(name).copied()
        }
    }

    let archive = emit::<DependabilityCategory>();
    let known: HashMap<String, ContentAddress> = archive
        .nodes
        .iter()
        .map(|n| (n.name.clone(), n.address().unwrap()))
        .collect();
    let rebound = rebind_nodes(&archive, &Selfish(known)).unwrap();
    assert!(
        rebound.iter().all(|r| r.is_bound()),
        "every concept of a real ontology must rebind to itself"
    );
}
