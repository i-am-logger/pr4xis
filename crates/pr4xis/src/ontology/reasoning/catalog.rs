//! Structural-properties catalog — the canonical mapping from a
//! relation-kind name to the structural axioms that apply to edges of
//! that kind.
//!
//! # Ontological placement (#168)
//!
//! The *content* of this catalog is the Relations ontology's
//! specification: Smith et al. (2005) OBO-RO names `Subsumption`,
//! `Parthood`, `Causation`, `Opposition` as canonical binary-relation
//! types and stipulates which algebraic properties (Tarski 1941
//! Calculus of Relations) attach to each. The Relations ontology in
//! `crates/domains/src/formal/relations/` is the ontological
//! declaration; this module is its Rust realisation, positioned in
//! core because core owns the `OnKind` axiom machinery.
//!
//! Per `feedback_one_ontology_per_module`: the rules live in one
//! place. Macros and runtime callers both consult the single
//! [`structural_axioms_for`] function — no per-ontology re-emission
//! of `NoCyclesOnKind<FooCategory>` / `NoCyclesOnKind<BarCategory>`
//! that differs only by type parameter.
//!
//! # Matching is by canonical name, not by type identity
//!
//! `pr4xis-derive` cannot reference `RelationsConcept` (would reverse
//! the workspace dep direction), so the catalog is keyed by the
//! `Debug` rendering of a kind value: `"Subsumption"`, `"Parthood"`,
//! etc. An ontology that uses non-canonical kind names (e.g.
//! `"Produces"`, `"Grounds"`) inherits no structural axioms — it may
//! still declare them as hand-written domain axioms.
//!
//! Literature:
//! - Smith et al. (2005) *Relations in Biomedical Ontologies* (OBO-RO) —
//!   canonical relation types and their algebraic properties
//! - Tarski (1941) *Calculus of Relations* — the structural properties
//!   themselves (transitive, antisymmetric, irreflexive, …)
//! - Gruber (1993) *A Translation Approach to Portable Ontology
//!   Specifications* — ontologies describe concepts, relations, and
//!   axioms uniformly; the catalog is one of those axiom families
//! - Guarino (2009) *The Ontological Level* — separates ontological
//!   commitment from axiomatisation; this module is axiomatisation

use std::fmt::Debug;
use std::hash::Hash;

use crate::category::{Arrow, Category};
use crate::logic::Axiom;

use super::structural::{
    AntisymmetricOnKind, AsymmetricOnKind, IrreflexiveOnKind, NoCyclesOnKind, SymmetricOnKind,
};

/// Inherit the canonical structural axioms for every relation kind
/// used in `C::morphisms()`, matching by canonical name (OBO-RO).
///
/// - `Subsumption` → `NoCyclesOnKind`, `AntisymmetricOnKind`
/// - `Parthood` → `NoCyclesOnKind`
/// - `Causation` → `AsymmetricOnKind`, `IrreflexiveOnKind`
/// - `Opposition` → `SymmetricOnKind`, `IrreflexiveOnKind`
/// - anything else → no structural axioms (add a hand-written domain
///   axiom if you need one)
///
/// Typical call site — inside an ontology's `axioms()` method:
///
/// ```text
/// impl Ontology for FooOntology {
///     fn axioms() -> Vec<Box<dyn Axiom>> {
///         let mut all = structural_axioms_for::<Self::Cat>();
///         all.push(Box::new(MyDomainAxiom));
///         all
///     }
/// }
/// ```
///
/// Kinds are deduplicated by `PartialEq`. Ordering within the returned
/// vec follows the order kinds first appear in `C::morphisms()`.
pub fn structural_axioms_for<C>() -> Vec<Box<dyn Axiom>>
where
    C: Category + 'static,
    C::Object: Clone + Eq + Hash + 'static,
    C::Morphism: Arrow<Object = C::Object>,
    <C::Morphism as Arrow>::Kind: Debug + PartialEq + Clone + 'static,
{
    let mut distinct: Vec<<C::Morphism as Arrow>::Kind> = Vec::new();
    for m in C::morphisms() {
        let k = m.kind();
        if !distinct.iter().any(|existing| existing == &k) {
            distinct.push(k);
        }
    }

    let mut axioms: Vec<Box<dyn Axiom>> = Vec::new();
    for kind in distinct {
        let name = format!("{kind:?}");
        match name.as_str() {
            "Subsumption" => {
                axioms.push(Box::new(NoCyclesOnKind::<C>::new(kind)));
                axioms.push(Box::new(AntisymmetricOnKind::<C>::new(kind)));
            }
            "Parthood" => {
                axioms.push(Box::new(NoCyclesOnKind::<C>::new(kind)));
            }
            "Causation" => {
                axioms.push(Box::new(AsymmetricOnKind::<C>::new(kind)));
                axioms.push(Box::new(IrreflexiveOnKind::<C>::new(kind)));
            }
            "Opposition" => {
                axioms.push(Box::new(SymmetricOnKind::<C>::new(kind)));
                axioms.push(Box::new(IrreflexiveOnKind::<C>::new(kind)));
            }
            _ => {}
        }
    }
    axioms
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::{Concept, FinitelyGenerated};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Obj {
        X,
        Y,
        Z,
    }

    impl Concept for Obj {}
    impl FinitelyGenerated for Obj {
        fn variants() -> Vec<Self> {
            vec![Obj::X, Obj::Y, Obj::Z]
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Kind {
        Identity,
        Subsumption,
        Parthood,
        Causation,
        Opposition,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct M {
        from: Obj,
        to: Obj,
        kind: Kind,
    }

    impl Arrow for M {
        type Object = Obj;
        type Kind = Kind;
        fn source(&self) -> Obj {
            self.from
        }
        fn target(&self) -> Obj {
            self.to
        }
        fn kind(&self) -> Kind {
            self.kind
        }
    }

    struct Cat;
    impl Category for Cat {
        type Object = Obj;
        type Morphism = M;
        fn identity(o: &Obj) -> M {
            M {
                from: *o,
                to: *o,
                kind: Kind::Identity,
            }
        }
        fn compose(f: &M, g: &M) -> Option<M> {
            if f.to != g.from {
                return None;
            }
            Some(M {
                from: f.from,
                to: g.to,
                kind: Kind::Identity,
            })
        }
        fn morphisms() -> Vec<M> {
            vec![
                M {
                    from: Obj::X,
                    to: Obj::X,
                    kind: Kind::Identity,
                },
                M {
                    from: Obj::Y,
                    to: Obj::Y,
                    kind: Kind::Identity,
                },
                M {
                    from: Obj::Z,
                    to: Obj::Z,
                    kind: Kind::Identity,
                },
                // Subsumption: X ⊑ Y ⊑ Z — expect NoCycles + Antisymmetric
                M {
                    from: Obj::X,
                    to: Obj::Y,
                    kind: Kind::Subsumption,
                },
                M {
                    from: Obj::Y,
                    to: Obj::Z,
                    kind: Kind::Subsumption,
                },
                // Parthood: X has Y — expect NoCycles
                M {
                    from: Obj::X,
                    to: Obj::Y,
                    kind: Kind::Parthood,
                },
                // Causation: X → Y — expect Asymmetric + Irreflexive
                M {
                    from: Obj::X,
                    to: Obj::Y,
                    kind: Kind::Causation,
                },
                // Opposition: X ↔ Y — expect Symmetric + Irreflexive
                M {
                    from: Obj::X,
                    to: Obj::Y,
                    kind: Kind::Opposition,
                },
                M {
                    from: Obj::Y,
                    to: Obj::X,
                    kind: Kind::Opposition,
                },
            ]
        }
    }

    #[crate::praxis_value(Verifiable)]
    #[test]
    fn inherits_expected_count() {
        let axioms = structural_axioms_for::<Cat>();
        // Subsumption: 2 + Parthood: 1 + Causation: 2 + Opposition: 2 = 7
        // (Identity gets no structural axioms — correct: it's not in OBO-RO's canonical set)
        assert_eq!(axioms.len(), 7);
    }

    #[crate::praxis_value(Verifiable)]
    #[test]
    fn all_inherited_axioms_verify() {
        for a in structural_axioms_for::<Cat>() {
            a.verify().unwrap_or_else(|c| {
                panic!(
                    "inherited structural axiom failed: {}",
                    c.meta().name.as_str()
                )
            });
        }
    }
}
