//! Kind-parameterised structural axioms — replaces the per-primitive-trait
//! families (`NoCycles<TaxonomyDef>`, `NoCycles<MereologyDef>`, …) with a
//! single family per structural property that filters a `Category`'s
//! morphisms by edge kind.
//!
//! Rationale (issue #152): the axioms *are properties of relations*, not
//! type-level distinctions. Forcing each axiom into a separate trait
//! family (`TaxonomyDef` vs `MereologyDef`) is a category error. One
//! axiom type per property, filtered to the relation kind it applies
//! to, is the ontological shape.
//!
//! Each axiom here accepts:
//! 1. A `Category` `C` — the containing ontology's category
//! 2. A `kind_name: &'static str` — what relation concept the filter
//!    selects (must match a Relations ontology concept — e.g.
//!    "Subsumption", "Parthood", "Causation", "Opposition")
//! 3. A `filter: fn(&C::Morphism) -> bool` — picks out the
//!    kind-matching morphisms; generated per-ontology by the
//!    `pr4xis::ontology!` macro
//!
//! The `AxiomMeta` carries the kind name so the Lemon registry reports
//! e.g. `NoCyclesOnKind[Subsumption]` rather than the generic type name.
//!
//! Sources for the structural properties themselves:
//! - Tarski (1941) *Calculus of Relations* — the axiom names and their
//!   algebraic definitions
//! - Russell & Whitehead *Principia Mathematica* (1910–13) §§30–35 — binary
//!   relations and their structural properties
//! - Smith et al. (2005) OBO Relation Ontology — which properties attach
//!   to which relation types canonically

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::marker::PhantomData;

use crate::category::{Category, Relationship};
use crate::logic::axiom::{Axiom, AxiomMeta};
use crate::ontology::meta::{Citation, Label, ModulePath, OntologyName};

/// Collect (from, to) pairs from the category's morphisms, filtered to
/// the morphisms matching the kind filter.
fn kinded_pairs<C>(filter: fn(&C::Morphism) -> bool) -> Vec<(C::Object, C::Object)>
where
    C: Category,
    C::Object: Clone,
    C::Morphism: Relationship<Object = C::Object>,
{
    C::morphisms()
        .into_iter()
        .filter(filter)
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

fn meta_for(
    axiom_name: &'static str,
    kind_name: &'static str,
    citation: &'static str,
) -> AxiomMeta {
    AxiomMeta {
        name: OntologyName::new(format!("{axiom_name}[{kind_name}]")),
        description: Label::new(format!("{axiom_name} applied to edges of kind {kind_name}")),
        citation: Citation::parse_static(citation),
        module_path: ModulePath::new_static(module_path!()),
    }
}

// ---------------------------------------------------------------------------
// NoCyclesOnKind — filter a category's edges by kind; verify the resulting
// graph has no cycles (DAG). Applies canonically to Subsumption and Parthood.
// Source: Guarino (2009); Casati & Varzi (1999); Tarski (1941).
// ---------------------------------------------------------------------------

pub struct NoCyclesOnKind<C: Category> {
    kind_name: &'static str,
    filter: fn(&C::Morphism) -> bool,
    _marker: PhantomData<C>,
}

impl<C: Category> NoCyclesOnKind<C> {
    pub fn new(kind_name: &'static str, filter: fn(&C::Morphism) -> bool) -> Self {
        Self {
            kind_name,
            filter,
            _marker: PhantomData,
        }
    }
}

impl<C> Axiom for NoCyclesOnKind<C>
where
    C: Category,
    C::Object: Clone + Eq + Hash,
    C::Morphism: Relationship<Object = C::Object>,
{
    fn description(&self) -> &str {
        "edges of this kind have no cycles (form a DAG)"
    }

    fn holds(&self) -> bool {
        let pairs = kinded_pairs::<C>(self.filter);
        let adj = adjacency(&pairs);
        // No entity appears in its own reachable set.
        adj.keys().all(|e| !reachable_from(e, &adj).contains(e))
    }

    fn meta(&self) -> AxiomMeta {
        meta_for(
            "NoCyclesOnKind",
            self.kind_name,
            "Guarino (2009); Casati & Varzi (1999); Tarski (1941) Calculus of Relations",
        )
    }
}

// ---------------------------------------------------------------------------
// AntisymmetricOnKind — if (A, B) is an edge and A ≠ B, then (B, A) is not.
// Applies canonically to Subsumption (if A is-a B then B is not a A).
// Source: Guarino (2009); Tarski (1941); Mac Lane (1971) partial orders.
// ---------------------------------------------------------------------------

pub struct AntisymmetricOnKind<C: Category> {
    kind_name: &'static str,
    filter: fn(&C::Morphism) -> bool,
    _marker: PhantomData<C>,
}

impl<C: Category> AntisymmetricOnKind<C> {
    pub fn new(kind_name: &'static str, filter: fn(&C::Morphism) -> bool) -> Self {
        Self {
            kind_name,
            filter,
            _marker: PhantomData,
        }
    }
}

impl<C> Axiom for AntisymmetricOnKind<C>
where
    C: Category,
    C::Object: Clone + Eq + Hash,
    C::Morphism: Relationship<Object = C::Object>,
{
    fn description(&self) -> &str {
        "edges of this kind are antisymmetric: if (A, B) and A ≠ B, then not (B, A)"
    }

    fn holds(&self) -> bool {
        let pairs = kinded_pairs::<C>(self.filter);
        let set: HashSet<(C::Object, C::Object)> = pairs.iter().cloned().collect();
        for (a, b) in &pairs {
            if a != b && set.contains(&(b.clone(), a.clone())) {
                return false;
            }
        }
        true
    }

    fn meta(&self) -> AxiomMeta {
        meta_for(
            "AntisymmetricOnKind",
            self.kind_name,
            "Guarino (2009); Tarski (1941); Mac Lane (1971) partial orders",
        )
    }
}

// ---------------------------------------------------------------------------
// AsymmetricOnKind — no symmetric pair at all (stronger than antisymmetric:
// also excludes self-loops). Applies canonically to Causation.
// Source: Lewis (1973) Causation; Reichenbach (1956); Tarski (1941).
// ---------------------------------------------------------------------------

pub struct AsymmetricOnKind<C: Category> {
    kind_name: &'static str,
    filter: fn(&C::Morphism) -> bool,
    _marker: PhantomData<C>,
}

impl<C: Category> AsymmetricOnKind<C> {
    pub fn new(kind_name: &'static str, filter: fn(&C::Morphism) -> bool) -> Self {
        Self {
            kind_name,
            filter,
            _marker: PhantomData,
        }
    }
}

impl<C> Axiom for AsymmetricOnKind<C>
where
    C: Category,
    C::Object: Clone + Eq + Hash,
    C::Morphism: Relationship<Object = C::Object>,
{
    fn description(&self) -> &str {
        "edges of this kind are asymmetric: if (A, B) then not (B, A) and A ≠ B"
    }

    fn holds(&self) -> bool {
        let pairs = kinded_pairs::<C>(self.filter);
        let set: HashSet<(C::Object, C::Object)> = pairs.iter().cloned().collect();
        for (a, b) in &pairs {
            if a == b || set.contains(&(b.clone(), a.clone())) {
                return false;
            }
        }
        true
    }

    fn meta(&self) -> AxiomMeta {
        meta_for(
            "AsymmetricOnKind",
            self.kind_name,
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

pub struct SymmetricOnKind<C: Category> {
    kind_name: &'static str,
    filter: fn(&C::Morphism) -> bool,
    _marker: PhantomData<C>,
}

impl<C: Category> SymmetricOnKind<C> {
    pub fn new(kind_name: &'static str, filter: fn(&C::Morphism) -> bool) -> Self {
        Self {
            kind_name,
            filter,
            _marker: PhantomData,
        }
    }
}

impl<C> Axiom for SymmetricOnKind<C>
where
    C: Category,
    C::Object: Clone + Eq + Hash,
    C::Morphism: Relationship<Object = C::Object>,
{
    fn description(&self) -> &str {
        "edges of this kind are symmetric: (A, B) iff (B, A)"
    }

    fn holds(&self) -> bool {
        let pairs = kinded_pairs::<C>(self.filter);
        let set: HashSet<(C::Object, C::Object)> = pairs.iter().cloned().collect();
        for (a, b) in &pairs {
            if !set.contains(&(b.clone(), a.clone())) {
                return false;
            }
        }
        true
    }

    fn meta(&self) -> AxiomMeta {
        meta_for(
            "SymmetricOnKind",
            self.kind_name,
            "Aristotle Peri Hermeneias; Saussure (1916); Cruse (1986) Lexical Semantics; Tarski (1941)",
        )
    }
}

// ---------------------------------------------------------------------------
// IrreflexiveOnKind — no entity is its own image under the relation.
// Applies canonically to Opposition and Causation.
// Source: Aristotle Peri Hermeneias; Lewis (1973); Tarski (1941).
// ---------------------------------------------------------------------------

pub struct IrreflexiveOnKind<C: Category> {
    kind_name: &'static str,
    filter: fn(&C::Morphism) -> bool,
    _marker: PhantomData<C>,
}

impl<C: Category> IrreflexiveOnKind<C> {
    pub fn new(kind_name: &'static str, filter: fn(&C::Morphism) -> bool) -> Self {
        Self {
            kind_name,
            filter,
            _marker: PhantomData,
        }
    }
}

impl<C> Axiom for IrreflexiveOnKind<C>
where
    C: Category,
    C::Object: Clone + Eq,
    C::Morphism: Relationship<Object = C::Object>,
{
    fn description(&self) -> &str {
        "edges of this kind are irreflexive: no (A, A)"
    }

    fn holds(&self) -> bool {
        C::morphisms()
            .into_iter()
            .filter(self.filter)
            .all(|m| m.source() != m.target())
    }

    fn meta(&self) -> AxiomMeta {
        meta_for(
            "IrreflexiveOnKind",
            self.kind_name,
            "Aristotle Peri Hermeneias; Lewis (1973); Tarski (1941)",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::Entity;

    // A tiny test category with kinded morphisms.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestObj {
        A,
        B,
        C,
    }

    impl Entity for TestObj {
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

    impl Relationship for TestMorph {
        type Object = TestObj;
        fn source(&self) -> TestObj {
            self.from
        }
        fn target(&self) -> TestObj {
            self.to
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

    fn is_subsumption(m: &TestMorph) -> bool {
        m.kind == TestKind::Subsumption
    }
    fn is_opposition(m: &TestMorph) -> bool {
        m.kind == TestKind::Opposition
    }
    fn is_causation(m: &TestMorph) -> bool {
        m.kind == TestKind::Causation
    }

    #[test]
    fn no_cycles_holds_on_subsumption() {
        let ax = NoCyclesOnKind::<TestCat>::new("Subsumption", is_subsumption);
        assert!(ax.holds());
    }

    #[test]
    fn antisymmetric_holds_on_subsumption() {
        let ax = AntisymmetricOnKind::<TestCat>::new("Subsumption", is_subsumption);
        assert!(ax.holds());
    }

    #[test]
    fn symmetric_holds_on_opposition() {
        let ax = SymmetricOnKind::<TestCat>::new("Opposition", is_opposition);
        assert!(ax.holds());
    }

    #[test]
    fn irreflexive_holds_on_opposition() {
        let ax = IrreflexiveOnKind::<TestCat>::new("Opposition", is_opposition);
        assert!(ax.holds());
    }

    #[test]
    fn asymmetric_holds_on_causation() {
        let ax = AsymmetricOnKind::<TestCat>::new("Causation", is_causation);
        assert!(ax.holds());
    }

    #[test]
    fn symmetric_fails_on_causation() {
        // Causation has (A, B) but not (B, A) — symmetric must fail.
        let ax = SymmetricOnKind::<TestCat>::new("Causation", is_causation);
        assert!(!ax.holds());
    }

    #[test]
    fn meta_carries_kind_name() {
        let ax = NoCyclesOnKind::<TestCat>::new("Subsumption", is_subsumption);
        assert_eq!(ax.meta().name.as_str(), "NoCyclesOnKind[Subsumption]");
        assert!(!ax.meta().citation.as_str().is_empty());
    }
}
