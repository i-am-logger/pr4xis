//! Whole-graph `GraphSnapshot` machinery (#271 effort B) — select a slice of
//! the [`PraxisKnowledgeGraph`](super), content-address it as a Merkle DAG,
//! and rehydrate it through the same fail-closed admit gate the `.prx`
//! archive uses, re-binding behavioural nodes to the running binary.
//!
//! This is the whole-graph generalisation of the archive storage substratum:
//! it REUSES the prx primitives ([`source_content_hash`], gzip/rkyv, the
//! `raw_hash::verify` typed-claim gate) and expresses selection
//! ONTOLOGICALLY — as the **relational image** of a [`RootSet`] under an
//! [`EdgeKindFilter`], computed through the category's own
//! [`morphisms_from`](pr4xis::category::Category::morphisms_from) over the
//! transitive closure the `ontology!` macro materializes at compile time
//! (NOT a re-derived traversal; see [`compute_reachable`]). It adds NO new
//! ontology edges (the selection concepts `RootSet` / `EdgeKindFilter` /
//! `ReachableSubgraph` / `UnboundReference` and their `has_a` edges already
//! exist, OUTSIDE the 12-concept `ArchiveIntoGraph` image), so the
//! fully-faithful functor stays certified.
//!
//! # Literature
//!
//! - **Merkle (1987)** CRYPTO '87; **Benet (2014)** IPFS arXiv:1407.3561;
//!   Git content-addressed DAG — per-node content addressing.
//! - **Lamb & Zacchiroli (2021)** IEEE Software 39(2) (arXiv:2104.06020) —
//!   reproducible builds (the same inputs reproduce the same address).
//! - **NIST (2015)** FIPS 180-4 §6.2 (SHA-256).

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec::Vec};

use pr4xis::category::{Category, Concept};
use pr4xis::ontology::axiom_by_name;

use super::ontology::{
    PraxisKnowledgeGraphCategory, PraxisKnowledgeGraphConcept, PraxisKnowledgeGraphRelation,
    PraxisKnowledgeGraphRelationKind,
};
use crate::formal::meta::artifact_identity::ontology::{
    ClaimData, IdentityClaim, IdentityConcept, VerificationResult,
};
use crate::formal::meta::artifact_identity::schemes::raw_hash;
use crate::formal::meta::well_behaved_lens::lens_by_name;
// Shared `.prx` primitives — the SAME content-hash, gzip/rkyv codecs, and
// typed-claim integrity gate the archive uses (never a parallel hash/codec).
use crate::social::software::markup::xml::owl::prx::{gunzip, gzip, source_content_hash};

// =============================================================================
// Selection — RootSet / EdgeKindFilter / ReachableSubgraph / compute_reachable
// =============================================================================

/// The seed concepts a selection starts from. A runtime realisation of the
/// ontology's `RootSet` concept (already declared, outside the archive image).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSet(pub Vec<PraxisKnowledgeGraphConcept>);

/// The edge kinds a selection traverses. A runtime realisation of the
/// ontology's `EdgeKindFilter` concept. Membership is set-based
/// (`filter.0.contains(&kind)`), generalising the scalar `kind == filter` of
/// the inlined BFS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeKindFilter(pub Vec<PraxisKnowledgeGraphRelationKind>);

/// The closed slice [`compute_reachable`] produces — a runtime realisation of
/// the ontology's `ReachableSubgraph` concept (which `has_a UnboundReference`).
///
/// `nodes` is the reachable concept set; `in_edges` are the filtered-kind
/// edges entirely within the slice; `unbound` are the filtered-kind edges
/// that LEAVE the slice (`from ∈ nodes`, `to ∉ nodes`) — the
/// `UnboundReference`s that, if non-empty, mean the slice is NOT closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachableSubgraph {
    pub nodes: Vec<PraxisKnowledgeGraphConcept>,
    pub in_edges: Vec<PraxisKnowledgeGraphRelation>,
    pub unbound: Vec<PraxisKnowledgeGraphRelation>,
}

/// The slice of the knowledge graph reachable from `roots` under the relation
/// kinds in `filter`.
///
/// This is a DIRECT categorical query, **not a graph traversal**. The
/// `ontology!` macro materializes the transitive closure of each transitive
/// relation (Subsumption / Parthood / Causation — OBO-RO `transitive_over`;
/// Floyd–Warshall at macro-expand time), so the closed category's
/// [`morphisms_from`](pr4xis::category::Category::morphisms_from) of a root
/// ALREADY yields every transitively-reachable target in ONE step — there is
/// no frontier to walk. Reachability is therefore the **relational image** of
/// the roots under the filtered, already-closed morphisms. (One-step
/// completeness holds for the OBO-RO TRANSITIVE kinds the macro closes —
/// Subsumption / Parthood / Causation; a non-transitive filter, e.g. Identity
/// or Opposition, has no multi-hop closure, so any further-hop edge simply
/// leaves the one-step image and is recorded as `unbound` — fail-closed at
/// emit, never silently lost.)
///
/// The category is PARTIAL — heterogeneous kinds do not compose (`compose`
/// returns `None`, #166) — so the image is taken **per kind**. A filtered edge
/// that leaves the slice (`from` in, `to` out) is a genuine `UnboundReference`
/// (a cross-kind or dangling reference the partial category does not absorb),
/// recorded in [`ReachableSubgraph::unbound`]; the slice is CLOSED iff that is
/// empty. (Iteration here only *collects* the image set — the reachability
/// itself is the materialized closure, queried through the category's own
/// `morphisms_from`, never re-derived.)
pub fn compute_reachable(roots: &RootSet, filter: &EdgeKindFilter) -> ReachableSubgraph {
    // The filtered arrows out of the roots — the Category's own
    // `morphisms_from` accessor over the CLOSED relation, not a hand-rolled
    // walk over the whole edge list. Because the closure is materialized, this
    // single image is already the full transitively-reachable set.
    let image_targets =
        |objs: &[PraxisKnowledgeGraphConcept]| -> Vec<PraxisKnowledgeGraphRelation> {
            objs.iter()
                .flat_map(PraxisKnowledgeGraphCategory::morphisms_from)
                .filter(|m| filter.0.contains(&m.kind))
                .collect()
        };

    // Node set = the roots together with the image targets (one step: the
    // closure already contains every multi-hop edge, so this is closed under
    // each transitive kind).
    let mut nodes = roots.0.clone();
    for target in image_targets(&roots.0).iter().map(|m| m.to) {
        if !nodes.contains(&target) {
            nodes.push(target);
        }
    }

    // The slice's own morphisms: every filtered arrow out of an in-slice node,
    // partitioned into those that stay in the slice (its `in_edges`) and those
    // that leave it (`UnboundReference`s — cross-kind or out-of-slice).
    let (in_edges, unbound) = image_targets(&nodes)
        .into_iter()
        .partition(|m| nodes.contains(&m.to));

    ReachableSubgraph {
        nodes,
        in_edges,
        unbound,
    }
}

// =============================================================================
// Name ⇄ type bijections — the ontological identities the wire carries.
// =============================================================================
//
// Every wire identity is a stable ONTOLOGY NAME (`Concept::name` for a node,
// the canonical kind name for an edge), re-resolved to its typed value on
// load — never a positional enum index or u32 discriminant. (`Concept::name`
// is the Lemon canonical form today; grounding it in an English lexical entry
// is a parked future review — see project_concept_name_lexical_grounding.)

/// Re-resolve a [`PraxisKnowledgeGraphConcept`] from its stable
/// [`Concept::name`], over the ontology's own `variants()` enumerator.
/// Fail-closed `None` for an unknown name.
fn concept_from_name(name: &str) -> Option<PraxisKnowledgeGraphConcept> {
    PraxisKnowledgeGraphConcept::variants()
        .into_iter()
        .find(|c| c.name() == name)
}

/// Canonical name of a relation kind — the by-name identity of an edge's
/// `RelationKind` (which is a category KIND, not a `Concept`, and is not
/// rkyv-serializable, so it crosses the wire as its name). Total over the five
/// fixed kinds.
///
/// TRACKED (project_271b_deferred_review_findings): this hand-rolled match is
/// the ONE spot in the snapshot where an identity is not re-resolved through an
/// ontology accessor — because the `ontology!` macro generates `RelationKind`
/// with only `Debug` (no `Concept` / `name()` / `variants()`). The clean fix is
/// UPSTREAM: generate `RelationKind::name()` + `variants()` in `pr4xis-derive`,
/// then re-resolve kinds through that accessor. Until then these arms are
/// canonical and `kind_name` / `kind_from_name` MUST stay in sync.
fn kind_name(kind: PraxisKnowledgeGraphRelationKind) -> &'static str {
    use PraxisKnowledgeGraphRelationKind as K;
    match kind {
        K::Identity => "Identity",
        K::Subsumption => "Subsumption",
        K::Parthood => "Parthood",
        K::Causation => "Causation",
        K::Opposition => "Opposition",
    }
}

/// Re-resolve a relation kind from its canonical name. Fail-closed `None`.
fn kind_from_name(name: &str) -> Option<PraxisKnowledgeGraphRelationKind> {
    use PraxisKnowledgeGraphRelationKind as K;
    Some(match name {
        "Identity" => K::Identity,
        "Subsumption" => K::Subsumption,
        "Parthood" => K::Parthood,
        "Causation" => K::Causation,
        "Opposition" => K::Opposition,
        _ => return None,
    })
}

// =============================================================================
// Wire form — name-keyed nodes + Merkle-edges-by-address (the GraphSnapshot).
// =============================================================================

/// A node to persist, in the pair-ontology shape `(structural kind, binding
/// identity)`: `kind` is the structural-knowledge node concept it is (a
/// `ConceptNode` for a structural concept, an `AxiomNode`/`LensNode` for a
/// behavioural one); `identity` is the stable [`Concept::name`] of the
/// concept, or the registered axiom/lens binding name the receiver re-binds by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    pub kind: PraxisKnowledgeGraphConcept,
    pub identity: String,
}

/// The content address of a node — the SHA-256 of its `(node-kind, identity)`
/// name-pair, so two nodes share an address iff they share their ontological
/// identity (the `MerkleDedupCorrect` iff, at graph scale). The `\u{1f}` unit
/// separator is unambiguous (names carry no control chars).
fn node_address(node_kind: &str, identity: &str) -> String {
    // Precondition: node-kind + identity are ontology names / registry keys
    // (`Concept::name`, axiom/lens binding names) — control-char-free — so the
    // `\u{1f}` unit separator is unambiguous. Guarded in debug/test builds.
    debug_assert!(
        !node_kind.contains('\u{1f}') && !identity.contains('\u{1f}'),
        "node identity must be control-char-free for the address separator to be unambiguous"
    );
    source_content_hash(format!("{node_kind}\u{1f}{identity}").as_bytes())
}

/// One persisted node, name-keyed and per-node content-addressed (the Merkle
/// DAG's `ContentAddressableNode`).
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SnapshotNode {
    /// The structural-node concept this node is, by [`Concept::name`]
    /// (`"ConceptNode"` / `"AxiomNode"` / `"LensNode"` / …).
    pub node_kind: String,
    /// The stable identity — a concept name, or a registered binding name.
    pub identity: String,
    /// Per-node content address = [`node_address`]`(node_kind, identity)`.
    pub address: String,
}

/// One persisted edge — a `MerkleEdge`-by-name between two node identities.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SnapshotEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
}

/// The rkyv-serializable `GraphSnapshot` envelope: a canonically-ordered set
/// of name-keyed nodes + Merkle-edges. Its `GraphVersion` is the SHA-256 of
/// these bytes (a DERIVED label, NOT stored inside them), so re-emitting the
/// same slice reproduces the same address.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SnapshotEnvelope {
    pub nodes: Vec<SnapshotNode>,
    pub edges: Vec<SnapshotEdge>,
}

/// Fail-closed errors from emitting/loading a `GraphSnapshot` — every variant
/// a refusal, mirroring the `.prx` `PrxError` grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    /// rkyv serialization / (bytecheck-validated) deserialization failed.
    Rkyv(String),
    /// gzip (RFC 1952) compression / decompression failed.
    Gzip(String),
    /// The selected slice is not closed — a filtered edge leaves it (an
    /// `UnboundReference`), so it cannot be emitted as a self-contained graph.
    SelectionLeftSlice {
        from: String,
        to: String,
        kind: String,
    },
    /// Two nodes share an ontological identity `(node_kind, identity)` — the
    /// canonical order would collapse them; refuse rather than silently dedup.
    AddressCollision { node_kind: String, identity: String },
    /// The `MerkleRoot` re-derived from the installed bytes does not match the
    /// trusted `GraphVersion` pin — a poisoned or wrong snapshot, refused
    /// before anything is materialized.
    MerkleRootMismatch { expected: String, found: String },
    /// A rehydrated node's binding name does not resolve in THIS binary's
    /// registries (`axiom_by_name` / `lens_by_name` / the ontology's concepts,
    /// or an unsupported behavioural kind) — version skew. Fail-closed: the
    /// whole snapshot is refused, never a partial graph.
    UnboundReference { node_kind: String, identity: String },
    /// A content-hash `IntegrityClaim` could not be evaluated (the `raw_hash`
    /// verifier returned `Unverifiable`). Fail-closed.
    IntegrityUnverifiable { reason: String },
}

impl core::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SnapshotError::Rkyv(m) => write!(f, "snapshot rkyv error: {m}"),
            SnapshotError::Gzip(m) => write!(f, "snapshot gzip error: {m}"),
            SnapshotError::SelectionLeftSlice { from, to, kind } => write!(
                f,
                "selection is not closed: edge {from} --{kind}--> {to} leaves the slice"
            ),
            SnapshotError::AddressCollision {
                node_kind,
                identity,
            } => write!(
                f,
                "address collision: two nodes share the identity ({node_kind}, {identity})"
            ),
            SnapshotError::MerkleRootMismatch { expected, found } => write!(
                f,
                "snapshot MerkleRoot mismatch: pin is {expected}, installed bytes hash to \
                 {found} — refusing to materialize"
            ),
            SnapshotError::UnboundReference {
                node_kind,
                identity,
            } => write!(
                f,
                "unbound reference: ({node_kind}, {identity}) does not resolve in this binary \
                 — refusing the snapshot"
            ),
            SnapshotError::IntegrityUnverifiable { reason } => {
                write!(f, "snapshot integrity unverifiable: {reason}")
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

// =============================================================================
// Load — gunzip → bytecheck → MerkleRoot gate → eager re-bind by name.
// =============================================================================

/// Discharge the `MerkleRoot` content-hash `IntegrityClaim` over `bytes`
/// against the trusted `GraphVersion` pin, through the SAME
/// [`raw_hash::verify`] primitive the `.prx` load gate uses — never a
/// `String==`. `raw_hash::verify` re-hashes `bytes`, so the pin is checked
/// against bytes actually present.
fn verify_merkle_root(bytes: &[u8], trusted_pin: &str) -> Result<(), SnapshotError> {
    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: ClaimData::Sha256(trusted_pin.to_string()),
    };
    match raw_hash::verify(&claim, bytes) {
        VerificationResult::Verified(_) => Ok(()),
        VerificationResult::Mismatch { expected, actual } => {
            Err(SnapshotError::MerkleRootMismatch {
                expected,
                found: actual,
            })
        }
        VerificationResult::Unverifiable { reason } => {
            Err(SnapshotError::IntegrityUnverifiable { reason })
        }
    }
}

/// Resolve a rehydrated node's `(node_kind, identity)` against THIS binary's
/// registries — the eager re-bind. A `ConceptNode` must name a live concept
/// (`concept_from_name`); an `AxiomNode` must re-bind to a registered axiom
/// (`axiom_by_name`); a `LensNode` to a registered lens handle
/// (`lens_by_name` — an asymmetric re-bind: a `&'static LensRegistration`, not
/// a runnable lens value); any other behavioural kind (FunctorNode /
/// AdjunctionNode) has no resolver and is refused. Fail-closed `Err` on any
/// miss. On wasm32 the axiom/lens slices are empty, so behavioural nodes are
/// refused there — the correct fail-closed behaviour.
fn rebind_node(node: &SnapshotNode) -> Result<(), SnapshotError> {
    let resolved = match node.node_kind.as_str() {
        "ConceptNode" => concept_from_name(&node.identity).is_some(),
        "AxiomNode" => axiom_by_name(&node.identity).is_some(),
        "LensNode" => lens_by_name(&node.identity).is_some(),
        // FunctorNode / AdjunctionNode / anything else: no resolver exists yet
        // (the functor/adjunction registries are provenance-only), so a node of
        // that kind cannot be re-bound — refuse rather than half-materialize.
        _ => false,
    };
    if resolved {
        Ok(())
    } else {
        Err(SnapshotError::UnboundReference {
            node_kind: node.node_kind.clone(),
            identity: node.identity.clone(),
        })
    }
}

/// Load a `GraphSnapshot` blob into a verified, re-bound [`SnapshotEnvelope`],
/// gated on the trusted `GraphVersion` pin. Mirrors the `.prx` admit gate.
///
/// 1. **DECODE** — gunzip (RFC 1952); rkyv `from_bytes` with bytecheck (a
///    corrupt/truncated blob fails closed).
/// 2. **MERKLEROOT GATE** — re-derive the content address from the rkyv bytes
///    and discharge it against `trusted_pin` via [`verify_merkle_root`]; a
///    poisoned slice mutates the address and is refused here.
/// 3. **EAGER RE-BIND** — resolve every node's `(node_kind, identity)` against
///    this binary's registries ([`rebind_node`]); on the FIRST unresolvable
///    binding, refuse the whole snapshot (no partial graph). Also confirm each
///    edge's kind + endpoints re-resolve by name.
/// 4. Return the verified, fully-rebindable envelope.
pub fn load_snapshot(gz: &[u8], trusted_pin: &str) -> Result<SnapshotEnvelope, SnapshotError> {
    // 1. Decode.
    let bytes = gunzip(gz).map_err(|e| SnapshotError::Gzip(e.to_string()))?;
    let mut aligned = rkyv::util::AlignedVec::<16>::new();
    aligned.extend_from_slice(&bytes);
    let envelope: SnapshotEnvelope =
        rkyv::from_bytes::<SnapshotEnvelope, rkyv::rancor::Error>(&aligned)
            .map_err(|e| SnapshotError::Rkyv(e.to_string()))?;

    // 2. MerkleRoot gate — over the EXACT decoded bytes (gzip-level-independent).
    verify_merkle_root(&bytes, trusted_pin)?;

    // 3. Eager re-bind — every node must resolve, every edge's kind + endpoints
    // must re-resolve by name. Fail-closed on the first miss.
    for node in &envelope.nodes {
        rebind_node(node)?;
    }
    for edge in &envelope.edges {
        // Re-bind the edge ONTOLOGICALLY — not "are the three names resolvable"
        // but "is this actually a morphism of the category". The materialized
        // closure makes the membership check exact + one-step
        // (`morphisms_from` over the closed relation), so a fabricated edge
        // between two real concepts that is NOT a morphism is refused —
        // symmetric with the node re-bind against the live registries, never
        // trusting the byte-gate alone.
        let is_morphism = match (
            kind_from_name(&edge.kind),
            concept_from_name(&edge.from),
            concept_from_name(&edge.to),
        ) {
            (Some(k), Some(f), Some(t)) => PraxisKnowledgeGraphCategory::morphisms_from(&f)
                .iter()
                .any(|m| m.to == t && m.kind == k),
            _ => false,
        };
        if !is_morphism {
            return Err(SnapshotError::UnboundReference {
                node_kind: "RelationEdge".to_string(),
                identity: format!("{} --{}--> {}", edge.from, edge.kind, edge.to),
            });
        }
    }

    Ok(envelope)
}

/// Emit a `GraphSnapshot` of a closed `slice` (plus the behavioural
/// `bindings`) as a gzip-wrapped rkyv blob, returning it with its
/// `GraphVersion` (the `MerkleRoot`).
///
/// 1. **SELECT guard** — refuse a slice that is not closed (`unbound`
///    non-empty), enforcing `SelectionClosedUnderEdgeKinds` on what is emitted.
/// 2. **Name + content-address** every node: the slice's concepts as
///    `ConceptNode`s (identity = [`Concept::name`]), plus the `bindings` (the
///    behavioural `AxiomNode`/`LensNode`s by registered name); each addressed
///    by [`node_address`]. Edges are the slice's `in_edges`, by name.
/// 3. **Canonical order** nodes by `(node_kind, identity)` and edges by
///    `(kind, from, to)` — a total order (identities are unique) — and refuse
///    any shared identity ([`SnapshotError::AddressCollision`]).
/// 4. **Address** = SHA-256 of the rkyv bytes (the `GraphVersion`, derived),
///    then gzip. Re-emitting the same slice reproduces the same address.
pub fn emit_snapshot(
    slice: &ReachableSubgraph,
    bindings: &[GraphNode],
) -> Result<(Vec<u8>, String), SnapshotError> {
    // 1. The slice must be closed.
    if let Some(u) = slice.unbound.first() {
        return Err(SnapshotError::SelectionLeftSlice {
            from: u.from.name().to_string(),
            to: u.to.name().to_string(),
            kind: kind_name(u.kind).to_string(),
        });
    }

    // 2. Name + address every node — structural concepts as ConceptNodes, then
    // the behavioural bindings.
    let structural = slice.nodes.iter().map(|c| GraphNode {
        kind: PraxisKnowledgeGraphConcept::ConceptNode,
        identity: c.name().to_string(),
    });
    let mut nodes: Vec<SnapshotNode> = structural
        .chain(bindings.iter().cloned())
        .map(|gn| {
            let node_kind = gn.kind.name().to_string();
            let address = node_address(&node_kind, &gn.identity);
            SnapshotNode {
                node_kind,
                identity: gn.identity,
                address,
            }
        })
        .collect();
    let mut edges: Vec<SnapshotEdge> = slice
        .in_edges
        .iter()
        .map(|m| SnapshotEdge {
            from: m.from.name().to_string(),
            to: m.to.name().to_string(),
            kind: kind_name(m.kind).to_string(),
        })
        .collect();

    // 3. Canonical total order + collision refusal.
    nodes.sort_by(|a, b| (&a.node_kind, &a.identity).cmp(&(&b.node_kind, &b.identity)));
    edges.sort_by(|a, b| (&a.kind, &a.from, &a.to).cmp(&(&b.kind, &b.from, &b.to)));
    if let Some(w) = nodes
        .windows(2)
        .find(|w| (w[0].node_kind == w[1].node_kind) && (w[0].identity == w[1].identity))
    {
        return Err(SnapshotError::AddressCollision {
            node_kind: w[0].node_kind.clone(),
            identity: w[0].identity.clone(),
        });
    }

    // 4. Serialize, content-address (GraphVersion = MerkleRoot), gzip.
    let envelope = SnapshotEnvelope { nodes, edges };
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&envelope)
        .map(|v| v.to_vec())
        .map_err(|e| SnapshotError::Rkyv(e.to_string()))?;
    let merkle_root = source_content_hash(&bytes);
    let gz = gzip(&bytes).map_err(|e| SnapshotError::Gzip(e.to_string()))?;
    Ok((gz, merkle_root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use PraxisKnowledgeGraphConcept as C;
    use PraxisKnowledgeGraphRelationKind as K;

    /// The single-root single-filter image reproduces the exact slice
    /// `SelectionClosedUnderEdgeKinds` asserts: from `MerkleRoot` through
    /// `Subsumption`, the slice reaches `ContentAddressableNode`, excludes
    /// `SourcePin`, and is CLOSED (no edge leaves it — the materialized
    /// closure guarantees the one-step image is closed under a transitive
    /// kind).
    #[test]
    fn single_root_single_filter_reproduces_original_slice() {
        let sub = compute_reachable(
            &RootSet(alloc::vec![C::MerkleRoot]),
            &EdgeKindFilter(alloc::vec![K::Subsumption]),
        );
        assert!(
            sub.unbound.is_empty(),
            "the is_a slice from MerkleRoot is closed"
        );
        assert!(
            sub.nodes.contains(&C::ContentAddressableNode),
            "MerkleRoot reaches ContentAddressableNode via Subsumption"
        );
        assert!(
            !sub.nodes.contains(&C::SourcePin),
            "the slice is not the whole graph — SourcePin is excluded"
        );
    }

    /// Multi-root / multi-kind slices a wider region than a single root.
    #[test]
    fn multi_root_multi_kind_widens_the_slice() {
        let single = compute_reachable(
            &RootSet(alloc::vec![C::MerkleRoot]),
            &EdgeKindFilter(alloc::vec![K::Subsumption]),
        );
        let multi = compute_reachable(
            &RootSet(alloc::vec![C::MerkleRoot, C::GraphSnapshot]),
            &EdgeKindFilter(alloc::vec![K::Subsumption, K::Parthood]),
        );
        assert!(
            multi.nodes.len() >= single.nodes.len(),
            "more roots + more kinds cannot shrink the reachable set"
        );
        // Every node the single-root Subsumption slice reached is still in the
        // wider slice (monotonicity).
        for n in &single.nodes {
            assert!(multi.nodes.contains(n), "wider slice keeps {n:?}");
        }
    }

    // ── emit / load (the GraphSnapshot wire protocol) ───────────────────

    /// A fixed, closed slice — the `MerkleRoot` Subsumption image.
    fn fixed_slice() -> ReachableSubgraph {
        compute_reachable(
            &RootSet(alloc::vec![C::MerkleRoot]),
            &EdgeKindFilter(alloc::vec![K::Subsumption]),
        )
    }

    /// A registered axiom name (registered via `register_axiom!(_, constructor)`)
    /// — `axiom_by_name` resolves it, so a snapshot binding to it re-binds.
    const REGISTERED_AXIOM: &str = "AxiomBindingComplete";

    /// Two independent emits of the same slice + bindings produce the same
    /// `GraphVersion` (`MerkleRoot`) — reproducibility at the address level
    /// (the canonical order + name-keyed addressing pin it; no enum-index or
    /// HashMap iteration leaks into the bytes).
    #[test]
    fn snapshot_emit_is_address_deterministic() {
        let bindings = alloc::vec![GraphNode {
            kind: C::AxiomNode,
            identity: REGISTERED_AXIOM.to_string(),
        }];
        let (_gz1, root1) = emit_snapshot(&fixed_slice(), &bindings).expect("emit 1");
        let (_gz2, root2) = emit_snapshot(&fixed_slice(), &bindings).expect("emit 2");
        assert_eq!(root1, root2, "same slice → same MerkleRoot");
    }

    /// emit → load round-trips the slice, preserves the node set, gives every
    /// node a distinct content address, and RE-BINDS the behavioural node by
    /// name (the registered axiom resolves through `axiom_by_name`).
    #[test]
    fn snapshot_round_trips_preserves_nodes_and_rebinds() {
        let slice = fixed_slice();
        let bindings = alloc::vec![GraphNode {
            kind: C::AxiomNode,
            identity: REGISTERED_AXIOM.to_string(),
        }];
        let (gz, root) = emit_snapshot(&slice, &bindings).expect("emit");
        let env = load_snapshot(&gz, &root).expect("load + gate + rebind");

        // One ConceptNode per slice concept, plus the one AxiomNode binding.
        assert_eq!(env.nodes.len(), slice.nodes.len() + 1);
        // Every node content address is distinct (no spurious collapse).
        let mut addrs: Vec<&str> = env.nodes.iter().map(|n| n.address.as_str()).collect();
        addrs.sort_unstable();
        let before = addrs.len();
        addrs.dedup();
        assert_eq!(addrs.len(), before, "node addresses are distinct");
        // The behavioural node is present + named — it re-bound (load would
        // have refused with UnboundReference otherwise).
        assert!(
            env.nodes
                .iter()
                .any(|n| n.node_kind == "AxiomNode" && n.identity == REGISTERED_AXIOM),
            "the AxiomNode binding survives and re-binds"
        );
    }

    /// Two nodes sharing an identity are refused at emit (no silent dedup).
    #[test]
    fn snapshot_emit_rejects_address_collision() {
        let dup = alloc::vec![
            GraphNode {
                kind: C::AxiomNode,
                identity: "dup".to_string()
            },
            GraphNode {
                kind: C::AxiomNode,
                identity: "dup".to_string()
            },
        ];
        let err = emit_snapshot(&fixed_slice(), &dup).expect_err("duplicate identity must refuse");
        assert!(
            matches!(err, SnapshotError::AddressCollision { .. }),
            "got {err:?}"
        );
    }

    /// A slice that is NOT closed (a filtered edge leaves it) cannot be emitted.
    #[test]
    fn snapshot_emit_rejects_unclosed_slice() {
        let open = ReachableSubgraph {
            nodes: alloc::vec![C::MerkleRoot],
            in_edges: Vec::new(),
            unbound: alloc::vec![PraxisKnowledgeGraphRelation {
                from: C::MerkleRoot,
                to: C::SourcePin,
                kind: K::Subsumption,
            }],
        };
        let err = emit_snapshot(&open, &[]).expect_err("unclosed slice must refuse");
        assert!(
            matches!(err, SnapshotError::SelectionLeftSlice { .. }),
            "got {err:?}"
        );
    }

    /// The MerkleRoot gate: a genuine snapshot loaded against a DIFFERENT
    /// slice's pin is refused (poison detection — the address binds the slice).
    #[test]
    fn snapshot_load_rejects_wrong_merkle_root() {
        let (_gz_a, root_a) = emit_snapshot(&fixed_slice(), &[]).expect("emit A");
        let bindings = alloc::vec![GraphNode {
            kind: C::AxiomNode,
            identity: REGISTERED_AXIOM.to_string(),
        }];
        let (gz_b, root_b) = emit_snapshot(&fixed_slice(), &bindings).expect("emit B");
        assert_ne!(root_a, root_b, "different slices → different roots");
        let err = load_snapshot(&gz_b, &root_a).expect_err("wrong pin must be refused");
        assert!(
            matches!(err, SnapshotError::MerkleRootMismatch { .. }),
            "got {err:?}"
        );
    }

    /// Eager re-bind teeth: a snapshot carrying an UNREGISTERED axiom binding
    /// passes the MerkleRoot gate but is refused at re-bind (fail-closed), and
    /// no partial graph is materialized.
    #[test]
    fn snapshot_load_rejects_unbound_axiom() {
        let bindings = alloc::vec![GraphNode {
            kind: C::AxiomNode,
            identity: "__praxis_unregistered_axiom_binding__".to_string(),
        }];
        let (gz, root) = emit_snapshot(&fixed_slice(), &bindings).expect("emit");
        let err = load_snapshot(&gz, &root).expect_err("unregistered binding must be refused");
        assert!(
            matches!(err, SnapshotError::UnboundReference { .. }),
            "got {err:?}"
        );
    }

    /// An edge is re-bound ONTOLOGICALLY: a fabricated edge between two real
    /// concepts that is NOT a morphism of the category is refused on load (not
    /// trusted by the byte-gate / name-resolvability alone). The blob is built
    /// directly with its genuine MerkleRoot, so only the edge re-bind rejects it.
    #[test]
    fn snapshot_load_rejects_non_morphism_edge() {
        let env = SnapshotEnvelope {
            nodes: Vec::new(),
            edges: alloc::vec![SnapshotEdge {
                from: C::LensNode.name().to_string(),
                to: C::SourcePin.name().to_string(),
                // LensNode --Subsumption--> SourcePin is NOT a morphism.
                kind: "Subsumption".to_string(),
            }],
        };
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&env)
            .expect("serialize")
            .to_vec();
        let root = source_content_hash(&bytes);
        let gz = gzip(&bytes).expect("gzip");
        let err = load_snapshot(&gz, &root).expect_err("non-morphism edge must be refused");
        assert!(
            matches!(err, SnapshotError::UnboundReference { .. }),
            "got {err:?}"
        );
    }
}
