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

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::category::category_theory::is_grounding_functor_kind;
use pr4xis_runtime::address::ContentAddress;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::codec::CodecError;
use pr4xis_runtime::connection::GeneratorAction;
use pr4xis_runtime::definition::EdgeTarget;
use pr4xis_runtime::grounding::{LinkError, ground, type_lens};
use pr4xis_runtime::lens::archive_lens::{ArchivedGeneratorActionView, archived_grounded};
use pr4xis_runtime::ontology::{RuntimeOntology, materialize, subsumption_kind};

use crate::cognitive::linguistics::english::English;
use crate::cognitive::linguistics::english::bridge::{ENGLISH_ONTOLOGY, english_atom_address};

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
            // Fail-closed to the copula's default kind when the functor's
            // `map_morphism` table is empty — derived from the one typed Relations
            // vocabulary (`subsumption_kind()`), never a bare string literal.
            .unwrap_or_else(|| subsumption_kind().name);
        // Fail-closed: a declared target concept NAME absent from the present peer
        // surfaces as GroundTargetAbsent (never a silently dropped edge).
        current = ground(
            &current,
            type_lens(map_object, &relation, &conn.target, peer),
        )?;
    }
    dedup_edges(&mut current);
    Ok(current)
}

/// Re-ground a LOADED set in place — the loader's grounding pass, applied at
/// every install so grounding is a pure function of the loaded set, INDEPENDENT
/// of load order.
///
/// For each loaded ontology, its declared grounding functors are read straight
/// off its ARCHIVED view (connections, `map_object`, `map_morphism`), each
/// declared target atom is resolved PER NAME against the peers' archived views
/// (`node_by_name` single-node decodes; English through
/// [`english_atom_address`]), and the desired edges' PRESENCE is checked
/// against the archived nodes via
/// [`archived_grounded`] — so the COMMON re-install case (every edge already
/// minted) costs ZERO owned decodes and NO transient peer projections. Only a
/// slot that genuinely needs new edges (a fresh install) pays ONE
/// `to_owned_archive` of ITSELF, is extended, and is RE-MATERIALIZED in place
/// under its own name. Edge-for-edge equivalent to running [`ground_declared`]
/// over an owned peer set (the reference implementation, kept for direct
/// callers) — same connection order, same node order, same fail-closed
/// verdicts — without the former per-install transient (the whole-English
/// `project_archive_with_forms` projection + a `to_owned_archive` of EVERY
/// peer: +82.79 MiB per install, now ~0 steady-state).
///
/// This is the re-ground-on-peer-arrival mechanism: when a grounding TARGET (a
/// base such as `LegalSources`) arrives — even AFTER a source that grounds into
/// it — the pass grounds that source on the base's arrival, so
/// `reaches(section, law)` is green whether the base loads before or after the
/// source.
///
/// Idempotent: an ontology already carrying its grounding edges passes the
/// archived presence check (exact-duplicate-free), so its root is unchanged and
/// it is NOT re-materialized. A grounding functor whose target is not yet
/// loaded DEFERS (that ontology is left unchanged this pass) — the typed
/// [`MissingPeerArchive`](LinkError::MissingPeerArchive) is the direct-call
/// verdict; here a later pass (when the peer arrives) completes it, never a silent
/// wrong bind.
///
/// # Contract: grounding targets are pure bases (one-level grounding), guarded
///
/// A functor's target atoms are resolved against the target's CURRENT node
/// addresses. A grounding TARGET must therefore be a pure base — an ontology
/// that declares no grounding functor of its own (every current target:
/// `LegalSources`, English, a domain taxonomy). A multi-level chain — a target
/// that *itself* grounds into a further base — is NOT supported:
/// re-materializing that target (to add ITS grounding edges) changes its node
/// content-addresses mid-pass, so a source grounded against the earlier
/// addresses would dangle SILENTLY. This is no longer left to prose: it is
/// enforced by `guard_one_level_grounding`, a FAIL-CLOSED runtime guard that
/// refuses the whole pass with [`LinkError::GroundingTargetIsSource`] when any
/// loaded ontology is BOTH a grounding target and a grounding source (which is
/// also what makes the live per-name peer reads equivalent to the old frozen
/// pre-grounding peer set). Grounding-target-that-grounds is future work (it
/// needs the peer set re-derived in dependency order, a topological pass);
/// until then the guard makes the unsupported case LOUD.
///
/// # Fail-closed vs deferred
///
/// A [`MissingPeerArchive`](LinkError::MissingPeerArchive) — a declared target
/// ontology not yet loaded — DEFERS (that slot is left ungrounded; a later pass
/// grounds it when the peer arrives). Every OTHER grounding fault is LOUD and
/// aborts the pass with the typed error: a declared target NAME absent from a
/// PRESENT peer ([`GroundTargetAbsent`](LinkError::GroundTargetAbsent)), a root
/// skew, a codec fault, or the multi-level guard. The caller surfaces it (a load
/// is refused, a REPL prints it) rather than installing a silently-mis-grounded
/// set.
pub fn ground_loaded_set(
    loaded: &mut [Rc<RuntimeOntology>],
    english: &English,
) -> Result<(), LinkError> {
    // FAIL-CLOSED one-level guard (the documented contract, enforced): refuse the
    // whole pass if any loaded ontology is both a grounding target and a grounding
    // source — the unsupported multi-level chain that would dangle silently. Reads
    // each ontology's connections straight off the ARCHIVED views (zero decode).
    guard_one_level_grounding(loaded)?;

    // The loaded peer NAMES — the membership test the MissingPeerArchive deferral
    // reads. The pass holds NO owned peer archives at all: a functor's handful of
    // declared target atoms are resolved PER NAME against the peers' archived
    // views ([`resolve_target_atom`]) — English through the per-name
    // [`english_atom_address`] (never the whole `project_archive_with_forms`
    // projection, the measured +82.79 MiB per-install transient this pass used to
    // pay), a loaded peer through its `node_by_name` single-node decode (never
    // `to_owned_archive` over a whole peer). The one-level guard above is what
    // makes reading peers live equivalent to the old frozen pre-grounding peer
    // set: a grounding TARGET is never itself re-materialized by this pass.
    let loaded_names: BTreeSet<String> =
        loaded.iter().map(|o| o.id().as_str().to_string()).collect();

    // (target ontology, concept name) → resolved atom address. Bounded by the
    // number of DECLARED functor targets (a handful), shared across slots, and
    // dropped when the pass returns.
    let mut atom_cache: BTreeMap<(String, String), ContentAddress> = BTreeMap::new();

    for i in 0..loaded.len() {
        let slot = Rc::clone(&loaded[i]);
        let name = slot.id().clone();

        // One planned grounding connection: the relation kind its edges assert,
        // its target ontology, and the functor's `map_object` rows.
        struct Plan {
            relation: String,
            target: String,
            map: Vec<(String, String)>,
        }

        // ── Phase 1+2 (ZERO owned decodes): walk the slot's grounding
        // connections IN CONNECTION ORDER (the old sequential `ground_declared`
        // semantics — a loud fault on an earlier connection fires before a later
        // connection's deferral), resolve each declared target atom per NODE (in
        // node order, exactly the old lens's error order), and check the desired
        // edge's PRESENCE against the archived nodes via [`archived_grounded`].
        // A missing peer ARCHIVE DEFERS this ontology (a later pass completes it
        // once the target loads — base-first makes that the common no-defer
        // case). EVERY other fault — a declared target NAME absent from a
        // present peer ([`LinkError::GroundTargetAbsent`]), a codec fault — is
        // LOUD: it aborts the pass so the caller refuses rather than installing
        // a silently-mis-grounded set.
        let mut plans: Vec<Plan> = Vec::new();
        let mut deferred = false;
        let mut all_present = true;
        let archive_view = slot.archive();
        for conn in archive_view.connections.iter() {
            if !is_grounding_functor_kind(conn.kind.as_str()) {
                continue;
            }
            // A grounding functor is a Functor action; a non-Functor kind that
            // still reaches InstanceFunctor is malformed — skip rather than
            // guess (verbatim the `ground_declared` stance).
            let ArchivedGeneratorActionView::Functor {
                map_object,
                map_morphism,
            } = &conn.action
            else {
                continue;
            };
            let target = conn.target.as_str();
            if target != ENGLISH_ONTOLOGY && !loaded_names.contains(target) {
                // The old per-slot `Err(MissingPeerArchive)` deferral — including
                // its discard of any EARLIER connection's would-be edges.
                deferred = true;
                break;
            }
            // Fail-closed to the copula's default kind when the functor's
            // `map_morphism` table is empty — derived from the one typed
            // Relations vocabulary (`subsumption_kind()`), never a bare string
            // literal (verbatim `ground_declared`).
            let relation = map_morphism
                .iter()
                .next()
                .map(|pair| pair.1.as_str().to_string())
                .unwrap_or_else(|| subsumption_kind().name);
            let plan = Plan {
                relation,
                target: target.to_string(),
                map: map_object
                    .iter()
                    .map(|pair| (pair.0.as_str().to_string(), pair.1.as_str().to_string()))
                    .collect(),
            };
            // Resolve + presence-check THIS connection over the archived nodes.
            for node in archive_view.nodes.iter() {
                let Some((_, concept)) = plan
                    .map
                    .iter()
                    .find(|(kind, _)| kind.as_str() == node.kind.as_str())
                else {
                    continue; // a node whose kind is undeclared is a legitimate no-op
                };
                let atom = resolve_target_atom(
                    &mut atom_cache,
                    &plan.target,
                    concept,
                    node.kind.as_str(),
                    loaded,
                    english,
                )?;
                let present = node.edges.iter().any(|edge| {
                    edge.0.as_str() == plan.relation.as_str()
                        && archived_grounded(&edge.1)
                            .is_some_and(|(ontology, a)| ontology == plan.target && a == atom)
                });
                if !present {
                    all_present = false;
                }
            }
            plans.push(plan);
        }
        if deferred || plans.is_empty() || all_present {
            // Deferred, nothing declared, or already grounded (the common
            // re-install case) — ZERO owned decodes, no re-materialize (roots
            // stay stable, the invariant "loading X adds only X's data" holds
            // for every other ontology).
            continue;
        }

        // ── Phase 3 (the ONE owned decode, only for a slot that genuinely
        // needs new edges — a fresh install): rebuild THIS slot, extend the
        // missing edges in the same (connection, node) order `ground_declared`
        // minted them, exact-duplicate-free, and re-materialize in place under
        // its own name.
        //
        // INVARIANT (not a swallow): `to_owned_archive` is `ArchiveLens::get`
        // over a buffer that `materialize` `bytecheck`-VALIDATED and holds
        // immutable, so a MATERIALIZED `RuntimeOntology` never fails to decode
        // (`materialized_ontology_to_owned_archive_never_fails` proves it); on
        // the impossible failure the slot is left unchanged — never a silently
        // mis-grounded install.
        let Ok(mut archive) = slot.to_owned_archive() else {
            continue;
        };
        for plan in &plans {
            for node in &mut archive.nodes {
                let Some((_, concept)) = plan.map.iter().find(|(kind, _)| kind == &node.kind)
                else {
                    continue;
                };
                let atom = resolve_target_atom(
                    &mut atom_cache,
                    &plan.target,
                    concept,
                    node.kind.as_str(),
                    loaded,
                    english,
                )?;
                let edge = (
                    plan.relation.clone(),
                    EdgeTarget::Grounded {
                        ontology: plan.target.clone(),
                        atom,
                    },
                );
                // Exact-duplicate-free (the `dedup_edges` idempotence contract).
                if !node.edges.contains(&edge) {
                    node.edges.push(edge);
                }
            }
        }
        if let Ok(regrounded) = materialize(archive, name) {
            loaded[i] = Rc::new(regrounded);
        }
    }
    Ok(())
}

/// Resolve ONE declared grounding target `(target ontology, concept name)` to
/// its atom address, against the LIVE loaded set — through the per-name English
/// resolver ([`english_atom_address`]) for the seeded `english_wordnet` target,
/// or a single-node archived decode
/// ([`RuntimeOntology::node_by_name`]) for a loaded peer. Cached per pass (the
/// declared-target set is a handful), so a ~40k-node USC title grounding into
/// one `LegalSources` concept resolves ONE atom, once.
///
/// Fail-closed, mirroring [`type_lens`]: a declared concept absent from the
/// present peer is [`LinkError::GroundTargetAbsent`] (never a silently dropped
/// edge); an address/codec fault is [`LinkError::Codec`]. The caller has
/// already deferred on a missing peer, so an absent peer here is unreachable —
/// kept as the typed [`LinkError::MissingPeerArchive`] rather than a panic.
fn resolve_target_atom(
    cache: &mut BTreeMap<(String, String), ContentAddress>,
    target: &str,
    concept: &str,
    node_kind: &str,
    loaded: &[Rc<RuntimeOntology>],
    english: &English,
) -> Result<ContentAddress, LinkError> {
    let key = (target.to_string(), concept.to_string());
    if let Some(&atom) = cache.get(&key) {
        return Ok(atom);
    }
    let absent = || LinkError::GroundTargetAbsent {
        kind: node_kind.to_string(),
        target: concept.to_string(),
        peer: target.to_string(),
    };
    let atom = if target == ENGLISH_ONTOLOGY {
        english_atom_address(english, concept)
            .map_err(LinkError::Codec)?
            .ok_or_else(absent)?
    } else {
        let peer = loaded
            .iter()
            .find(|o| o.id().as_str() == target)
            .ok_or_else(|| LinkError::MissingPeerArchive {
                ontology: target.to_string(),
            })?;
        match peer.node_by_name(concept) {
            None => return Err(absent()),
            Some(Ok(node)) => node.address().map_err(LinkError::Codec)?,
            // Defensively unreachable (a validated buffer's single-node decode);
            // typed, never a panic.
            Some(Err(e)) => {
                return Err(LinkError::Codec(CodecError::Decode(format!(
                    "peer {target:?} node {concept:?}: {e}"
                ))));
            }
        }
    };
    cache.insert(key, atom);
    Ok(atom)
}

/// FAIL-CLOSED one-level-grounding guard — refuse the whole pass if any LOADED
/// ontology is BOTH a grounding target and a grounding source.
///
/// [`ground_loaded_set`] resolves a functor's target atoms against the loaded
/// peers' CURRENT archived nodes. If a loaded ontology used as a target were
/// itself a source, grounding it would re-materialize it — shifting the very
/// addresses another source grounded (or is about to ground) against, leaving
/// that source's edges dangling SILENTLY. That multi-level chain is
/// unsupported; this guard makes it LOUD
/// ([`GroundingTargetIsSource`](LinkError::GroundingTargetIsSource)) instead of
/// dangling — and it is ALSO what makes the pass's live per-name peer reads
/// equivalent to the old frozen pre-grounding peer set: a target is never
/// re-materialized mid-pass. The supported cases are untouched: into-English
/// targets `english_wordnet` (never a loaded slot, so never in `sources`), and
/// USC→`LegalSources` targets a pure base (declares no grounding functor).
fn guard_one_level_grounding(loaded: &[Rc<RuntimeOntology>]) -> Result<(), LinkError> {
    // A loaded ontology is a SOURCE if it declares any grounding functor; the
    // TARGETS are the ontologies those functors point at. Read each ontology's
    // connections straight off its ARCHIVED view — zero owned decode.
    let mut sources: BTreeSet<String> = BTreeSet::new();
    let mut targets: BTreeSet<String> = BTreeSet::new();
    for o in loaded {
        let name = o.id().as_str();
        for conn in o.archive().connections.iter() {
            if is_grounding_functor_kind(conn.kind.as_str()) {
                sources.insert(name.to_string());
                targets.insert(conn.target.as_str().to_string());
            }
        }
    }
    // A loaded ontology that is both a target and a source is the multi-level chain.
    if let Some(target) = targets.intersection(&sources).next() {
        return Err(LinkError::GroundingTargetIsSource {
            target: target.clone(),
        });
    }
    Ok(())
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
    use crate::cognitive::linguistics::english::bridge::project_archive_with_forms;
    use pr4xis_runtime::connection::Connection;
    use pr4xis_runtime::definition::Definition;

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

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn materialized_ontology_to_owned_archive_never_fails() {
        // The INVARIANT behind the `if let Ok(archive)` peer-build skip (and the
        // reused `peers.get` in the slot loop): `to_owned_archive` is
        // `ArchiveLens::get` over a buffer `materialize` bytecheck-VALIDATED and
        // holds immutable, so a materialized `RuntimeOntology` always decodes. This
        // proves the skip branch is a defensive no-op, never a silent swallow of
        // real data — every materialized ontology (a pure base, a grounding source,
        // and a grounded result) round-trips to an owned archive.
        use pr4xis::ontology::meta::OntologyName;

        let base = materialize(taxonomy(), OntologyName::new_static("taxonomy")).unwrap();
        assert!(
            base.to_owned_archive().is_ok(),
            "a materialized base must decode to its owned archive"
        );

        let source = materialize(
            menagerie("InstanceFunctor"),
            OntologyName::new_static("menagerie"),
        )
        .unwrap();
        assert!(
            source.to_owned_archive().is_ok(),
            "a materialized grounding source must decode to its owned archive"
        );

        // A GROUNDED result (edges minted) is still a validated buffer once
        // re-materialized — it decodes too.
        let mut peers = BTreeMap::new();
        peers.insert("taxonomy".to_string(), taxonomy());
        let grounded = ground_declared(&menagerie("InstanceFunctor"), &peers).unwrap();
        let regrounded = materialize(grounded, OntologyName::new_static("menagerie")).unwrap();
        assert!(
            regrounded.to_owned_archive().is_ok(),
            "a materialized grounded result must decode to its owned archive"
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

    use proptest::prelude::*;

    proptest! {
        /// ∀-strengthening of [`re_grounding_is_idempotent`]: over a generated
        /// menagerie (any set of `Pet` nodes grounding into the fixed `Dog ⊑ Animal`
        /// taxonomy), grounding twice equals grounding once — `ground_declared` is
        /// idempotent (the re-ground-on-peer-arrival contract) for every generated
        /// node set, not just the one-node witness.
        #[test]
        fn prop_grounding_is_idempotent(
            names in prop::collection::hash_set("[a-z]{1,6}", 0..6)
        ) {
            let mut peers = BTreeMap::new();
            peers.insert("taxonomy".to_string(), taxonomy());
            let nodes: alloc::vec::Vec<Definition> = names
                .iter()
                .map(|n| Definition {
                    kind: "Pet".to_string(),
                    name: n.clone(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: Some("a generated pet".to_string()),
                })
                .collect();
            let archive = Archive {
                nodes,
                connections: alloc::vec![Connection {
                    kind: "InstanceFunctor".to_string(),
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
            };
            let once = ground_declared(&archive, &peers).expect("first grounding");
            let twice = ground_declared(&once, &peers).expect("second grounding");
            prop_assert_eq!(once, twice);
        }
    }

    pr4xis::register_praxis_value!(prop_grounding_is_idempotent, Deterministic);

    /// A one-node menagerie whose functor declares a target concept NAME the peer
    /// does not hold (`Pet ↦ <target>`) — the authoring-error fixture for FIX 2.
    fn menagerie_targeting(target: &str) -> Archive {
        let mut m = menagerie("InstanceFunctor");
        if let GeneratorAction::Functor { map_object, .. } = &mut m.connections[0].action {
            map_object[0] = ("Pet".to_string(), target.to_string());
        }
        m
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_declared_but_absent_target_fails_closed() {
        // The peer (`taxonomy`) IS present, but the functor declares
        // `Pet ↦ DOESNOTEXIST`, a concept the peer does not hold. That is a
        // declared-but-unrealizable grounding — it must FAIL CLOSED with a typed
        // GroundTargetAbsent, NOT be silently installed ungrounded (the FIX 2 bug).
        let mut peers = BTreeMap::new();
        peers.insert("taxonomy".to_string(), taxonomy());
        assert_eq!(
            ground_declared(&menagerie_targeting("DOESNOTEXIST"), &peers),
            Err(LinkError::GroundTargetAbsent {
                kind: "Pet".to_string(),
                target: "DOESNOTEXIST".to_string(),
                peer: "taxonomy".to_string(),
            }),
            "a present peer that lacks the declared concept fails closed, never a silent drop"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_multi_level_chain_triggers_the_one_level_guard() {
        // Build a 2-level chain of LOADED ontologies: `menagerie` grounds into
        // `taxonomy`, and `taxonomy` ITSELF grounds into `base`. `taxonomy` is thus
        // both a grounding target (of menagerie) and a grounding source (into base)
        // — the unsupported multi-level chain. `ground_loaded_set` must refuse the
        // whole pass, LOUD, rather than silently dangle (the FIX 3 contract).
        use pr4xis::ontology::meta::OntologyName;

        // A pure base: one node, no grounding functor.
        let base = Archive {
            nodes: alloc::vec![Definition {
                kind: "Concept".to_string(),
                name: "Root".to_string(),
                edges: alloc::vec![],
                axioms: alloc::vec![],
                lexical: Some("the root of the base".to_string()),
            }],
            connections: alloc::vec![],
        };
        // A taxonomy that is ALSO a source: its `Dog ⊑ Animal` nodes plus a
        // grounding functor typing its `Concept` nodes into `base:Root`.
        let mut taxonomy_src = taxonomy();
        taxonomy_src.connections.push(Connection {
            kind: "InstanceFunctor".to_string(),
            source: "taxonomy".to_string(),
            target: "base".to_string(),
            action: GeneratorAction::Functor {
                map_object: alloc::vec![("Concept".to_string(), "Root".to_string())],
                map_morphism: alloc::vec![("instantiates".to_string(), "Subsumption".to_string())],
            },
            laws: alloc::vec!["PreservesTyping".to_string()],
        });

        let set: alloc::vec::Vec<Rc<RuntimeOntology>> = alloc::vec![
            Rc::new(
                materialize(
                    menagerie("InstanceFunctor"),
                    OntologyName::new_static("menagerie")
                )
                .unwrap()
            ),
            Rc::new(materialize(taxonomy_src, OntologyName::new_static("taxonomy")).unwrap()),
            Rc::new(materialize(base, OntologyName::new_static("base")).unwrap()),
        ];
        let mut set = set;
        assert_eq!(
            ground_loaded_set(&mut set, English::sample_static()),
            Err(LinkError::GroundingTargetIsSource {
                target: "taxonomy".to_string(),
            }),
            "a target that is itself a source is the unsupported multi-level chain — LOUD, not dangling"
        );
    }

    // -----------------------------------------------------------------------
    // W2.2 DORMANCY DISCHARGE — a REAL committed `.prx` that CARRIES an
    // into-English grounding functor, loaded FROM DISK BYTES fail-closed, so the
    // functor-as-data path is proven end-to-end (not only an inline Rust
    // `Connection` literal). A menagerie declares `Canine ↦ english_wordnet:s-dog`
    // as data; the loaded node then inherits English's is-a chain and "is rex an
    // animal" answers through WordNet's `s-dog ⊑ s-mammal ⊑ s-animal`, while an
    // UNDECLARED `Mineral` node (surface an animal word) does NOT link (§9).
    // -----------------------------------------------------------------------

    /// The committed into-English menagerie `.prx` — a domain taxonomy that
    /// CARRIES its into-English `InstanceFunctor` as data (nodes + one Connection),
    /// admitted only against its baked root.
    const MENAGERIE_INTO_ENGLISH_PRX: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/taxonomy/menagerie_into_english.prx"
    ));

    /// The trusted Merkle root of [`MENAGERIE_INTO_ENGLISH_PRX`] — the fail-closed
    /// integrity pin (file ⇔ pin coherence asserted below; regenerate with the
    /// `--ignored regenerate_menagerie_into_english_prx` test and bake the printed
    /// root here).
    const MENAGERIE_INTO_ENGLISH_ROOT_HEX: &str =
        "fa28bdaa273d8553764770a152e007ff72fda7f8cb47da3caa9bac05741722e6";

    /// The into-English menagerie as an [`Archive`] — the SOURCE OF TRUTH the
    /// committed `.prx` must equal. TWO nodes: a DECLARED `Canine` (`rex`, typed by
    /// the functor as `english_wordnet:s-dog`) and an UNDECLARED `Mineral`
    /// (`salmon`, whose surface is literally an English animal word yet carries NO
    /// functor entry). ONE Connection: the into-English `InstanceFunctor`
    /// (`Canine ↦ s-dog`, `denotes ↦ Subsumption`). Built from code ONLY here (the
    /// regenerate + drift guard); the runtime loads it from the committed bytes.
    fn menagerie_into_english_archive() -> Archive {
        Archive {
            nodes: alloc::vec![
                Definition {
                    kind: "Canine".to_string(),
                    name: "rex".to_string(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: Some("a companion dog kept in the menagerie".to_string()),
                },
                Definition {
                    kind: "Mineral".to_string(),
                    name: "salmon".to_string(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: Some(
                        "a specimen labelled with the fish word 'salmon' but typed a \
                         Mineral — an UNDECLARED control: it carries no into-English \
                         functor entry, so it must NOT link to 'animal' (§9)."
                            .to_string()
                    ),
                },
            ],
            connections: alloc::vec![Connection {
                // The grounding-functor kind the meta-ontology discriminator reaches
                // to InstanceFunctor — NOT an ad-hoc string.
                kind: "InstanceFunctor".to_string(),
                source: "menagerie".to_string(),
                target: ENGLISH_ONTOLOGY.to_string(),
                action: GeneratorAction::Functor {
                    // DECLARED-TYPE grounding: the node KIND `Canine` types as the
                    // WordNet synset `s-dog` (its original_id) — NOT a surface match.
                    map_object: alloc::vec![("Canine".to_string(), "s-dog".to_string())],
                    // The typing edge asserts Subsumption (is-a into English's chain).
                    map_morphism: alloc::vec![("denotes".to_string(), "Subsumption".to_string())],
                },
                laws: alloc::vec!["PreservesTyping".to_string()],
            }],
        }
    }

    /// The committed into-English `.prx` path.
    fn committed_menagerie_into_english_prx_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/taxonomy/menagerie_into_english.prx")
    }

    /// Load the committed menagerie `.prx` FROM DISK BYTES — FAIL-CLOSED against
    /// [`MENAGERIE_INTO_ENGLISH_ROOT_HEX`]; a tampered/stale file is refused.
    fn load_menagerie_into_english() -> Archive {
        let root =
            pr4xis_runtime::address::ContentAddress::from_hex(MENAGERIE_INTO_ENGLISH_ROOT_HEX)
                .expect("MENAGERIE_INTO_ENGLISH_ROOT_HEX is valid 64-hex");
        pr4xis_runtime::load::load(MENAGERIE_INTO_ENGLISH_PRX, root)
            .expect("committed menagerie_into_english.prx must load against its baked root")
    }

    /// REGENERATE PATH (`--ignored`, WRITES): re-emit the committed
    /// `menagerie_into_english.prx` from [`menagerie_into_english_archive`], then
    /// PRINT the root to bake into [`MENAGERIE_INTO_ENGLISH_ROOT_HEX`]. Mirrors the
    /// USC `regenerate_usc_legal_sources_functor_prx` pattern. Run:
    /// `cargo test -p pr4xis-domains --features prx -- --ignored regenerate_menagerie_into_english_prx`.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    #[ignore = "WRITES the committed .prx; run explicitly to regenerate"]
    fn regenerate_menagerie_into_english_prx() {
        let archive = menagerie_into_english_archive();
        let bytes = pr4xis_runtime::load::emit(&archive).expect("encode menagerie .prx");
        let out = committed_menagerie_into_english_prx_path();
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).expect("create data/taxonomy/");
        }
        std::fs::write(&out, &bytes).expect("write menagerie_into_english.prx");
        let root = archive.root().expect("root").to_hex();
        eprintln!("wrote {} ({} bytes)", out.display(), bytes.len());
        println!("MENAGERIE_INTO_ENGLISH_ROOT_HEX = {root}");
    }

    /// STALENESS GUARD (normal suite): the committed `.prx` must be a FRESH emit of
    /// the source-of-truth archive, and its baked root must match.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn committed_menagerie_into_english_prx_matches_source() {
        let archive = menagerie_into_english_archive();
        let fresh = pr4xis_runtime::load::emit(&archive).expect("encode");
        let committed = std::fs::read(committed_menagerie_into_english_prx_path())
            .expect("read committed menagerie_into_english.prx");
        assert_eq!(
            fresh, committed,
            "committed menagerie_into_english.prx is STALE — regenerate with \
             `--ignored regenerate_menagerie_into_english_prx` and bake the printed root"
        );
        assert_eq!(
            archive.root().unwrap().to_hex(),
            MENAGERIE_INTO_ENGLISH_ROOT_HEX,
            "MENAGERIE_INTO_ENGLISH_ROOT_HEX is STALE vs the committed archive"
        );
    }

    /// THE DORMANCY-DISCHARGE PROOF: load the committed `.prx` FROM DISK BYTES
    /// fail-closed, ground its declared into-English functor against the transient
    /// English target peer, materialize, compose, and answer a CONCEPTUAL question
    /// ("is rex an animal") through English's synset taxonomy — proving the
    /// functor-as-data path end-to-end from bytes, not an inline literal.
    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn the_committed_into_english_prx_answers_through_english_from_disk_bytes() {
        use crate::cognitive::linguistics::composed::{ComposedReasoner, GroundedConcept};
        use crate::cognitive::linguistics::english::{English, LexicalReasoner};
        use pr4xis::ontology::meta::OntologyName;
        use pr4xis_runtime::ontology::subsumption_kind;

        // FROM DISK BYTES, fail-closed: the committed file loads against its root…
        let archive = load_menagerie_into_english();
        // …and a WRONG root is refused (the fail-closed leg).
        assert!(
            pr4xis_runtime::load::load(
                MENAGERIE_INTO_ENGLISH_PRX,
                pr4xis_runtime::address::ContentAddress::of(b"wrong")
            )
            .is_err(),
            "a wrong root is refused — the load is fail-closed"
        );

        // Ground the declared into-English functor against the TRANSIENT English
        // target peer (the lean pre-materialize atom layer), materialize, compose.
        let english = English::sample_static();
        let mut peers: BTreeMap<String, Archive> = BTreeMap::new();
        peers.insert(
            ENGLISH_ONTOLOGY.to_string(),
            project_archive_with_forms(english),
        );
        let grounded =
            ground_declared(&archive, &peers).expect("grounds against the English target peer");
        let onto = materialize(grounded, OntologyName::new_static("menagerie"))
            .expect("the grounded menagerie materializes");
        let composed = ComposedReasoner::new(english, alloc::vec![Rc::new(onto)]);

        // GATE (i): English is NEVER a loaded ontology — it entered only as a
        // transient grounding target, never a `RuntimeOntology` slot.
        assert!(
            composed
                .loaded()
                .iter()
                .all(|o| o.id().as_str() != ENGLISH_ONTOLOGY),
            "english_wordnet must never be a loaded ontology"
        );

        let subsumption = subsumption_kind();
        let loaded_id = |surface: &str| {
            composed
                .lookup(surface)
                .iter()
                .copied()
                .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::Loaded(_))))
                .unwrap_or_else(|| panic!("no loaded concept resolves for {surface:?}"))
        };
        let english_id = |surface: &str| {
            composed
                .lookup(surface)
                .iter()
                .copied()
                .find(|&id| matches!(composed.decode(id), Some(GroundedConcept::English(_))))
                .unwrap_or_else(|| panic!("no english concept resolves for {surface:?}"))
        };

        let rex = loaded_id("rex");
        let animal = english_id("animal");
        // DECLARED: rex (kind Canine ↦ s-dog) reaches English's `animal` through
        // WordNet's own s-dog ⊑ s-mammal ⊑ s-animal chain — answered FROM DISK.
        assert!(
            composed.reaches(rex, animal, &subsumption),
            "rex (declared Canine ↦ s-dog) is an animal via English's is-a chain"
        );
        // GATE (ii) §9 negative control: the UNDECLARED `salmon` (kind Mineral, an
        // animal-word surface) does NOT link — no declared functor entry, no path.
        let salmon = loaded_id("salmon");
        assert!(
            !composed.reaches(salmon, animal, &subsumption),
            "the undeclared Mineral 'salmon' must NOT link to animal (surface-match declined, §9)"
        );
        // GATE (iii) directional: English's `animal` does not reach the loaded rex.
        assert!(
            !composed.reaches(animal, rex, &subsumption),
            "reaches into English is directional — animal does not reach the loaded node"
        );
    }

    // -----------------------------------------------------------------------
    // RESOLVER EQUIVALENCE — the per-name resolvers the archived-view pass
    // uses must agree, address-for-address, with the owned reference path
    // (`project_archive_with_forms` + `type_lens`'s first-match scan) they
    // replaced. These are the keystone legs for the +82.79 MiB transient kill.
    // -----------------------------------------------------------------------

    /// ADDRESS EQUALITY, exhaustive over the sample corpus: for EVERY node the
    /// full with-forms projection mints (every synset AND every form atom),
    /// the per-name resolver [`english_atom_address`] returns exactly the
    /// address `type_lens`'s first-match scan over that projection would bind
    /// — and an absent name resolves to `None`, never a fabricated atom.
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn per_name_english_resolver_agrees_with_the_full_projection() {
        let english = English::sample_static();
        let full = project_archive_with_forms(english);
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for node in &full.nodes {
            if !seen.insert(node.name.as_str()) {
                continue; // first-match semantics: only the first node binds
            }
            let expected = node.address().expect("projected node addresses");
            let resolved = english_atom_address(english, &node.name)
                .expect("per-name resolution does not fault")
                .unwrap_or_else(|| panic!("{:?} must resolve per-name", node.name));
            assert_eq!(
                resolved, expected,
                "per-name atom for {:?} must equal the full projection's",
                node.name
            );
        }
        assert_eq!(
            english_atom_address(english, "no-such-synset-or-word").unwrap(),
            None,
            "an absent name resolves to None (the caller's fail-closed GroundTargetAbsent)"
        );
    }

    /// PASS EQUIVALENCE: `ground_loaded_set` (archived views, per-name atoms)
    /// produces the SAME grounded archive as the reference `ground_declared`
    /// over an owned peer set — same root, edge-for-edge — for both a loaded
    /// peer target (menagerie → taxonomy) and the seeded English target; and a
    /// SECOND pass over the grounded set changes nothing and re-materializes
    /// nothing (roots stable — the zero-decode idempotent re-install case).
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn ground_loaded_set_matches_the_owned_reference_pass_and_is_idempotent() {
        use pr4xis::ontology::meta::OntologyName;

        // Loaded-peer leg: menagerie (InstanceFunctor → taxonomy) + taxonomy.
        let mut set = alloc::vec![
            Rc::new(
                materialize(
                    menagerie("InstanceFunctor"),
                    OntologyName::new_static("menagerie")
                )
                .unwrap()
            ),
            Rc::new(materialize(taxonomy(), OntologyName::new_static("taxonomy")).unwrap()),
        ];
        ground_loaded_set(&mut set, English::sample_static()).expect("grounds");

        // The reference: ground_declared over the owned peer set.
        let mut peers = BTreeMap::new();
        peers.insert("taxonomy".to_string(), taxonomy());
        let reference = ground_declared(&menagerie("InstanceFunctor"), &peers).unwrap();
        assert_eq!(
            set[0].to_owned_archive().unwrap(),
            reference,
            "the archived-view pass minted exactly the reference edges"
        );

        // English leg: the committed into-English menagerie grounds identically
        // through the per-name resolver and through the owned English peer.
        let english = English::sample_static();
        let mut en_set = alloc::vec![Rc::new(
            materialize(
                menagerie_into_english_archive(),
                OntologyName::new_static("menagerie")
            )
            .unwrap()
        )];
        ground_loaded_set(&mut en_set, english).expect("grounds into English");
        let mut en_peers = BTreeMap::new();
        en_peers.insert(
            ENGLISH_ONTOLOGY.to_string(),
            project_archive_with_forms(english),
        );
        let en_reference = ground_declared(&menagerie_into_english_archive(), &en_peers).unwrap();
        assert_eq!(
            en_set[0].to_owned_archive().unwrap(),
            en_reference,
            "the per-name English resolver minted exactly the reference edges"
        );

        // Idempotence: a second pass changes no root (the zero-decode skip).
        let roots: alloc::vec::Vec<_> = en_set.iter().map(|o| o.root()).collect();
        ground_loaded_set(&mut en_set, english).expect("re-grounds");
        assert_eq!(
            en_set
                .iter()
                .map(|o| o.root())
                .collect::<alloc::vec::Vec<_>>(),
            roots,
            "an already-grounded set re-grounds to the same roots (idempotent)"
        );
    }
}
