//! Emit — project a live, compiled praxis ontology (a `Category`) into a `.prx`
//! [`Archive`].
//!
//! This is the COMPILER half of the chrysalis. Where the runtime kernel works on
//! `Archive` *data*, this bridges the compile-time `ontology!` world into it: a
//! real, macro-generated ontology becomes a content-addressed `.prx` that
//! round-trips through the very runtime defined by the meta-`.prx`.
//!
//! Gated on the `emit` feature, which deps `pr4xis` (the category model); the
//! kernel itself carries no compile-time coupling. The projection is faithful to
//! the structure `Category` exposes: each `Concept` variant becomes a node, its
//! edges the outgoing morphisms (the materialized transitive closure) as
//! `(relation-kind, target)`, and its lexical grounding the concept's
//! ONTOLEX-Lemon gloss via [`Concept::lexical`] — so a `.prx` loaded with no
//! access to the compile-time `labels()` table (e.g. in the browser) still
//! carries each concept's meaning. (The connection nodes — functors/adjunctions
//! — are the next refinement; they need the registry's per-ontology
//! axiom/constructor slices.)

use pr4xis::category::{Arrow, Category, Concept, FinitelyGenerated};

use crate::archive::Archive;
use crate::definition::Definition;

/// Project the compiled ontology `Cat` into a `.prx` [`Archive`]: one node per
/// `Concept` variant, edges from `morphisms_from` as `(relation-kind, target)`,
/// identity self-loops dropped, edges sorted for a canonical address.
pub fn emit<Cat: Category>() -> Archive
where
    // Emits one node per concept variant — enumerates the objects, so the
    // compiled ontology being projected must be finitely generated (closed-world).
    Cat::Object: FinitelyGenerated,
{
    let nodes = <Cat::Object as FinitelyGenerated>::variants()
        .iter()
        .map(|obj| {
            let mut edges: Vec<(String, String)> = Cat::morphisms_from(obj)
                .iter()
                .filter(|m| m.target() != *obj) // drop identity self-loops
                .map(|m| (format!("{:?}", m.kind()), m.target().name().to_string()))
                .collect();
            edges.sort();
            edges.dedup();
            // Carry the concept's lexical grounding INTO the `.prx`: its
            // ONTOLEX-Lemon gloss (the `Definition`/sense text). The runtime
            // `Definition.lexical` is the serialized gloss string, so project
            // the structured `Lexical` to its definition text here — sourced
            // generically from `Concept::lexical()` (which the `ontology!`
            // macro fills from its labels table), never a per-ontology hack.
            // An ungrounded concept stays `None` (honest absence).
            let lexical = obj.lexical().map(|lex| lex.definition.as_str().to_string());
            Definition {
                kind: "Concept".to_string(),
                name: obj.name().to_string(),
                edges,
                // ## Honestly deferred — per-concept axiom derivation.
                //
                // The meta-`.prx` declares an `Axiom` concept that
                // `Constrains` a `Concept` (`meta::ontology`), and
                // `Definition::address` canonically folds the `axioms` field
                // into a node's content-address. The field is therefore REAL,
                // not vestigial — a definition carrying a non-empty `axioms`
                // Vec round-trips byte-exact through the codec and changes the
                // address (proven in this module's
                // `axioms_field_is_wired_through_the_codec_round_trip` test).
                //
                // What `emit` does NOT yet do is DERIVE the axioms that govern
                // each concept FROM the compiled ontology. The closed-world
                // axiom constructors live in `pr4xis::ontology::reasoning`,
                // keyed by typed `Category`/`Kind`, not exposed as a
                // per-concept slice this projection can enumerate generically
                // (the same registry gap that defers
                // `Connection::laws`-resolution in `ontology::materialize`).
                // Mirroring that honest deferral: this emit does not claim to
                // have projected per-concept axioms — it leaves the field
                // empty rather than silently contradicting the meta's `Axiom`
                // concept with a fabricated one. Deferred to a tracked
                // follow-up (the registry's per-ontology axiom slice).
                axioms: Vec::new(),
                lexical,
            }
        })
        .collect();
    Archive {
        nodes,
        // ## Honestly deferred — connection-node derivation.
        //
        // The meta-`.prx` declares `Connection` (and its `Functor` /
        // `Adjunction` / `Lens` / `NaturalTransformation` families) and the
        // `Archive` `Contains` both `Concept`s and `Connection`s; an
        // `Archive`'s Merkle root folds in every connection's address, so the
        // field is REAL (proven by `archive::tests::a_connection_contributes_
        // to_the_root` and this module's connections-wiring round-trip test).
        //
        // What `emit` does NOT yet do is DERIVE connection nodes
        // (functors / adjunctions between ontologies) from the compiled
        // world: that needs the registry's per-ontology
        // functor/adjunction/constructor slices, which are not exposed as a
        // generic projection surface here (noted in the module header). Rather
        // than emit a fabricated or always-trivial connection, this leaves the
        // set empty — honest absence, not a stub. Deferred to the same tracked
        // follow-up as per-concept axiom derivation.
        connections: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::ContentAddress;
    use crate::load;
    use crate::rebind::{RebindTarget, rebind_nodes};
    use std::collections::{BTreeSet, HashMap};

    // A small REAL ontology — exactly what a domain ontology is, in miniature:
    // generated by the same `ontology!` macro, materialized transitive closure
    // and all.
    pr4xis::ontology! {
        name: "Org",
        source: "pr4xis-runtime emit test fixture",
        concepts: [Employer, Employee, Person, Agent],
        labels: {
            Employer: ("en", "Employer", "One who employs."),
            Employee: ("en", "Employee", "One who is employed."),
            Person: ("en", "Person", "A human being."),
            Agent: ("en", "Agent", "One who acts."),
        },
        is_a: [
            (Employer, Person),
            (Employee, Person),
            (Person, Agent),
        ],
    }

    #[test]
    fn emits_every_concept_as_a_node() {
        let archive = emit::<OrgCategory>();
        let names: BTreeSet<&str> = archive.nodes.iter().map(|n| n.name.as_str()).collect();
        for c in ["Employer", "Employee", "Person", "Agent"] {
            assert!(names.contains(c), "missing concept node {c}");
        }
    }

    #[test]
    fn emits_the_subsumption_closure_as_edges() {
        let archive = emit::<OrgCategory>();
        let employer = archive.nodes.iter().find(|n| n.name == "Employer").unwrap();
        let targets: BTreeSet<&str> = employer.edges.iter().map(|(_, t)| t.as_str()).collect();
        // direct: Employer is_a Person; closure: Employer is_a Agent.
        assert!(targets.contains("Person"), "Employer → Person missing");
        assert!(
            targets.contains("Agent"),
            "closure Employer → Agent missing"
        );
    }

    #[test]
    fn emits_each_concepts_gloss_as_its_lexical_and_round_trips() {
        // The labels table the `ontology!` macro generated for this fixture —
        // the authoritative source the emitted gloss must match (tuple shape:
        // (variant, lang, surface, gloss)).
        let glosses: HashMap<&str, &str> = OrgOntology::labels()
            .iter()
            .map(|(_, _, surface, gloss)| (*surface, *gloss))
            .collect();

        let archive = emit::<OrgCategory>();
        // Every emitted node carries its concept's gloss — non-None and equal
        // to the macro's labels table (proving the gloss travels IN the `.prx`,
        // not only in the compile-time `labels()` side table).
        for node in &archive.nodes {
            let expected = glosses.get(node.name.as_str()).copied();
            assert_eq!(
                node.lexical.as_deref(),
                expected,
                "node {} must carry its labels-table gloss",
                node.name
            );
            assert!(
                node.lexical.is_some(),
                "every glossed concept must emit a lexical; {} did not",
                node.name
            );
        }
        assert_eq!(
            archive.nodes.iter().filter(|n| n.lexical.is_some()).count(),
            glosses.len(),
            "every labelled concept must contribute a gloss"
        );

        // The gloss survives the byte-exact round-trip through load.
        let bytes = load::emit(&archive).unwrap();
        let loaded = load::load(&bytes, archive.root().unwrap()).unwrap();
        assert_eq!(loaded, archive);
        let employer = loaded.nodes.iter().find(|n| n.name == "Employer").unwrap();
        assert_eq!(employer.lexical.as_deref(), Some("One who employs."));
    }

    #[test]
    fn emitted_ontology_round_trips_through_the_runtime() {
        let archive = emit::<OrgCategory>();
        let bytes = load::emit(&archive).unwrap();
        let loaded = load::load(&bytes, archive.root().unwrap()).unwrap();
        assert_eq!(loaded, archive);
    }

    #[test]
    fn axioms_field_is_wired_through_the_codec_round_trip() {
        // The meta-`.prx` declares an `Axiom` concept that `Constrains` a
        // `Concept`, and `Definition::address` folds `axioms` into the
        // content-address — but `emit` does not yet DERIVE per-concept axioms
        // (see the honest-deferral note at the `axioms: Vec::new()` site). This
        // test proves the field is nonetheless REAL, not vestigial: a manually
        // constructed `Definition` carrying a NON-EMPTY axioms Vec survives the
        // `emit -> load` codec round-trip byte-exact, AND the axioms field is
        // load-bearing on the content-address (two definitions differing ONLY
        // in `axioms` get different addresses, so the archive root differs too).
        let with_axioms = Definition {
            kind: "Concept".to_string(),
            name: "Employer".to_string(),
            edges: vec![("Subsumption".to_string(), "Agent".to_string())],
            axioms: vec![
                "EmployerIsAgent".to_string(),
                "EmployerHiresEmployee".to_string(),
            ],
            lexical: Some("employer".to_string()),
        };
        // Same node with NO axioms — differs ONLY in the axioms field.
        let without_axioms = Definition {
            axioms: Vec::new(),
            ..with_axioms.clone()
        };

        // Difference-detection: the axioms field changes the node address...
        assert_ne!(
            with_axioms.address().unwrap(),
            without_axioms.address().unwrap(),
            "axioms must be load-bearing on the definition address"
        );

        let archive = Archive {
            nodes: vec![with_axioms.clone()],
            connections: Vec::new(),
        };
        let archive_no_axioms = Archive {
            nodes: vec![without_axioms],
            connections: Vec::new(),
        };
        // ...and therefore on the archive root (the content-address the load
        // gate checks against).
        assert_ne!(
            archive.root().unwrap(),
            archive_no_axioms.root().unwrap(),
            "the axioms field must reach the archive root"
        );

        // The axioms survive the codec round-trip byte-exact, fail-closed
        // against the archive's own root.
        let bytes = load::emit(&archive).unwrap();
        let loaded = load::load(&bytes, archive.root().unwrap()).unwrap();
        assert_eq!(loaded, archive, "the archive must round-trip faithfully");
        let node = loaded.nodes.iter().find(|n| n.name == "Employer").unwrap();
        assert_eq!(
            node.axioms,
            vec![
                "EmployerIsAgent".to_string(),
                "EmployerHiresEmployee".to_string(),
            ],
            "the non-empty axioms Vec must survive the round-trip byte-exact"
        );
    }

    #[test]
    fn connections_are_wired_through_the_codec_round_trip() {
        use crate::connection::{Connection, GeneratorAction};

        // Companion to the axioms-wiring proof, for the `Archive.connections`
        // field `emit` also leaves empty (see the honest-deferral note at the
        // `connections: Vec::new()` site). A manually constructed connection
        // survives the `emit -> load` round-trip byte-exact AND is load-bearing
        // on the archive root (an archive with a connection differs from the
        // same archive without it).
        let connection = Connection {
            kind: "FullyFaithful".to_string(),
            source: "Employer".to_string(),
            target: "Agent".to_string(),
            action: GeneratorAction::Functor {
                map_object: vec![("Employer".to_string(), "Agent".to_string())],
                map_morphism: vec![("Subsumption".to_string(), "Subsumption".to_string())],
            },
            laws: vec!["PreservesComposition".to_string()],
        };
        let node = Definition {
            kind: "Concept".to_string(),
            name: "Employer".to_string(),
            edges: Vec::new(),
            axioms: Vec::new(),
            lexical: None,
        };

        let with_conn = Archive {
            nodes: vec![node.clone()],
            connections: vec![connection.clone()],
        };
        let without_conn = Archive {
            nodes: vec![node],
            connections: Vec::new(),
        };
        // Difference-detection: the connection reaches the archive root.
        assert_ne!(
            with_conn.root().unwrap(),
            without_conn.root().unwrap(),
            "a connection must be load-bearing on the archive root"
        );

        // The connection survives the round-trip byte-exact, fail-closed.
        let bytes = load::emit(&with_conn).unwrap();
        let loaded = load::load(&bytes, with_conn.root().unwrap()).unwrap();
        assert_eq!(loaded, with_conn, "the archive must round-trip faithfully");
        assert_eq!(
            loaded.connections,
            vec![connection],
            "the connection must survive the round-trip byte-exact"
        );
    }

    #[test]
    fn emitted_ontology_rebinds_against_itself() {
        struct Selfish(HashMap<String, ContentAddress>);
        impl RebindTarget for Selfish {
            fn address_of(&self, name: &str) -> Option<ContentAddress> {
                self.0.get(name).copied()
            }
        }
        let archive = emit::<OrgCategory>();
        let known: HashMap<String, ContentAddress> = archive
            .nodes
            .iter()
            .map(|n| (n.name.clone(), n.address().unwrap()))
            .collect();
        let rebound = rebind_nodes(&archive, &Selfish(known)).unwrap();
        assert!(
            rebound.iter().all(|r| r.is_bound()),
            "a freshly-emitted ontology must rebind to itself"
        );
    }
}
