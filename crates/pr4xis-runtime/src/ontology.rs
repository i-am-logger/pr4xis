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
//! # Materialization IS the free-functor image (not a query-time BFS)
//!
//! The [`MaterializedClosure`] is computed ONCE, at [`materialize`] time, by
//! re-folding the archive's *generating* edges per transitive [`RelationKind`]
//! into their transitive-closure set. This re-fold is the runtime analogue of
//! the `ontology!` macro's compile-time Floyd-Warshall
//! (`pr4xis-derive/ontology.rs`), and categorically it is the image of the free
//! functor `FreeCategory<Q> → ReachCat` into the thin reachability category —
//! exactly the `Collapse` / `ReachCat` reflection in
//! [`quiver`](pr4xis::category::quiver) (CWM II.7): every generating path
//! collapses to its `(source, target)` reachability arrow. We **always** re-fold
//! from the generators and never trust a stored closure (a `.prx`'s edges may
//! themselves be a closure, but the materialize step does not depend on that —
//! the closure of a closure is the same closure, so the re-fold is correct
//! either way).
//!
//! Materialization is *not* query. Once the closure is folded, every query is an
//! O(1) relational-image LOOKUP into the pre-folded `BTreeSet` — never a
//! per-query traversal. A query-time BFS would be the forbidden mechanical
//! anti-pattern; reachability here is the *image* of the materialized functor.
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
use pr4xis::category::quiver::ReachabilityClosure;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use crate::address::ContentAddress;
use crate::archive::Archive;
use crate::codec::CodecError;
use crate::definition::EdgeTarget;

extern crate alloc;
#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// The canonically transitive relation kinds — the relations whose closure is
/// taken (OBO-RO `transitive_over`, Smith et al. 2005): subsumption (OWL
/// subClassOf), parthood (Casati & Varzi 1999), causation (Lewis 1973
/// counterfactual chains). These are exactly the kinds the `ontology!` macro
/// (`pr4xis-derive/ontology.rs`) folds over — the same authoritative
/// reading, so the runtime closure matches the compile-time one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationKind {
    /// `is-a` — OWL `subClassOf`; Guarino (2009). Transitive.
    Subsumption,
    /// `part-of` — Casati & Varzi (1999) *Parts and Places*. Transitive.
    Parthood,
    /// `causes` — Lewis (1973) *Causation*; counterfactual chains. Transitive.
    Causation,
}

impl RelationKind {
    /// The canonical relation-kind name, as it appears in a `.prx`
    /// [`Definition::edges`](crate::definition::Definition::edges) — the same
    /// identifier `emit` writes via `format!("{:?}", kind)` and the macro emits.
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationKind::Subsumption => "Subsumption",
            RelationKind::Parthood => "Parthood",
            RelationKind::Causation => "Causation",
        }
    }

    /// Parse a relation-kind name into its transitive [`RelationKind`], or
    /// `None` if the name is not one of the canonically transitive kinds
    /// (e.g. `Opposition` / `Equivalence`, which are symmetric, not transitive).
    pub fn from_edge_kind(name: &str) -> Option<Self> {
        match name {
            "Subsumption" => Some(RelationKind::Subsumption),
            "Parthood" => Some(RelationKind::Parthood),
            "Causation" => Some(RelationKind::Causation),
            _ => None,
        }
    }

    /// Every transitive kind whose closure the materializer folds.
    pub fn transitive() -> [RelationKind; 3] {
        [
            RelationKind::Subsumption,
            RelationKind::Parthood,
            RelationKind::Causation,
        ]
    }
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
/// morphism as data. Carries its [`RelationKind`] so the closure can be folded
/// per kind.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeEdge {
    pub source: ConceptRef,
    pub kind: RelationKind,
    pub target: ConceptRef,
}

/// The materialized transitive closure — per transitive [`RelationKind`], the
/// set of `(ConceptRef → ConceptRef)` pairs reachable along that kind's
/// generating edges.
///
/// Computed ONCE at [`materialize`] time by re-folding the generators (never a
/// stored closure). Keyed by `ConceptRef` so an N-ontology composite can union
/// these maps. Every query is an O(1) relational-image lookup into `reachable`
/// — never a traversal.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaterializedClosure {
    /// `kind → ReachabilityClosure` — one shared
    /// [`ReachabilityClosure`]
    /// per transitive relation kind. The fold, the fixpoint, and the
    /// O(1)-lookup invariant all live in that one shared construct (the same one
    /// the English hypernym closure uses); this type only partitions it by kind
    /// and re-exposes the reachable SET (for the existing `BTreeSet`-returning
    /// query surface).
    reachable: BTreeMap<RelationKind, ReachabilityClosure<ConceptRef>>,
}

impl MaterializedClosure {
    /// Re-fold the closure from `edges` — the categorical free-functor image
    /// into the thin reachability category (Mac Lane 1971 CWM II.7: the unique
    /// functor `FreeCategory<Q> → ReachCat` that collapses each generating path
    /// to its `(source, target)` reachability arrow). Runtime analogue of the
    /// `ontology!` macro's compile-time Floyd-Warshall.
    ///
    /// This is **materialization**, not query: the transitive closure is
    /// saturated here, once, so every later query is an O(1) lookup. We always
    /// fold from the GENERATING edges and never trust a pre-stored closure. The
    /// per-kind fold delegates to the shared
    /// [`ReachabilityClosure`] —
    /// no bespoke fixpoint loop lives here.
    pub fn fold(edges: &[RuntimeEdge]) -> Self {
        let mut reachable: BTreeMap<RelationKind, ReachabilityClosure<ConceptRef>> =
            BTreeMap::new();
        for kind in RelationKind::transitive() {
            let closure = ReachabilityClosure::fold(
                edges
                    .iter()
                    .filter(|e| e.kind == kind)
                    .map(|e| (e.source.clone(), e.target.clone())),
            );
            reachable.insert(kind, closure);
        }
        Self { reachable }
    }

    /// The pre-folded reachable set from `source` along `kind` — an O(1)
    /// relational-image lookup, never a traversal. Empty set if `source` has no
    /// outgoing edges of `kind`. This is the STRICT reachable set (descendants
    /// of `source`; the reflexive `source → source` arrow is implicit in the
    /// shared closure and is not included here, matching the prior behavior).
    pub fn reachable_from(&self, source: &ConceptRef, kind: RelationKind) -> BTreeSet<ConceptRef> {
        self.reachable
            .get(&kind)
            .map(|closure| {
                closure
                    .strict_image(source)
                    .into_iter()
                    .map(|(v, _)| v)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Does `source` reach `target` along `kind`? — a direct O(1) membership
    /// query into the shared closure, never materializing the intermediate set.
    /// (Strict reachability: a vertex does not reach itself along the closure
    /// here, matching [`reachable_from`](Self::reachable_from); the reflexive
    /// `is-a` case is the caller's `child == ancestor` short-circuit.)
    pub fn reaches(&self, source: &ConceptRef, target: &ConceptRef, kind: RelationKind) -> bool {
        self.reachable
            .get(&kind)
            .is_some_and(|closure| source != target && closure.reaches(source, target))
    }

    /// The reflexive Subsumption (hypernym) image of `c` — `c` itself plus every
    /// ancestor reachable up the is-a closure, each with its minimal is-a
    /// distance. A lookup over the materialized set; empty (apart from `c`) when
    /// `c` has no Subsumption ancestors. This is the loaded-ontology analogue of
    /// `English::ancestors`, sharing the same
    /// [`ReachabilityClosure`].
    pub fn subsumption_image(&self, c: &ConceptRef) -> Vec<(ConceptRef, u32)> {
        self.reachable
            .get(&RelationKind::Subsumption)
            .map(|closure| closure.strict_image(c))
            .unwrap_or_default()
    }

    /// The lattice MEET of `a` and `b` over the Subsumption closure — the nearest
    /// shared hypernym (`strict_ancestors(b) ∩ ancestors(a)`, nearest-first),
    /// ties broken by `(ontology, name)`. The categorical meet over the
    /// materialized set, never a hand-BFS.
    pub fn subsumption_meet(&self, a: &ConceptRef, b: &ConceptRef) -> Option<ConceptRef> {
        self.reachable
            .get(&RelationKind::Subsumption)
            .and_then(|closure| {
                closure.meet_by(a, b, |c| (c.ontology.as_str().to_string(), c.name.clone()))
            })
    }

    /// The ordered hypernym chain `[child, …, ancestor]` (nearest-first) over the
    /// Subsumption closure when `child` is-a `ancestor`, else `None` — the is-a
    /// evidence path, read off the materialized closure rather than hand-walked.
    pub fn subsumption_chain(
        &self,
        child: &ConceptRef,
        ancestor: &ConceptRef,
    ) -> Option<Vec<ConceptRef>> {
        let closure = self.reachable.get(&RelationKind::Subsumption)?;
        if child != ancestor && !closure.reaches(child, ancestor) {
            return None;
        }
        // Reflexive ancestors of `child` that still reach `ancestor` lie on a
        // child⇝ancestor path; order them nearest-first by is-a distance.
        let mut chain: Vec<(ConceptRef, u32)> = closure
            .reflexive_image(child)
            .into_iter()
            .filter(|(x, _)| x == ancestor || closure.reaches(x, ancestor))
            .collect();
        chain.sort_unstable_by(|(a, da), (b, db)| {
            da.cmp(db)
                .then_with(|| a.ontology.as_str().cmp(b.ontology.as_str()))
                .then_with(|| a.name.cmp(&b.name))
        });
        Some(chain.into_iter().map(|(v, _)| v).collect())
    }

    /// Union another closure into this one — the structural hook for an
    /// N-ontology composite. The union of two reachability sets is itself a
    /// reachability relation; we re-fold the combined generating image so the
    /// result stays a valid materialized closure (with correct shortest-path
    /// grading) rather than a hand-merged set. (Single-ontology callers never
    /// need it; it exists so the closure is N-ready by construction.)
    pub fn union(&mut self, other: &MaterializedClosure) {
        for (kind, other_closure) in &other.reachable {
            let into = self.reachable.entry(*kind).or_default();
            // Re-fold from the union of both closures' (source → target) pairs.
            // Folding an already-closed pair set is idempotent, so this recovers
            // the combined closure correctly.
            let merged =
                ReachabilityClosure::fold(into.edges_iter().chain(other_closure.edges_iter()));
            *into = merged;
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
        }
    }
}

impl std::error::Error for MaterializeError {}

/// One loaded `.prx` as a single typed, queryable ontology.
///
/// Identity is the content address: two `RuntimeOntology`s are equal iff their
/// archive roots ([`Archive::root`]) agree. The materialized closure is held
/// alongside the archive (the open form) and the root (the identity).
#[derive(Debug, Clone)]
pub struct RuntimeOntology {
    id: OntologyName,
    root: ContentAddress,
    archive: Archive,
    closure: MaterializedClosure,
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

    /// The open form this ontology was materialized from.
    pub fn archive(&self) -> &Archive {
        &self.archive
    }

    /// The materialized transitive closure.
    pub fn closure(&self) -> &MaterializedClosure {
        &self.closure
    }

    /// A [`ConceptRef`] in this ontology for `name` (does not check the name
    /// exists — use it to build query arguments against a known node).
    pub fn concept(&self, name: impl Into<String>) -> ConceptRef {
        ConceptRef::new(self.id.clone(), name)
    }

    // --- query surface (relational image over the materialized closure) ---

    /// Every outgoing GENERATING edge from `c` — the morphisms departing this
    /// concept, as typed [`RuntimeEdge`]s. (Generating edges, not the closure;
    /// the closure is served by [`reachable_from`](Self::reachable_from).)
    pub fn morphisms_from(&self, c: &ConceptRef) -> Vec<RuntimeEdge> {
        self.archive
            .nodes
            .iter()
            .filter(|n| n.name == c.name)
            .flat_map(|n| {
                n.edges.iter().filter_map(move |(kind_name, target)| {
                    // Only LOCAL edges are morphisms within this ontology; a
                    // Grounded target is a cross-ontology atom, resolved by the
                    // ContainsAtom step, not a generator of this graph.
                    let name = target.local_name()?;
                    RelationKind::from_edge_kind(kind_name).map(|kind| RuntimeEdge {
                        source: c.clone(),
                        kind,
                        target: ConceptRef::new(self.id.clone(), name.to_string()),
                    })
                })
            })
            .collect()
    }

    /// The pre-folded reachable set from `c` along `kind` — an O(1) relational-
    /// image lookup into the materialized closure. Never a query-time BFS.
    pub fn reachable_from(&self, c: &ConceptRef, kind: RelationKind) -> BTreeSet<ConceptRef> {
        self.closure.reachable_from(c, kind)
    }

    /// Is `child` a `ancestor`? — membership of `ancestor` in `child`'s
    /// Subsumption closure. Returns the witnessing [`Verdict`] (never a bool): a
    /// [`Proof`](pr4xis::logic::proof::Proof) carrying the relation when it
    /// holds, a [`Counterexample`](pr4xis::logic::proof::Counterexample) when it
    /// does not. The decision is a closure-membership lookup, not a traversal.
    pub fn is_a(&self, child: &ConceptRef, ancestor: &ConceptRef) -> Verdict {
        let holds = self
            .closure
            .reaches(child, ancestor, RelationKind::Subsumption);
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
        self.archive
            .nodes
            .iter()
            .find(|n| n.name == c.name)
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
/// 4. Re-fold the materialized closure from the generating edges (the free-
///    functor image; never a stored closure).
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
            // Only the canonically transitive kinds participate in the closure;
            // non-transitive kinds (Opposition / Equivalence / custom) are still
            // referentially validated above, but are not folded.
            if let Some(kind) = RelationKind::from_edge_kind(kind_name) {
                edges.push(RuntimeEdge {
                    source: ConceptRef::new(id.clone(), node.name.clone()),
                    kind,
                    target: ConceptRef::new(id.clone(), local.clone()),
                });
            }
        }
    }

    // 4. Re-fold the closure from the generators — the free-functor image.
    let closure = MaterializedClosure::fold(&edges);

    Ok(RuntimeOntology {
        id,
        root,
        archive,
        closure,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::Definition;
    use crate::emit;

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
        // Relational image over the materialized closure — Employer ⊑ Person ⊑
        // Agent collapses to Employer → Agent. O(1) lookup, no traversal.
        let descendants = onto.reachable_from(&employer, RelationKind::Subsumption);
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
        // emit() already emits the materialized closure as edges; re-folding it
        // must yield the same closure (closure of a closure = closure) — the
        // re-fold never trusts, and is correct regardless of, the stored form.
        let onto = org();
        let employer = onto.concept("Employer");
        let direct_then_closed = onto.reachable_from(&employer, RelationKind::Subsumption);
        // Folding the generating edges again over the already-materialized
        // ontology's edges is stable.
        let refolded = MaterializedClosure::fold(
            &onto
                .archive()
                .nodes
                .iter()
                .flat_map(|n| onto.morphisms_from(&onto.concept(n.name.clone())))
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            refolded.reachable_from(&employer, RelationKind::Subsumption),
            direct_then_closed
        );
    }
}
