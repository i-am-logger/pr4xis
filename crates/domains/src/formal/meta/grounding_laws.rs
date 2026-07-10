//! Runnable, cited engine-law axioms for the declared-grounding pass
//! ([`ground_declared`](crate::formal::meta::grounding::ground_declared)) — the loader step
//! that mints a loaded archive's cross-ontology TYPE edges from the grounding
//! functors it carries AS DATA.
//!
//! A grounding functor rides in a `.prx` as a
//! [`Connection`](pr4xis_runtime::connection::Connection) whose `laws` field is
//! DATA (`["PreservesTyping"]`) — a declaration, not a proof. This module lifts
//! the pass's REAL, falsifiable contracts into registered, discoverable
//! `Axiom`s verifying over witness archives with teeth, mirroring the
//! `reach_laws` / `packed_csr_laws` shape:
//!
//! - [`GroundingExtensionOnly`](crate::formal::meta::grounding_laws::GroundingExtensionOnly)
//!   — the pass only ADDS grounded edges: every
//!   source node (kind, name, axioms, lexical, its own edges — in order) and
//!   every source connection survives verbatim; the appended tail is grounded
//!   edges into declared functor targets, nothing else. The closure-operator
//!   "extensive" law, `x ≤ c(x)`.
//! - [`GroundingIdempotent`](crate::formal::meta::grounding_laws::GroundingIdempotent)
//!   — re-running the pass over an already-grounded
//!   archive is a no-op (`c(c(x)) = c(x)`) — the re-ground-on-peer-arrival
//!   contract.
//! - [`GroundingFailClosed`](crate::formal::meta::grounding_laws::GroundingFailClosed)
//!   — the defer/refuse semantics: a declared target
//!   ontology with no supplied peer is the typed
//!   [`MissingPeerArchive`](pr4xis_runtime::grounding::LinkError::MissingPeerArchive)
//!   (the load path's
//!   DEFERRAL verdict), and a declared concept NAME absent from a PRESENT peer
//!   is the LOUD
//!   [`GroundTargetAbsent`](pr4xis_runtime::grounding::LinkError::GroundTargetAbsent) — never
//!   a silent empty grounding, never a silently dropped edge.
//!
//! # Literature
//!
//! - **Spivak, D. I. (2012)** "Functorial data migration", *Information and
//!   Computation* 217, 31–51 — an instance is a functor into the schema; the
//!   grounding pass applies that functor, carried as data.
//! - **Davey & Priestley (2002)** *Introduction to Lattices and Order*, 2nd
//!   ed., Cambridge University Press — closure operators: extensive
//!   (`x ≤ c(x)`) and idempotent (`c(c(x)) = c(x)`), the two laws the pass
//!   satisfies over duplicate-free sources.
//! - **Saltzer & Schroeder (1975)** "The Protection of Information in Computer
//!   Systems", *Proceedings of the IEEE* 63(9), 1278–1308 — the fail-safe
//!   defaults principle the defer/refuse semantics realize.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::category::category_theory::is_grounding_functor_kind;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::connection::{Connection, GeneratorAction};
use pr4xis_runtime::definition::{Definition, EdgeTarget};
use pr4xis_runtime::grounding::LinkError;

use super::grounding::ground_declared;

// ── witness archives ─────────────────────────────────────────────────────────

/// A witness node.
fn node(kind: &str, name: &str, edges: Vec<(String, EdgeTarget)>) -> Definition {
    Definition {
        kind: kind.to_string(),
        name: name.to_string(),
        edges,
        axioms: alloc::vec![],
        lexical: Some(alloc::format!("witness {name}")),
    }
}

/// A witness grounding functor connection (`kind` decides whether the
/// meta-ontology discriminator admits it).
fn functor(
    kind: &str,
    target: &str,
    map_object: Vec<(String, String)>,
    map_morphism: Vec<(String, String)>,
) -> Connection {
    Connection {
        kind: kind.to_string(),
        source: "witness".to_string(),
        target: target.to_string(),
        action: GeneratorAction::Functor {
            map_object,
            map_morphism,
        },
        // The declaration the axioms make RUNNABLE — data, not proof.
        laws: alloc::vec!["PreservesTyping".to_string()],
    }
}

/// The two witness peers: a `taxonomy` (`Dog ⊑ Animal`) and a `geology`
/// (`Quartz ⊑ Rock`).
fn peers() -> BTreeMap<String, Archive> {
    let taxonomy = Archive {
        nodes: alloc::vec![
            node(
                "Concept",
                "Dog",
                alloc::vec![(
                    "Subsumption".to_string(),
                    EdgeTarget::Local("Animal".to_string())
                )],
            ),
            node("Concept", "Animal", alloc::vec![]),
        ],
        connections: alloc::vec![],
    };
    let geology = Archive {
        nodes: alloc::vec![
            node(
                "Concept",
                "Quartz",
                alloc::vec![(
                    "Subsumption".to_string(),
                    EdgeTarget::Local("Rock".to_string())
                )],
            ),
            node("Concept", "Rock", alloc::vec![]),
        ],
        connections: alloc::vec![],
    };
    let mut peers = BTreeMap::new();
    peers.insert("taxonomy".to_string(), taxonomy);
    peers.insert("geology".to_string(), geology);
    peers
}

/// The witness SOURCE archives the extension-only and idempotence laws range
/// over — multiple shapes, all duplicate-free (the pass's stated domain):
/// - an EMPTY archive that still declares a functor (nothing to ground);
/// - the one-node instance (`Pet ↦ taxonomy:Dog`);
/// - a multi-functor, multi-kind archive: two functors into two peers (one
///   with an EMPTY `map_morphism`, exercising the typed copula default), a
///   node with a PRE-EXISTING local edge (the survival the extension law
///   pins), an UNMAPPED kind (a legitimate no-op), and a NON-grounding
///   `FullyFaithful` connection the discriminator must leave untouched;
/// - an ALREADY-GROUNDED archive (the one-node instance after one pass).
fn witness_sources() -> Vec<Archive> {
    let one_node = Archive {
        nodes: alloc::vec![node("Pet", "rex", alloc::vec![])],
        connections: alloc::vec![functor(
            "InstanceFunctor",
            "taxonomy",
            alloc::vec![("Pet".to_string(), "Dog".to_string())],
            alloc::vec![("instantiates".to_string(), "Subsumption".to_string())],
        )],
    };
    let already_grounded =
        ground_declared(&one_node, &peers()).expect("the one-node witness grounds");
    alloc::vec![
        Archive {
            nodes: alloc::vec![],
            connections: alloc::vec![functor(
                "InstanceFunctor",
                "taxonomy",
                alloc::vec![("Pet".to_string(), "Dog".to_string())],
                alloc::vec![],
            )],
        },
        one_node,
        Archive {
            nodes: alloc::vec![
                node(
                    "Pet",
                    "rex",
                    // A pre-existing source edge — extension-only must keep it,
                    // verbatim, ahead of any minted grounding edge.
                    alloc::vec![(
                        "companionOf".to_string(),
                        EdgeTarget::Local("pebble".to_string())
                    )],
                ),
                node("Stone", "pebble", alloc::vec![]),
                node("Flower", "rose", alloc::vec![]), // unmapped kind — no-op
            ],
            connections: alloc::vec![
                functor(
                    "InstanceFunctor",
                    "taxonomy",
                    alloc::vec![("Pet".to_string(), "Dog".to_string())],
                    alloc::vec![("instantiates".to_string(), "Subsumption".to_string())],
                ),
                // EMPTY map_morphism — the minted kind falls back to the typed
                // Relations copula default (`subsumption_kind()`), never a bare
                // string literal.
                functor(
                    "InstanceFunctor",
                    "geology",
                    alloc::vec![("Stone".to_string(), "Quartz".to_string())],
                    alloc::vec![],
                ),
                // A schema relabel — NOT a grounding functor; must mint nothing.
                functor(
                    "FullyFaithful",
                    "taxonomy",
                    alloc::vec![("Flower".to_string(), "Dog".to_string())],
                    alloc::vec![],
                ),
            ],
        },
        already_grounded,
    ]
}

// ── the predicates ───────────────────────────────────────────────────────────

/// EXTENSION-ONLY on one witness: the grounded result preserves the source
/// verbatim — same connections, same node count and order, each node's
/// identity fields intact and its OWN edges an untouched PREFIX of the
/// grounded edges — and every APPENDED edge is a
/// [`Grounded`](EdgeTarget::Grounded) edge into an ontology some declared
/// grounding functor targets. Nothing is dropped, nothing else is invented.
pub(crate) fn extension_only_on(source: &Archive, peers: &BTreeMap<String, Archive>) -> bool {
    let Ok(grounded) = ground_declared(source, peers) else {
        return false;
    };
    grounded.connections == source.connections
        && grounded.nodes.len() == source.nodes.len()
        && source.nodes.iter().zip(&grounded.nodes).all(|(s, g)| {
            s.kind == g.kind
                && s.name == g.name
                && s.axioms == g.axioms
                && s.lexical == g.lexical
                && g.edges.len() >= s.edges.len()
                && g.edges[..s.edges.len()] == s.edges[..]
                && g.edges[s.edges.len()..].iter().all(|(_, target)| {
                    matches!(target, EdgeTarget::Grounded { ontology, .. }
                        if source.connections.iter().any(|c|
                            is_grounding_functor_kind(&c.kind) && c.target == *ontology))
                })
        })
}

/// IDEMPOTENCE on one witness: grounding twice equals grounding once —
/// `c(c(x)) = c(x)`, the re-ground-on-peer-arrival contract.
pub(crate) fn idempotent_on(source: &Archive, peers: &BTreeMap<String, Archive>) -> bool {
    let Ok(once) = ground_declared(source, peers) else {
        return false;
    };
    ground_declared(&once, peers).as_ref() == Ok(&once)
}

// ── the axioms ───────────────────────────────────────────────────────────────

/// EXTENSION-ONLY: the declared-grounding pass only ADDS typed grounded edges
/// — over a duplicate-free source it preserves every source node (kind, name,
/// axioms, lexical, its own edges in order) and every source connection
/// verbatim, and every appended edge is a `Grounded` edge into a declared
/// grounding functor's target. The closure-operator EXTENSIVE law, `x ≤ c(x)`.
///
/// TEETH: the multi-functor witness carries a node with a PRE-EXISTING local
/// edge and a non-grounding `FullyFaithful` connection — a pass that dropped
/// or reordered a source edge, rewrote a node, or let the relabel mint an edge
/// fails the prefix / connection equality here.
pub struct GroundingExtensionOnly;

impl Axiom for GroundingExtensionOnly {
    fn verify(&self) -> Verdict {
        let peers = peers();
        if witness_sources()
            .iter()
            .all(|source| extension_only_on(source, &peers))
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GroundingExtensionOnly",
        "the declared-grounding pass only ADDS grounded edges: source nodes, their own edges (as an untouched prefix) and connections survive verbatim, and every appended edge is a Grounded edge into a declared functor target",
        "Spivak (2012) Functorial data migration, Information and Computation 217, 31-51; Davey & Priestley (2002) Introduction to Lattices and Order, 2nd ed., Cambridge University Press — the extensive law x <= c(x)"
    );
}

pr4xis::register_axiom!(GroundingExtensionOnly, constructor);

/// IDEMPOTENCE: re-running the declared-grounding pass over an already-grounded
/// archive changes nothing — `c(c(x)) = c(x)` over every witness shape
/// (empty, one-node, multi-functor/multi-peer with the copula default, and an
/// archive that was ALREADY grounded once). This is the registered form of the
/// re-ground-on-peer-arrival contract (`ground_loaded_set` re-runs the pass at
/// every install).
pub struct GroundingIdempotent;

impl Axiom for GroundingIdempotent {
    fn verify(&self) -> Verdict {
        let peers = peers();
        if witness_sources()
            .iter()
            .all(|source| idempotent_on(source, &peers))
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GroundingIdempotent",
        "re-running the declared-grounding pass over an already-grounded archive is a no-op: ground(ground(x)) == ground(x) over every witness shape — the re-ground-on-peer-arrival contract",
        "Davey & Priestley (2002) Introduction to Lattices and Order, 2nd ed., Cambridge University Press — closure operators are idempotent: c(c(x)) = c(x)"
    );
}

pr4xis::register_axiom!(GroundingIdempotent, constructor);

/// FAIL-CLOSED defer/refuse semantics: a declared grounding whose target
/// ontology has NO supplied peer is the typed
/// [`MissingPeerArchive`](LinkError::MissingPeerArchive) (the load path's
/// DEFERRAL — install ungrounded, re-ground when the peer arrives), and a
/// declared concept NAME absent from a PRESENT peer is the LOUD
/// [`GroundTargetAbsent`](LinkError::GroundTargetAbsent) with the exact
/// (kind, target, peer) triple — never a silent empty grounding, never a
/// silently dropped edge. Both verdicts are exercised, and the SAME functor
/// against the full peer set succeeds (the errors are about the environment,
/// not the functor).
pub struct GroundingFailClosed;

impl Axiom for GroundingFailClosed {
    fn verify(&self) -> Verdict {
        let source = Archive {
            nodes: alloc::vec![node("Pet", "rex", alloc::vec![])],
            connections: alloc::vec![functor(
                "InstanceFunctor",
                "taxonomy",
                alloc::vec![("Pet".to_string(), "Dog".to_string())],
                alloc::vec![("instantiates".to_string(), "Subsumption".to_string())],
            )],
        };
        // DEFER leg: the declared target ontology is not among the peers.
        let empty: BTreeMap<String, Archive> = BTreeMap::new();
        let defers = ground_declared(&source, &empty)
            == Err(LinkError::MissingPeerArchive {
                ontology: "taxonomy".to_string(),
            });

        // REFUSE leg: the peer IS present but lacks the declared concept.
        let mut ghost = source.clone();
        if let GeneratorAction::Functor { map_object, .. } = &mut ghost.connections[0].action {
            map_object[0] = ("Pet".to_string(), "Ghost".to_string());
        }
        let refuses = ground_declared(&ghost, &peers())
            == Err(LinkError::GroundTargetAbsent {
                kind: "Pet".to_string(),
                target: "Ghost".to_string(),
                peer: "taxonomy".to_string(),
            });

        // CONTRAST leg: the same functor against the full peer set grounds.
        let grounds = ground_declared(&source, &peers()).is_ok();

        if defers && refuses && grounds {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GroundingFailClosed",
        "declared grounding fails closed: an unsupplied target peer is the typed MissingPeerArchive deferral and a declared concept absent from a PRESENT peer is the loud GroundTargetAbsent — never a silent empty grounding, never a silently dropped edge",
        "Saltzer & Schroeder (1975) The Protection of Information in Computer Systems, Proceedings of the IEEE 63(9), 1278-1308 — the fail-safe defaults principle"
    );
}

pr4xis::register_axiom!(GroundingFailClosed, constructor);

// ── laws-hold + discoverability (the reach_laws shape) ───────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    use pr4xis::ontology::registry::axiom_by_name;

    /// The three grounding-pass laws hold over their witnesses.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn grounding_laws_hold() {
        assert!(
            GroundingExtensionOnly.verify().is_ok(),
            "the pass only ADDS grounded edges — the source survives verbatim"
        );
        assert!(
            GroundingIdempotent.verify().is_ok(),
            "ground(ground(x)) == ground(x)"
        );
        assert!(
            GroundingFailClosed.verify().is_ok(),
            "MissingPeerArchive defers; GroundTargetAbsent is loud"
        );
    }

    /// The three axioms re-bind by name through the registry — discoverable
    /// exactly as any statute's law is (the load-time rebind gate).
    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn grounding_laws_discoverable_via_registry() {
        for name in [
            "GroundingExtensionOnly",
            "GroundingIdempotent",
            "GroundingFailClosed",
        ] {
            assert!(
                axiom_by_name(name).is_some(),
                "grounding law {name} must re-bind through the registry"
            );
        }
    }
}
