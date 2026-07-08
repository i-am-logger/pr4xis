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
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis::category::category_theory::is_grounding_functor_kind;
use pr4xis_runtime::archive::Archive;
use pr4xis_runtime::connection::GeneratorAction;
use pr4xis_runtime::grounding::{LinkError, ground, type_lens};
use pr4xis_runtime::ontology::{RuntimeOntology, materialize, subsumption_kind};

use crate::cognitive::linguistics::english::English;
use crate::cognitive::linguistics::english::bridge::{
    ENGLISH_ONTOLOGY, project_archive_with_forms,
};

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
/// # Contract: grounding targets are pure bases (one-level grounding), guarded
///
/// The peer set is frozen from the loaded archives *before* any slot is grounded,
/// so a functor's target atoms are the target's *pre-grounding* node addresses. A
/// grounding TARGET must therefore be a pure base — an ontology that declares no
/// grounding functor of its own (every current target: `LegalSources`, English, a
/// domain taxonomy). A multi-level chain — a target that *itself* grounds into a
/// further base — is NOT supported: re-materializing that target (to add ITS
/// grounding edges) changes its node content-addresses, which the already-frozen
/// peer set would not reflect, so a source grounded against its pre-grounding
/// addresses would dangle SILENTLY. This is no longer left to prose: it is enforced
/// by [`guard_one_level_grounding`], a FAIL-CLOSED runtime guard that refuses the
/// whole pass with [`LinkError::GroundingTargetIsSource`] when any loaded ontology
/// is BOTH a grounding target and a grounding source. Grounding-target-that-grounds
/// is future work (it needs the peer set re-derived in dependency order, a
/// topological pass); until then the guard makes the unsupported case LOUD.
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
    // The peer set, built ONCE: every loaded ontology's owned archive, addressed by
    // name — a grounding functor's target atoms are its peer archive's node
    // addresses. Base ontologies (the grounding targets) are small; a large corpus
    // is present too but is a grounding SOURCE, never looked up as a target. This is
    // the SINGLE owning decode per ontology for the whole pass: it is reused by the
    // one-level guard, by the per-slot peer lookups, AND as each slot's own
    // pre-grounding archive (`raw`) — no O(N) re-decode per slot, no third decode in
    // the guard.
    let mut peers: BTreeMap<String, Archive> = BTreeMap::new();
    // Seed English as a grounding TARGET peer — so a `.prx` that DECLARES an
    // into-English InstanceFunctor (its `kind ↦ synset` map_object) grounds its
    // typed nodes onto WordNet synsets, exactly as a domain source grounds into
    // `LegalSources`. The peer is the LEAN pre-materialize atom layer
    // ([`project_archive_with_forms`] — a transient `Vec<Definition>` dropped when
    // this pass returns): CATEGORICALLY NOT [`english_runtime_ontology`], the
    // measured +183 MiB `apply_then_materialize`. English is NEVER installed as a
    // loaded `RuntimeOntology`; it enters grounding only as this transient target
    // archive, so `loaded` never gains an `english_wordnet` slot. A loaded set with
    // no into-English functor simply never resolves against this peer (it costs the
    // one transient projection and nothing else).
    peers.insert(
        ENGLISH_ONTOLOGY.to_string(),
        project_archive_with_forms(english),
    );
    for o in loaded.iter() {
        // INVARIANT (not a swallow): `to_owned_archive` is `ArchiveLens::get` over a
        // buffer that `materialize` `bytecheck`-VALIDATED and holds immutable, so a
        // MATERIALIZED `RuntimeOntology` never fails to decode. The `if let Ok` is a
        // defensive guard the invariant makes unreachable in practice
        // (`materialized_ontology_to_owned_archive_never_fails` proves it); on the
        // impossible failure the ontology is simply absent from the peer set and its
        // slot skipped below — never a silently mis-grounded install.
        if let Ok(archive) = o.to_owned_archive() {
            peers.insert(o.id().as_str().to_string(), archive);
        }
    }
    // FAIL-CLOSED one-level guard (the documented contract, enforced): refuse the
    // whole pass if any loaded ontology is both a grounding target and a grounding
    // source — the unsupported multi-level chain that would dangle silently. Reads
    // each ontology's connections from the frozen peer set (no re-decode).
    guard_one_level_grounding(loaded, &peers)?;
    for slot in loaded.iter_mut() {
        let name = slot.id().clone();
        // The slot's own pre-grounding archive IS its frozen peer entry — reuse it
        // rather than decode a third time. Absent only if its `to_owned_archive`
        // failed above (the documented-unreachable defensive skip).
        let Some(raw) = peers.get(name.as_str()) else {
            continue;
        };
        // Per connection: a missing peer ARCHIVE DEFERS this ontology (a later pass
        // completes it once the target loads — base-first makes that the common
        // no-defer case). EVERY other fault — a declared target NAME absent from a
        // present peer, a root skew, a codec fault — is LOUD: it aborts the pass so
        // the caller refuses rather than installing a silently-mis-grounded set.
        let grounded = match ground_declared(raw, &peers) {
            Ok(grounded) => grounded,
            Err(LinkError::MissingPeerArchive { .. }) => continue,
            Err(loud) => return Err(loud),
        };
        if &grounded == raw {
            // Already grounded (or nothing to ground) — no content change, so no
            // re-materialize (roots stay stable, the invariant "loading X adds only
            // X's data" holds for every other ontology).
            continue;
        }
        if let Ok(regrounded) = materialize(grounded, name) {
            *slot = Rc::new(regrounded);
        }
    }
    Ok(())
}

/// FAIL-CLOSED one-level-grounding guard — refuse the whole pass if any LOADED
/// ontology is BOTH a grounding target and a grounding source.
///
/// [`ground_loaded_set`] freezes the peer set before grounding, so a grounding
/// target's atoms are its *pre-grounding* addresses. If a loaded ontology used as
/// a target were itself a source, grounding it would re-materialize it — shifting
/// the very addresses another source grounded against, leaving that source's edges
/// dangling SILENTLY. That multi-level chain is unsupported; this guard makes it
/// LOUD ([`GroundingTargetIsSource`](LinkError::GroundingTargetIsSource)) instead
/// of dangling. The supported cases are untouched: into-English targets
/// `english_wordnet` (never a loaded slot, so never in `sources`), and
/// USC→`LegalSources` targets a pure base (declares no grounding functor).
fn guard_one_level_grounding(
    loaded: &[Rc<RuntimeOntology>],
    peers: &BTreeMap<String, Archive>,
) -> Result<(), LinkError> {
    // A loaded ontology is a SOURCE if it declares any grounding functor; the
    // TARGETS are the ontologies those functors point at. Read each ontology's
    // connections from the FROZEN peer set the caller already decoded — no re-decode
    // (English is in `peers` but never in `loaded`, so it is never a source).
    let mut sources: BTreeSet<String> = BTreeSet::new();
    let mut targets: BTreeSet<String> = BTreeSet::new();
    for o in loaded {
        let name = o.id().as_str();
        let Some(archive) = peers.get(name) else {
            continue;
        };
        for conn in &archive.connections {
            if is_grounding_functor_kind(&conn.kind) {
                sources.insert(name.to_string());
                targets.insert(conn.target.clone());
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
}
