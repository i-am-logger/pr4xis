//! Kind-parameterised structural axioms — replaces the per-primitive-trait
//! families (`NoCycles<TaxonomyDef>`, `NoCycles<MereologyDef>`, …) with a
//! single family per structural property that filters a `Category`'s
//! morphisms by its typed relation kind.
//!
//! # Why typed, not stringly typed (#163)
//!
//! Each OnKind axiom is parameterised over a `Category` `C` and compares
//! against a typed instance of `<C::Morphism as Arrow>::Kind`. No
//! `kind_name: &'static str`, no separate filter predicate — the kind
//! value itself is the filter. Framework-neutral: each ontology
//! instantiates with its own local kind enum (e.g. `FooRelationKind`),
//! or with an imported shared kind type (e.g. a Relations-ontology
//! concept), without core committing to any specific relations ontology.
//!
//! Per Gruber (1993) *KAS* 5: "ontology = formally-named relations" —
//! typed entities, not string conventions.
//! Per Smith et al. (2005) OBO-RO: every relation is a canonical named
//! type. Encoding that as a compile-time-checked `Kind` value matches
//! the literature.
//!
//! # Rationale (issue #152, #163)
//!
//! The axioms *are properties of relations*, not type-level distinctions.
//! Forcing each axiom into a separate trait family (`TaxonomyDef` vs
//! `MereologyDef`) is a category error — that's #152. Forcing the
//! relation-identity through a `&'static str` parameter is a separate
//! category error — that's #163. Both resolve in this module: one axiom
//! type per structural property, filtered by typed kind value.
//!
//! Sources for the structural properties themselves:
//! - Tarski (1941) *Calculus of Relations* — the axiom names and their
//!   algebraic definitions
//! - Russell & Whitehead *Principia Mathematica* (1910–13) §§30–35 — binary
//!   relations and their structural properties
//! - Smith et al. (2005) OBO Relation Ontology — which properties attach
//!   to which relation types canonically

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::category::{Arrow, Category};
use crate::logic::axiom::Axiom;
use crate::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use crate::ontology::meta::{Citation, Label, OntologyName};

type KindOf<C> = <<C as Category>::Morphism as Arrow>::Kind;

/// Collect (from, to) pairs from the category's morphisms whose kind
/// equals `kind`.
fn kinded_pairs<C>(kind: KindOf<C>) -> Vec<(C::Object, C::Object)>
where
    C: Category,
    C::Object: Clone,
    C::Morphism: Arrow<Object = C::Object>,
    KindOf<C>: PartialEq,
{
    C::morphisms()
        .into_iter()
        .filter(|m| m.kind() == kind)
        .map(|m| (m.source(), m.target()))
        .collect()
}

fn adjacency<E: Clone + Eq + Hash>(pairs: &[(E, E)]) -> HashMap<E, Vec<E>> {
    let mut map: HashMap<E, Vec<E>> = HashMap::new();
    for (from, to) in pairs {
        map.entry(from.clone()).or_default().push(to.clone());
    }
    map
}

fn reachable_from<E: Clone + Eq + Hash>(start: &E, adj: &HashMap<E, Vec<E>>) -> HashSet<E> {
    let mut visited: HashSet<E> = HashSet::new();
    let mut queue: VecDeque<E> = VecDeque::new();
    if let Some(neighbors) = adj.get(start) {
        for n in neighbors {
            if visited.insert(n.clone()) {
                queue.push_back(n.clone());
            }
        }
    }
    while let Some(current) = queue.pop_front() {
        if let Some(neighbors) = adj.get(&current) {
            for n in neighbors {
                if visited.insert(n.clone()) {
                    queue.push_back(n.clone());
                }
            }
        }
    }
    visited
}

/// Build the `name()` override for an OnKind axiom, projecting the typed
/// kind via `Debug` into an identifier like `"NoCyclesOnKind[Subsumption]"`.
/// Kinds are enum-like, so their Debug output is their variant name —
/// exactly the identifier the Lemon registry wants.
fn name_with_kind<K: Debug>(axiom_name: &'static str, kind: &K) -> OntologyName {
    OntologyName::new(format!("{axiom_name}[{kind:?}]"))
}

/// Build the `description()` override for an OnKind axiom.
fn description_with_kind<K: Debug>(axiom_name: &'static str, kind: &K) -> Label {
    Label::new(format!("{axiom_name} applied to edges of kind {kind:?}"))
}

// ---------------------------------------------------------------------------
// NoCyclesOnKind — filter a category's edges by kind; verify the resulting
// graph has no cycles (DAG). Applies canonically to Subsumption and Parthood.
// Source: Guarino (2009); Casati & Varzi (1999); Tarski (1941).
// ---------------------------------------------------------------------------

pub struct NoCyclesOnKind<C: Category>
where
    C::Morphism: Arrow,
{
    kind: KindOf<C>,
    _marker: PhantomData<C>,
}

impl<C: Category> NoCyclesOnKind<C>
where
    C::Morphism: Arrow,
{
    pub fn new(kind: KindOf<C>) -> Self {
        Self {
            kind,
            _marker: PhantomData,
        }
    }
}

impl<C> Axiom for NoCyclesOnKind<C>
where
    C: Category,
    C::Object: Clone + Eq + Hash,
    C::Morphism: Arrow<Object = C::Object>,
    KindOf<C>: PartialEq,
{
    fn verify(&self) -> Verdict {
        let pairs = kinded_pairs::<C>(self.kind);
        let adj = adjacency(&pairs);
        if adj.keys().all(|e| !reachable_from(e, &adj).contains(e)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    fn name(&self) -> OntologyName {
        name_with_kind("NoCyclesOnKind", &self.kind)
    }

    fn description(&self) -> Label {
        description_with_kind("NoCyclesOnKind", &self.kind)
    }

    fn citation(&self) -> Citation {
        Citation::parse_static(
            "Guarino (2009); Casati & Varzi (1999); Tarski (1941) Calculus of Relations",
        )
    }
}

// ---------------------------------------------------------------------------
// AntisymmetricOnKind — if (A, B) is an edge and A ≠ B, then (B, A) is not.
// Applies canonically to Subsumption (if A is-a B then B is not a A).
// Source: Guarino (2009); Tarski (1941); Mac Lane (1971) partial orders.
// ---------------------------------------------------------------------------

pub struct AntisymmetricOnKind<C: Category>
where
    C::Morphism: Arrow,
{
    kind: KindOf<C>,
    _marker: PhantomData<C>,
}

impl<C: Category> AntisymmetricOnKind<C>
where
    C::Morphism: Arrow,
{
    pub fn new(kind: KindOf<C>) -> Self {
        Self {
            kind,
            _marker: PhantomData,
        }
    }
}

impl<C> Axiom for AntisymmetricOnKind<C>
where
    C: Category,
    C::Object: Clone + Eq + Hash,
    C::Morphism: Arrow<Object = C::Object>,
    KindOf<C>: PartialEq,
{
    fn verify(&self) -> Verdict {
        let pairs = kinded_pairs::<C>(self.kind);
        let set: HashSet<(C::Object, C::Object)> = pairs.iter().cloned().collect();
        if pairs
            .iter()
            .all(|(a, b)| a == b || !set.contains(&(b.clone(), a.clone())))
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    fn name(&self) -> OntologyName {
        name_with_kind("AntisymmetricOnKind", &self.kind)
    }

    fn description(&self) -> Label {
        description_with_kind("AntisymmetricOnKind", &self.kind)
    }

    fn citation(&self) -> Citation {
        Citation::parse_static("Guarino (2009); Tarski (1941); Mac Lane (1971) partial orders")
    }
}

// ---------------------------------------------------------------------------
// AsymmetricOnKind — no symmetric pair at all (stronger than antisymmetric:
// also excludes self-loops). Applies canonically to Causation.
// Source: Lewis (1973) Causation; Reichenbach (1956); Tarski (1941).
// ---------------------------------------------------------------------------

pub struct AsymmetricOnKind<C: Category>
where
    C::Morphism: Arrow,
{
    kind: KindOf<C>,
    _marker: PhantomData<C>,
}

impl<C: Category> AsymmetricOnKind<C>
where
    C::Morphism: Arrow,
{
    pub fn new(kind: KindOf<C>) -> Self {
        Self {
            kind,
            _marker: PhantomData,
        }
    }
}

impl<C> Axiom for AsymmetricOnKind<C>
where
    C: Category,
    C::Object: Clone + Eq + Hash,
    C::Morphism: Arrow<Object = C::Object>,
    KindOf<C>: PartialEq,
{
    fn verify(&self) -> Verdict {
        let pairs = kinded_pairs::<C>(self.kind);
        let set: HashSet<(C::Object, C::Object)> = pairs.iter().cloned().collect();
        if pairs
            .iter()
            .all(|(a, b)| a != b && !set.contains(&(b.clone(), a.clone())))
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    fn name(&self) -> OntologyName {
        name_with_kind("AsymmetricOnKind", &self.kind)
    }

    fn description(&self) -> Label {
        description_with_kind("AsymmetricOnKind", &self.kind)
    }

    fn citation(&self) -> Citation {
        Citation::parse_static(
            "Lewis (1973) Causation; Reichenbach (1956) Direction of Time; Tarski (1941)",
        )
    }
}

// ---------------------------------------------------------------------------
// SymmetricOnKind — every edge's reverse is also an edge. Applies canonically
// to Opposition and Equivalence.
// Source: Aristotle *Peri Hermeneias* Square of Opposition; Saussure (1916);
// Cruse (1986) *Lexical Semantics*; Tarski (1941).
// ---------------------------------------------------------------------------

pub struct SymmetricOnKind<C: Category>
where
    C::Morphism: Arrow,
{
    kind: KindOf<C>,
    _marker: PhantomData<C>,
}

impl<C: Category> SymmetricOnKind<C>
where
    C::Morphism: Arrow,
{
    pub fn new(kind: KindOf<C>) -> Self {
        Self {
            kind,
            _marker: PhantomData,
        }
    }
}

impl<C> Axiom for SymmetricOnKind<C>
where
    C: Category,
    C::Object: Clone + Eq + Hash,
    C::Morphism: Arrow<Object = C::Object>,
    KindOf<C>: PartialEq,
{
    fn verify(&self) -> Verdict {
        let pairs = kinded_pairs::<C>(self.kind);
        let set: HashSet<(C::Object, C::Object)> = pairs.iter().cloned().collect();
        if pairs
            .iter()
            .all(|(a, b)| set.contains(&(b.clone(), a.clone())))
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    fn name(&self) -> OntologyName {
        name_with_kind("SymmetricOnKind", &self.kind)
    }

    fn description(&self) -> Label {
        description_with_kind("SymmetricOnKind", &self.kind)
    }

    fn citation(&self) -> Citation {
        Citation::parse_static(
            "Aristotle Peri Hermeneias; Saussure (1916); Cruse (1986) Lexical Semantics; Tarski (1941)",
        )
    }
}

// ---------------------------------------------------------------------------
// IrreflexiveOnKind — no entity is its own image under the relation.
// Applies canonically to Opposition and Causation.
// Source: Aristotle Peri Hermeneias; Lewis (1973); Tarski (1941).
// ---------------------------------------------------------------------------

pub struct IrreflexiveOnKind<C: Category>
where
    C::Morphism: Arrow,
{
    kind: KindOf<C>,
    _marker: PhantomData<C>,
}

impl<C: Category> IrreflexiveOnKind<C>
where
    C::Morphism: Arrow,
{
    pub fn new(kind: KindOf<C>) -> Self {
        Self {
            kind,
            _marker: PhantomData,
        }
    }
}

impl<C> Axiom for IrreflexiveOnKind<C>
where
    C: Category,
    C::Object: Clone + Eq,
    C::Morphism: Arrow<Object = C::Object>,
    KindOf<C>: PartialEq,
{
    fn verify(&self) -> Verdict {
        if C::morphisms()
            .into_iter()
            .filter(|m| m.kind() == self.kind)
            .all(|m| m.source() != m.target())
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    fn name(&self) -> OntologyName {
        name_with_kind("IrreflexiveOnKind", &self.kind)
    }

    fn description(&self) -> Label {
        description_with_kind("IrreflexiveOnKind", &self.kind)
    }

    fn citation(&self) -> Citation {
        Citation::parse_static("Aristotle Peri Hermeneias; Lewis (1973); Tarski (1941)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::Concept;

    // A tiny test category with kinded morphisms.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestObj {
        A,
        B,
        C,
    }

    impl Concept for TestObj {
        fn variants() -> Vec<Self> {
            vec![TestObj::A, TestObj::B, TestObj::C]
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestKind {
        Identity,
        Subsumption,
        Opposition,
        Causation,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestMorph {
        from: TestObj,
        to: TestObj,
        kind: TestKind,
    }

    impl Arrow for TestMorph {
        type Object = TestObj;
        type Kind = TestKind;
        fn source(&self) -> TestObj {
            self.from
        }
        fn target(&self) -> TestObj {
            self.to
        }
        fn kind(&self) -> TestKind {
            self.kind
        }
    }

    struct TestCat;
    impl Category for TestCat {
        type Object = TestObj;
        type Morphism = TestMorph;
        fn identity(obj: &TestObj) -> TestMorph {
            TestMorph {
                from: *obj,
                to: *obj,
                kind: TestKind::Identity,
            }
        }
        fn compose(f: &TestMorph, g: &TestMorph) -> Option<TestMorph> {
            if f.to != g.from {
                return None;
            }
            Some(TestMorph {
                from: f.from,
                to: g.to,
                kind: TestKind::Identity,
            })
        }
        fn morphisms() -> Vec<TestMorph> {
            vec![
                // Identities
                TestMorph {
                    from: TestObj::A,
                    to: TestObj::A,
                    kind: TestKind::Identity,
                },
                TestMorph {
                    from: TestObj::B,
                    to: TestObj::B,
                    kind: TestKind::Identity,
                },
                TestMorph {
                    from: TestObj::C,
                    to: TestObj::C,
                    kind: TestKind::Identity,
                },
                // Subsumption chain: A ⊑ B ⊑ C (DAG, antisymmetric)
                TestMorph {
                    from: TestObj::A,
                    to: TestObj::B,
                    kind: TestKind::Subsumption,
                },
                TestMorph {
                    from: TestObj::B,
                    to: TestObj::C,
                    kind: TestKind::Subsumption,
                },
                // Opposition pair: A ↔ B (symmetric, irreflexive)
                TestMorph {
                    from: TestObj::A,
                    to: TestObj::B,
                    kind: TestKind::Opposition,
                },
                TestMorph {
                    from: TestObj::B,
                    to: TestObj::A,
                    kind: TestKind::Opposition,
                },
                // Causation: A → B (asymmetric, irreflexive)
                TestMorph {
                    from: TestObj::A,
                    to: TestObj::B,
                    kind: TestKind::Causation,
                },
            ]
        }
    }

    /// Pattern-match helpers — the ontological test shape. The claim IS
    /// the Axiom; the test bridges the Verdict to Rust's panic-based test
    /// harness. No `.is_ok()`/`.is_err()` bool shortcuts (see
    /// `feedback_core_no_bool_api`).
    fn expect_proves<A: Axiom>(axiom: A) {
        match axiom.verify() {
            Ok(_) => {}
            Err(c) => panic!("expected proof but got counterexample: {}", c.meta().name),
        }
    }

    fn expect_refutes<A: Axiom>(axiom: A) {
        match axiom.verify() {
            Err(_) => {}
            Ok(p) => panic!("expected counterexample but got proof: {}", p.meta().name),
        }
    }

    #[test]
    fn no_cycles_holds_on_subsumption() {
        expect_proves(NoCyclesOnKind::<TestCat>::new(TestKind::Subsumption));
    }

    #[test]
    fn antisymmetric_holds_on_subsumption() {
        expect_proves(AntisymmetricOnKind::<TestCat>::new(TestKind::Subsumption));
    }

    #[test]
    fn symmetric_holds_on_opposition() {
        expect_proves(SymmetricOnKind::<TestCat>::new(TestKind::Opposition));
    }

    #[test]
    fn irreflexive_holds_on_opposition() {
        expect_proves(IrreflexiveOnKind::<TestCat>::new(TestKind::Opposition));
    }

    #[test]
    fn asymmetric_holds_on_causation() {
        expect_proves(AsymmetricOnKind::<TestCat>::new(TestKind::Causation));
    }

    #[test]
    fn symmetric_fails_on_causation() {
        // Causation has (A, B) but not (B, A) — symmetric must refute.
        expect_refutes(SymmetricOnKind::<TestCat>::new(TestKind::Causation));
    }

    #[test]
    fn meta_carries_kind_identifier() {
        let ax = NoCyclesOnKind::<TestCat>::new(TestKind::Subsumption);
        assert_eq!(ax.meta().name.as_str(), "NoCyclesOnKind[Subsumption]");
        assert!(!ax.meta().citation.as_str().is_empty());
    }
}
