//! Integration — a REAL domain ontology emitted through `pr4xis-runtime`.
//!
//! Where the runtime's own emit test uses a 4-concept toy fixture, this feeds it
//! the actual Dependability ontology (Avizienis et al. 2004's Fault/Error/Failure
//! taxonomy, ~44 concepts) straight from its macro-generated `Category`, and
//! shows it round-trips through the runtime and rebinds against itself — the
//! chrysalis ingesting one of praxis's real ontologies, not a toy.

use std::collections::HashMap;

use pr4xis::category::{Category, FinitelyGenerated};
use pr4xis_domains::applied::dependability::ontology::DependabilityCategory;
use pr4xis_domains::applied::resilience::ontology::ResilienceCategory;
use pr4xis_domains::formal::causation::CausationCategory;
use pr4xis_domains::formal::classification::ClassificationCategory;
use pr4xis_domains::formal::mereology::MereologyTheoryCategory;
use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::archive::Archive;
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

// ---------------------------------------------------------------------------
// The `.prx` FORMAT round-trip law (issue #30), over several REAL ontologies.
//
// ## What this law is — and what it is NOT
//
// This asserts the **format** round-trip: the ontology serialization
// (`Archive`) is a WELL-BEHAVED, DETERMINISTIC, CONTENT-ADDRESSED lens through
// `pr4xis-runtime`'s codec. For every compiled ontology in the set below:
//
//   1. `load` is the inverse of `emit`:  archive == load(emit(archive), root)
//      (the fail-closed load gate admits it against its OWN re-derived root).
//   2. Serialization is DETERMINISTIC: re-emitting the SAME archive produces
//      BYTE-IDENTICAL `.prx` bytes (canonical DAG-CBOR, no nondeterminism).
//   3. The content-address (Merkle root) is STABLE across re-emit.
//
// It is checked over a SET of real compiled ontologies (`Dependability`,
// `Resilience`, `Causation`, `Classification`, `MereologyTheory`) so the law is
// genuinely "all-sources" for the compiled world, not a single fixture.
//
// ## NOT claimed here — the byte-exact SOURCE round-trip
//
// This is the FORMAT round-trip (`.prx` → bytes → `.prx`), NOT the byte-exact
// SOURCE round-trip (`compile source → .prx → decompile → source`, with
// `hash(out) == hash(in)`). The source round-trip needs the decompile lens
// (the ontology → source writer, issue #15), which is DEFERRED per the #28
// scope decision (USC-only decompile; the all-format source writers are a
// follow-on). No source bytes are reconstructed or hashed here; this test makes
// no claim about source-byte-exactness.

/// Assert the `.prx` FORMAT round-trip law for one compiled ontology `Cat`:
/// well-behaved (`load ∘ emit == id`), deterministic (byte-identical re-emit),
/// content-addressed (stable root). Returns the node count so the driver can
/// confirm it exercised a non-trivial ontology.
fn assert_format_round_trip_law<Cat: Category + pr4xis::category::DomainAxiomatized + 'static>(
    name: &str,
) -> usize
where
    Cat::Object: FinitelyGenerated,
    <Cat::Morphism as pr4xis::category::Arrow>::Kind:
        core::fmt::Debug + PartialEq + Clone + 'static,
{
    let archive: Archive = emit::<Cat>();

    // Non-vacuous: a real ontology must carry real structure (nodes + edges),
    // otherwise the law below would hold trivially over an empty archive.
    assert!(
        archive.nodes.len() >= 3,
        "{name}: expected a real ontology, got {} nodes",
        archive.nodes.len()
    );
    let total_edges: usize = archive.nodes.iter().map(|n| n.edges.len()).sum();
    assert!(
        total_edges > 0,
        "{name}: a real ontology must carry relations"
    );

    let root = archive.root().expect("root must derive");

    // (1) load is the inverse of emit — fail-closed against the archive's OWN
    //     re-derived root (the load gate re-derives, never trusts the wire).
    let bytes = load::emit(&archive).expect("emit to .prx bytes");
    let loaded = load::load(&bytes, root).expect("load against own root");
    assert_eq!(
        loaded, archive,
        "{name}: load(emit(archive)) must equal archive (well-behaved)"
    );

    // (2) Serialization is DETERMINISTIC: re-emitting the same archive yields
    //     byte-identical `.prx`. (Also re-emit the LOADED archive: round-trip
    //     then re-serialize must reproduce the very same bytes.)
    let bytes_again = load::emit(&archive).expect("re-emit");
    assert_eq!(
        bytes, bytes_again,
        "{name}: re-emitting the same archive must be byte-identical (deterministic)"
    );
    let bytes_from_loaded = load::emit(&loaded).expect("emit the loaded archive");
    assert_eq!(
        bytes, bytes_from_loaded,
        "{name}: emit(load(emit(a))) must reproduce the same bytes"
    );

    // (3) The content-address (root) is STABLE across re-emit and round-trip.
    assert_eq!(
        root,
        archive.root().expect("root again"),
        "{name}: the root must be stable across re-derivation"
    );
    assert_eq!(
        root,
        loaded.root().expect("loaded root"),
        "{name}: the root must survive the round-trip"
    );

    // Difference-detection (the law is non-trivial): perturbing the archive by
    // ONE node must change BOTH the bytes and the root — proving the round-trip
    // is faithful to content, not vacuously true for everything.
    let mut perturbed = archive.clone();
    perturbed.nodes.remove(0);
    let perturbed_bytes = load::emit(&perturbed).expect("emit perturbed");
    assert_ne!(
        bytes, perturbed_bytes,
        "{name}: dropping a node must change the .prx bytes"
    );
    assert_ne!(
        root,
        perturbed.root().expect("perturbed root"),
        "{name}: dropping a node must change the root"
    );

    archive.nodes.len()
}

#[test]
fn prx_format_round_trip_law_holds_over_real_ontologies() {
    // Iterate the law over a SET of real compiled ontologies — "all-sources"
    // for the compiled world, not one. (Generics over distinct `Category`
    // types can't share a `Vec`, so the set is enumerated by type here; each
    // call drives the full format-law over that ontology's emitted `.prx`.)
    let dependability = assert_format_round_trip_law::<DependabilityCategory>("Dependability");
    let resilience = assert_format_round_trip_law::<ResilienceCategory>("Resilience");
    let causation = assert_format_round_trip_law::<CausationCategory>("Causation");
    let classification = assert_format_round_trip_law::<ClassificationCategory>("Classification");
    let mereology = assert_format_round_trip_law::<MereologyTheoryCategory>("MereologyTheory");

    // Sanity: the set really exercised several distinct, non-trivial ontologies.
    let counts = [
        dependability,
        resilience,
        causation,
        classification,
        mereology,
    ];
    assert!(
        counts.iter().all(|&n| n >= 3),
        "every ontology in the set must be non-trivial; got {counts:?}"
    );
    assert!(
        counts.iter().sum::<usize>() >= 40,
        "the set must cover a substantial number of concepts; got {counts:?}"
    );
}
