//! RuntimeOntology — a loaded `.prx` [`Archive`] interpreted as ONE typed,
//! queryable ontology over the NOW-RELAXED open-world category.
//!
//! This is the convergence step of the runtime: where [`load`](crate::load)
//! admits the bytes and [`rebind`](crate::rebind) binds nodes by address, this
//! module turns the whole admitted archive into a single interpreted object that
//! a running system *queries* — "what does Employer subsume?", "is Employer an
//! Agent?", "what is this node's gloss?".
//!
//! # The open-world vertex
//!
//! Each archive node becomes a [`ConceptRef`] — the typed `(ontology, name)`
//! pair that IS the vertex identity. `ConceptRef` impls [`Concept`] (it has a
//! [`name()`](Concept::name)) but deliberately NOT
//! [`FinitelyGenerated`](pr4xis::category::FinitelyGenerated): a
//! runtime vertex materialized from a loaded `.prx` cannot be enumerated at the
//! type level — that is precisely the open-world case the Concept /
//! FinitelyGenerated relaxation exists for (Reiter 1978; `entity.rs`). Identity
//! is the typed pair, never `String ==`.
//!
//! # Reachability is the free-functor image, evaluated LAZILY per vertex
//!
//! The reasoning answer is the image of the free functor `FreeCategory<Q> →
//! ReachCat` into the thin reachability category — the `Collapse` / `ReachCat`
//! reflection in [`quiver`](pr4xis::category::quiver) (CWM II.7): every
//! generating path collapses to its `(source, target)` reachability arrow. The
//! [`MaterializedClosure`] holds only the *generating adjacency* per transitive
//! relation-kind (linear in the edge count); it does **not** eagerly saturate
//! and store the whole transitive-closure set (which is `O(V · depth)` — for
//! English's 107,519 concepts, hundreds of MB of pre-folded owned pairs). The
//! reflexive-transitive image of a *queried* vertex is instead computed ON
//! DEMAND by a bounded, cycle-safe breadth-first walk over the generators
//! (shortest-path grading; Moore 1959), and **memoized** per source: the first
//! query for a vertex pays the walk, every later one hits the memo with no
//! re-traversal (an `is_a` membership test then borrows the cached image
//! allocation-free; an image query copies it, O(image)). A
//! running chat touches a handful of vertices, so the resident footprint is the
//! adjacency plus the images actually asked for — never the 107k-concept
//! closure nobody queried.
//!
//! This is a REPRESENTATION change, not a semantics change. The lazy per-vertex
//! image is exactly the eager fold's image restricted to that vertex — same
//! reachable set, same shortest-path distances (BFS min-hops over the same
//! generators equals the Floyd–Warshall fixpoint over the same generators) — so
//! every query answer is SET-and-distance-identical to the eagerly-saturated
//! form. (The image-returning methods now enumerate in deterministic BFS order,
//! not the eager `HashMap`'s arbitrary order — a strictly-more-canonical order
//! on which no caller relies: `chain` sorts, `meet` argmins, `reachable_from`
//! collects into a `BTreeSet`.)
//! A `.prx`'s edges may themselves already be a closure; the walk does not
//! depend on that (the closure of a closure is the same closure), so it is
//! correct either way, exactly as the eager fold was.
//!
//! # Identity is the content address
//!
//! Two [`RuntimeOntology`]s are equal iff their archive roots agree — content-
//! address identity (`archive.root()`), the same identity rule the load gate and
//! `Archive` use. The closure is keyed by `ConceptRef` carrying its source
//! ontology so a later N-ontology composite can union closures and compose
//! cross-ontology edges; we build a single ontology now, but the structure is
//! N-ready.
//!
//! Literature:
//! - Mac Lane (1971) *Categories for the Working Mathematician* II.7 — the
//!   free–forgetful adjunction Grph ⊣ Cat; the free category on a graph and the
//!   unique functor extending an interpretation of its generators. The closure
//!   re-fold IS that functor's image into the thin reachability category.
//! - Fong & Spivak (2019) *Seven Sketches* Ch. 3 — finitely-presented
//!   categories via generators and relations (the finite-presentation theorem):
//!   a structure-preserving map is determined by its finite action on
//!   generators, which is why folding the generators recovers the whole closure.
//! - Smith et al. (2005) OBO Relation Ontology — `transitive_over`: Subsumption,
//!   Parthood, Causation are the canonically transitive relation kinds (the set
//!   the macro and `compose` also fold over).
//! - Reiter (1978) *On Closed World Data Bases* — the open/closed-world split the
//!   Concept / FinitelyGenerated relaxation realizes.

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use core::cell::RefCell;

use pr4xis::category::Concept;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use crate::address::ContentAddress;
use crate::apply::apply;
use crate::archive::Archive;
use crate::codec::CodecError;
use crate::connection::GeneratorAction;
use crate::definition::EdgeTarget;
use crate::lens::archive_lens::{
    ArchiveLens, ArchiveLensError, ArchivedArchiveView, archived_grounded, archived_local_name,
};

use rkyv::util::AlignedVec;

extern crate alloc;
#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// The one Relations vocabulary every edge kind-name resolves into. A relation
/// kind is NOT a closed Rust enum and NOT a bare string — it is a [`ConceptRef`]
/// in the loaded "Relations" ontology (`docs/praxis-self-aware-architecture` §11).
/// One shared vocabulary means an `Org` `Subsumption` edge and an English
/// `Subsumption` edge name the SAME kind, so closures compose across ontologies.
const RELATIONS_VOCAB: OntologyName = OntologyName::new_static("Relations");

/// Resolve an edge's kind-name (a wire string) to its [`ConceptRef`] in the one
/// `RELATIONS_VOCAB` vocabulary — THE blessed kind-name→concept lowering
/// (praxis-way rule 11: strings are WIRE, crossed by a single lowering). Every
/// edge-construction site ([`materialize`], [`RuntimeOntology::morphisms_from`])
/// and every kind a caller hands the query surface goes through here, so a
/// relation kind is one typed value, never a hand-assembled string compared with
/// `==`. The kind-name stays byte-exact on the wire; only its in-memory identity
/// becomes a concept.
pub fn relations_kind(name: impl Into<String>) -> ConceptRef {
    ConceptRef::new(RELATIONS_VOCAB.clone(), name)
}

/// The Subsumption (`is-a`) relation kind as a [`ConceptRef`] — OWL `subClassOf`
/// (Guarino 2009), the most-queried kind. The blessed handle callers pass to
/// [`RuntimeOntology::reachable_from`] / [`MaterializedClosure::reaches`] and that
/// the closure internals (`is_a`, the subsumption images) key on, so the one
/// Subsumption identity lives here, not re-spelled at each call site.
pub fn subsumption_kind() -> ConceptRef {
    relations_kind("Subsumption")
}

/// The open-world runtime vertex — a typed `(ontology, name)` pair.
///
/// This is the materialized concept of a loaded `.prx`: it carries its source
/// ontology so closures from different ontologies can be unioned and composed
/// across without a name collision. It impls [`Concept`] (identity + `name()`)
/// but NOT [`FinitelyGenerated`](pr4xis::category::FinitelyGenerated) — a
/// runtime vertex is open-world and cannot be enumerated at the type level.
///
/// Equality is the typed pair, never `String ==` of the bare name: `Person` in
/// ontology `A` is a different concept from `Person` in ontology `B`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConceptRef {
    /// The source ontology this concept belongs to.
    pub ontology: OntologyName,
    /// The concept's name within that ontology.
    pub name: String,
}

impl ConceptRef {
    /// A concept reference in `ontology` named `name`.
    pub fn new(ontology: OntologyName, name: impl Into<String>) -> Self {
        Self {
            ontology,
            name: name.into(),
        }
    }
}

// `ConceptRef` is a `Concept` (it has a name + identity) but deliberately does
// NOT impl `FinitelyGenerated`: the open-world vertex cannot be enumerated. This
// is the whole point of the Concept / FinitelyGenerated relaxation.
impl Concept for ConceptRef {
    fn name(&self) -> &'static str {
        // `Concept::name` returns `&'static str` for the compile-time enums; a
        // runtime vertex's name lives in `self.name`. We do not leak a `String`
        // to obtain a `'static` borrow — callers that need the runtime name use
        // [`ConceptRef::name`] (the field) directly. The trait method exists to
        // satisfy the open-world `Concept` bound; the *typed identity* is the
        // whole `ConceptRef`, which is what every query keys on.
        ""
    }
}

/// A directed, typed runtime edge between two [`ConceptRef`]s — the generating
/// morphism as data. Carries its kind as a [`ConceptRef`] in the one Relations
/// vocabulary (minted by [`relations_kind`]) so the closure can be folded per
/// kind.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeEdge {
    pub source: ConceptRef,
    pub kind: ConceptRef,
    pub target: ConceptRef,
}

/// A CROSS-ONTOLOGY typed edge departing a node — the foreign-atom half
/// [`RuntimeEdge`] / [`morphisms_from`](RuntimeOntology::morphisms_from) drop
/// (they carry only LOCAL generators). It names the connected `ontology` and the
/// content `atom` of the target node there; the `kind` is the edge's relation
/// (minted via [`relations_kind`]) so a reachability query can match it against
/// the kind it asks along. The atom is resolved to the peer's `Definition` — and
/// thence a [`ConceptRef`] — by the generic
/// [`AtomResolver`](crate::grounding::AtomResolver), the same primitive the
/// lexical `denotes` floor already uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroundedEdge {
    /// The edge's relation kind, in the one Relations vocabulary.
    pub kind: ConceptRef,
    /// The connected ontology the target atom lives in.
    pub ontology: String,
    /// The content address of the target atom in that ontology.
    pub atom: ContentAddress,
}

/// Lazy, memoized reachability over ONE transitive relation-kind's GENERATING
/// adjacency — the per-kind engine [`MaterializedClosure`] partitions by kind.
///
/// It stores only the direct generators (`source → targets`), linear in the
/// edge count — NOT the eagerly-saturated `O(V · depth)` transitive closure.
/// The reflexive-transitive image of a vertex is the free functor
/// `FreeCategory<Q> → ReachCat` (Mac Lane 1971 CWM II.7) restricted to that
/// vertex, computed ON DEMAND by a bounded, cycle-safe breadth-first walk over
/// the generators and MEMOIZED per source: the first query for a vertex pays
/// the walk, every later one hits the memo with no re-traversal (a `reaches`
/// membership test borrows the cached image allocation-free; an image query
/// copies it). The walk's hop count is the
/// minimal number of generating edges to each reachable vertex (BFS shortest
/// path; Moore 1959) — the same shortest-path grading the eager
/// Floyd–Warshall fold produced over the same generators, so the graded image
/// is identical.
///
/// Literature:
/// - Mac Lane (1971) *CWM* II.7 — the free category on a quiver and the unique
///   functor into the thin reachability category; this image IS that functor's
///   action, evaluated lazily per vertex rather than saturated in bulk.
/// - Moore (1959) "The shortest path through a maze" — breadth-first search
///   yields the minimal-hop distance to every reachable vertex; the grading
///   here is that distance over the generating edges.
#[derive(Debug, Default)]
struct LazyKindReach {
    /// `source → direct targets` — the generators of this kind. Deduped, and
    /// self-loops are dropped (a `source == target` generator carries nothing
    /// beyond the implicit reflexive arrow), matching the eager fold's handling.
    adjacency: BTreeMap<ConceptRef, Vec<ConceptRef>>,
    /// Memoized STRICT images: `source → [(descendant, min hops)]`, filled on
    /// first query. Interior mutability keeps the query surface `&self`. This
    /// `RefCell` makes `LazyKindReach` — hence `MaterializedClosure` and
    /// `RuntimeOntology` — `!Sync`; that is a DELIBERATE INVARIANT of the runtime
    /// (chat / wasm are single-threaded, and no `static`/`OnceLock` holds a
    /// `RuntimeOntology`). A future threaded native server that shares
    /// `&RuntimeOntology` across threads must put this memo behind a `Mutex`, not
    /// weaken the invariant silently. The cache is a *derived* view of
    /// `adjacency` — never part of identity — so equality and the union merge
    /// ignore it.
    memo: RefCell<BTreeMap<ConceptRef, Vec<(ConceptRef, u32)>>>,
}

impl Clone for LazyKindReach {
    fn clone(&self) -> Self {
        Self {
            adjacency: self.adjacency.clone(),
            // The memo is a valid derived cache of the same adjacency; carrying
            // it over a clone is sound (it could equally be dropped empty).
            memo: RefCell::new(self.memo.borrow().clone()),
        }
    }
}

impl PartialEq for LazyKindReach {
    /// Identity is the GENERATORS; the memo is a derived cache, not part of it.
    fn eq(&self, other: &Self) -> bool {
        self.adjacency == other.adjacency
    }
}

impl Eq for LazyKindReach {}

impl LazyKindReach {
    /// Add a generating edge `source → target`. Self-loops are dropped and
    /// duplicate targets deduped, so the adjacency stays the minimal generator
    /// set the eager fold would have consumed.
    fn insert_edge(&mut self, source: ConceptRef, target: ConceptRef) {
        if source == target {
            return;
        }
        let targets = self.adjacency.entry(source).or_default();
        if !targets.contains(&target) {
            targets.push(target);
        }
    }

    /// The cycle-safe BFS walk that computes `source`'s STRICT reachable image
    /// from the generators — the pure kernel behind both [`strict_image`] (which
    /// memoizes + clones it out) and [`reaches`] (which scans it without
    /// cloning). The first time a vertex is enqueued is along a shortest
    /// (fewest-hop) path, so its recorded hop count is minimal (Moore 1959); the
    /// `seen` set makes the walk terminating even on cyclic generators (each
    /// vertex is enqueued at most once) — bounded by the reachable-set size,
    /// never divergent.
    ///
    /// [`strict_image`]: Self::strict_image
    /// [`reaches`]: Self::reaches
    fn compute_image(&self, source: &ConceptRef) -> Vec<(ConceptRef, u32)> {
        let mut image: Vec<(ConceptRef, u32)> = Vec::new();
        let mut seen: BTreeSet<ConceptRef> = BTreeSet::new();
        seen.insert(source.clone());
        let mut queue: VecDeque<(ConceptRef, u32)> = VecDeque::new();
        queue.push_back((source.clone(), 0));
        while let Some((vertex, hops)) = queue.pop_front() {
            if let Some(targets) = self.adjacency.get(&vertex) {
                for target in targets {
                    if seen.insert(target.clone()) {
                        image.push((target.clone(), hops + 1));
                        queue.push_back((target.clone(), hops + 1));
                    }
                }
            }
        }
        image
    }

    /// The STRICT reachable image of `source` — its descendants under the
    /// closure (excluding `source`), each with the minimal number of generating
    /// edges to it. Computed once by a cycle-safe BFS, then memoized; the
    /// reflexive `source → source` (hop 0) arrow is implicit and added by
    /// [`reflexive_image`](Self::reflexive_image).
    fn strict_image(&self, source: &ConceptRef) -> Vec<(ConceptRef, u32)> {
        if let Some(hit) = self.memo.borrow().get(source) {
            return hit.clone();
        }
        let image = self.compute_image(source);
        self.memo.borrow_mut().insert(source.clone(), image.clone());
        image
    }

    /// Does `source` STRICTLY reach `target`? — membership in `source`'s
    /// (memoized) strict image. The reflexive `source == target` case is the
    /// caller's decision (per-relation), never assumed here.
    ///
    /// Membership only, so it borrows the cached image and scans it WITHOUT
    /// cloning — the eager form's O(1) `contains`; a full-image clone here would
    /// be wasteful on the hot is-a path. On a miss it computes the image, tests
    /// it, then stores it (populating the memo for later `strict_image` calls).
    fn reaches(&self, source: &ConceptRef, target: &ConceptRef) -> bool {
        if let Some(hit) = self.memo.borrow().get(source) {
            return hit.iter().any(|(v, _)| v == target);
        }
        let image = self.compute_image(source);
        let found = image.iter().any(|(v, _)| v == target);
        self.memo.borrow_mut().insert(source.clone(), image);
        found
    }

    /// The reflexive image of `source` — `source` at hop 0 plus its strict
    /// image.
    fn reflexive_image(&self, source: &ConceptRef) -> Vec<(ConceptRef, u32)> {
        let mut out = alloc::vec![(source.clone(), 0u32)];
        out.extend(self.strict_image(source));
        out
    }

    /// Whether this kind has ANY generating edge — the basis for
    /// [`MaterializedClosure::populated_kinds`]. A kind whose only generators
    /// were self-loops (all dropped) has empty adjacency and is not populated,
    /// matching the eager form (whose closure would have been empty).
    fn has_edges(&self) -> bool {
        self.adjacency.values().any(|targets| !targets.is_empty())
    }

    /// The lattice meet of `a` and `b` — the nearest vertex in
    /// `strict_image(b) ∩ reflexive_image(a)`, ranked by hops from `b` (nearest
    /// first), ties broken by `tie_key`. Verbatim the semantics of the eager
    /// `ReachabilityClosure::meet_by`, over the lazily-computed images.
    fn meet_by<K: Ord>(
        &self,
        a: &ConceptRef,
        b: &ConceptRef,
        tie_key: impl Fn(&ConceptRef) -> K,
    ) -> Option<ConceptRef> {
        let anc_a: BTreeSet<ConceptRef> = self
            .reflexive_image(a)
            .into_iter()
            .map(|(v, _)| v)
            .collect();
        self.strict_image(b)
            .into_iter()
            .filter(|(v, _)| anc_a.contains(v))
            .min_by(|(v1, d1), (v2, d2)| d1.cmp(d2).then_with(|| tie_key(v1).cmp(&tie_key(v2))))
            .map(|(v, _)| v)
    }
}

/// The reachability answer of a materialized ontology — per transitive
/// relation-kind [`ConceptRef`], the generating adjacency, queried LAZILY.
///
/// It holds only the generators (linear in the edge count), NOT the
/// eagerly-saturated transitive-closure set; the reflexive-transitive image of
/// a queried vertex is computed on demand and memoized (see [`LazyKindReach`]).
/// Keyed by `ConceptRef` so an N-ontology composite can [`union`](Self::union)
/// these maps. Every query answer is identical to the eagerly-folded form — a
/// representation change (footprint), not a semantics change.
#[derive(Debug, Clone, Default)]
pub struct MaterializedClosure {
    /// `kind → LazyKindReach` — the per-kind generating adjacency. An entry
    /// exists for EVERY declared transitive kind (even one with no edges), so
    /// [`populated_kinds`](Self::populated_kinds) filters the truly non-empty.
    per_kind: BTreeMap<ConceptRef, LazyKindReach>,
}

impl PartialEq for MaterializedClosure {
    /// Identity is the per-kind generators; the memoized images are a derived
    /// cache and are ignored (that comparison lives in [`LazyKindReach`]).
    fn eq(&self, other: &Self) -> bool {
        self.per_kind == other.per_kind
    }
}

impl Eq for MaterializedClosure {}

impl MaterializedClosure {
    /// Build the per-kind generating adjacency from `edges` — the free-functor
    /// image `FreeCategory<Q> → ReachCat` (Mac Lane 1971 CWM II.7) captured as
    /// its generators, NOT saturated. Unlike the former eager fold, no
    /// transitive closure is computed here; reachability is evaluated lazily per
    /// vertex at query time (see the module docs and [`LazyKindReach`]). We keep
    /// only the GENERATING edges and never trust a pre-stored closure: a `.prx`
    /// whose edges are already a closure just supplies redundant generators the
    /// BFS ignores (the closure of a closure is the same closure).
    pub fn fold(edges: &[RuntimeEdge], transitive: &BTreeSet<ConceptRef>) -> Self {
        let mut per_kind: BTreeMap<ConceptRef, LazyKindReach> = BTreeMap::new();
        // `transitive` is the LOADED transitive-kind vocabulary (the kinds OWL-RL
        // marks `Transitive`); one adjacency per kind in it — never a hardcoded
        // array. An entry is inserted for every kind (even edge-less), matching
        // the eager form so `populated_kinds` reports identically.
        for kind in transitive {
            let mut reach = LazyKindReach::default();
            for edge in edges.iter().filter(|e| &e.kind == kind) {
                reach.insert_edge(edge.source.clone(), edge.target.clone());
            }
            per_kind.insert(kind.clone(), reach);
        }
        Self { per_kind }
    }

    /// The reachable set from `source` along `kind` — the STRICT reachable set
    /// (descendants of `source`; the reflexive `source → source` arrow is
    /// implicit and not included, matching the prior behavior). Computed by a
    /// bounded, memoized BFS on first ask, an O(1) cache hit after. Empty set if
    /// `source` has no outgoing edges of `kind`.
    pub fn reachable_from(&self, source: &ConceptRef, kind: ConceptRef) -> BTreeSet<ConceptRef> {
        self.per_kind
            .get(&kind)
            .map(|reach| {
                reach
                    .strict_image(source)
                    .into_iter()
                    .map(|(v, _)| v)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Does `source` reach `target` along `kind`? — membership in `source`'s
    /// (memoized) strict image. (Strict reachability: a vertex does not reach
    /// itself here, matching [`reachable_from`](Self::reachable_from); the
    /// reflexive `is-a` case is the caller's `child == ancestor` short-circuit.)
    pub fn reaches(&self, source: &ConceptRef, target: &ConceptRef, kind: ConceptRef) -> bool {
        self.per_kind
            .get(&kind)
            .is_some_and(|reach| source != target && reach.reaches(source, target))
    }

    /// The relation kinds this ontology actually POPULATES — the keys with at
    /// least one generating edge. A transitive kind with no edges (an entry is
    /// held for EVERY declared transitive kind) is omitted, so this reports what
    /// the ontology can really answer, not what the vocabulary permits. The
    /// data-driven basis for an ontology's CAPABILITIES (doc §4.7): a USC
    /// mereology populates `Parthood`, an OWL vocabulary `Subsumption` — read
    /// off the loaded data, not hardcoded.
    pub fn populated_kinds(&self) -> Vec<ConceptRef> {
        self.per_kind
            .iter()
            .filter(|(_, reach)| reach.has_edges())
            .map(|(kind, _)| kind.clone())
            .collect()
    }

    /// The strict image of `c` along the relation `kind` — every node reachable
    /// from it under that kind (excluding `c`), each with its minimal hop count.
    /// RELATION-PARAMETRIC: `image(c, Subsumption)` is the hypernym ancestors,
    /// `image(c, Parthood)` the wholes `c` is transitively part of. A bounded,
    /// memoized per-vertex BFS, not a bulk-saturated lookup.
    pub fn image(&self, c: &ConceptRef, kind: &ConceptRef) -> Vec<(ConceptRef, u32)> {
        self.per_kind
            .get(kind)
            .map(|reach| reach.strict_image(c))
            .unwrap_or_default()
    }

    /// The Subsumption (hypernym) image of `c` — `image(c, Subsumption)`.
    /// The loaded-ontology analogue of `English::ancestors`.
    pub fn subsumption_image(&self, c: &ConceptRef) -> Vec<(ConceptRef, u32)> {
        self.image(c, &subsumption_kind())
    }

    /// The lattice MEET of `a` and `b` over the relation `kind` — the nearest
    /// node both reach (`strict_image(b) ∩ reflexive_image(a)`, nearest-first),
    /// ties broken by `(ontology, name)`. RELATION-PARAMETRIC: the nearest
    /// common hypernym for Subsumption, the nearest common whole for Parthood.
    pub fn meet(&self, a: &ConceptRef, b: &ConceptRef, kind: &ConceptRef) -> Option<ConceptRef> {
        self.per_kind.get(kind).and_then(|reach| {
            reach.meet_by(a, b, |c| (c.ontology.as_str().to_string(), c.name.clone()))
        })
    }

    /// The lattice meet over the Subsumption closure — `meet(a, b, Subsumption)`,
    /// the nearest shared hypernym.
    pub fn subsumption_meet(&self, a: &ConceptRef, b: &ConceptRef) -> Option<ConceptRef> {
        self.meet(a, b, &subsumption_kind())
    }

    /// The ordered chain `[child, …, ancestor]` (nearest-first) along the relation
    /// `kind` when `child` reaches `ancestor`, else `None` — the EVIDENCE path,
    /// read off the (lazily-computed) reachability rather than hand-walked.
    /// RELATION-PARAMETRIC: the is-a chain for Subsumption, the part-of chain
    /// (`subsection → section → title`) for Parthood.
    pub fn chain(
        &self,
        child: &ConceptRef,
        ancestor: &ConceptRef,
        kind: &ConceptRef,
    ) -> Option<Vec<ConceptRef>> {
        let reach = self.per_kind.get(kind)?;
        if child != ancestor && !reach.reaches(child, ancestor) {
            return None;
        }
        // Reflexive ancestors of `child` that still reach `ancestor` lie on a
        // child⇝ancestor path; order them nearest-first by is-a distance.
        let mut chain: Vec<(ConceptRef, u32)> = reach
            .reflexive_image(child)
            .into_iter()
            .filter(|(x, _)| x == ancestor || reach.reaches(x, ancestor))
            .collect();
        chain.sort_unstable_by(|(a, da), (b, db)| {
            da.cmp(db)
                .then_with(|| a.ontology.as_str().cmp(b.ontology.as_str()))
                .then_with(|| a.name.cmp(&b.name))
        });
        Some(chain.into_iter().map(|(v, _)| v).collect())
    }

    /// The ordered hypernym chain over the Subsumption closure — `chain(child,
    /// ancestor, Subsumption)`, the is-a evidence path.
    pub fn subsumption_chain(
        &self,
        child: &ConceptRef,
        ancestor: &ConceptRef,
    ) -> Option<Vec<ConceptRef>> {
        self.chain(child, ancestor, &subsumption_kind())
    }

    /// Union another closure into this one — the structural hook for an
    /// N-ontology composite. We merge the GENERATING adjacency per kind (the
    /// union of two generator sets generates the union reachability), and
    /// invalidate the affected memo so later queries recompute over the enlarged
    /// graph. Simpler and stricter than the former eager re-fold: no closure is
    /// materialized, and BFS over the merged generators is exactly the union
    /// reachability. (Single-ontology callers never need it; it exists so the
    /// closure is N-ready by construction.)
    pub fn union(&mut self, other: &MaterializedClosure) {
        for (kind, other_reach) in &other.per_kind {
            let into = self.per_kind.entry(kind.clone()).or_default();
            for (source, targets) in &other_reach.adjacency {
                for target in targets {
                    into.insert_edge(source.clone(), target.clone());
                }
            }
            // The adjacency grew — any cached images are now stale; drop them so
            // the next query re-walks the merged generators.
            into.memo.borrow_mut().clear();
        }
    }
}

/// Why an [`Archive`] could not be materialized into a [`RuntimeOntology`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterializeError {
    /// The archive's Merkle root could not be derived (codec failure).
    Root(CodecError),
    /// An edge names an endpoint that is not a declared node — referential
    /// closure is violated. Carries a [`DanglingEdge`] counterexample naming the
    /// orphan, never a silent bool.
    DanglingEdge(DanglingEdge),
    /// The `rkyv` cache buffer failed `bytecheck` validation — a corrupted
    /// transcode (in [`materialize`]) or a corrupted input buffer (in
    /// [`materialize_bytes`]). Fail-closed rather than storing an unvalidated
    /// buffer the zero-copy query path would `access_unchecked`.
    Archive(ArchiveLensError),
}

/// The typed counterexample to referential closure: an edge whose `endpoint`
/// (the `orphan` name) is not among the archive's declared nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanglingEdge {
    /// The source node the dangling edge departs from.
    pub source: String,
    /// The relation-kind name on the edge.
    pub kind: String,
    /// The endpoint name that has no declaring node — the orphan.
    pub orphan: String,
}

impl core::fmt::Display for MaterializeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MaterializeError::Root(e) => write!(f, "materialize: derive root: {e}"),
            MaterializeError::DanglingEdge(d) => write!(
                f,
                "materialize: dangling edge {}--{}-->{}: target node {:?} is not declared",
                d.source, d.kind, d.orphan, d.orphan
            ),
            MaterializeError::Archive(e) => write!(f, "materialize: rkyv cache buffer: {e}"),
        }
    }
}

impl std::error::Error for MaterializeError {}

/// One loaded `.prx` as a single typed, queryable ontology.
///
/// Identity is the content address: two `RuntimeOntology`s are equal iff their
/// archive roots ([`Archive::root`]) agree. The reachability engine (the
/// per-kind generating adjacency, queried lazily) is held alongside the archived
/// buffer (the open form) and the root (the identity).
///
/// # The open form is the archived BUFFER, reasoned over in place (Step 1c)
///
/// The archive is NOT held as an owned [`Archive`] of `String`/`Vec`-heavy
/// [`Definition`]s; it is held as its `rkyv` local-cache bytes
/// ([`ArchiveLens::put_aligned`]), `bytecheck`-validated ONCE at materialize.
/// Every query reads a borrowed [`ArchivedArchiveView`] straight out of that
/// immutable, 16-aligned buffer ([`archive`](Self::archive)) — zero owned
/// rebuild. This is the runtime half of "reason over the archived buffer, not an
/// owned graph" (review §3.1, Lever A): the loaded USC / OWL / legal-source
/// path stops materializing a second owned copy of the whole graph. The closure
/// keys on owned [`ConceptRef`]s (never archived references), so the buffer is a
/// plain `&self` field, not a self-referential struct.
#[derive(Clone)]
pub struct RuntimeOntology {
    id: OntologyName,
    root: ContentAddress,
    /// The `rkyv` cache bytes of the archive — 16-aligned, `bytecheck`-validated
    /// at materialize, immutable thereafter. Queried zero-copy via
    /// [`archive`](Self::archive).
    buf: AlignedVec<16>,
    closure: MaterializedClosure,
}

impl core::fmt::Debug for RuntimeOntology {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // The buffer is opaque cache bytes; print its length, not its contents.
        f.debug_struct("RuntimeOntology")
            .field("id", &self.id)
            .field("root", &self.root)
            .field("buf_len", &self.buf.len())
            .field("closure", &self.closure)
            .finish()
    }
}

impl PartialEq for RuntimeOntology {
    /// Content-address identity: equal iff the archive roots agree. The `id`
    /// label and the (re-derivable) closure are not part of identity.
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}

impl Eq for RuntimeOntology {}

impl RuntimeOntology {
    /// The ontology's name.
    pub fn id(&self) -> &OntologyName {
        &self.id
    }

    /// The content-address identity — the archive's Merkle root.
    pub fn root(&self) -> ContentAddress {
        self.root
    }

    /// The open form this ontology was materialized from — a borrowed,
    /// zero-copy [`ArchivedArchiveView`] over the validated `rkyv` buffer (no
    /// owned rebuild). Its fields mirror [`Archive`] but every leaf is its
    /// archived form (`ArchivedString`, `ArchivedVec`, …); read an edge target's
    /// local name with [`archived_local_name`].
    pub fn archive(&self) -> &ArchivedArchiveView {
        // SAFETY: `self.buf` was `bytecheck`-validated by `ArchiveLens::access`
        // at materialize time and is immutable for the lifetime of `self`, so the
        // zero-copy access is sound. This is the deliberate `access_unchecked`
        // that pays bytecheck exactly once — the memory notes' never-used unsafe.
        unsafe { ArchiveLens::access_unchecked(self.buf.as_slice()) }
    }

    /// The reachability engine — the per-kind generating adjacency, queried
    /// lazily (see [`MaterializedClosure`]).
    pub fn closure(&self) -> &MaterializedClosure {
        &self.closure
    }

    /// A [`ConceptRef`] in this ontology for `name` (does not check the name
    /// exists — use it to build query arguments against a known node).
    pub fn concept(&self, name: impl Into<String>) -> ConceptRef {
        ConceptRef::new(self.id.clone(), name)
    }

    // --- query surface (lazily-computed reachability over the generators) ---

    /// Every outgoing GENERATING edge from `c` — the morphisms departing this
    /// concept, as typed [`RuntimeEdge`]s. (Generating edges, not the closure;
    /// the closure is served by [`reachable_from`](Self::reachable_from).)
    pub fn morphisms_from(&self, c: &ConceptRef) -> Vec<RuntimeEdge> {
        self.archive()
            .nodes
            .iter()
            .filter(|n| n.name == c.name.as_str())
            .flat_map(|n| {
                n.edges.iter().filter_map(move |edge| {
                    // Archived edges are `ArchivedTuple2(kind_name, target)`.
                    let (kind_name, target) = (&edge.0, &edge.1);
                    // Only LOCAL edges are morphisms within this ontology; a
                    // Grounded target is a cross-ontology atom, resolved by the
                    // ContainsAtom step, not a generator of this graph.
                    let name = archived_local_name(target)?;
                    // EVERY local edge is a morphism now — its kind-name resolves
                    // into the one Relations vocabulary; no kind is dropped (the
                    // old `from_edge_kind` 3-kind filter is gone).
                    Some(RuntimeEdge {
                        source: c.clone(),
                        kind: relations_kind(kind_name.as_str()),
                        target: ConceptRef::new(self.id.clone(), name.to_string()),
                    })
                })
            })
            .collect()
    }

    /// Every CROSS-ONTOLOGY grounded edge departing `c` — the foreign-atom edges
    /// [`morphisms_from`](Self::morphisms_from) deliberately drops. Read straight
    /// off the archived buffer via [`archived_grounded`]; each carries its relation
    /// `kind`, the connected `ontology`, and the target `atom` a resolver binds.
    ///
    /// This is the read half of the type-grounding path: a loaded USC section
    /// carries a grounded edge into `LegalSources` (its `Statute` typing), minted
    /// by the `usc_legal_sources_functor` grounding lens; a composed reasoner
    /// reads it here, resolves the atom to the peer concept, and CONTINUES its
    /// reachability query inside the peer ontology's closure.
    pub fn grounded_edges_from(&self, c: &ConceptRef) -> Vec<GroundedEdge> {
        self.archive()
            .nodes
            .iter()
            .filter(|n| n.name == c.name.as_str())
            .flat_map(|n| {
                n.edges.iter().filter_map(move |edge| {
                    let (kind_name, target) = (&edge.0, &edge.1);
                    let (ontology, atom) = archived_grounded(target)?;
                    Some(GroundedEdge {
                        kind: relations_kind(kind_name.as_str()),
                        ontology: ontology.to_string(),
                        atom,
                    })
                })
            })
            .collect()
    }

    /// The owned [`Archive`] this ontology was materialized from — the OWNING GET
    /// over the retained `rkyv` buffer ([`ArchiveLens::get`]). The zero-copy query
    /// path reads [`archive`](Self::archive) instead; this is for the rare caller
    /// that needs the owned form (e.g. a peer archive an
    /// [`AtomResolver`](crate::grounding::AtomResolver) indexes by
    /// [`Definition::address`](crate::definition::Definition::address), which the
    /// archived view does not carry). Fail-closed on a corrupted buffer, though the
    /// buffer was already `bytecheck`-validated at materialize.
    pub fn to_owned_archive(&self) -> Result<Archive, ArchiveLensError> {
        ArchiveLens::get(self.buf.as_slice())
    }

    /// The reachable set from `c` along `kind` — the strict descendants under
    /// that relation, computed by a bounded, memoized BFS over the generators
    /// (O(1) once cached).
    pub fn reachable_from(&self, c: &ConceptRef, kind: ConceptRef) -> BTreeSet<ConceptRef> {
        self.closure.reachable_from(c, kind)
    }

    /// Is `child` a `ancestor`? — membership of `ancestor` in `child`'s
    /// Subsumption closure. Returns the witnessing [`Verdict`] (never a bool): a
    /// [`Proof`](pr4xis::logic::proof::Proof) carrying the relation when it
    /// holds, a [`Counterexample`](pr4xis::logic::proof::Counterexample) when it
    /// does not. The decision is a membership test in `child`'s (memoized)
    /// Subsumption image.
    pub fn is_a(&self, child: &ConceptRef, ancestor: &ConceptRef) -> Verdict {
        let holds = self.closure.reaches(child, ancestor, subsumption_kind());
        let meta = self.is_a_meta(child, ancestor);
        if holds {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }

    /// The node's lexical grounding (its Lemon gloss / canonical English form),
    /// if the declaring [`Definition`](crate::definition::Definition) carries
    /// one.
    pub fn lexical(&self, c: &ConceptRef) -> Option<&str> {
        self.archive()
            .nodes
            .iter()
            .find(|n| n.name == c.name.as_str())
            .and_then(|n| n.lexical.as_deref())
    }

    /// Provenance for an `is_a` verdict — names the witnessed subsumption
    /// claim and cites the transitive-closure reading.
    fn is_a_meta(&self, child: &ConceptRef, ancestor: &ConceptRef) -> Provenance {
        Provenance {
            name: OntologyName::new(alloc::format!(
                "IsA[{}/{} ⊑ {}/{}]",
                child.ontology,
                child.name,
                ancestor.ontology,
                ancestor.name
            )),
            description: Label::new(alloc::format!(
                "{} is-a {} via the Subsumption transitive closure",
                child.name,
                ancestor.name
            )),
            citation: Citation::parse_static(
                "Guarino (2009); Smith et al. (2005) OBO Relation Ontology (transitive_over); Mac Lane (1971) CWM II.7",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

/// Materialize an [`Archive`] into a single typed [`RuntimeOntology`].
///
/// 1. Capture the archive's Merkle root (the content-address identity).
/// 2. Build the generating edges over [`ConceptRef`] from each node's
///    `Definition.edges`.
/// 3. VALIDATE referential closure: every edge endpoint must be a declared node
///    — a dangling target returns a typed [`MaterializeError::DanglingEdge`]
///    with a [`DanglingEdge`] counterexample naming the orphan, never a silent
///    bool. (Self-edges to the node's own name are trivially closed.)
/// 4. Capture the generating adjacency (the free-functor image, kept as its
///    generators; reachability is evaluated lazily at query time, never a stored
///    closure).
///
/// ## Honestly deferred — connection-law verification
///
/// A `.prx` carries [`Connection::laws`](crate::connection::Connection::laws) as
/// *names of laws as data* (e.g. `"PreservesComposition"`). Resolving those law
/// names to runnable [`Axiom`](pr4xis::logic::axiom::Axiom)s and verifying them
/// at materialize time requires a name→axiom registry that does not yet exist in
/// `pr4xis-runtime` (the closed-world axiom constructors live in
/// `pr4xis::ontology::reasoning`, keyed by typed `Category`/`Kind`, not by a
/// wire-name string). Wiring that resolver in here would balloon this step into
/// a second feature. It is therefore deferred to a tracked follow-up rather than
/// stubbed: this materializer does NOT silently return `Ok` while pretending to
/// have verified laws — it simply does not claim to verify them. See the runtime
/// convergence thread / follow-up issue "resolve Connection.laws-as-data to
/// runnable Axioms at materialize time". Referential closure (step 3) IS
/// verified and fails closed.
pub fn materialize(
    archive: Archive,
    id: OntologyName,
) -> Result<RuntimeOntology, MaterializeError> {
    // Derive the content root + verify referential closure + capture the
    // generating adjacency over the OWNED archive (the root needs `Definition`
    // addressing, which the archived view does not carry).
    let (root, closure) = analyze(&archive, &id)?;

    // Transcode the owned archive into its 16-aligned `rkyv` cache bytes (the
    // ArchiveLens PUT), then drop the owned archive — the buffer is the retained
    // open form. Validate ONCE with `bytecheck` here so every later zero-copy
    // query over the buffer is sound without re-paying validation.
    let buf = ArchiveLens::put_aligned(&archive);
    ArchiveLens::access(buf.as_slice()).map_err(MaterializeError::Archive)?;

    Ok(RuntimeOntology {
        id,
        root,
        buf,
        closure,
    })
}

/// Materialize a [`RuntimeOntology`] from `rkyv` cache bytes already in hand —
/// the buffer-first sibling of [`materialize`] for a caller that holds an
/// [`ArchiveLens::put_aligned`] buffer (e.g. a cached `.prx` blob) rather than an
/// owned [`Archive`].
///
/// The buffer is `bytecheck`-validated once, then read back to an owned
/// [`Archive`] SOLELY to derive the content root and verify referential closure
/// (both need `Definition` addressing); the validated buffer itself is kept
/// verbatim as the retained open form — no re-PUT. The resulting ontology is
/// query-identical to `materialize(ArchiveLens::get(&buf)?, id)`.
pub fn materialize_bytes(
    buf: AlignedVec<16>,
    id: OntologyName,
) -> Result<RuntimeOntology, MaterializeError> {
    // Validate the incoming buffer before it is ever `access_unchecked`-ed.
    ArchiveLens::access(buf.as_slice()).map_err(MaterializeError::Archive)?;
    // Owning decode only to derive root + referential closure + generators.
    let archive = ArchiveLens::get(buf.as_slice()).map_err(MaterializeError::Archive)?;
    let (root, closure) = analyze(&archive, &id)?;
    Ok(RuntimeOntology {
        id,
        root,
        buf,
        closure,
    })
}

/// Derive the content-address root and the per-kind generating adjacency of an
/// owned [`Archive`], validating referential closure — the shared kernel of
/// [`materialize`] and [`materialize_bytes`].
///
/// 1. Capture the archive's Merkle root (the content-address identity).
/// 2. Build the generating edges over [`ConceptRef`] from each node's
///    `Definition.edges`.
/// 3. VALIDATE referential closure: every LOCAL edge endpoint must be a declared
///    node — a dangling target returns [`MaterializeError::DanglingEdge`]. A
///    GROUNDED target is a foreign atom held as a cross-ontology edge (resolved
///    by the ContainsAtom step), not validated here.
/// 4. Capture the generating adjacency (the free-functor image kept as its
///    generators; reachability is evaluated lazily at query time).
fn analyze(
    archive: &Archive,
    id: &OntologyName,
) -> Result<(ContentAddress, MaterializedClosure), MaterializeError> {
    // 1. Capture the content-address identity up front.
    let root = archive.root().map_err(MaterializeError::Root)?;

    // The declared node names — the referential universe.
    let declared: BTreeSet<&str> = archive.nodes.iter().map(|n| n.name.as_str()).collect();

    // 2 + 3. Build the generating edges over ConceptRef, validating that every
    // endpoint is a declared node (referential closure) as we go.
    let mut edges: Vec<RuntimeEdge> = Vec::new();
    for node in &archive.nodes {
        for (kind_name, target) in &node.edges {
            let local = match target {
                // A LOCAL target must name a declared node — referential closure.
                EdgeTarget::Local(name) => name,
                // A GROUNDED target is a foreign atom (by content address) in a
                // connected ontology — it is HELD as a cross-ontology edge, not
                // validated against this archive's names and not folded into the
                // local closure. Resolution against the connected ontology is the
                // ContainsAtom step (fail-closed there); carrying it unresolved
                // here is the open-world span endpoint.
                EdgeTarget::Grounded { .. } => continue,
            };
            if !declared.contains(local.as_str()) {
                return Err(MaterializeError::DanglingEdge(DanglingEdge {
                    source: node.name.clone(),
                    kind: kind_name.clone(),
                    orphan: local.clone(),
                }));
            }
            // EVERY referentially-valid local edge becomes a generator; its
            // kind-name resolves into the one Relations vocabulary. WHICH kinds
            // actually close is decided by the loaded transitive set threaded to
            // `fold` below — not by dropping edges here (the old `from_edge_kind`
            // 3-kind filter is gone). A non-transitive kind simply contributes no
            // closed pairs.
            edges.push(RuntimeEdge {
                source: ConceptRef::new(id.clone(), node.name.clone()),
                kind: relations_kind(kind_name.clone()),
                target: ConceptRef::new(id.clone(), local.clone()),
            });
        }
    }

    // 4. Capture the generating adjacency per kind — the free-functor image kept
    // as its generators — over the loaded transitive-kind set, read from the
    // Relations vocabulary cache ([`declared_transitive_kinds`]); the `ontology!`
    // macro reads its own copy of the SAME cache, so the compile-time and runtime
    // reachability range over identical kinds. Reachability is then evaluated
    // lazily per queried vertex (see [`MaterializedClosure`]).
    let closure = MaterializedClosure::fold(&edges, &declared_transitive_kinds());

    Ok((root, closure))
}

/// The project-less core of every envelope loader: interpret a *raw* source
/// [`Archive`] through a functor `action` (the data-driven free extension,
/// [`apply`]) and [`materialize`] the praxis image under `id`. This is the
/// `apply → materialize` tail the owl / english / usc bridges each open-coded
/// byte-for-byte; the format-specific *projection* (a domain struct → a raw
/// [`Archive`]) stays the caller's step — the only genuinely per-format code, so
/// the kernel carries no `match` on format.
///
/// `action` is `&`[`GeneratorAction`] — exactly [`apply`]'s parameter — so each
/// bridge keeps owning its fail-closed `*_functor()` connection load (the
/// integrity pin) and passes only `&conn.action`. The kernel never sees the
/// committed `.prx` bytes or any root hex; nothing is re-pinned.
///
/// [`apply`] is infallible on a `Functor` action (it fails closed only on a
/// non-`Functor` [`GeneratorAction`]); the bridges always pass a functor
/// connection, so the only fallible step is [`materialize`], whose
/// [`MaterializeError`] propagates.
pub fn apply_then_materialize(
    action: &GeneratorAction,
    source: &Archive,
    id: OntologyName,
) -> Result<RuntimeOntology, MaterializeError> {
    let praxis = apply(action, source).expect(
        "a Functor action, which `apply` always interprets (fail-closed only on non-Functor)",
    );
    materialize(praxis, id)
}

/// The transitive relation-kind vocabulary the kernel's [`materialize`] folds the
/// closure over — READ from `relations_transitive_kinds.txt`, a distilled,
/// drift-guarded cache of the Relations ontology's `(R, Transitive, HasProperty)`
/// declarations (the ONE source of truth that [`transitive_kinds`] reads off a
/// loaded archive).
///
/// This is the runtime half of "one declaration, two readers"; the build-time
/// half is the `ontology!` macro (`pr4xis-derive`), which reads its OWN copy of
/// the same cache. The kernel cannot dep `domains` to load Relations directly, so
/// it reads this committed projection — loaded data, never a hardcoded allowlist
/// (the former 3-kind bootstrap). The cache is regenerated by, and drift-guarded
/// against, `emit::<RelationsCategory>()` + [`transitive_kinds`] in the `domains`
/// test suite; a hand-edit or a stale cache fails that test. Because both tiers
/// read the same cache, the compile-time and runtime closures fold identical kinds
/// — no divergence.
fn declared_transitive_kinds() -> BTreeSet<ConceptRef> {
    const RELATIONS_TRANSITIVE_KINDS_SRC: &str = include_str!("relations_transitive_kinds.txt");
    RELATIONS_TRANSITIVE_KINDS_SRC
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(relations_kind)
        .collect()
}

/// The relation-kind wire name carrying a relation's OWL-style properties — the
/// `kind` of the `(R, Property, HasProperty)` morphism the Relations ontology
/// declares (`domains/.../formal/relations/ontology.rs`). The `.prx` edge it
/// lowers to is `("HasProperty", Local(<property>))`.
const HAS_PROPERTY_REL: &str = "HasProperty";

/// The OWL `TransitiveProperty` marker concept — the `Local` edge target whose
/// presence under [`HAS_PROPERTY_REL`] asserts that a relation kind is
/// transitive (OWL-RL `prp-trp`: `Transitive(p) ∧ p(x,y) ∧ p(y,z) → p(x,z)`).
const TRANSITIVE_CONCEPT: &str = "Transitive";

/// The transitive relation kinds DECLARED in a loaded Relations ontology — the
/// runtime mirror of the compile-time `RelationProperty::get` query in
/// `domains/.../formal/relations/ontology.rs`, which reads the SAME
/// `(R, Transitive, HasProperty)` morphisms over the typed `Category`.
///
/// This is the live-archive reading of the same POLICY that
/// `declared_transitive_kinds` caches for the closure fold: a relation kind is
/// transitive iff the
/// Relations ontology asserts `Transitive(R)` — the OWL `TransitiveProperty`
/// membership read off the loaded edge, never a Rust constant. The closure
/// engine ([`ReachabilityClosure::fold`](pr4xis::category::quiver::ReachabilityClosure))
/// is unchanged; only the SET of kinds it folds over becomes data.
///
/// # The blessed wire-boundary lowering
///
/// This is the ONE place the relation-property wire names (`"HasProperty"`,
/// `"Transitive"`) are read (praxis-way rule 11: strings are WIRE, crossed by a
/// single lowering). Every result is a typed [`ConceptRef`] keyed in `relations`'
/// own ontology — `Subsumption` in `Relations` is a distinct, queryable concept,
/// not a bare string. Downstream the closure keys on these `ConceptRef`s; the
/// strings do not appear again.
pub fn transitive_kinds(relations: &RuntimeOntology) -> BTreeSet<ConceptRef> {
    relations
        .archive()
        .nodes
        .iter()
        .filter(|node| {
            node.edges.iter().any(|edge| {
                // Archived edges are `ArchivedTuple2(rel, target)`.
                edge.0 == HAS_PROPERTY_REL
                    && matches!(archived_local_name(&edge.1), Some(name) if name == TRANSITIVE_CONCEPT)
            })
        })
        .map(|node| ConceptRef::new(relations.id().clone(), node.name.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::Definition;
    use crate::emit;

    /// A5 — `apply_then_materialize` is exactly `materialize(apply(action, source), id)`:
    /// the `apply → materialize` tail the owl / usc / english bridges now share. The
    /// in-crate regression gate for the kernel loader (the bridge tests are the
    /// integration proof that behaviour is preserved end-to-end).
    #[test]
    fn apply_then_materialize_is_its_two_steps() {
        let synset = |name: &str, hyper: Option<&str>, gloss: &str| Definition {
            kind: "Synset".into(),
            name: name.into(),
            edges: match hyper {
                Some(h) => vec![("hypernym".to_string(), EdgeTarget::Local(h.to_string()))],
                None => vec![],
            },
            axioms: vec![],
            lexical: Some(gloss.into()),
        };
        // A raw source archive (Synset/hypernym) + the functor that relabels it
        // into praxis (Concept/Subsumption) — the shape every envelope projects to.
        let source = Archive {
            nodes: vec![
                synset("dog", Some("animal"), "a dog"),
                synset("animal", None, "an animal"),
            ],
            connections: vec![],
        };
        let action = GeneratorAction::Functor {
            map_object: vec![("Synset".to_string(), "Concept".to_string())],
            map_morphism: vec![("hypernym".to_string(), "Subsumption".to_string())],
        };
        let id = OntologyName::new_static("MiniLoaded");
        let via = apply_then_materialize(&action, &source, id.clone()).unwrap();
        let by_hand = materialize(apply(&action, &source).unwrap(), id).unwrap();
        // RuntimeOntology isn't PartialEq; its content root IS its identity.
        assert_eq!(
            via.root(),
            by_hand.root(),
            "the loader is exactly its two steps"
        );
        assert_eq!(via.id(), by_hand.id());
    }

    // The same REAL miniature ontology the emit module exercises: Org, with the
    // Subsumption taxonomy Employer/Employee ⊑ Person ⊑ Agent, materialized
    // transitive closure and all.
    pr4xis::ontology! {
        name: "Org",
        source: "pr4xis-runtime ontology materialize test fixture",
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

    fn org() -> RuntimeOntology {
        let archive = emit::emit::<OrgCategory>();
        materialize(archive, OntologyName::new_static("Org")).expect("Org materializes")
    }

    #[test]
    fn employer_reaches_agent_via_the_subsumption_closure() {
        let onto = org();
        let employer = onto.concept("Employer");
        let agent = onto.concept("Agent");
        // Reachability image over the generators — Employer ⊑ Person ⊑ Agent
        // collapses to Employer → Agent (a bounded, memoized BFS, O(1) once cached).
        let descendants = onto.reachable_from(&employer, subsumption_kind());
        assert!(
            descendants.contains(&agent),
            "Employer must reach Agent through the Subsumption closure; got {descendants:?}"
        );
        // Person, too (the direct generator).
        assert!(descendants.contains(&onto.concept("Person")));
    }

    #[test]
    fn is_a_returns_a_verdict_carrying_the_claim() {
        let onto = org();
        let employer = onto.concept("Employer");
        let agent = onto.concept("Agent");
        // The claim IS the Verdict — pattern-match it, never `.is_ok()`.
        match onto.is_a(&employer, &agent) {
            Ok(proof) => {
                let name = proof.meta().name;
                assert!(
                    name.as_str().contains("Employer") && name.as_str().contains("Agent"),
                    "the proof must name the witnessed Employer ⊑ Agent claim; got {name}"
                );
            }
            Err(c) => panic!("expected Employer is-a Agent to be proven; got {:?}", c),
        }
        // And the negative direction refutes (Agent is not an Employer).
        match onto.is_a(&agent, &employer) {
            Err(_) => {}
            Ok(p) => panic!("Agent is-a Employer must refute; got proof {:?}", p),
        }
    }

    #[test]
    fn content_address_identity_equality() {
        // Same archive → same root → equal RuntimeOntologies (content-address
        // identity), even materialized independently.
        let archive = emit::emit::<OrgCategory>();
        let a = materialize(archive.clone(), OntologyName::new_static("Org")).unwrap();
        let b = materialize(archive, OntologyName::new_static("Org")).unwrap();
        assert_eq!(a.root(), b.root());
        assert_eq!(a, b);

        // A structurally different archive → different root → not equal.
        let mut other_archive = emit::emit::<OrgCategory>();
        other_archive.nodes.push(Definition {
            kind: "Concept".into(),
            name: "Stranger".into(),
            edges: alloc::vec![],
            axioms: alloc::vec![],
            lexical: None,
        });
        let other = materialize(other_archive, OntologyName::new_static("Org")).unwrap();
        assert_ne!(a, other);
    }

    #[test]
    fn materialize_bytes_matches_materialize_over_the_same_archive() {
        // The buffer-first loader is query-identical to the owned-archive loader:
        // same content root, same lexical, same reachability — it just skips the
        // owned Archive the caller already transcoded.
        let archive = emit::emit::<OrgCategory>();
        let by_archive =
            materialize(archive.clone(), OntologyName::new_static("Org")).expect("materializes");

        let buf = ArchiveLens::put_aligned(&archive);
        let by_bytes = materialize_bytes(buf, OntologyName::new_static("Org"))
            .expect("materializes from bytes");

        assert_eq!(by_archive.root(), by_bytes.root(), "same content root");
        assert_eq!(by_archive, by_bytes, "content-address identity holds");
        // Same reachability image and same gloss, read over the two open forms.
        let employer = by_bytes.concept("Employer");
        assert_eq!(
            by_archive.reachable_from(&employer, subsumption_kind()),
            by_bytes.reachable_from(&employer, subsumption_kind()),
            "same Subsumption reachable set"
        );
        assert_eq!(
            by_archive.lexical(&employer),
            by_bytes.lexical(&employer),
            "same gloss"
        );
    }

    #[test]
    fn materialize_bytes_rejects_a_corrupted_buffer() {
        // A truncated cache buffer fails closed before it is ever queried.
        let archive = emit::emit::<OrgCategory>();
        let full = ArchiveLens::put_aligned(&archive);
        let mut truncated = AlignedVec::<16>::new();
        truncated.extend_from_slice(&full.as_slice()[..full.len() / 2]);
        assert!(
            matches!(
                materialize_bytes(truncated, OntologyName::new_static("Org")),
                Err(MaterializeError::Archive(_))
            ),
            "a truncated rkyv buffer must fail closed at materialize_bytes"
        );
    }

    #[test]
    fn referential_closure_counterexample_on_a_dangling_edge() {
        // Hand-built archive: an edge whose target node is not declared.
        let archive = Archive {
            nodes: alloc::vec![Definition {
                kind: "Concept".into(),
                name: "Employer".into(),
                edges: alloc::vec![("Subsumption".into(), "Ghost".into())],
                axioms: alloc::vec![],
                lexical: None,
            }],
            connections: alloc::vec![],
        };
        match materialize(archive, OntologyName::new_static("Broken")) {
            Err(MaterializeError::DanglingEdge(d)) => {
                assert_eq!(d.source, "Employer");
                assert_eq!(d.kind, "Subsumption");
                assert_eq!(d.orphan, "Ghost", "the counterexample must name the orphan");
            }
            other => panic!("expected a DanglingEdge counterexample; got {other:?}"),
        }
    }

    #[test]
    fn transitive_kinds_reads_the_loaded_owl_transitive_property() {
        // The runtime mirror of `RelationProperty::get`: a relation kind is
        // transitive iff the loaded ontology asserts `Transitive(R)` (the
        // `("HasProperty", Local("Transitive"))` edge, OWL `TransitiveProperty`)
        // — NOT a Rust constant. Hand-built Relations-shaped archive: Subsumption
        // is transitive, Opposition is merely symmetric. The Transitive/Symmetric
        // marker concepts are declared so referential closure holds.
        let archive = Archive {
            nodes: alloc::vec![
                Definition {
                    kind: "Concept".into(),
                    name: "Subsumption".into(),
                    edges: alloc::vec![("HasProperty".into(), "Transitive".into())],
                    axioms: alloc::vec![],
                    lexical: None,
                },
                Definition {
                    kind: "Concept".into(),
                    name: "Opposition".into(),
                    edges: alloc::vec![("HasProperty".into(), "Symmetric".into())],
                    axioms: alloc::vec![],
                    lexical: None,
                },
                Definition {
                    kind: "Concept".into(),
                    name: "Transitive".into(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: None,
                },
                Definition {
                    kind: "Concept".into(),
                    name: "Symmetric".into(),
                    edges: alloc::vec![],
                    axioms: alloc::vec![],
                    lexical: None,
                },
            ],
            connections: alloc::vec![],
        };
        let relations =
            materialize(archive, OntologyName::new_static("Relations")).expect("materializes");
        let kinds = transitive_kinds(&relations);

        let relations_id = OntologyName::new_static("Relations");
        assert!(
            kinds.contains(&ConceptRef::new(relations_id.clone(), "Subsumption")),
            "Subsumption asserts Transitive(R) → must be a transitive kind; got {kinds:?}"
        );
        assert!(
            !kinds.contains(&ConceptRef::new(relations_id.clone(), "Opposition")),
            "Opposition is symmetric, not transitive → must be excluded; got {kinds:?}"
        );
        assert!(
            !kinds.contains(&ConceptRef::new(relations_id, "Transitive")),
            "the Transitive marker is not itself a transitive relation kind; got {kinds:?}"
        );
        assert_eq!(
            kinds.len(),
            1,
            "exactly one transitive kind in this fixture; got {kinds:?}"
        );
    }

    #[test]
    fn lexical_lookup_returns_a_nodes_gloss() {
        // This test isolates the `lexical()` reader: build a minimal archive
        // whose single node declares its Lemon gloss directly, materialize it,
        // and read the gloss back. (The emit path's own gloss projection is
        // covered by `emit::tests::emits_each_concepts_gloss_as_its_lexical_and_round_trips`.)
        let archive = Archive {
            nodes: alloc::vec![Definition {
                kind: "Concept".into(),
                name: "Employer".into(),
                edges: alloc::vec![],
                axioms: alloc::vec![],
                lexical: Some("One who employs.".into()),
            }],
            connections: alloc::vec![],
        };
        let onto = materialize(archive, OntologyName::new_static("Org")).unwrap();
        assert_eq!(
            onto.lexical(&onto.concept("Employer")),
            Some("One who employs.")
        );
        // A concept with no declared gloss → None (honest absence).
        assert_eq!(onto.lexical(&onto.concept("Nobody")), None);
    }

    #[test]
    fn closure_refold_is_idempotent_on_a_prefolded_input() {
        // emit() already emits the closure's transitive edges as generators;
        // capturing them again as adjacency and querying must yield the same
        // reachable set (closure of a closure = closure) — the BFS is correct
        // regardless of whether the stored edges are already transitively closed.
        let onto = org();
        let employer = onto.concept("Employer");
        let direct_then_closed = onto.reachable_from(&employer, subsumption_kind());
        // Folding the generating edges again over the already-materialized
        // ontology's edges is stable.
        let refolded = MaterializedClosure::fold(
            &onto
                .archive()
                .nodes
                .iter()
                .flat_map(|n| onto.morphisms_from(&onto.concept(n.name.to_string())))
                .collect::<Vec<_>>(),
            &declared_transitive_kinds(),
        );
        assert_eq!(
            refolded.reachable_from(&employer, subsumption_kind()),
            direct_then_closed
        );
    }

    /// A concept reference in a throwaway test ontology `T`.
    fn tref(name: &str) -> ConceptRef {
        ConceptRef::new(OntologyName::new_static("T"), name)
    }

    /// One Subsumption edge `source → target`.
    fn sub_edge(source: &ConceptRef, target: &ConceptRef) -> RuntimeEdge {
        RuntimeEdge {
            kind: subsumption_kind(),
            source: source.clone(),
            target: target.clone(),
        }
    }

    #[test]
    fn union_merges_generators_and_recomputes_across_the_seam() {
        // The N-ontology composite hook (no production caller yet). Two closures
        // whose generators only compose ACROSS the merge seam: A → B and B → C.
        // Neither alone reaches A ⇝ C; the union must.
        let kind = subsumption_kind();
        let mut transitive = BTreeSet::new();
        transitive.insert(kind.clone());
        let (a, b, cc) = (tref("A"), tref("B"), tref("C"));

        let mut left = MaterializedClosure::fold(&[sub_edge(&a, &b)], &transitive);
        let right = MaterializedClosure::fold(&[sub_edge(&b, &cc)], &transitive);

        // Warm `left`'s memo for A BEFORE the union, so we exercise the memo
        // invalidation: pre-union, A reaches only B.
        assert!(left.reaches(&a, &b, kind.clone()));
        assert!(!left.reaches(&a, &cc, kind.clone()));

        left.union(&right);

        // Post-union the merged generators saturate across the seam: A ⇝ B ⇝ C.
        // (A stale memo — not invalidated by `union` — would still answer "A
        // reaches only B" here, so this pins the invalidation.)
        assert!(
            left.reaches(&a, &cc, kind.clone()),
            "union must merge generators so A transitively reaches C"
        );
        let img = left.image(&a, &kind);
        // Shortest-path grading survives the merge: A → B one hop, A → C two.
        assert_eq!(img.iter().find(|(v, _)| v == &b).map(|(_, d)| *d), Some(1));
        assert_eq!(img.iter().find(|(v, _)| v == &cc).map(|(_, d)| *d), Some(2));
    }

    #[test]
    fn lazy_matches_eager_on_a_diamond_and_a_cycle() {
        // The two cases the synthetic memory probe never exercises — a
        // multi-parent diamond and a directed cycle — pinned against the eager
        // Floyd–Warshall engine directly, so lazy/eager equivalence is proven
        // exactly where a hand-rolled BFS is most likely to diverge.
        use pr4xis::category::quiver::ReachabilityClosure;
        let kind = subsumption_kind();
        let mut transitive = BTreeSet::new();
        transitive.insert(kind.clone());

        let (a, b, cc, d, x, y, z) = (
            tref("A"),
            tref("B"),
            tref("C"),
            tref("D"),
            tref("X"),
            tref("Y"),
            tref("Z"),
        );
        // Diamond: D → B, D → C, B → A, C → A (A reachable two ways, distance 2).
        // Cycle: X → Y → Z → X (must terminate via the `seen` guard).
        let raw = [
            (d.clone(), b.clone()),
            (d.clone(), cc.clone()),
            (b.clone(), a.clone()),
            (cc.clone(), a.clone()),
            (x.clone(), y.clone()),
            (y.clone(), z.clone()),
            (z.clone(), x.clone()),
        ];
        let edges: Vec<RuntimeEdge> = raw.iter().map(|(s, t)| sub_edge(s, t)).collect();

        let lazy = MaterializedClosure::fold(&edges, &transitive);
        let eager = ReachabilityClosure::fold(raw.iter().cloned());

        // Every vertex's strict image — set AND shortest-path distance — matches
        // the eager engine's.
        for v in [&a, &b, &cc, &d, &x, &y, &z] {
            let lazy_img: BTreeMap<ConceptRef, u32> = lazy.image(v, &kind).into_iter().collect();
            let eager_img: BTreeMap<ConceptRef, u32> = eager.strict_image(v).into_iter().collect();
            assert_eq!(lazy_img, eager_img, "lazy/eager images diverge at {v:?}");
        }

        // The diamond's shared descendant A is reached from D at distance 2 (via
        // either parent), and the cycle walk terminated (no divergence above).
        assert_eq!(
            lazy.image(&d, &kind)
                .into_iter()
                .find(|(v, _)| v == &a)
                .map(|(_, hops)| hops),
            Some(2)
        );
    }
}
