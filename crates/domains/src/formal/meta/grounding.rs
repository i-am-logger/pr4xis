//! Declared grounding — the GENERAL loader step that mints a loaded archive's
//! cross-ontology TYPE edges from the grounding functors it carries AS DATA.
//!
//! An [`Archive`] declares its cross-ontology grounding as
//! [`Connection`](pr4xis_runtime::connection::Connection)s: a functor-as-data
//! whose `kind` names its categorical family. [`ground_declared`] is the one
//! step that reads those connections, keeps ONLY the grounding (instance)
//! functors — decided by the META-ONTOLOGY query
//! [`is_grounding_functor_kind`],
//! never a `kind == "InstanceFunctor"` string match and never a `source != target`
//! test — and MINTS one typed
//! [`Grounded`](pr4xis_runtime::definition::EdgeTarget::Grounded) edge per typed
//! node into the LOADED peer's atoms, via the generic
//! [`type_lens`] + [`ground`].
//!
//! It is SOURCE-AGNOSTIC: a USC title grounding into `LegalSources`, a menagerie
//! grounding into a `taxonomy`, or any `.prx` declaring an instance functor
//! grounds the SAME way. The one USC special case (`ground_legal_types` +
//! `emit::<LegalSourcesCategory>()`) collapses into this general path — the USC
//! projection now just APPENDS its grounding `Connection` as data.
//!
//! # Fail-closed
//!
//! A declared grounding functor whose `target` ontology is not among the supplied
//! `peers` returns a typed
//! [`MissingPeerArchive`](pr4xis_runtime::grounding::LinkError::MissingPeerArchive)
//! — never a silent empty grounding. The load path treats that as a DEFERRAL
//! (install ungrounded, re-ground when the peer arrives) under the base-first
//! contract; a direct caller sees the typed error.
//!
//! # Idempotent
//!
//! Grounding is EXACT-DUPLICATE-free: re-running [`ground_declared`] over an
//! already-grounded archive (the re-ground-on-peer-arrival path) does not
//! double-mint an edge it already carries, so re-grounding is a safe re-materialize
//! of the source.

use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::category::category_theory::is_grounding_functor_kind;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::connection::GeneratorAction;
use pr4xis_runtime::grounding::{LinkError, ground, type_lens};
use pr4xis_runtime::ontology::{RuntimeOntology, materialize};

/// The reachability kind an instance-functor's typing edge asserts, fail-closed to
/// `Subsumption` (the copula's default kind) when the functor's `map_morphism`
/// table is empty.
const DEFAULT_TYPING_RELATION: &str = "Subsumption";

/// Mint the cross-ontology TYPE edges an [`Archive`] declares — the general
/// grounding step.
///
/// For each [`Connection`](pr4xis_runtime::connection::Connection) in
/// `archive.connections` whose `kind` is a grounding (instance) functor per the
/// meta-ontology, and whose action is a
/// [`Functor`](pr4xis_runtime::connection::GeneratorAction::Functor), one
/// `Grounded` edge is minted per typed node into the LOADED `peers[target]`
/// archive's atoms (via [`type_lens`]). A grounding functor whose `target` is not
/// a supplied peer fails closed with
/// [`MissingPeerArchive`](LinkError::MissingPeerArchive). Non-grounding
/// connections (schema relabels, natural transformations, …) are left untouched —
/// they are `apply`'s job, not grounding's. Exact-duplicate edges are not
/// re-minted, so re-running is idempotent.
pub fn ground_declared(
    archive: &Archive,
    peers: &BTreeMap<String, Archive>,
) -> Result<Archive, LinkError> {
    let mut current = archive.clone();
    for conn in &archive.connections {
        if !is_grounding_functor_kind(&conn.kind) {
            continue;
        }
        let GeneratorAction::Functor {
            map_object,
            map_morphism,
        } = &conn.action
        else {
            // A grounding functor is a Functor action; a non-Functor kind that
            // still reaches InstanceFunctor is malformed — skip rather than guess.
            continue;
        };
        let peer = peers
            .get(&conn.target)
            .ok_or_else(|| LinkError::MissingPeerArchive {
                ontology: conn.target.clone(),
            })?;
        let relation = map_morphism
            .iter()
            .map(|(_, target)| target.clone())
            .next()
            .unwrap_or_else(|| DEFAULT_TYPING_RELATION.to_string());
        current = ground(
            &current,
            type_lens(map_object, &relation, &conn.target, peer),
        );
    }
    dedup_edges(&mut current);
    Ok(current)
}

/// Re-ground a LOADED set in place — the loader's grounding pass, applied at
/// every install so grounding is a pure function of the loaded set, INDEPENDENT
/// of load order.
///
/// For each loaded ontology, its declared grounding functors are minted against
/// the current peer set (every loaded ontology's owned archive, by name) via
/// [`ground_declared`]; an ontology whose content genuinely changes (edges newly
/// minted) is RE-MATERIALIZED in place under its own name. This is the
/// re-ground-on-peer-arrival mechanism: when a grounding TARGET (a base such as
/// `LegalSources`) arrives — even AFTER a source that grounds into it — the pass
/// grounds that source on the base's arrival, so `reaches(section, law)` is green
/// whether the base loads before or after the source.
///
/// Idempotent: an ontology already carrying its grounding edges re-grounds to the
/// SAME archive (exact-duplicate-free), so its root is unchanged and it is NOT
/// re-materialized. A grounding functor whose target is not yet loaded DEFERS
/// (that ontology is left unchanged this pass) — the typed
/// [`MissingPeerArchive`](LinkError::MissingPeerArchive) is the direct-call
/// verdict; here a later pass (when the peer arrives) completes it, never a silent
/// wrong bind.
///
/// # Contract: grounding targets are pure bases (one-level grounding)
///
/// The peer set is frozen from the loaded archives *before* any slot is grounded,
/// so a functor's target atoms are the target's *pre-grounding* node addresses. A
/// grounding TARGET is therefore expected to be a pure base — an ontology that
/// declares no grounding functor of its own (every current target: `LegalSources`,
/// English, a domain taxonomy). A multi-level chain — a target that *itself*
/// grounds into a further base — is NOT yet supported: re-materializing that target
/// (line below) changes its node content-addresses, which the already-frozen peer
/// set would not reflect, so a source grounded against its pre-grounding addresses
/// would dangle. Grounding-target-that-grounds is future work (it needs the peer
/// set re-derived in dependency order, a topological pass); for now the base-first
/// corpus makes every target a leaf and this pass is exact.
pub fn ground_loaded_set(loaded: &mut [Rc<RuntimeOntology>]) {
    // The peer set: every loaded ontology's owned archive, addressed by name — a
    // grounding functor's target atoms are its peer archive's node addresses. Base
    // ontologies (the grounding targets) are small; a large corpus is present too
    // but is a grounding SOURCE, never looked up as a target.
    let mut peers: BTreeMap<String, Archive> = BTreeMap::new();
    for o in loaded.iter() {
        if let Ok(archive) = o.to_owned_archive() {
            peers.insert(o.id().as_str().to_string(), archive);
        }
    }
    for slot in loaded.iter_mut() {
        let Ok(raw) = slot.to_owned_archive() else {
            continue;
        };
        // Fail-closed per connection; a missing peer DEFERS this ontology (a later
        // pass completes it once the target loads — base-first makes that the
        // common no-defer case).
        let Ok(grounded) = ground_declared(&raw, &peers) else {
            continue;
        };
        if grounded == raw {
            // Already grounded (or nothing to ground) — no content change, so no
            // re-materialize (roots stay stable, the invariant "loading X adds only
            // X's data" holds for every other ontology).
            continue;
        }
        let name = slot.id().clone();
        if let Ok(regrounded) = materialize(grounded, name) {
            *slot = Rc::new(regrounded);
        }
    }
}

/// Drop exact-duplicate `(kind, target)` edges per node (order-preserving), so a
/// re-run of [`ground_declared`] over an already-grounded archive is idempotent.
fn dedup_edges(archive: &mut Archive) {
    for node in &mut archive.nodes {
        let mut seen: Vec<(String, pr4xis_runtime::definition::EdgeTarget)> = Vec::new();
        node.edges.retain(|edge| {
            if seen.contains(edge) {
                false
            } else {
                seen.push(edge.clone());
                true
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis_runtime::connection::Connection;
    use pr4xis_runtime::definition::{Definition, EdgeTarget};

    /// A tiny two-node target taxonomy: `Dog ⊑ Animal`.
    fn taxonomy() -> Archive {
        Archive {
            nodes: alloc::vec![
                Definition {
                    kind: "Concept".to_string(),
                    name: "Dog".to_string(),
                    edges: alloc::vec![(
                        "Subsumption".to_string(),
                        EdgeTarget::Local("Animal".to_string())
                    )],
                    axioms: alloc::vec![],
                    lexical: Some("a domesticated canine".to_string()),
                },
                Definition {
                    kind: "Concept".to_string(),
                    name: "Animal".to_string(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: Some("a living organism".to_string()),
                },
            ],
            connections: alloc::vec![],
        }
    }

    /// A one-node instance archive carrying a grounding (InstanceFunctor)
    /// connection typing its `Pet` kind as a `taxonomy:Dog`.
    fn menagerie(kind: &str) -> Archive {
        Archive {
            nodes: alloc::vec![Definition {
                kind: "Pet".to_string(),
                name: "rex".to_string(),
                edges: alloc::vec![],
                axioms: alloc::vec![],
                lexical: Some("a good dog".to_string()),
            }],
            connections: alloc::vec![Connection {
                kind: kind.to_string(),
                source: "menagerie".to_string(),
                target: "taxonomy".to_string(),
                action: GeneratorAction::Functor {
                    map_object: alloc::vec![("Pet".to_string(), "Dog".to_string())],
                    map_morphism: alloc::vec![(
                        "instantiates".to_string(),
                        "Subsumption".to_string()
                    )],
                },
                laws: alloc::vec!["PreservesTyping".to_string()],
            }],
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn grounds_a_declared_instance_functor_into_the_peer() {
        let mut peers = BTreeMap::new();
        peers.insert("taxonomy".to_string(), taxonomy());
        let grounded = ground_declared(&menagerie("InstanceFunctor"), &peers).unwrap();
        let dog_atom = taxonomy()
            .nodes
            .iter()
            .find(|n| n.name == "Dog")
            .unwrap()
            .address()
            .unwrap();
        let rex = grounded.nodes.iter().find(|n| n.name == "rex").unwrap();
        assert!(
            rex.edges.iter().any(|(k, t)| k == "Subsumption"
                && matches!(t, EdgeTarget::Grounded { ontology, atom }
                    if ontology == "taxonomy" && *atom == dog_atom)),
            "the InstanceFunctor mints rex --Subsumption--> taxonomy:Dog"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_schema_relabel_functor_is_not_grounded() {
        // `FullyFaithful` is the kind the USC/OWL `apply` schema-relabels carry.
        // It is not a registered InstanceFunctor-reaching concept, so the
        // discriminator excludes it (fail-closed on an unknown/non-instance kind)
        // and NO edge is minted. The reachability exclusion of a *known* non-
        // instance Functor is tested in the meta-ontology
        // (`is_grounding_functor_kind_discriminates_instance_from_relabel`).
        let mut peers = BTreeMap::new();
        peers.insert("taxonomy".to_string(), taxonomy());
        let grounded = ground_declared(&menagerie("FullyFaithful"), &peers).unwrap();
        let rex = grounded.nodes.iter().find(|n| n.name == "rex").unwrap();
        assert!(
            rex.edges.is_empty(),
            "a schema-relabel functor mints no grounding edge (discriminator excludes it)"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_missing_peer_fails_closed() {
        // The declared target `taxonomy` is not supplied — fail-closed, never a
        // silent empty grounding.
        let empty: BTreeMap<String, Archive> = BTreeMap::new();
        assert_eq!(
            ground_declared(&menagerie("InstanceFunctor"), &empty),
            Err(LinkError::MissingPeerArchive {
                ontology: "taxonomy".to_string(),
            })
        );
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn re_grounding_is_idempotent() {
        // Re-running ground_declared over an already-grounded archive does not
        // double-mint the type edge (the re-ground-on-peer-arrival contract).
        let mut peers = BTreeMap::new();
        peers.insert("taxonomy".to_string(), taxonomy());
        let once = ground_declared(&menagerie("InstanceFunctor"), &peers).unwrap();
        let twice = ground_declared(&once, &peers).unwrap();
        let rex = twice.nodes.iter().find(|n| n.name == "rex").unwrap();
        assert_eq!(
            rex.edges.len(),
            1,
            "re-grounding mints no duplicate — exactly one type edge"
        );
    }
}
