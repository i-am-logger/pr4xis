//! Runnable axioms of the [`PraxisKnowledgeGraph`](super::ontology)
//! ontology — each `verify()` is a predicate that RUNS against real
//! machinery (the `.prx` realisation, the `ArchiveIntoGraph` functor, and
//! the registries), never a doc-comment claim
//! (`feedback_praxis_as_compiler_self_describing`).
//!
//! Gated on `feature = "prx"`, where the realisation and the storage
//! substratum exist.
//!
//! ## What ships in #272
//!
//! - The **eight storage axioms** of [`OntologyArchiveStorage`](crate::formal::meta::ontology_archive),
//!   re-exported as graph axioms. The [`ArchiveIntoGraph`](super::functor)
//!   functor is the formal statement that they ARE graph axioms; its
//!   full-and-faithful law (`FunctorLawPreservation`) proves the carry-over
//!   is structure-preserving.
//! - [`FunctorLawPreservation`] — the embedding is functorial AND fully
//!   faithful (the machine proof behind `KIND = FullyFaithful`).
//! - [`AxiomBindingComplete`] + [`UnboundReferenceFailsClosed`] — the
//!   load-time re-bind handler table (`pr4xis::ontology::axiom_by_name`):
//!   every persisted `AxiomNode` re-binds to a live predicate, and an
//!   unresolvable binding fails closed.
//! - [`LensLawPreservation`] (delegates to the lens harness over every
//!   registered lens), [`SelectionClosedUnderEdgeKinds`] (the relational-image
//!   selection over the materialized closure), and [`PairOntologyRoundTrip`]
//!   (the `(data, binding)` pair round-trip — extended whole-graph in #271-B).
//!
//! ## What #271 effort B adds
//!
//! - [`GraphSnapshotReproducible`] + the whole-graph leg of
//!   [`PairOntologyRoundTrip`] — now that the [`snapshot`](super::snapshot)
//!   emit/load/re-bind machinery exists, these run the REAL select → emit →
//!   pin → reload → re-bind loop (before B a passing `verify()` would have
//!   been a stub).
//! - [`LensBindingComplete`] — the lens analogue of [`AxiomBindingComplete`]
//!   over `lens_by_name`.
//! - [`NodeAddressIsDefinitionBearing`] — node addresses derive from each
//!   node's DEFINITION through the runtime's one typed lowering, never from
//!   its bare name (the G5 closure at graph scale).
//!
//! ## Deferred, deliberately NOT declared here (no-stub doctrine)
//!
//! `AdjunctionTrianglePreservation` waits on an adjunction in the graph's
//! binding set (the `AdjunctionTriangleLaw` machinery already exists in
//! `pr4xis::category::laws`); `AttestationChainVerifiable` waits on
//! TUF/in-toto/SLSA + an authenticated signed pin. A passing `verify()` for
//! either today would be a stub. (`IntegrityClaimVerifiable`, W3C SRI, is
//! now realised in [`archive`] and carried over in `domain_axioms`.)

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use pr4xis::category::Concept;
use pr4xis::category::laws::{fully_faithful_law_axioms, functor_law_axioms};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, axiom_by_name, axiom_constructors};

use pr4xis_runtime::emit::definition_of;

use super::functor::ArchiveIntoGraph;
use super::ontology::{
    PraxisKnowledgeGraphConcept, PraxisKnowledgeGraphRelation, PraxisKnowledgeGraphRelationKind,
};
use super::snapshot::{
    EdgeKindFilter, GraphNode, RootSet, SnapshotError, compute_reachable, emit_snapshot,
    load_snapshot,
};
use crate::formal::meta::ontology_archive::axioms as archive;
use crate::formal::meta::well_behaved_lens::harness::RoundTripHarnessAllVerified;
use crate::formal::meta::well_behaved_lens::{lens_by_name, lens_registrations};
use alloc::string::ToString;

/// The `ArchiveIntoGraph` embedding preserves functor structure AND is
/// fully faithful — identity + composition + faithful (injective on each
/// hom-set) + full onto its image. This is the machine proof that the
/// archive's persisted nodes and edges round-trip into the graph without
/// loss or collision, the carry-over witness for the eight re-exported
/// storage axioms. Mac Lane (1971) CWM Ch. I §3-4.
pub struct FunctorLawPreservation;

impl Axiom for FunctorLawPreservation {
    fn verify(&self) -> Verdict {
        let mut laws = functor_law_axioms::<ArchiveIntoGraph>();
        laws.extend(fully_faithful_law_axioms::<ArchiveIntoGraph>());
        for law in laws {
            if law.verify().is_err() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "FunctorLawPreservation",
        "the ArchiveIntoGraph embedding preserves id/composition and is fully faithful (faithful + full onto image)",
        "Mac Lane (1971) Categories for the Working Mathematician Ch. I §3-4"
    );
}

pr4xis::register_axiom!(FunctorLawPreservation, constructor);

/// Every registered axiom constructor re-binds by its stable name:
/// reconstruct each axiom, resolve it through the handler table
/// ([`axiom_by_name`]), and confirm the resolved axiom carries the same
/// name. A persisted `AxiomNode` therefore re-binds to a live predicate on
/// load — no dangling unbound binding survives.
pub struct AxiomBindingComplete;

impl Axiom for AxiomBindingComplete {
    fn verify(&self) -> Verdict {
        for a in axiom_constructors() {
            let name = a.name();
            // The resolved axiom's identity is checked through the typed
            // `OntologyName` equality, not a raw `&str ==`.
            match axiom_by_name(name.as_str()) {
                Some(b) if b.name() == name => {}
                _ => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "AxiomBindingComplete",
        "every persisted AxiomNode re-binds to a registered axiom constructor by its stable name",
        "graph-rebind invariant; Samuel et al. (2010) Survivable Key Compromise in Software Update Systems (TUF), CCS '10"
    );
}

pr4xis::register_axiom!(AxiomBindingComplete, constructor);

/// A binding the receiver cannot resolve fails closed: resolving an
/// unregistered axiom name returns `None`, so the load gate refuses rather
/// than silently producing a partial graph.
pub struct UnboundReferenceFailsClosed;

impl Axiom for UnboundReferenceFailsClosed {
    fn verify(&self) -> Verdict {
        let unbound = "__praxis_unregistered_axiom_binding__";
        if axiom_by_name(unbound).is_some() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "UnboundReferenceFailsClosed",
        "an unresolvable binding identity resolves to None (fail-closed), never a partial graph",
        "load-gate discipline; Samuel et al. (2010) Survivable Key Compromise in Software Update Systems (TUF), CCS '10"
    );
}

pr4xis::register_axiom!(UnboundReferenceFailsClosed, constructor);

/// Every persisted `LensNode`'s get/put round-trip law holds on
/// rehydration — delegated to the lens harness, which dispatches each
/// registered lens on its `RoundTripFidelity` and checks its signature
/// against `praxis.lock`. Foster et al. (2007) §2.2.
pub struct LensLawPreservation;

impl Axiom for LensLawPreservation {
    fn verify(&self) -> Verdict {
        if RoundTripHarnessAllVerified.verify().is_err() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "LensLawPreservation",
        "every persisted LensNode's get/put round-trip law holds on rehydration (the lens harness verifies all registered lenses)",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(LensLawPreservation, constructor);

/// A slice computed from `(roots, edge-kind filter)` contains every node
/// reachable from the roots through the chosen kinds, and no filtered edge
/// leaves it (a leaving reference would be an `UnboundReference`). Driven
/// through the ontological relational-image selection
/// ([`compute_reachable`]) over the
/// transitive closure the macro materializes — a direct categorical query,
/// not a re-derived traversal.
pub struct SelectionClosedUnderEdgeKinds;

impl Axiom for SelectionClosedUnderEdgeKinds {
    fn verify(&self) -> Verdict {
        use PraxisKnowledgeGraphConcept as C;
        use PraxisKnowledgeGraphRelationKind as K;
        // Representative selection: root = MerkleRoot, filter = Subsumption —
        // now driven through the first-class selection (#271 effort B); the
        // teeth below are preserved verbatim from the inlined BFS.
        let sub = compute_reachable(
            &RootSet(vec![C::MerkleRoot]),
            &EdgeKindFilter(vec![K::Subsumption]),
        );
        // Closure — no filtered edge leaves the slice (a leaving edge is an
        // `UnboundReference`).
        if !sub.unbound.is_empty() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        // Non-vacuous — MerkleRoot reaches ContentAddressableNode through
        // Subsumption, but the slice is NOT the whole graph (SourcePin is
        // unreachable from MerkleRoot via is_a, so it is excluded).
        if !sub.nodes.contains(&C::ContentAddressableNode) || sub.nodes.contains(&C::SourcePin) {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SelectionClosedUnderEdgeKinds",
        "a slice from (roots, edge-kind filter) contains every node reachable through those kinds, and no filtered edge leaves it",
        "graph-reachability invariant; Mac Lane (1971) Categories for the Working Mathematician Ch. I §1"
    );
}

pr4xis::register_axiom!(SelectionClosedUnderEdgeKinds, constructor);

/// The pair-ontology `(structural data, binding)` round-trips together for
/// the archive subset: the structural data survives the emit/load lens
/// ([`archive::EmitLoadWellBehaved`], Foster 2007 §2.2) and a behavioural
/// node's binding re-binds by name on the receiving side ([`axiom_by_name`])
/// — the composition a future wire transfer's slice → emit → receive → re-bind
/// would reduce to (the whole-graph leg awaits the #271 snapshot machinery; the
/// inter-node negotiation itself is deferred).
pub struct PairOntologyRoundTrip;

impl Axiom for PairOntologyRoundTrip {
    fn verify(&self) -> Verdict {
        // Structural leg — the archived data round-trips through rkyv bytes.
        if archive::EmitLoadWellBehaved.verify().is_err() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        // Binding leg — a persisted axiom binding re-binds by its name.
        if axiom_by_name(AxiomBindingComplete.name().as_str()).is_none() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        // Whole-graph leg (#271 effort B) — the (data, binding) pair round-trips
        // over a REAL snapshot, no longer the archive subset only: the
        // structural node-set survives emit/load AND a behavioural AxiomNode
        // re-binds by name. This lands now that the snapshot machinery exists.
        {
            use PraxisKnowledgeGraphConcept as C;
            use PraxisKnowledgeGraphRelationKind as K;
            let slice = compute_reachable(
                &RootSet(vec![C::MerkleRoot]),
                &EdgeKindFilter(vec![K::Subsumption]),
            );
            // Cover BOTH behavioural re-bind paths: a registered AxiomNode
            // (axiom_by_name) AND a registered LensNode (lens_by_name) — so this
            // leg exercises the lens re-bind end-to-end, not just the axiom one.
            let bindings = vec![
                GraphNode {
                    kind: C::AxiomNode,
                    identity: AxiomBindingComplete.name().as_str().to_string(),
                },
                GraphNode {
                    kind: C::LensNode,
                    identity: "usc_title_18@pl-119-90".to_string(),
                },
            ];
            let Ok((gz, root)) = emit_snapshot(&slice, &bindings) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            let Ok(env) = load_snapshot(&gz, &root) else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            // Structural data preserved EXACTLY — every slice concept survives as
            // a ConceptNode, and no spurious ConceptNode is added or dropped.
            let concept_nodes: Vec<&str> = env
                .nodes
                .iter()
                .filter(|n| n.node_kind == "ConceptNode")
                .map(|n| n.identity.as_str())
                .collect();
            let structural_ok = concept_nodes.len() == slice.nodes.len()
                && slice
                    .nodes
                    .iter()
                    .all(|c| concept_nodes.iter().any(|&id| id == c.name()));
            // Both behavioural bindings re-bound by name (load would have refused
            // with UnboundReference otherwise).
            let axiom_ok = env.nodes.iter().any(|n| {
                n.node_kind == "AxiomNode" && n.identity == AxiomBindingComplete.name().as_str()
            });
            let lens_ok = env
                .nodes
                .iter()
                .any(|n| n.node_kind == "LensNode" && n.identity == "usc_title_18@pl-119-90");
            if !(structural_ok && axiom_ok && lens_ok) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "PairOntologyRoundTrip",
        "the (structural data, binding) pair round-trips together — the archive subset AND a whole-graph snapshot: structural concepts survive emit/load and the AxiomNode + LensNode bindings re-bind by name",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2; graph-rebind invariant"
    );
}

pr4xis::register_axiom!(PairOntologyRoundTrip, constructor);

/// The frozen `GraphVersion` (`MerkleRoot`) of [`GraphSnapshotReproducible`]'s
/// fixed slice — a known-answer pin captured once and committed (the
/// [`archive::MerkleHashDeterministic`] KAT pattern). It pins `root1` against a
/// STRUCTURALLY-INDEPENDENT literal, not a value re-derived by the emit path
/// under test, so the pin leg is not `h(x) == h(x)`. A change here is a
/// conscious GraphVersion bump (an address-preimage or rkyv-layout change), the
/// desired property.
///
/// The literal is the content digest (BLAKE3) of the rkyv-serialized envelope bytes, portable
/// across every praxis target because rkyv runs with the default `FixedUsize`
/// (u32, little-endian — `crates/domains/Cargo.toml` enables no
/// `pointer_width_*` / `big_endian` feature, so the layout is identical on
/// native and wasm32). Adding such a feature or bumping the rkyv wire layout
/// would invalidate this literal on the affected target — surfaced precisely by
/// this axiom failing there (the conscious bump).
const GRAPH_SNAPSHOT_KAT_ROOT: &str =
    "96d1c8b09d35209f3f2108b26c7c8e73126ffa7437178918cf4fa22ad5fd1885";

/// A whole-graph [`GraphSnapshot`](super::snapshot) round-trips reproducibly
/// and fail-closed: a fixed, closed slice plus a behavioural binding emit to a
/// stable `MerkleRoot` (known-answer), reload through the real gate, and refuse
/// a poisoned slice or an unbindable node. Lands now that the #271 snapshot
/// machinery exists — before it, a passing `verify()` would have been a stub.
pub struct GraphSnapshotReproducible;

impl Axiom for GraphSnapshotReproducible {
    fn verify(&self) -> Verdict {
        use PraxisKnowledgeGraphConcept as C;
        use PraxisKnowledgeGraphRelationKind as K;
        let roots = RootSet(vec![C::MerkleRoot]);
        let filter = EdgeKindFilter(vec![K::Subsumption]);
        let registered = || {
            vec![GraphNode {
                kind: C::AxiomNode,
                identity: AxiomBindingComplete.name().as_str().to_string(),
            }]
        };

        // Non-vacuous + closed.
        let slice = compute_reachable(&roots, &filter);
        if slice.nodes.len() < 2 || !slice.unbound.is_empty() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        // Address-level reproducibility: a SECOND independent emit yields the
        // same MerkleRoot (backed by RkyvDeterminism + the canonical name-keyed
        // order). Not the gz bytes — gzip byte-stability is not a registered
        // axiom; CompressionRoundTrip proves only gunzip(gzip(x)) == x.
        let (Ok((gz1, root1)), Ok((_gz2, root2))) = (
            emit_snapshot(&slice, &registered()),
            emit_snapshot(&compute_reachable(&roots, &filter), &registered()),
        ) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        if root1 != root2 {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        // Pin against the frozen, structurally-independent KAT literal, then
        // load through the real gate.
        if root1 != GRAPH_SNAPSHOT_KAT_ROOT || load_snapshot(&gz1, &root1).is_err() {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        // POISON: a different binding changes the MerkleRoot, so the poisoned
        // blob is refused against root1 (the address binds the slice).
        let poisoned = vec![GraphNode {
            kind: C::AxiomNode,
            identity: UnboundReferenceFailsClosed.name().as_str().to_string(),
        }];
        let Ok((poisoned_gz, _)) = emit_snapshot(&slice, &poisoned) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        if !matches!(
            load_snapshot(&poisoned_gz, &root1),
            Err(SnapshotError::MerkleRootMismatch { .. })
        ) {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        // UNBOUND: an unregistered binding passes the gate (its own genuine
        // root) but is refused at eager re-bind, materializing nothing.
        let unbound = vec![GraphNode {
            kind: C::AxiomNode,
            identity: "__praxis_unregistered_axiom_binding__".to_string(),
        }];
        let Ok((unbound_gz, unbound_root)) = emit_snapshot(&slice, &unbound) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        if !matches!(
            load_snapshot(&unbound_gz, &unbound_root),
            Err(SnapshotError::UnboundReference { .. })
        ) {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "GraphSnapshotReproducible",
        "a fixed graph slice emits to a stable MerkleRoot (known-answer), reloads through the fail-closed gate, and refuses a poisoned slice or an unbindable node",
        "Merkle (1987) A Digital Signature Based on a Conventional Encryption Function, CRYPTO '87; Benet (2014) IPFS: Content-Addressed, Versioned, P2P File System; Lamb & Zacchiroli (2021) Reproducible Builds, IEEE Software 39(2); Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020) BLAKE3"
    );
}

pr4xis::register_axiom!(GraphSnapshotReproducible, constructor);

/// Every persisted node's content address is DEFINITION-BEARING — derived,
/// through the runtime's ONE typed lowering
/// ([`definition_of`]), from what the
/// slice carries of the node (kind + name + in-slice morphisms + gloss),
/// never from its bare name. Verified against the REAL emit/load loop: each
/// rehydrated `ConceptNode`'s address equals an independently recomputed
/// lowering of its typed concept, and the same name over a different
/// definition addresses differently — the G5 refutation leg a name-pair hash
/// could not pass.
pub struct NodeAddressIsDefinitionBearing;

impl Axiom for NodeAddressIsDefinitionBearing {
    fn verify(&self) -> Verdict {
        use PraxisKnowledgeGraphConcept as C;
        use PraxisKnowledgeGraphRelationKind as K;
        let slice = compute_reachable(
            &RootSet(vec![C::MerkleRoot]),
            &EdgeKindFilter(vec![K::Subsumption]),
        );
        let Ok((gz, root)) = emit_snapshot(&slice, &[]) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let Ok(env) = load_snapshot(&gz, &root) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };

        // Agreement leg: every emitted ConceptNode's address IS the typed
        // lowering's address, recomputed here independently of the emit path.
        for c in &slice.nodes {
            let morphisms: Vec<PraxisKnowledgeGraphRelation> = slice
                .in_edges
                .iter()
                .filter(|m| m.from == *c)
                .cloned()
                .collect();
            let Ok(expected) = definition_of(&C::ConceptNode, c, &morphisms).address() else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            };
            let agrees = env
                .nodes
                .iter()
                .any(|n| n.identity == c.name() && n.address == expected.to_hex());
            if !agrees {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }

        // G5 refutation leg: the SAME name over a DIFFERENT definition (a
        // node's full in-slice lowering vs its edge-less projection) must
        // address differently.
        let Some(edged) = slice
            .nodes
            .iter()
            .find(|c| slice.in_edges.iter().any(|m| m.from == **c))
        else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let morphisms: Vec<PraxisKnowledgeGraphRelation> = slice
            .in_edges
            .iter()
            .filter(|m| m.from == *edged)
            .cloned()
            .collect();
        let bare: [PraxisKnowledgeGraphRelation; 0] = [];
        let (Ok(full), Ok(empty)) = (
            definition_of(&C::ConceptNode, edged, &morphisms).address(),
            definition_of(&C::ConceptNode, edged, &bare).address(),
        ) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        if full == empty {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "NodeAddressIsDefinitionBearing",
        "every persisted node's content address derives from its DEFINITION (kind + name + in-slice morphisms + gloss) through the runtime's one typed lowering — the same name over a different definition addresses differently, so address agreement is agreement on meaning, not on spelling",
        "Merkle (1987) A Digital Signature Based on a Conventional Encryption Function, CRYPTO '87; Farrell, Kutscher & Dannewitz (2013) RFC 6920 Naming Things with Hashes; Benet (2014) IPFS: Content-Addressed, Versioned, P2P File System"
    );
}

pr4xis::register_axiom!(NodeAddressIsDefinitionBearing, constructor);

/// Every registered `LensNode` re-binds to its lens registration HANDLE by key
/// (`lens_by_name`) — the lens analogue of [`AxiomBindingComplete`]. An
/// asymmetric re-bind: a `LensNode` resolves to a `&'static LensRegistration`
/// (a fn-pointer record), not a runnable lens value (the lens trait is not
/// dyn-compatible). Non-vacuous on native (production lenses are linked);
/// empty/fail-closed on wasm32.
pub struct LensBindingComplete;

impl Axiom for LensBindingComplete {
    fn verify(&self) -> Verdict {
        for reg in lens_registrations() {
            // Re-derive the lookup key INDEPENDENTLY from (source_name,
            // source_version) — the same `name@version` `register_lens!` builds
            // — and resolve THAT through the live registry. Drift between the
            // stored key and the derived one is a catchable defect; this is not
            // `x == x` (the resolver is queried with an independently-built key).
            let derived = alloc::format!("{}@{}", reg.source_name, reg.source_version);
            match lens_by_name(&derived) {
                Some(r) if r.key == reg.key => {}
                _ => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
            }
            // Discriminating power — a bogus key must NOT resolve (fail-closed),
            // proving the resolver the load-time `LensNode` re-bind depends on
            // actually distinguishes, rather than accepting anything.
            let bogus = alloc::format!("{}__praxis_unregistered_lens__", reg.key);
            if lens_by_name(&bogus).is_some() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "LensBindingComplete",
        "every registered LensNode re-binds by its independently re-derived name@version key, and the resolver refuses an unregistered key",
        "graph-rebind invariant; Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2"
    );
}

pr4xis::register_axiom!(LensBindingComplete, constructor);

/// The runnable domain axioms of the knowledge-graph ontology — spliced
/// into [`PraxisKnowledgeGraphOntology::axioms`](super::ontology) under
/// `feature = "prx"`. The eight archive axioms are re-exported (the functor
/// is the formal carry-over); the three whole-graph axioms exercise the
/// embedding and the re-bind handler table.
pub fn domain_axioms() -> Vec<Box<dyn Axiom>> {
    vec![
        // The storage substratum (OntologyArchiveStorage), carried over by
        // the fully-faithful ArchiveIntoGraph functor.
        Box::new(archive::MerkleHashDeterministic),
        Box::new(archive::MerkleDedupCorrect),
        Box::new(archive::CompressionRoundTrip),
        Box::new(archive::RkyvDeterminism),
        Box::new(archive::EmitLoadWellBehaved),
        Box::new(archive::SourceHashFaithfulness),
        Box::new(archive::LoadGateFailsClosed),
        Box::new(archive::IntegrityClaimVerifiable),
        // Whole-graph axioms.
        Box::new(FunctorLawPreservation),
        Box::new(AxiomBindingComplete),
        Box::new(UnboundReferenceFailsClosed),
        Box::new(LensLawPreservation),
        Box::new(SelectionClosedUnderEdgeKinds),
        Box::new(PairOntologyRoundTrip),
        // Whole-graph snapshot axioms (#271 effort B) — unlocked now that the
        // GraphSnapshot emit/load/re-bind machinery exists.
        Box::new(GraphSnapshotReproducible),
        Box::new(NodeAddressIsDefinitionBearing),
        Box::new(LensBindingComplete),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every domain axiom holds against the real machinery (the `.prx`
    /// realisation, the ArchiveIntoGraph functor, and the registries).
    ///
    /// `LensLawPreservation` delegates to the full round-trip lens harness over
    /// every registered lens (parsing every small on-disk source). Its dedicated
    /// owner — `ci_gate_passes` (well_behaved_lens::harness::tests) — runs that
    /// IDENTICAL pass on every push, so re-running it here is pure duplication.
    /// This test therefore SKIPS that one leg's `verify()` (matched by its stable
    /// axiom name) while still asserting it is PRESENT in `domain_axioms()` (the
    /// cheap non-vacuity check), so the axiom set stays complete and the costly
    /// pass is run exactly once, by its owner. All other legs run in full.
    #[test]
    fn all_domain_axioms_hold() {
        // Guard against a future vacuous AxiomBindingComplete: the re-bind
        // handler table must actually carry registrations to round-trip
        // (the three #272 axioms register via the constructor arm).
        assert!(
            !axiom_constructors().is_empty(),
            "AXIOM_CONSTRUCTORS must be populated for the re-bind axioms to have teeth"
        );
        // Non-vacuity guard for LensBindingComplete: production lenses must be
        // linked on native, so the lens re-bind path has teeth (analogue of the
        // AXIOM_CONSTRUCTORS guard above).
        assert!(
            !lens_registrations().is_empty(),
            "LENS_REGISTRATIONS must be populated for LensBindingComplete to have teeth"
        );
        // The leg whose heavy pass `ci_gate_passes` already owns. Verified PRESENT
        // below (non-vacuity), then skipped in the verify loop.
        let lens_law = LensLawPreservation.name();
        assert!(
            domain_axioms().iter().any(|ax| ax.name() == lens_law),
            "LensLawPreservation must remain in domain_axioms() — its presence (not a \
             re-run of its harness pass) is what this set asserts; ci_gate_passes runs the pass"
        );
        for ax in domain_axioms() {
            if ax.name() == lens_law {
                continue; // owned by ci_gate_passes — see the doc comment above
            }
            ax.verify()
                .unwrap_or_else(|c| panic!("axiom failed: {}", c.meta().name.as_str()));
        }
    }

    /// Prints the [`GraphSnapshotReproducible`] fixture's MerkleRoot so a
    /// conscious GraphVersion bump (an address-preimage or rkyv-layout
    /// change) can re-bless [`GRAPH_SNAPSHOT_KAT_ROOT`].
    #[test]
    #[ignore = "prints the snapshot KAT root for re-pinning; not an assertion"]
    fn print_graph_snapshot_kat_root() {
        use PraxisKnowledgeGraphConcept as C;
        use PraxisKnowledgeGraphRelationKind as K;
        let slice = compute_reachable(
            &RootSet(vec![C::MerkleRoot]),
            &EdgeKindFilter(vec![K::Subsumption]),
        );
        let bindings = vec![GraphNode {
            kind: C::AxiomNode,
            identity: AxiomBindingComplete.name().as_str().to_string(),
        }];
        let (_gz, root) = emit_snapshot(&slice, &bindings).expect("the KAT fixture emits");
        println!("GRAPH_SNAPSHOT_KAT_ROOT = {root}");
    }
}
