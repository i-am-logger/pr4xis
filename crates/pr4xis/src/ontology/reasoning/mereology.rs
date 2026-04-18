#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};
use core::marker::PhantomData;

use crate::category::Category;
use crate::category::entity::Concept;
use crate::category::relationship::Relationship;

use super::graph;

/// Domains implement this to declare their part-whole relationships.
///
/// A mereology is a DAG of has-a relationships.
/// If A has-a B, then B is a part of A.
pub trait MereologyDef {
    type Concept: Concept;
    /// Direct has-a pairs: (whole, part).
    fn relations() -> Vec<(Self::Concept, Self::Concept)>;
}

/// Has-a relationship morphism: whole has-a part.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HasA<E: Concept> {
    pub whole: E,
    pub part: E,
}

impl<E: Concept> Relationship for HasA<E> {
    type Object = E;
    type Kind = ();
    fn source(&self) -> E {
        self.whole.clone()
    }
    fn target(&self) -> E {
        self.part.clone()
    }
    fn kind(&self) {}
}

/// Category adapter for a mereology.
///
/// Objects are the entities. Morphisms are has-a relationships
/// (direct + identity + transitive closure).
pub struct MereologyCategory<T: MereologyDef> {
    _marker: PhantomData<T>,
}

impl<T: MereologyDef> Category for MereologyCategory<T> {
    type Object = T::Concept;
    type Morphism = HasA<T::Concept>;

    fn identity(obj: &T::Concept) -> HasA<T::Concept> {
        HasA {
            whole: obj.clone(),
            part: obj.clone(),
        }
    }

    fn compose(f: &HasA<T::Concept>, g: &HasA<T::Concept>) -> Option<HasA<T::Concept>> {
        if f.part != g.whole {
            return None;
        }
        Some(HasA {
            whole: f.whole.clone(),
            part: g.part.clone(),
        })
    }

    fn morphisms() -> Vec<HasA<T::Concept>> {
        let entities = T::Concept::variants();
        let adj = graph::adjacency_map(&T::relations());

        let mut morphisms = Vec::new();
        for entity in &entities {
            morphisms.push(Self::identity(entity));
            for part in graph::reachable(entity, &adj) {
                morphisms.push(HasA {
                    whole: entity.clone(),
                    part,
                });
            }
        }
        morphisms
    }
}

// ---- Query functions ----

/// All direct and transitive parts of a whole. Does not include the entity itself.
pub fn parts_of<T: MereologyDef>(whole: &T::Concept) -> Vec<T::Concept> {
    let adj = graph::adjacency_map(&T::relations());
    graph::reachable(whole, &adj)
}

/// All wholes that transitively contain this part. Does not include the entity itself.
pub fn whole_of<T: MereologyDef>(part: &T::Concept) -> Vec<T::Concept> {
    let adj = graph::reverse_adjacency_map(&T::relations());
    graph::reachable(part, &adj)
}

// ---- Axioms ----

/// Axiom: the mereology has no cycles (it is a DAG).
pub struct NoCycles<T: MereologyDef> {
    _marker: PhantomData<T>,
}

impl<T: MereologyDef> NoCycles<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: MereologyDef> Default for NoCycles<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: MereologyDef> crate::logic::Axiom for NoCycles<T> {
    fn description(&self) -> &str {
        "mereology has no cycles (part-whole is a DAG)"
    }

    fn holds(&self) -> bool {
        let adj = graph::adjacency_map(&T::relations());
        T::Concept::variants()
            .iter()
            .all(|entity| !graph::has_cycle(entity, &adj))
    }

    crate::axiom_meta!(
        "NoCycles[Mereology]",
        "mereology has no cycles (part-whole is a DAG)",
        "Casati & Varzi (1999) 'Parts and Places' — Classical Extensional Mereology"
    );
}

/// Axiom: weak supplementation — if A has-a B (and A != B),
/// then A has at least one other direct part C != B.
pub struct WeakSupplementation<T: MereologyDef> {
    _marker: PhantomData<T>,
}

impl<T: MereologyDef> WeakSupplementation<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: MereologyDef> Default for WeakSupplementation<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: MereologyDef> crate::logic::Axiom for WeakSupplementation<T> {
    fn description(&self) -> &str {
        "weak supplementation: every proper whole has at least two direct parts"
    }

    fn holds(&self) -> bool {
        let direct = T::relations();
        let adj = graph::adjacency_map(
            &direct
                .iter()
                .filter(|(w, p)| w != p)
                .cloned()
                .collect::<Vec<_>>(),
        );
        adj.values().all(|parts| parts.len() >= 2)
    }

    crate::axiom_meta!(
        "WeakSupplementation[Mereology]",
        "weak supplementation: every proper whole has at least two direct parts",
        "Simons (1987) 'Parts: A Study in Ontology'; Casati & Varzi (1999)"
    );
}

// ---- Algebraic structure integrations ----

/// Query whole and part relationships independently using applicative.
///
/// The parts and wholes of two entities are independent queries —
/// neither depends on the other's result. This is applicative, not monadic.
///
/// Reference: McBride & Paterson, "Applicative Programming with Effects" (2008)
#[allow(clippy::type_complexity)]
pub fn applicative_parts_wholes<T: MereologyDef>(
    entity_a: &T::Concept,
    entity_b: &T::Concept,
) -> crate::category::Ap<crate::category::Product<Vec<T::Concept>, Vec<T::Concept>>> {
    let parts_a = crate::category::Ap::pure(parts_of::<T>(entity_a));
    let wholes_b = crate::category::Ap::pure(whole_of::<T>(entity_b));
    parts_a.map2(wholes_b, |parts, wholes| {
        crate::category::Product::new(parts, wholes)
    })
}

/// Unfold a mereology tree from a root using anamorphism.
///
/// Produces a Cofree tree where each node carries an entity
/// and its children are its direct parts.
///
/// Reference: Meijer, Fokkinga & Paterson (1991)
pub fn unfold_mereology<T: MereologyDef + 'static>()
-> crate::category::algebra::Coalgebra<T::Concept, T::Concept>
where
    T::Concept: Clone + core::fmt::Debug,
{
    let relations = T::relations();
    crate::category::algebra::Coalgebra::new(move |whole: &T::Concept| {
        let parts: Vec<T::Concept> = relations
            .iter()
            .filter(|(w, _)| w == whole)
            .map(|(_, part)| part.clone())
            .collect();
        (whole.clone(), parts)
    })
}

/// Lens into the mereology of an entity: view/modify its parts.
///
/// Reference: van Laarhoven (2009), Pickering et al. (2017)
pub fn parts_lens<T: MereologyDef + 'static>()
-> crate::category::optics::Lens<T::Concept, Vec<T::Concept>>
where
    T::Concept: Clone + core::fmt::Debug,
{
    crate::category::optics::Lens::new(
        |whole: &T::Concept| parts_of::<T>(whole),
        |_whole: &T::Concept, _new_parts: Vec<T::Concept>| {
            // Mereology is declarative — can't "set" parts at runtime.
            // Return the whole unchanged (read-only lens).
            _whole.clone()
        },
    )
}

/// Yoneda profile for mereology.
pub fn yoneda_profile<T: MereologyDef>(
    entity: &T::Concept,
) -> crate::category::yoneda::YonedaProfile<MereologyCategory<T>> {
    crate::category::yoneda::YonedaProfile::of(entity)
}
