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
//! form. (The image-returning methods enumerate in the shared kernel's
//! CANONICAL `(hops, ConceptRef::Ord)` order — see
//! [`pr4xis::category::reach`], the one graded-reach kernel this engine
//! delegates its walks to — not the eager `HashMap`'s arbitrary order:
//! `chain` sorts by the same contract, `meet` argmins by it, `reachable_from`
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

use alloc::collections::{BTreeMap, BTreeSet};

use pr4xis::category::Concept;
use pr4xis::category::reach::{Cached, ReachSubstrate, ReachView};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use std::sync::OnceLock;

use crate::address::ContentAddress;
use crate::apply::apply;
use crate::archive::Archive;
use crate::codec::CodecError;
use crate::connection::GeneratorAction;
use crate::definition::Definition;
use crate::definition::EdgeTarget;
use crate::lens::archive_lens::{
    ArchiveLens, ArchiveLensError, ArchivedArchiveView, ArchivedDefinitionView,
    archived_definition_to_owned, archived_grounded, archived_local_name,
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
/// in the loaded "Relations" ontology (self-aware-architecture design note §11).
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
    /// The connected ontology the target atom lives in — the typed in-memory
    /// [`OntologyName`], LIFTED once here from the archived wire form (the
    /// `EdgeTarget::Grounded.ontology` wire `String`) exactly as `kind` is lifted
    /// into the Relations vocabulary. It sits beside [`ConceptRef::ontology`] (also
    /// `OntologyName`), so the resolver keys never re-wrap a bare `String`.
    pub ontology: OntologyName,
    /// The content address of the target atom in that ontology.
    pub atom: ContentAddress,
}

/// The reachability answer of a materialized ontology — per transitive
/// relation-kind [`ConceptRef`], the generating adjacency, queried LAZILY
/// through the ONE generic graded-reach engine
/// ([`pr4xis::category::reach`]: the [`ReachSubstrate`] this type implements,
/// the [`Cached`] memo policy, and the per-call [`ReachView`]).
///
/// It holds only the generators (linear in the edge count), NOT the
/// eagerly-saturated `O(V · depth)` transitive-closure set. The
/// reflexive-transitive image of a queried vertex is the free functor
/// `FreeCategory<Q> → ReachCat` (Mac Lane 1971 CWM II.7) restricted to that
/// vertex, computed ON DEMAND by the kernel's bounded, cycle-safe,
/// hop-graded breadth-first walk (Moore 1959) and MEMOIZED per `(kind,
/// source)` by the [`Cached`] policy: the first query for a vertex pays the
/// walk, every later one hits the memo with no re-traversal (a `reaches`
/// membership test scans the cached image allocation-free; an image query
/// copies it out). The vertices are typed `ConceptRef`s (resident once, in
/// the sorted vertex table) so an N-ontology composite can
/// [`union`](Self::union) closures across ontologies. Every query answer is
/// identical to the eagerly-folded form — a representation change
/// (footprint), not a semantics change.
///
/// The memo's `RefCell` makes `MaterializedClosure` — and `RuntimeOntology` —
/// `!Sync`; that is a DELIBERATE INVARIANT of the runtime (chat / wasm are
/// single-threaded, and no `static`/`OnceLock` holds a `RuntimeOntology`). A
/// future threaded native server that shares `&RuntimeOntology` across threads
/// must put the memo behind a `Mutex` (or instantiate the engine with the
/// `Sync` [`Uncached`](pr4xis::category::reach::Uncached) policy), never
/// weaken the invariant silently. The memo is a *derived* view of the
/// adjacency — never part of identity — so equality ignores it and
/// [`union`](Self::union) rebuilds it empty.
///
/// # The representation — one vertex table + per-kind CSR over `u32` ids
///
/// Every distinct [`ConceptRef`] endpoint of the generating edges is stored
/// ONCE, in a table sorted by `ConceptRef::Ord`; a vertex's `u32` id is its
/// table position, so **`u32::Ord` on ids is order-isomorphic to
/// `ConceptRef::Ord` on the vertices they name**. The kernel's determinism
/// contract (`(hops, V::Ord)` — see [`pr4xis::category::reach`]) therefore
/// yields byte-identical answers whether the engine runs over `ConceptRef`
/// vertices or over their ids: mapping the id-graded output back through the
/// table IS the `ConceptRef`-graded output. Each kind's adjacency is a
/// compressed-sparse-row pair (`offsets` + `targets`, both `u32`) over those
/// ids — the [`TaxonomyStore`-style CSR](pr4xis::category::reach) shape,
/// owned and built at [`fold`](Self::fold) — so a vertex name is resident
/// once (the table), not once per edge endpoint as in the former
/// `BTreeMap<ConceptRef, BTreeMap<ConceptRef, Vec<ConceptRef>>>` adjacency.
/// Queries arrive as `ConceptRef`s and are resolved to ids at the public
/// boundary by binary search on the table; a concept absent from the table
/// has no generating edges, and each method answers exactly what the engine
/// answers for an edge-less vertex (empty image / `false` / `None` /
/// singleton reflexive chain).
#[derive(Debug, Clone, Default)]
pub struct MaterializedClosure {
    /// The vertex TABLE: every distinct `ConceptRef` endpoint of the
    /// generating edges, sorted by `ConceptRef::Ord`, resident ONCE. A
    /// vertex's `u32` id is its position here — the id order IS the
    /// `ConceptRef` order (the isomorphism the tie-break pins rely on).
    vertices: Vec<ConceptRef>,
    /// `kind → CSR adjacency over vertex ids`. An entry exists for EVERY
    /// declared transitive kind (an edge-less kind holds the empty CSR) AND
    /// for every NON-transitive kind that actually generated an edge
    /// (Opposition and any other kind outside [`Self::transitive`] — an
    /// edge-less non-transitive kind gets no entry, since there is no
    /// declared-but-empty set to track for it the way there is for
    /// transitive kinds). [`populated_kinds`](Self::populated_kinds) filters
    /// the truly non-empty. Per `(kind, source)` row, targets are deduped and
    /// self-loops dropped (a `source == target` generator carries nothing
    /// beyond the implicit reflexive arrow), matching the former eager
    /// fold's handling.
    adjacency: BTreeMap<ConceptRef, KindCsr>,
    /// The DECLARED transitive-kind vocabulary this closure was folded
    /// against (Subsumption, Parthood, …) — [`reaches`](Self::reaches) /
    /// [`reachable_from`](Self::reachable_from) / [`chain`](Self::chain)
    /// dispatch on membership here: a kind IN this set answers through the
    /// BFS-graded reachability engine (multi-hop, memoized); a kind whose
    /// [`adjacency`](Self::adjacency) entry exists but is NOT in this set
    /// (Opposition, and any other non-transitive relation a functor
    /// relabels edges into) answers through a single DIRECT edge check —
    /// never chained, since "opposite of an opposite" is not a coherent
    /// closure (Casati & Varzi 1999's disjointness reading of Opposition;
    /// contrast [`Parthood`](crate::ontology::opposition_relation_kind),
    /// which IS transitive and so IS in this set). A kind in neither set nor
    /// adjacency has no generating edges at all — honest `false`/empty,
    /// exactly as before this field existed.
    transitive: BTreeSet<ConceptRef>,
    /// The engine's memo — nested per kind (`kind → source id → image`),
    /// storing exactly the kernel's canonical `(hops, u32::Ord)`-ordered
    /// output over vertex ids (≡ `(hops, ConceptRef::Ord)` via the table).
    /// Only ever populated for a TRANSITIVE kind — the direct single-edge
    /// path for a non-transitive kind never touches the memo.
    memo: Cached<ConceptRef, u32>,
}

/// One relation kind's generating adjacency in compressed-sparse-row form over
/// the closure's `u32` vertex ids: vertex `v`'s direct targets are
/// `targets[offsets[v] .. offsets[v + 1]]`, sorted ascending and deduped. An
/// edge-less kind is the empty CSR (both vectors empty); [`Self::targets_of`]
/// answers the empty slice for any row the offsets do not cover.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct KindCsr {
    /// Row offsets into `targets` — `vertex count + 1` entries for a populated
    /// kind, empty for an edge-less kind.
    offsets: Vec<u32>,
    /// Target vertex ids, grouped per source row, each row sorted ascending.
    targets: Vec<u32>,
}

impl KindCsr {
    /// The direct targets of vertex `v` — the CSR row, or the empty slice for
    /// a row outside the offsets (an edge-less kind or an unknown id).
    fn targets_of(&self, v: u32) -> &[u32] {
        let row = v as usize;
        match (self.offsets.get(row), self.offsets.get(row + 1)) {
            (Some(&lo), Some(&hi)) => &self.targets[lo as usize..hi as usize],
            _ => &[],
        }
    }
}

impl PartialEq for MaterializedClosure {
    /// Identity is the per-kind generator SET (the table + CSR are canonical:
    /// sorted, deduped) PLUS which kinds are transitive (it changes how a
    /// query over the SAME edges answers); the memoized images are a derived
    /// cache and are ignored.
    fn eq(&self, other: &Self) -> bool {
        self.vertices == other.vertices
            && self.adjacency == other.adjacency
            && self.transitive == other.transitive
    }
}

impl Eq for MaterializedClosure {}

impl ReachSubstrate for MaterializedClosure {
    type Kind = ConceptRef;
    type Vertex = u32;

    /// The direct generating targets of `vertex` along `kind` — one CSR row
    /// read; empty for an undeclared kind or an edge-less vertex.
    fn neighbors<'s>(
        &'s self,
        kind: &ConceptRef,
        vertex: &u32,
    ) -> impl Iterator<Item = u32> + use<'s> {
        self.adjacency
            .get(kind)
            .map(|csr| csr.targets_of(*vertex))
            .unwrap_or(&[])
            .iter()
            .copied()
    }
}

impl MaterializedClosure {
    /// Mint the generic engine's per-call [`ReachView`] over this substrate,
    /// bound to `kind` — three shared references, free to construct. Minted
    /// PER CALL, never stored: a `ReachView` field would borrow this struct's
    /// own `adjacency` and `memo` (a self-borrow).
    fn view<'s>(&'s self, kind: &'s ConceptRef) -> ReachView<'s, Self, Cached<ConceptRef, u32>> {
        ReachView::new(self, &self.memo, kind)
    }

    /// Resolve a query-boundary `ConceptRef` to its table id — binary search
    /// on the sorted vertex table. `None` means the concept is not an endpoint
    /// of any generating edge (so its image is empty by construction).
    fn id_of(&self, c: &ConceptRef) -> Option<u32> {
        self.vertices.binary_search(c).ok().map(|i| i as u32)
    }

    /// The `ConceptRef` a table id names — the inverse boundary map.
    fn concept_of(&self, id: u32) -> &ConceptRef {
        &self.vertices[id as usize]
    }

    /// Build the vertex table and the per-kind CSR adjacency from an edge
    /// multiset — the shared kernel of [`fold`](Self::fold) and
    /// [`union`](Self::union). `kinds` is the full key set (an edge-less kind
    /// gets the empty CSR); `edges` are `(kind, source, target)` triples whose
    /// kind is in `kinds`. Self-loops are dropped and duplicates deduped here,
    /// once, for both callers.
    fn build(
        kinds: BTreeSet<ConceptRef>,
        edges: &[(&ConceptRef, &ConceptRef, &ConceptRef)],
    ) -> Self {
        // The vertex table: every distinct endpoint of a non-self-loop edge,
        // in ConceptRef::Ord order — so table position (the u32 id) is
        // order-isomorphic to ConceptRef::Ord.
        let mut vertex_set: BTreeSet<&ConceptRef> = BTreeSet::new();
        for &(_, source, target) in edges {
            if source != target {
                vertex_set.insert(source);
                vertex_set.insert(target);
            }
        }
        let vertices: Vec<ConceptRef> = vertex_set.into_iter().cloned().collect();
        // The id space is u32 (the CSR cell width); a loaded corpus is
        // 5-6 orders of magnitude below this bound.
        u32::try_from(vertices.len()).expect("vertex table exceeds the u32 id space");

        // Group the edge id-pairs per kind, then sort + dedup: sorted-by-
        // (source, target) pairs ARE the CSR rows in order, targets sorted
        // within each row.
        let id_of = |c: &ConceptRef| -> u32 {
            vertices
                .binary_search(c)
                .expect("every edge endpoint was just inserted into the table") as u32
        };
        let mut per_kind: BTreeMap<&ConceptRef, Vec<(u32, u32)>> = BTreeMap::new();
        for &(kind, source, target) in edges {
            if source != target {
                per_kind
                    .entry(kind)
                    .or_default()
                    .push((id_of(source), id_of(target)));
            }
        }

        let mut adjacency: BTreeMap<ConceptRef, KindCsr> = kinds
            .into_iter()
            .map(|kind| (kind, KindCsr::default()))
            .collect();
        for (kind, mut pairs) in per_kind {
            pairs.sort_unstable();
            pairs.dedup();
            // Prefix-sum offsets over the sorted pairs; the pair order is the
            // row-grouped target order.
            let mut offsets = alloc::vec![0u32; vertices.len() + 1];
            for &(source, _) in &pairs {
                offsets[source as usize + 1] += 1;
            }
            for i in 1..offsets.len() {
                offsets[i] += offsets[i - 1];
            }
            let targets: Vec<u32> = pairs.into_iter().map(|(_, target)| target).collect();
            *adjacency
                .get_mut(kind)
                .expect("every edge kind is in the closure's declared kind set") =
                KindCsr { offsets, targets };
        }

        Self {
            vertices,
            adjacency,
            // Set by the caller ([`fold`](Self::fold) / [`union`](Self::union))
            // after this returns — `build` itself is transitivity-agnostic, it
            // just lays out whatever `kinds`/`edges` it is given.
            transitive: BTreeSet::new(),
            memo: Cached::default(),
        }
    }

    /// Every generating edge as `(kind, source, target)` references — the
    /// CSR read back through the vertex table, for [`union`](Self::union)'s
    /// rebuild.
    fn edge_refs(&self) -> impl Iterator<Item = (&ConceptRef, &ConceptRef, &ConceptRef)> {
        self.adjacency.iter().flat_map(move |(kind, csr)| {
            (0..self.vertices.len() as u32).flat_map(move |row| {
                csr.targets_of(row).iter().map(move |&target| {
                    (
                        kind,
                        &self.vertices[row as usize],
                        &self.vertices[target as usize],
                    )
                })
            })
        })
    }

    /// Build the per-kind generating adjacency from `edges` — the free-functor
    /// image `FreeCategory<Q> → ReachCat` (Mac Lane 1971 CWM II.7) captured as
    /// its generators, NOT saturated. No transitive closure is computed here;
    /// reachability is evaluated lazily per vertex at query time (see the
    /// struct docs). We keep only the GENERATING edges and never trust a
    /// pre-stored closure: a `.prx` whose edges are already a closure just
    /// supplies redundant generators the BFS ignores (the closure of a closure
    /// is the same closure).
    ///
    /// `transitive` is the LOADED transitive-kind vocabulary (the kinds OWL-RL
    /// marks `Transitive`); one adjacency per kind in it — never a hardcoded
    /// array — plus an entry for every kind (even edge-less), matching the
    /// eager form so `populated_kinds` reports identically for transitive
    /// kinds. An edge whose kind is NOT in `transitive` is not dropped: it
    /// still generates a CSR row (so [`reaches`](Self::reaches) can answer a
    /// DIRECT single-edge check for it — Opposition and any other
    /// non-transitive relation), it is just excluded from the BFS-graded
    /// engine's kind set (a non-transitive kind has no multi-hop closure).
    pub fn fold(edges: &[RuntimeEdge], transitive: &BTreeSet<ConceptRef>) -> Self {
        let triples: Vec<(&ConceptRef, &ConceptRef, &ConceptRef)> = edges
            .iter()
            .map(|e| (&e.kind, &e.source, &e.target))
            .collect();
        let mut kinds = transitive.clone();
        kinds.extend(edges.iter().map(|e| e.kind.clone()));
        let mut closure = Self::build(kinds, &triples);
        closure.transitive = transitive.clone();
        closure
    }

    /// The reachable set from `source` along `kind` — the STRICT reachable set
    /// (descendants of `source`; the reflexive `source → source` arrow is
    /// implicit and not included, matching the prior behavior). For a
    /// TRANSITIVE kind, computed by a bounded, memoized BFS on first ask, an
    /// O(1) cache hit after. For a kind outside `Self::transitive`
    /// (Opposition, …), this is exactly the direct generating edges — one CSR
    /// row read, never chained (see the field doc on `transitive`). Empty set
    /// if `source` has no outgoing edges of `kind`.
    pub fn reachable_from(&self, source: &ConceptRef, kind: ConceptRef) -> BTreeSet<ConceptRef> {
        let Some(csr) = self.adjacency.get(&kind) else {
            return BTreeSet::new();
        };
        let Some(source_id) = self.id_of(source) else {
            return BTreeSet::new();
        };
        if !self.transitive.contains(&kind) {
            return csr
                .targets_of(source_id)
                .iter()
                .map(|&v| self.concept_of(v).clone())
                .collect();
        }
        self.view(&kind)
            .strict_image(&source_id)
            .into_iter()
            .map(|(v, _)| self.concept_of(v).clone())
            .collect()
    }

    /// Does `source` reach `target` along `kind`? For a TRANSITIVE kind,
    /// membership in `source`'s (memoized) strict image (multi-hop BFS).
    /// (Strict reachability: a vertex does not reach itself here, matching
    /// [`reachable_from`](Self::reachable_from); the reflexive `is-a` case is
    /// the caller's `child == ancestor` short-circuit.) For a kind outside
    /// `Self::transitive` (Opposition, …), a SINGLE direct-edge check —
    /// never chained, so a real antonym pair answers true without treating
    /// "opposite of an opposite" as reachable. An endpoint absent from the
    /// vertex table has no generating edges, so the answer is `false` without
    /// a walk — exactly what the walk would say.
    pub fn reaches(&self, source: &ConceptRef, target: &ConceptRef, kind: ConceptRef) -> bool {
        if source == target {
            return false;
        }
        let Some(csr) = self.adjacency.get(&kind) else {
            return false;
        };
        let (Some(source_id), Some(target_id)) = (self.id_of(source), self.id_of(target)) else {
            return false;
        };
        if !self.transitive.contains(&kind) {
            return csr.targets_of(source_id).contains(&target_id);
        }
        self.view(&kind).reaches(&source_id, &target_id)
    }

    /// The relation kinds this ontology actually POPULATES — the keys with at
    /// least one generating edge. A transitive kind with no edges (an entry is
    /// held for EVERY declared transitive kind) is omitted, so this reports what
    /// the ontology can really answer, not what the vocabulary permits. The
    /// data-driven basis for an ontology's CAPABILITIES (doc §4.7): a USC
    /// mereology populates `Parthood`, an OWL vocabulary `Subsumption` — read
    /// off the loaded data, not hardcoded.
    pub fn populated_kinds(&self) -> Vec<ConceptRef> {
        self.adjacency
            .iter()
            .filter(|(_, csr)| !csr.targets.is_empty())
            .map(|(kind, _)| kind.clone())
            .collect()
    }

    /// The strict image of `c` along the relation `kind` — every node reachable
    /// from it under that kind (excluding `c`), each with its minimal hop count.
    /// RELATION-PARAMETRIC: `image(c, Subsumption)` is the hypernym ancestors,
    /// `image(c, Parthood)` the wholes `c` is transitively part of. A bounded,
    /// memoized per-vertex BFS, not a bulk-saturated lookup.
    pub fn image(&self, c: &ConceptRef, kind: &ConceptRef) -> Vec<(ConceptRef, u32)> {
        // A multi-hop "image with hop count" has no coherent meaning for a
        // kind outside `transitive` (Opposition, …) — see the field doc on
        // `transitive` and `reaches`'s single-edge branch. Honest empty,
        // exactly the pre-existing answer for an undeclared kind.
        if !self.adjacency.contains_key(kind) || !self.transitive.contains(kind) {
            return Vec::new();
        }
        let Some(c_id) = self.id_of(c) else {
            return Vec::new();
        };
        // The kernel's canonical (hops, u32::Ord) order maps back through the
        // table to (hops, ConceptRef::Ord) — the id order IS the vertex order.
        self.view(kind)
            .strict_image(&c_id)
            .into_iter()
            .map(|(v, hops)| (self.concept_of(v).clone(), hops))
            .collect()
    }

    /// The Subsumption (hypernym) image of `c` — `image(c, Subsumption)`.
    /// The loaded-ontology analogue of `English::ancestors`.
    pub fn subsumption_image(&self, c: &ConceptRef) -> Vec<(ConceptRef, u32)> {
        self.image(c, &subsumption_kind())
    }

    /// The lattice MEET of `a` and `b` over the relation `kind` — the nearest
    /// node both reach (`strict_image(b) ∩ reflexive_image(a)`, nearest-first),
    /// ties broken by `ConceptRef`'s derived `(ontology, name)` order — the
    /// shared kernel's `(hops, V::Ord)` contract, applied over this engine's
    /// MEMOIZED images so the meet keeps hitting (and warming) the memo.
    /// RELATION-PARAMETRIC: the nearest common hypernym for Subsumption, the
    /// nearest common whole for Parthood.
    pub fn meet(&self, a: &ConceptRef, b: &ConceptRef, kind: &ConceptRef) -> Option<ConceptRef> {
        // "Nearest shared ancestor" has no coherent meaning for a kind
        // outside `transitive` (Opposition, …) — see the field doc on
        // `transitive`. Honest `None`, exactly the pre-existing answer for
        // an undeclared kind.
        if !self.adjacency.contains_key(kind) || !self.transitive.contains(kind) {
            return None;
        }
        // An endpoint absent from the table has no edges: `b`'s strict image
        // is empty, and an edge-less `a` can never appear in a strict image —
        // the meet is None either way, exactly as the walk would answer.
        let (Some(a_id), Some(b_id)) = (self.id_of(a), self.id_of(b)) else {
            return None;
        };
        self.view(kind)
            .meet(&a_id, &b_id)
            .map(|v| self.concept_of(v).clone())
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
        // An ordered multi-hop path has no coherent meaning for a kind
        // outside `transitive` (Opposition, …) — see the field doc on
        // `transitive`. Honest `None`, exactly the pre-existing answer for
        // an undeclared kind.
        if !self.adjacency.contains_key(kind) || !self.transitive.contains(kind) {
            return None;
        }
        // The engine's chain: reflexive ancestors of `child` that still reach
        // `ancestor` lie on a child⇝ancestor path, in the kernel's canonical
        // `(hops, ConceptRef::Ord)` order, evaluated through the memo.
        match (self.id_of(child), self.id_of(ancestor)) {
            (Some(child_id), Some(ancestor_id)) => {
                self.view(kind).chain(&child_id, &ancestor_id).map(|ids| {
                    ids.into_iter()
                        .map(|v| self.concept_of(v).clone())
                        .collect()
                })
            }
            // An edge-less vertex reflexively reaches itself and nothing else:
            // the kernel's chain(c, c) over the empty adjacency is the
            // singleton — preserved at the boundary.
            _ if child == ancestor => Some(alloc::vec![child.clone()]),
            // Otherwise one endpoint has no edges, so child never reaches
            // ancestor — the kernel's honest None.
            _ => None,
        }
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
    /// N-ontology composite. The edge MULTISETS are merged and the vertex
    /// table + per-kind CSR REBUILT over the union (the union of two generator
    /// sets generates the union reachability); the kind key set is the union
    /// of both (an edge-less declared kind stays declared). Union is an
    /// install-time operation, so the rebuild cost is paid once, off the query
    /// path. The rebuilt memo starts empty — the memo is a DERIVED view of the
    /// adjacency and a stale image would answer the pre-union graph.
    /// (Single-ontology callers never need it; it exists so the closure is
    /// N-ready by construction.)
    pub fn union(&mut self, other: &MaterializedClosure) {
        let kinds: BTreeSet<ConceptRef> = self
            .adjacency
            .keys()
            .chain(other.adjacency.keys())
            .cloned()
            .collect();
        let triples: Vec<(&ConceptRef, &ConceptRef, &ConceptRef)> =
            self.edge_refs().chain(other.edge_refs()).collect();
        let mut rebuilt = Self::build(kinds, &triples);
        // The transitive-kind vocabulary is a GLOBAL authority
        // (`declared_transitive_kinds`), so both sides agree in practice;
        // unioned defensively rather than assumed, so a kind either side
        // declares transitive stays BFS-graded after the merge.
        rebuilt.transitive = self.transitive.union(&other.transitive).cloned().collect();
        *self = rebuilt;
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

/// The archive-node indices SORTED BY NODE NAME (ties by archive order) — the
/// by-name lookup index built once at materialize (see
/// [`RuntimeOntology::node_index`]). A lookup is a binary search whose name
/// comparisons read the ARCHIVED node names zero-copy out of the retained
/// buffer, so no node name is duplicated into an owned key (the former
/// `BTreeMap<String, Vec<usize>>` re-owned every name); duplicate-named nodes
/// occupy adjacent positions in archive order, reproducing the all-matches
/// behaviour of the full scan this index replaced.
type NodeIndex = Vec<u32>;

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
/// `Definition`s; it is held as its `rkyv` local-cache bytes
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
    /// The archive-node indices sorted by node name, built ONCE at materialize
    /// so a by-name node lookup is an O(log N) binary search — over the
    /// ARCHIVED names in `buf`, zero-copy, no owned key per name — rather than
    /// the former O(N) scan of every node. Duplicate-named nodes (admitted by
    /// the archive layer; they collapse only in the Merkle root) sort adjacent
    /// with archive order as the tie-break, preserving the exact all-matches
    /// semantics of the scan this replaces. Indices are into
    /// [`archive`](Self::archive)`().nodes`, whose order `rkyv` preserves
    /// verbatim from the owned [`Archive`]. Derived, not identity: like `buf`
    /// it is re-derivable from the archive and takes no part in `PartialEq`.
    node_index: NodeIndex,
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

    /// The archive nodes named `name`, in archive order — the O(log N) by-name
    /// node lookup every by-name query surface shares ([`morphisms_from`](Self::morphisms_from),
    /// [`grounded_edges_from`](Self::grounded_edges_from), [`lexical`](Self::lexical)),
    /// replacing their former `nodes.iter().filter(|n| n.name == …)` full scan.
    /// It yields EVERY node carrying the name (the archive layer admits duplicates),
    /// so a `filter` caller iterates all and a `find` caller takes the first — the
    /// same nodes, in the same order, the scan visited. Empty when the name is
    /// absent.
    fn nodes_named<'s>(
        &'s self,
        name: &str,
    ) -> impl Iterator<Item = &'s ArchivedDefinitionView> + 's {
        let nodes = &self.archive().nodes;
        // Binary-search the name-sorted index; the comparisons read the
        // archived names zero-copy. Duplicates sit adjacent (archive order),
        // so the matching run is `[start, end)`.
        let start = self
            .node_index
            .partition_point(|&i| nodes[i as usize].name.as_str() < name);
        let end = start
            + self.node_index[start..]
                .partition_point(|&i| nodes[i as usize].name.as_str() == name);
        self.node_index[start..end]
            .iter()
            .map(move |&i| &nodes[i as usize])
    }

    /// Every outgoing GENERATING edge from `c` — the morphisms departing this
    /// concept, as typed [`RuntimeEdge`]s. (Generating edges, not the closure;
    /// the closure is served by [`reachable_from`](Self::reachable_from).)
    pub fn morphisms_from(&self, c: &ConceptRef) -> Vec<RuntimeEdge> {
        self.nodes_named(c.name.as_str())
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
        self.nodes_named(c.name.as_str())
            .flat_map(|n| {
                n.edges.iter().filter_map(move |edge| {
                    let (kind_name, target) = (&edge.0, &edge.1);
                    let (ontology, atom) = archived_grounded(target)?;
                    Some(GroundedEdge {
                        kind: relations_kind(kind_name.as_str()),
                        // LIFT the archived wire `String` to the typed in-memory
                        // OntologyName — the single lowering boundary (see the
                        // `GroundedEdge.ontology` and `EdgeTarget::Grounded` docs).
                        ontology: OntologyName::new(ontology.to_string()),
                        atom,
                    })
                })
            })
            .collect()
    }

    /// The FIRST archive node named `name`, rebuilt as ONE owned
    /// [`Definition`] (`None` when the name is absent) — the per-node owned
    /// decode a grounding resolver uses to derive a single declared target's
    /// content address WITHOUT [`to_owned_archive`](Self::to_owned_archive)'s
    /// whole-graph decode. "First" is archive order, exactly the node the
    /// former whole-archive scan (`nodes.iter().find(|n| n.name == …)`)
    /// yielded, served by the O(log N) name index. Fail-closed on the
    /// (defensively-unreachable) decode fault of a validated buffer.
    pub fn node_by_name(&self, name: &str) -> Option<Result<Definition, ArchiveLensError>> {
        self.nodes_named(name)
            .next()
            .map(archived_definition_to_owned)
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
    /// if the declaring [`Definition`] carries
    /// one.
    pub fn lexical(&self, c: &ConceptRef) -> Option<&str> {
        self.nodes_named(c.name.as_str())
            .next()
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
    // generating adjacency AND the name→node index over the OWNED archive (the
    // root needs `Definition` addressing, which the archived view does not carry).
    let (root, closure, node_index) = analyze(&archive, &id)?;

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
        node_index,
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
    // Owning decode only to derive root + referential closure + generators +
    // the name→node index.
    let archive = ArchiveLens::get(buf.as_slice()).map_err(MaterializeError::Archive)?;
    let (root, closure, node_index) = analyze(&archive, &id)?;
    Ok(RuntimeOntology {
        id,
        root,
        buf,
        closure,
        node_index,
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
/// 5. Capture the name→node index (the archive-node indices sorted by node
///    name), so a by-name query is an O(log N) binary search over the archived
///    names, not an O(N) scan.
fn analyze(
    archive: &Archive,
    id: &OntologyName,
) -> Result<(ContentAddress, MaterializedClosure, NodeIndex), MaterializeError> {
    // 1. Capture the content-address identity up front.
    let root = archive.root().map_err(MaterializeError::Root)?;

    // The declared node names — the referential universe.
    let declared: BTreeSet<&str> = archive.nodes.iter().map(|n| n.name.as_str()).collect();

    // 2 + 3. Build the generating edges over ConceptRef, validating that every
    // endpoint is a declared node — referential closure — as we go.
    let mut edges: Vec<RuntimeEdge> = Vec::new();
    for node in archive.nodes.iter() {
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
    let closure = MaterializedClosure::fold(&edges, declared_transitive_kinds());

    // 5. The name→node index: archive-node indices sorted by node name (ties by
    // archive order, so duplicate-named nodes are visited exactly as the full
    // scan visited them). Lookups binary-search this against the ARCHIVED names
    // — the owned names here and the archived names in the retained buffer are
    // byte-identical (`rkyv` preserves them verbatim), so the sort order agrees.
    let mut node_index: NodeIndex = (0..archive.nodes.len())
        .map(|i| u32::try_from(i).expect("archive node count exceeds the u32 index space"))
        .collect();
    node_index.sort_unstable_by(|&a, &b| {
        archive.nodes[a as usize]
            .name
            .cmp(&archive.nodes[b as usize].name)
            .then(a.cmp(&b))
    });

    Ok((root, closure, node_index))
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
/// closure over — DERIVED once, at first use, from the loaded `morphism_kinds.prx`
/// (the `domains` Relations ontology, which the kernel already embeds + loads for
/// [`default_kind_vocab`](crate::recursive_address::load_relation_kinds)). A kind
/// is transitive iff Relations asserts `Transitive(R)` — the
/// `("HasProperty", Local("Transitive"))` edge, OWL `TransitiveProperty` — read
/// straight off the decoded archive, never a restated allowlist. This is exactly
/// [`transitive_kinds`]'s policy applied to the loaded authority; the two agree by
/// construction (same `.prx`, same predicate), which the `domains` drift test pins.
///
/// # Why read the OWNED archive, not [`materialize`] it
///
/// [`load_relation_kinds`](crate::recursive_address::load_relation_kinds) returns
/// the *decoded* archive with NO closure fold; reading it directly avoids
/// re-entering this `OnceLock` — materializing Relations would fold the closure at
/// [`materialize`]'s step 4, which calls back here. The closure over Relations' own
/// edges is not needed to enumerate its transitive kinds; only the raw
/// `HasProperty` edges are.
///
/// # The remaining compile-time half
///
/// The `ontology!` macro (`pr4xis-derive`) needs the same set at EXPANSION time to
/// generate same-kind transitive-composition arms, and a proc-macro can neither do
/// IO nor pull in the `.prx` decoder without adding a codec dependency to every
/// downstream compile — so it keeps one drift-guarded distilled projection
/// (`relations_transitive_kinds.txt`, the single sanctioned proc-macro exception).
/// The runtime no longer restates it.
fn declared_transitive_kinds() -> &'static BTreeSet<ConceptRef> {
    static TRANSITIVE_KINDS: OnceLock<BTreeSet<ConceptRef>> = OnceLock::new();
    TRANSITIVE_KINDS.get_or_init(|| {
        crate::recursive_address::load_relation_kinds()
            .nodes
            .iter()
            .filter(|node| {
                node.edges.iter().any(|(rel, target)| {
                    rel.as_str() == HAS_PROPERTY_REL
                        && matches!(target, EdgeTarget::Local(name) if name.as_str() == TRANSITIVE_CONCEPT)
                })
            })
            .map(|node| relations_kind(node.name.clone()))
            .collect()
    })
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
    #[pr4xis::praxis_value(Deterministic)]
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

    #[pr4xis::praxis_value(Verifiable)]
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

    #[pr4xis::praxis_value(Explainable, Verifiable)]
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

    #[pr4xis::praxis_value(Deterministic)]
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

    #[pr4xis::praxis_value(Deterministic)]
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

    #[pr4xis::praxis_value(Honest)]
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

    #[pr4xis::praxis_value(Honest, Explainable)]
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

    #[pr4xis::praxis_value(Verifiable)]
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

    #[pr4xis::praxis_value(Verifiable, Honest)]
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

    #[pr4xis::praxis_value(Deterministic)]
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
            declared_transitive_kinds(),
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

    #[pr4xis::praxis_value(Verifiable)]
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

    #[pr4xis::praxis_value(Deterministic)]
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

    /// The FIRST pin of the runtime tie-break (formerly doc-stated with ZERO
    /// tests): over a DAG diamond with two equal-distance ancestors, `meet` and
    /// `chain` break the distance tie by `ConceptRef`'s derived `(ontology,
    /// name)` order — the kernel's `(hops, V::Ord)` contract — NOT by edge
    /// (BFS discovery) order. The edges deliberately declare `Zed` before
    /// `Alpha`, so a discovery-order tie-break would answer `Zed`.
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn meet_and_chain_break_equal_distance_dag_ties_by_concept_ref_order() {
        let kind = subsumption_kind();
        let mut transitive = BTreeSet::new();
        transitive.insert(kind.clone());

        let (x, y, zed, alpha, root) = (
            tref("X"),
            tref("Y"),
            tref("Zed"),
            tref("Alpha"),
            tref("Root"),
        );
        // X and Y are both children of Zed AND Alpha (the tie pair, Zed-edges
        // first); both mids reach Root.
        let edges = [
            sub_edge(&x, &zed),
            sub_edge(&x, &alpha),
            sub_edge(&y, &zed),
            sub_edge(&y, &alpha),
            sub_edge(&zed, &root),
            sub_edge(&alpha, &root),
        ];
        let closure = MaterializedClosure::fold(&edges, &transitive);

        // meet(X, Y): Zed and Alpha are both common ancestors at distance 1
        // from Y — the tie. ConceptRef::Ord ranks "Alpha" < "Zed".
        assert_eq!(
            closure.meet(&x, &y, &kind),
            Some(alpha.clone()),
            "the equal-distance meet tie must go to the ConceptRef::Ord-minimal ancestor"
        );

        // chain(X, Root): the tied mids order as [Alpha, Zed] within their hop
        // level — (dist, ConceptRef::Ord), never declaration order.
        assert_eq!(
            closure.chain(&x, &root, &kind),
            Some(alloc::vec![
                x.clone(),
                alpha.clone(),
                zed.clone(),
                root.clone()
            ]),
            "the chain's equal-distance members must order by ConceptRef::Ord"
        );
    }

    /// The tie-break's FIELD ORDER pin: `ConceptRef`'s derived `Ord` compares
    /// `ontology` BEFORE `name`, so an equal-distance tie between ancestors in
    /// two ontologies goes to the smaller ONTOLOGY even when its concept NAME
    /// is larger. (Byte-identical to the former ad-hoc
    /// `(ontology.as_str(), name.clone())` key this replaces.)
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn meet_tie_orders_by_ontology_before_name() {
        let kind = subsumption_kind();
        let mut transitive = BTreeSet::new();
        transitive.insert(kind.clone());

        let x = tref("X");
        let y = tref("Y");
        // Ontology "A" carries the LARGER name, ontology "B" the smaller —
        // ontology-first ordering picks A:zz; name-first would pick B:aa.
        let a_zz = ConceptRef::new(OntologyName::new_static("A"), "zz");
        let b_aa = ConceptRef::new(OntologyName::new_static("B"), "aa");
        let edges = [
            sub_edge(&x, &b_aa),
            sub_edge(&x, &a_zz),
            sub_edge(&y, &b_aa),
            sub_edge(&y, &a_zz),
        ];
        let closure = MaterializedClosure::fold(&edges, &transitive);
        assert_eq!(
            closure.meet(&x, &y, &kind),
            Some(a_zz),
            "ConceptRef::Ord is ontology-then-name; the ontology component decides first"
        );
    }
}
