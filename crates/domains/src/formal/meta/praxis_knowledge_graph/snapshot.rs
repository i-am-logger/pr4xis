//! `GraphSnapshot` — the wire form of praxis's **whole-graph wire protocol**
//! (#272 / #271 effort B): select a slice of the [`PraxisKnowledgeGraph`](super)
//! (the whole graph by default), content-address it as a Merkle DAG, and
//! rehydrate it through the same fail-closed admit gate the `.prx` archive uses,
//! re-binding behavioural nodes to the running binary. This is how one praxis
//! instance hands its ontologies to another, verifiably.
//!
//! This is the whole-graph generalisation of the archive storage substratum:
//! it REUSES the prx primitives ([`ContentAddress`](pr4xis_runtime::address::ContentAddress), gzip/rkyv, the
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
//! - **Aumasson, O'Connor, Neves & Wilcox-O'Hearn (2020)** BLAKE3 (the
//!   content-address hash).

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec::Vec};

use pr4xis::category::{Arrow, Category, Concept, FinitelyGenerated, RelationKind};
use pr4xis::ontology::{adjunction_by_name, axiom_by_name, functor_by_name};
use pr4xis_runtime::codec;
use pr4xis_runtime::emit::{binding_definition, definition_of};

// `PraxisKnowledgeGraphConcept` is the PKG node-KIND vocabulary the generic
// snapshot still classifies against (`ConceptNode`/`AxiomNode`/…), so it stays
// in production scope. The PKG Category/Relation/RelationKind are now used only
// by the PKG test module (the generic fns name them only via turbofish there);
// gate them so non-test builds see no unused import.
use super::ontology::PraxisKnowledgeGraphConcept;
#[cfg(test)]
use super::ontology::{
    PraxisKnowledgeGraphCategory, PraxisKnowledgeGraphRelation, PraxisKnowledgeGraphRelationKind,
};
use crate::applied::data_provisioning::registry::LockDigest;
use crate::formal::meta::artifact_identity::ontology::{
    IdentityClaim, IdentityConcept, VerificationResult,
};
use crate::formal::meta::artifact_identity::schemes::raw_hash;
use crate::formal::meta::well_behaved_lens::lens_by_name;
// Shared `.prx` primitives — the SAME gzip/rkyv codecs and typed-claim
// integrity gate the archive uses (never a parallel hash/codec); the
// content-hash itself is the runtime [`ContentAddress`].
use crate::social::software::markup::xml::owl::prx::{gunzip, gzip};

// =============================================================================
// Selection — RootSet / EdgeKindFilter / ReachableSubgraph / compute_reachable
// =============================================================================

/// The relation-kind of a category's morphisms (mirrors `structural.rs`'s alias).
type KindOf<C> = <<C as Category>::Morphism as Arrow>::Kind;

/// The seed concepts a selection starts from — generic over any category `C`.
/// A runtime realisation of the ontology's `RootSet` concept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSet<C: Category>(pub Vec<C::Object>);

/// The edge kinds a selection traverses — generic over any category `C`. A
/// runtime realisation of the ontology's `EdgeKindFilter` concept. Membership is
/// set-based (`filter.0.contains(&kind)`), generalising the scalar `kind ==
/// filter` of the inlined BFS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeKindFilter<C: Category>(pub Vec<KindOf<C>>);

/// The closed slice [`compute_reachable`] produces — a runtime realisation of
/// the ontology's `ReachableSubgraph` concept (which `has_a UnboundReference`),
/// generic over any category `C`.
///
/// `nodes` is the reachable concept set; `in_edges` are the filtered-kind
/// edges entirely within the slice; `unbound` are the filtered-kind edges
/// that LEAVE the slice (`from ∈ nodes`, `to ∉ nodes`) — the
/// `UnboundReference`s that, if non-empty, mean the slice is NOT closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachableSubgraph<C: Category> {
    pub nodes: Vec<C::Object>,
    pub in_edges: Vec<C::Morphism>,
    pub unbound: Vec<C::Morphism>,
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
pub fn compute_reachable<C>(roots: &RootSet<C>, filter: &EdgeKindFilter<C>) -> ReachableSubgraph<C>
where
    C: Category,
    C::Object: Clone + PartialEq,
    C::Morphism: Arrow<Object = C::Object>,
    KindOf<C>: PartialEq,
{
    // The filtered arrows out of the roots — the Category's own
    // `morphisms_from` accessor over the CLOSED relation, not a hand-rolled
    // walk over the whole edge list. Because the closure is materialized, this
    // single image is already the full transitively-reachable set. The kind and
    // endpoint are read through `Arrow::kind`/`Arrow::target` — trait methods,
    // never a per-ontology struct field.
    let image_targets = |objs: &[C::Object]| -> Vec<C::Morphism> {
        objs.iter()
            .flat_map(C::morphisms_from)
            .filter(|m| filter.0.contains(&m.kind()))
            .collect()
    };

    // Node set = the roots together with the image targets (one step: the
    // closure already contains every multi-hop edge, so this is closed under
    // each transitive kind).
    let mut nodes = roots.0.clone();
    for target in image_targets(&roots.0).iter().map(|m| m.target()) {
        if !nodes.contains(&target) {
            nodes.push(target);
        }
    }

    // The slice's own morphisms: every filtered arrow out of an in-slice node,
    // partitioned into those that stay in the slice (its `in_edges`) and those
    // that leave it (`UnboundReference`s — cross-kind or out-of-slice).
    let (in_edges, unbound) = image_targets(&nodes)
        .into_iter()
        .partition(|m| nodes.contains(&m.target()));

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

/// Re-resolve a concept of ANY finitely-generated ontology `T` from its stable
/// [`Concept::name`], over `T`'s own `variants()` enumerator. Fail-closed `None`
/// for an unknown name. Used at two type instantiations: `T = PraxisKnowledgeGraphConcept`
/// for the structural NODE-KIND vocabulary, and `T = C::Object` for the loaded
/// ontology's own concept identities.
fn concept_from_name<T: FinitelyGenerated>(name: &str) -> Option<T> {
    T::variants().into_iter().find(|c| c.name() == name)
}

// An edge's relation kind crosses the wire as its ontology name via
// `RelationKind::name()` and re-resolves through `RelationKind::from_name()` —
// the kind-level parallels of `Concept::name`/`concept_from_name`, generated by
// the `ontology!` macro in `pr4xis-derive` (so there is no hand-rolled mirror).

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

/// One persisted node, name-keyed and per-node content-addressed (the Merkle
/// DAG's `ContentAddressableNode`).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct SnapshotNode {
    /// The structural-node concept this node is, by [`Concept::name`]
    /// (`"ConceptNode"` / `"AxiomNode"` / `"LensNode"` / …).
    pub node_kind: String,
    /// The stable identity — a concept name, or a registered binding name.
    pub identity: String,
    /// Per-node content address — DEFINITION-BEARING, via the runtime's one
    /// typed lowering ([`definition_of`] /
    /// [`Definition::address`](pr4xis_runtime::definition::Definition::address)):
    /// the canonical DAG-CBOR encoding of `(kind, name, sorted edges, axioms,
    /// lexical)`, hashed. Two nodes share an address iff they share their
    /// definition as this slice carries it (the `MerkleDedupCorrect` iff, at
    /// graph scale) — a same-name node with different edges addresses
    /// differently, closing the wire gap where a bare name-pair silently
    /// re-bound to a different meaning on the receiver (G5).
    pub address: String,
}

/// One persisted edge — a `MerkleEdge`-by-name between two node identities.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct SnapshotEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
}

/// The rkyv-serializable `GraphSnapshot` envelope: a canonically-ordered set
/// of name-keyed nodes + Merkle-edges. Its `GraphVersion` is the content digest of
/// these bytes (a DERIVED label, NOT stored inside them), so re-emitting the
/// same slice reproduces the same address.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
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
    /// The canonical DAG-CBOR encoding behind a node's definition-bearing
    /// address failed.
    Codec(String),
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
    /// An edge's endpoint is NOT carried as a `ConceptNode` in this slice's own
    /// node set — a dangling reference. The edge may be a genuine morphism of
    /// the global category, but a self-contained graph must carry both
    /// endpoints; load mirrors emit's closure guard. Fail-closed.
    DanglingEdge {
        from: String,
        kind: String,
        to: String,
    },
}

impl core::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SnapshotError::Rkyv(m) => write!(f, "snapshot rkyv error: {m}"),
            SnapshotError::Codec(m) => {
                write!(f, "snapshot definition-address codec error: {m}")
            }
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
            SnapshotError::DanglingEdge { from, kind, to } => write!(
                f,
                "dangling edge: {from} --{kind}--> {to} — an endpoint is not a node carried in \
                 the slice (not self-contained) — refusing the snapshot"
            ),
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
/// `String==`. The pin is a runtime-EMITTED address (a `GraphVersion` is
/// the slice's own content address), so the claim carries the one emit
/// algorithm ([`LockDigest::address`]); `raw_hash::verify` re-hashes
/// `bytes`, so the pin is checked against bytes actually present.
fn verify_merkle_root(bytes: &[u8], trusted_pin: &str) -> Result<(), SnapshotError> {
    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: LockDigest::address(trusted_pin).claim_data(),
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
/// a runnable lens value); a `FunctorNode` / `AdjunctionNode` to a registered
/// connection by its stable binding name (`functor_by_name` / `adjunction_by_name`
/// over `FUNCTOR_CONSTRUCTORS` / `ADJUNCTION_CONSTRUCTORS`). Fail-closed `Err` on
/// any miss — an unknown binding name resolves to nothing and is refused. On
/// wasm32 every constructor slice is empty, so behavioural nodes are refused
/// there — the correct fail-closed behaviour.
fn rebind_node<C>(node: &SnapshotNode) -> Result<(), SnapshotError>
where
    C: Category,
    C::Object: FinitelyGenerated,
{
    use PraxisKnowledgeGraphConcept as PkgC;
    // The node-KIND is the PKG STRUCTURAL vocabulary (`ConceptNode`/`AxiomNode`/…)
    // — resolve it against PKG, never `C`. Then dispatch on the VARIANT: a
    // `ConceptNode`'s IDENTITY names a concept of the LOADED ontology `C`, so it
    // resolves against `C::Object`; the behavioural arms hit the workspace-global
    // registries (PKG-independent), unchanged. A non-node kind, or an unknown
    // binding name in any registry, fails closed.
    let resolved = match concept_from_name::<PkgC>(&node.node_kind) {
        Some(PkgC::ConceptNode) => concept_from_name::<C::Object>(&node.identity).is_some(),
        Some(PkgC::AxiomNode) => axiom_by_name(&node.identity).is_some(),
        Some(PkgC::LensNode) => lens_by_name(&node.identity).is_some(),
        Some(PkgC::FunctorNode) => functor_by_name(&node.identity).is_some(),
        Some(PkgC::AdjunctionNode) => adjunction_by_name(&node.identity).is_some(),
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
///    and discharge it against `trusted_pin` via `verify_merkle_root`; a
///    poisoned slice mutates the address and is refused here.
/// 3. **EAGER RE-BIND** — resolve every node's `(node_kind, identity)` against
///    this binary's registries (`rebind_node`); on the FIRST unresolvable
///    binding, refuse the whole snapshot (no partial graph). Also confirm each
///    edge's kind + endpoints re-resolve by name.
/// 4. Return the verified, fully-rebindable envelope.
pub fn load_snapshot<C>(gz: &[u8], trusted_pin: &str) -> Result<SnapshotEnvelope, SnapshotError>
where
    C: Category,
    C::Object: FinitelyGenerated + PartialEq,
    C::Morphism: Arrow<Object = C::Object>,
    KindOf<C>: RelationKind,
{
    // 1. Decode.
    let bytes = gunzip(gz).map_err(|e| SnapshotError::Gzip(e.to_string()))?;
    let mut aligned = rkyv::util::AlignedVec::<16>::new();
    aligned.extend_from_slice(&bytes);
    let envelope: SnapshotEnvelope =
        rkyv::from_bytes::<SnapshotEnvelope, rkyv::rancor::Error>(&aligned)
            .map_err(|e| SnapshotError::Rkyv(e.to_string()))?;

    // 2. MerkleRoot gate — over the canonical DAG-CBOR of the DECODED envelope
    //    (toolchain-stable), NOT the rkyv layout. rkyv is now only the local
    //    transport. Correctness invariant: DAG-CBOR(rkyv-decoded envelope) ==
    //    DAG-CBOR(emitted envelope), because the rkyv round-trip is value-lossless
    //    and `serde_ipld_dagcbor` is a pure function of the value — so this
    //    re-derivation reproduces the address emit pinned. (A future field with a
    //    non-canonical serde repr — a float, an untagged enum — would break this;
    //    this comment is the tripwire.)
    let canonical =
        codec::canonical_encode(&envelope).map_err(|e| SnapshotError::Codec(e.to_string()))?;
    verify_merkle_root(&canonical, trusted_pin)?;

    // 3. Eager re-bind — every node must resolve, every edge's kind + endpoints
    // must re-resolve by name. Fail-closed on the first miss.
    for node in &envelope.nodes {
        rebind_node::<C>(node)?;
    }
    // G1 — referential self-containment: the concept identities this slice
    // actually CARRIES (its `ConceptNode`s). An edge endpoint absent here is a
    // dangling reference even when it is a genuine morphism of the global
    // category — load mirrors emit's closure guard.
    let slice_concepts: alloc::collections::BTreeSet<&str> = envelope
        .nodes
        .iter()
        .filter(|n| n.node_kind == PraxisKnowledgeGraphConcept::ConceptNode.name())
        .map(|n| n.identity.as_str())
        .collect();
    for edge in &envelope.edges {
        // Re-bind the edge ONTOLOGICALLY — not "are the three names resolvable"
        // but "is this actually a morphism of the category". The materialized
        // closure makes the membership check exact + one-step
        // (`morphisms_from` over the closed relation), so a fabricated edge
        // between two real concepts that is NOT a morphism is refused —
        // symmetric with the node re-bind against the live registries, never
        // trusting the byte-gate alone.
        let is_morphism = match (
            <KindOf<C> as RelationKind>::from_name(&edge.kind),
            concept_from_name::<C::Object>(&edge.from),
            concept_from_name::<C::Object>(&edge.to),
        ) {
            (Some(k), Some(f), Some(t)) => C::morphisms_from(&f)
                .iter()
                .any(|m| m.target() == t && m.kind() == k),
            _ => false,
        };
        if !is_morphism {
            return Err(SnapshotError::UnboundReference {
                node_kind: "RelationEdge".to_string(),
                identity: format!("{} --{}--> {}", edge.from, edge.kind, edge.to),
            });
        }
        // G1 — a genuine morphism whose endpoints are not BOTH carried as nodes
        // in this slice is a dangling edge — refuse (the byte-gate + morphism
        // check alone would admit a zero-node, one-real-edge blob).
        if !slice_concepts.contains(edge.from.as_str())
            || !slice_concepts.contains(edge.to.as_str())
        {
            return Err(SnapshotError::DanglingEdge {
                from: edge.from.clone(),
                kind: edge.kind.clone(),
                to: edge.to.clone(),
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
/// 2. **Name + definition-address** every node: the slice's concepts as
///    `ConceptNode`s (identity = [`Concept::name`]), plus the `bindings` (the
///    behavioural `AxiomNode`/`LensNode`s by registered name). Each address is
///    DEFINITION-BEARING through the runtime's one typed lowering
///    ([`definition_of`] over the node's in-slice morphisms + gloss;
///    [`binding_definition`] for the name-keyed behavioural nodes). Edges are
///    the slice's `in_edges`, by name.
/// 3. **Canonical order** nodes by `(node_kind, identity)` and edges by
///    `(kind, from, to)` — a total order (identities are unique) — and refuse
///    any shared identity ([`SnapshotError::AddressCollision`]).
/// 4. **Address** = the content digest of the rkyv bytes (the `GraphVersion`, derived),
///    then gzip. Re-emitting the same slice reproduces the same address.
pub fn emit_snapshot<C>(
    slice: &ReachableSubgraph<C>,
    bindings: &[GraphNode],
) -> Result<(Vec<u8>, String), SnapshotError>
where
    C: Category,
    C::Object: Concept + PartialEq,
    C::Morphism: Arrow<Object = C::Object>,
    KindOf<C>: RelationKind,
{
    // 1. The slice must be closed.
    if let Some(u) = slice.unbound.first() {
        return Err(SnapshotError::SelectionLeftSlice {
            from: u.source().name().to_string(),
            to: u.target().name().to_string(),
            kind: u.kind().name().to_string(),
        });
    }

    // 2. Name + definition-address every node THROUGH the runtime's one typed
    // lowering — the same boundary `.prx` emit crosses, so a concept lowers to
    // the same `Definition` (hence the same address) here as anywhere else.
    // A structural concept's definition is what this slice carries of it: its
    // in-slice outgoing morphisms (typed, from `in_edges`) and its
    // ONTOLEX-Lemon gloss. A behavioural binding is name-keyed BY DESIGN
    // (`binding_definition`): the receiver re-binds it through its own
    // registries, and load is the gate.
    let structural = slice.nodes.iter().map(|c| {
        let morphisms: Vec<C::Morphism> = slice
            .in_edges
            .iter()
            .filter(|m| m.source() == *c)
            .cloned()
            .collect();
        (
            // The node-KIND is the PKG structural `ConceptNode` (the `K` slot of
            // the generic `definition_of<K,O,M>`); the concept CONTENT `O` and its
            // morphisms `M` are the loaded ontology `C`'s own — independent params.
            GraphNode {
                kind: PraxisKnowledgeGraphConcept::ConceptNode,
                identity: c.name().to_string(),
            },
            definition_of::<PraxisKnowledgeGraphConcept, C::Object, C::Morphism>(
                &PraxisKnowledgeGraphConcept::ConceptNode,
                c,
                &morphisms,
            ),
        )
    });
    let behavioural = bindings.iter().cloned().map(|gn| {
        let definition = binding_definition(&gn.kind, &gn.identity);
        (gn, definition)
    });
    let mut nodes: Vec<SnapshotNode> = structural
        .chain(behavioural)
        .map(|(gn, definition)| {
            let address = definition
                .address()
                .map(|a| a.to_hex())
                .map_err(|e| SnapshotError::Codec(e.to_string()))?;
            Ok(SnapshotNode {
                node_kind: gn.kind.name().to_string(),
                identity: gn.identity,
                address,
            })
        })
        .collect::<Result<Vec<_>, SnapshotError>>()?;
    let mut edges: Vec<SnapshotEdge> = slice
        .in_edges
        .iter()
        .map(|m| SnapshotEdge {
            from: m.source().name().to_string(),
            to: m.target().name().to_string(),
            kind: m.kind().name().to_string(),
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
    // The address (GraphVersion) is BLAKE3 over the canonical DAG-CBOR of the
    // envelope VALUE — toolchain-independent by construction — NOT over the rkyv
    // `bytes` (whose FixedUsize/endianness layout is per-toolchain). `bytes` is now
    // only the local gz cache/transport; the address no longer depends on it.
    let merkle_root = codec::address_of(&envelope)
        .map_err(|e| SnapshotError::Codec(e.to_string()))?
        .to_hex();
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
        let sub = compute_reachable::<PraxisKnowledgeGraphCategory>(
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
        let single = compute_reachable::<PraxisKnowledgeGraphCategory>(
            &RootSet(alloc::vec![C::MerkleRoot]),
            &EdgeKindFilter(alloc::vec![K::Subsumption]),
        );
        let multi = compute_reachable::<PraxisKnowledgeGraphCategory>(
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

    // ── emit / load (the GraphSnapshot slice round-trip) ────────────────

    /// A fixed, closed slice — the `MerkleRoot` Subsumption image.
    fn fixed_slice() -> ReachableSubgraph<PraxisKnowledgeGraphCategory> {
        compute_reachable::<PraxisKnowledgeGraphCategory>(
            &RootSet(alloc::vec![C::MerkleRoot]),
            &EdgeKindFilter(alloc::vec![K::Subsumption]),
        )
    }

    /// A registered axiom name (registered via `register_axiom!(_, constructor)`)
    /// — `axiom_by_name` resolves it, so a snapshot binding to it re-binds.
    const REGISTERED_AXIOM: &str = "AxiomBindingComplete";

    /// The stable binding name of an actually-registered functor, read from the
    /// live registry — robust to the `meta().name` scheme (a clean `functor!`
    /// literal vs the default `type_name` placeholder, which differ), where a
    /// hardcoded constant would silently pin the wrong string. Any registered
    /// functor's name re-binds; we just need one.
    fn a_registered_functor_name() -> String {
        pr4xis::ontology::FUNCTOR_CONSTRUCTORS
            .iter()
            .map(|f| f())
            .next()
            .expect("at least one functor is registered in this binary")
            .name
            .as_str()
            .to_string()
    }

    /// A registered adjunction name — `register_adjunction!(AnalysisSynthesis)`
    /// (hearing/adjunctions.rs), whose hand-written `meta().name` is the literal
    /// `"AnalysisSynthesis"`. `adjunction_by_name` resolves it.
    const REGISTERED_ADJUNCTION: &str = "AnalysisSynthesis";

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
        let (_gz1, root1) =
            emit_snapshot::<PraxisKnowledgeGraphCategory>(&fixed_slice(), &bindings)
                .expect("emit 1");
        let (_gz2, root2) =
            emit_snapshot::<PraxisKnowledgeGraphCategory>(&fixed_slice(), &bindings)
                .expect("emit 2");
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
        let (gz, root) =
            emit_snapshot::<PraxisKnowledgeGraphCategory>(&slice, &bindings).expect("emit");
        let env = load_snapshot::<PraxisKnowledgeGraphCategory>(&gz, &root)
            .expect("load + gate + rebind");

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
        let err = emit_snapshot::<PraxisKnowledgeGraphCategory>(&fixed_slice(), &dup)
            .expect_err("duplicate identity must refuse");
        assert!(
            matches!(err, SnapshotError::AddressCollision { .. }),
            "got {err:?}"
        );
    }

    /// A slice that is NOT closed (a filtered edge leaves it) cannot be emitted.
    #[test]
    fn snapshot_emit_rejects_unclosed_slice() {
        let open = ReachableSubgraph::<PraxisKnowledgeGraphCategory> {
            nodes: alloc::vec![C::MerkleRoot],
            in_edges: Vec::new(),
            unbound: alloc::vec![PraxisKnowledgeGraphRelation {
                from: C::MerkleRoot,
                to: C::SourcePin,
                kind: K::Subsumption,
            }],
        };
        let err = emit_snapshot::<PraxisKnowledgeGraphCategory>(&open, &[])
            .expect_err("unclosed slice must refuse");
        assert!(
            matches!(err, SnapshotError::SelectionLeftSlice { .. }),
            "got {err:?}"
        );
    }

    /// The MerkleRoot gate: a genuine snapshot loaded against a DIFFERENT
    /// slice's pin is refused (poison detection — the address binds the slice).
    #[test]
    fn snapshot_load_rejects_wrong_merkle_root() {
        let (_gz_a, root_a) =
            emit_snapshot::<PraxisKnowledgeGraphCategory>(&fixed_slice(), &[]).expect("emit A");
        let bindings = alloc::vec![GraphNode {
            kind: C::AxiomNode,
            identity: REGISTERED_AXIOM.to_string(),
        }];
        let (gz_b, root_b) =
            emit_snapshot::<PraxisKnowledgeGraphCategory>(&fixed_slice(), &bindings)
                .expect("emit B");
        assert_ne!(root_a, root_b, "different slices → different roots");
        let err = load_snapshot::<PraxisKnowledgeGraphCategory>(&gz_b, &root_a)
            .expect_err("wrong pin must be refused");
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
        let (gz, root) =
            emit_snapshot::<PraxisKnowledgeGraphCategory>(&fixed_slice(), &bindings).expect("emit");
        let err = load_snapshot::<PraxisKnowledgeGraphCategory>(&gz, &root)
            .expect_err("unregistered binding must be refused");
        assert!(
            matches!(err, SnapshotError::UnboundReference { .. }),
            "got {err:?}"
        );
    }

    /// A4 (the headline) — a `FunctorNode` binding to a REGISTERED functor
    /// re-binds: the snapshot loads instead of failing closed with
    /// `UnboundReference`, the node survives + is named. The connection-round-trip
    /// blocker — a snapshot carrying a functor is admitted + re-bound to this
    /// binary's live functor (`functor_by_name`).
    #[test]
    fn snapshot_functor_node_round_trips_and_rebinds() {
        let slice = fixed_slice();
        let fname = a_registered_functor_name();
        let bindings = alloc::vec![GraphNode {
            kind: C::FunctorNode,
            identity: fname.clone(),
        }];
        let (gz, root) =
            emit_snapshot::<PraxisKnowledgeGraphCategory>(&slice, &bindings).expect("emit");
        let env = load_snapshot::<PraxisKnowledgeGraphCategory>(&gz, &root)
            .expect("load + gate + rebind the FunctorNode");
        assert_eq!(env.nodes.len(), slice.nodes.len() + 1);
        assert!(
            env.nodes
                .iter()
                .any(|n| n.node_kind == "FunctorNode" && n.identity == fname),
            "the FunctorNode binding survives and re-binds (would be UnboundReference otherwise)"
        );
    }

    /// A4 — the `AdjunctionNode` arm, symmetric: a registered adjunction re-binds.
    #[test]
    fn snapshot_adjunction_node_round_trips_and_rebinds() {
        let slice = fixed_slice();
        let bindings = alloc::vec![GraphNode {
            kind: C::AdjunctionNode,
            identity: REGISTERED_ADJUNCTION.to_string(),
        }];
        let (gz, root) =
            emit_snapshot::<PraxisKnowledgeGraphCategory>(&slice, &bindings).expect("emit");
        let env = load_snapshot::<PraxisKnowledgeGraphCategory>(&gz, &root)
            .expect("load + gate + rebind the AdjunctionNode");
        assert!(
            env.nodes
                .iter()
                .any(|n| n.node_kind == "AdjunctionNode" && n.identity == REGISTERED_ADJUNCTION),
            "the AdjunctionNode binding survives and re-binds"
        );
    }

    /// A4 fail-closed teeth — a `FunctorNode` binding to an UNREGISTERED functor
    /// passes the MerkleRoot gate but is refused at re-bind (`UnboundReference`),
    /// no partial graph. The resolver accepts strictly more, never a fabrication.
    #[test]
    fn snapshot_load_rejects_unbound_functor() {
        let bindings = alloc::vec![GraphNode {
            kind: C::FunctorNode,
            identity: "__praxis_unregistered_functor_binding__".to_string(),
        }];
        let (gz, root) =
            emit_snapshot::<PraxisKnowledgeGraphCategory>(&fixed_slice(), &bindings).expect("emit");
        let err = load_snapshot::<PraxisKnowledgeGraphCategory>(&gz, &root)
            .expect_err("unregistered functor must be refused");
        assert!(
            matches!(err, SnapshotError::UnboundReference { .. }),
            "got {err:?}"
        );
    }

    /// A4 — the resolver contract directly: `functor_by_name` returns `None` on a
    /// miss (the `find`-miss path the fail-closed gate relies on). On wasm32 the
    /// `FUNCTOR_CONSTRUCTORS` slice is empty (linkme unsupported), so EVERY name →
    /// `None` → behavioural nodes are refused there — the cfg stub is byte-identical
    /// to `axiom_by_name`'s, so the wasm path inherits its proven fail-closed shape.
    #[test]
    fn functor_by_name_refuses_an_unregistered_name() {
        assert!(functor_by_name("__praxis_no_such_functor__").is_none());
        assert!(adjunction_by_name("__praxis_no_such_adjunction__").is_none());
        // And the registered names DO resolve (the positive control, native).
        assert!(functor_by_name(&a_registered_functor_name()).is_some());
        assert!(adjunction_by_name(REGISTERED_ADJUNCTION).is_some());
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
        // The genuine pin is now the DAG-CBOR address of the envelope (self-tracks
        // the A7 change); the gz still wraps the rkyv `bytes` as the cache.
        let root = codec::address_of(&env).expect("dag-cbor address").to_hex();
        let gz = gzip(&bytes).expect("gzip");
        let err = load_snapshot::<PraxisKnowledgeGraphCategory>(&gz, &root)
            .expect_err("non-morphism edge must be refused");
        assert!(
            matches!(err, SnapshotError::UnboundReference { .. }),
            "got {err:?}"
        );
    }

    /// G1 counterexample — referential self-containment. A blob with a GENUINE
    /// morphism edge (`MerkleRoot --Subsumption--> ContentAddressableNode`, the
    /// declared `is_a` edge) whose endpoints are NOT carried as nodes is a
    /// DANGLING edge, refused on load. Before this guard it passed the morphism
    /// gate and loaded `Ok` over an empty node set; load now mirrors emit's
    /// closure guard. The blob carries its genuine MerkleRoot, so ONLY the
    /// dangling check (not the byte-gate or the morphism check) rejects it.
    #[test]
    fn snapshot_load_rejects_dangling_edge() {
        let env = SnapshotEnvelope {
            nodes: Vec::new(),
            edges: alloc::vec![SnapshotEdge {
                from: C::MerkleRoot.name().to_string(),
                to: C::ContentAddressableNode.name().to_string(),
                kind: "Subsumption".to_string(),
            }],
        };
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&env)
            .expect("serialize")
            .to_vec();
        // The genuine pin is now the DAG-CBOR address of the envelope (self-tracks
        // the A7 change); the gz still wraps the rkyv `bytes` as the cache.
        let root = codec::address_of(&env).expect("dag-cbor address").to_hex();
        let gz = gzip(&bytes).expect("gzip");
        let err = load_snapshot::<PraxisKnowledgeGraphCategory>(&gz, &root)
            .expect_err("dangling edge must be refused");
        assert!(
            matches!(err, SnapshotError::DanglingEdge { .. }),
            "got {err:?}"
        );
    }

    proptest::proptest! {
        /// G1 property — referential self-containment holds for EVERY honestly
        /// emitted snapshot: over arbitrary roots, the round-trip loads and every
        /// loaded edge's endpoints are carried `ConceptNode`s (the positive of
        /// the dangling-edge guard — emit never produces a dangling edge, load
        /// never admits one).
        #[test]
        fn prop_emitted_snapshot_is_referentially_closed(
            root_ix in 0usize..PraxisKnowledgeGraphConcept::variants().len()
        ) {
            let root = PraxisKnowledgeGraphConcept::variants()[root_ix];
            let slice = compute_reachable::<PraxisKnowledgeGraphCategory>(
                &RootSet(alloc::vec![root]),
                &EdgeKindFilter(alloc::vec![K::Subsumption]),
            );
            // compute_reachable closes the slice for the transitive Subsumption
            // kind; emit refuses an unclosed one.
            proptest::prop_assume!(slice.unbound.is_empty());
            let (gz, pin) = emit_snapshot::<PraxisKnowledgeGraphCategory>(&slice, &[]).expect("emit closed slice");
            let env = load_snapshot::<PraxisKnowledgeGraphCategory>(&gz, &pin).expect("honest snapshot round-trips");
            let carried: alloc::collections::BTreeSet<&str> = env
                .nodes
                .iter()
                .filter(|n| n.node_kind == C::ConceptNode.name())
                .map(|n| n.identity.as_str())
                .collect();
            for edge in &env.edges {
                proptest::prop_assert!(carried.contains(edge.from.as_str()));
                proptest::prop_assert!(carried.contains(edge.to.as_str()));
            }
        }
    }
}

/// A6 — the genericity proof. The PKG monomorphization is exercised above; these
/// instantiate the SAME `compute_reachable`/`emit_snapshot`/`load_snapshot` over a
/// SECOND, unrelated `ontology!`-generated category, proving the generics are real
/// (not vacuous): generic emit lowers the wire kind via `RelationKind::name`, and
/// generic load re-resolves it via `RelationKind::from_name` + `concept_from_name`
/// over THIS ontology's own concepts. No production non-PKG caller exists yet, so
/// these two tests ARE the non-vacuity justification.
#[cfg(test)]
mod second_ontology_tests {
    use super::*;

    // A real second ontology — typed kinds + a macro-materialized is_a closure
    // (NOT a `type Kind = ()` stub). The `is_a` chain A⊑B⊑C is closed by the
    // macro's Floyd–Warshall to also carry the transitive A⊑C; D is isolated.
    pr4xis::ontology! {
        name: "SnapA6",
        source: "A6 generic-snapshot second-ontology fixture",
        concepts: [A, B, C, D],
        labels: {
            A: ("en", "A", "Root concept of the A6 snapshot fixture."),
            B: ("en", "B", "Mid concept."),
            C: ("en", "C", "Apex concept."),
            D: ("en", "D", "An isolated concept, off the A-reachable set."),
        },
        is_a: [
            (A, B),
            (B, C),
        ],
    }

    use SnapA6Concept as O;
    use SnapA6RelationKind as OK;
    type Cat = SnapA6Category;

    /// (i) `compute_reachable` over a SECOND ontology: roots={A}, the Subsumption
    /// chain A⊑B⊑C is closed (the macro materializes A⊑C), the isolated D excluded.
    #[test]
    fn second_ontology_compute_reachable_closure() {
        let sub = compute_reachable::<Cat>(
            &RootSet::<Cat>(alloc::vec![O::A]),
            &EdgeKindFilter::<Cat>(alloc::vec![OK::Subsumption]),
        );
        assert!(sub.unbound.is_empty(), "A⊑B⊑C is closed under Subsumption");
        assert!(sub.nodes.contains(&O::A), "root A is in the slice");
        assert!(sub.nodes.contains(&O::B), "A reaches B");
        assert!(
            sub.nodes.contains(&O::C),
            "A reaches C via the materialized transitive closure"
        );
        assert!(!sub.nodes.contains(&O::D), "isolated D is not reached");
    }

    /// (ii) full EMIT→LOAD round-trip over the SECOND ontology — generic emit
    /// (`RelationKind::name` on the wire edge) AND generic load (rebind the O
    /// concepts via `concept_from_name::<SnapA6Concept>` + `RelationKind::from_name`).
    #[test]
    fn second_ontology_emit_load_round_trip() {
        let slice = compute_reachable::<Cat>(
            &RootSet::<Cat>(alloc::vec![O::A]),
            &EdgeKindFilter::<Cat>(alloc::vec![OK::Subsumption]),
        );
        assert!(slice.unbound.is_empty(), "closed slice is emittable");

        // No behavioural bindings — a pure concept-only graph, so the round-trip
        // exercises ONLY the O-concept / O-kind wire path.
        let (gz, root) = emit_snapshot::<Cat>(&slice, &[]).expect("generic emit");
        let env = load_snapshot::<Cat>(&gz, &root).expect("generic load + gate + rebind");

        assert_eq!(env.nodes.len(), slice.nodes.len());
        assert_eq!(env.nodes.len(), 3, "A, B, C — D is not in the slice");
        for ident in ["A", "B", "C"] {
            assert!(
                env.nodes
                    .iter()
                    .any(|n| n.node_kind == "ConceptNode" && n.identity == ident),
                "ConceptNode {ident} survives the round-trip"
            );
        }
        let mut addrs: alloc::vec::Vec<&str> =
            env.nodes.iter().map(|n| n.address.as_str()).collect();
        addrs.sort_unstable();
        let before = addrs.len();
        addrs.dedup();
        assert_eq!(addrs.len(), before, "node addresses are distinct");
        // The materialized A⊑C edge round-trips: generic emit lowered its kind via
        // `RelationKind::name`, load re-resolved it via `RelationKind::from_name`.
        assert!(
            env.edges
                .iter()
                .any(|e| e.from == "A" && e.to == "C" && e.kind == "Subsumption"),
            "the transitive A⊑C edge round-trips by canonical kind name"
        );
    }
}
