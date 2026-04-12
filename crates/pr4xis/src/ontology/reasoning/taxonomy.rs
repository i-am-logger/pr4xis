use std::marker::PhantomData;

use crate::category::Category;
use crate::category::entity::Entity;
use crate::category::relationship::Relationship;
use crate::ontology::Quality;

use super::graph;

/// Domains implement this to declare their is-a taxonomy.
///
/// A taxonomy is a directed acyclic graph (DAG) of subsumption relationships.
/// If A is-a B, then A inherits all qualities of B.
pub trait TaxonomyDef {
    type Entity: Entity;
    /// Direct is-a pairs: (child, parent).
    fn relations() -> Vec<(Self::Entity, Self::Entity)>;
}

/// Is-a relationship morphism: child is-a parent.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IsA<E: Entity> {
    pub child: E,
    pub parent: E,
}

impl<E: Entity> Relationship for IsA<E> {
    type Object = E;
    fn source(&self) -> E {
        self.child.clone()
    }
    fn target(&self) -> E {
        self.parent.clone()
    }
}

/// Category adapter for a taxonomy.
///
/// Objects are the entities. Morphisms are is-a relationships
/// (direct relations + identity + transitive closure).
pub struct TaxonomyCategory<T: TaxonomyDef> {
    _marker: PhantomData<T>,
}

impl<T: TaxonomyDef> Category for TaxonomyCategory<T> {
    type Object = T::Entity;
    type Morphism = IsA<T::Entity>;

    fn identity(obj: &T::Entity) -> IsA<T::Entity> {
        IsA {
            child: obj.clone(),
            parent: obj.clone(),
        }
    }

    fn compose(f: &IsA<T::Entity>, g: &IsA<T::Entity>) -> Option<IsA<T::Entity>> {
        if f.parent != g.child {
            return None;
        }
        Some(IsA {
            child: f.child.clone(),
            parent: g.parent.clone(),
        })
    }

    fn morphisms() -> Vec<IsA<T::Entity>> {
        let entities = T::Entity::variants();
        let adj = graph::adjacency_map(&T::relations());

        let mut morphisms = Vec::new();
        for entity in &entities {
            morphisms.push(Self::identity(entity));
            for ancestor in graph::reachable(entity, &adj) {
                morphisms.push(IsA {
                    child: entity.clone(),
                    parent: ancestor,
                });
            }
        }
        morphisms
    }
}

// ---- Query functions ----

/// Check if `child` is-a `ancestor` (transitively).
pub fn is_a<T: TaxonomyDef>(child: &T::Entity, ancestor: &T::Entity) -> bool {
    if child == ancestor {
        return true;
    }
    ancestors::<T>(child).contains(ancestor)
}

/// All ancestors of an entity (transitive). Does not include the entity itself.
pub fn ancestors<T: TaxonomyDef>(entity: &T::Entity) -> Vec<T::Entity> {
    let adj = graph::adjacency_map(&T::relations());
    graph::reachable(entity, &adj)
}

/// All descendants of an entity (transitive). Does not include the entity itself.
pub fn descendants<T: TaxonomyDef>(entity: &T::Entity) -> Vec<T::Entity> {
    let adj = graph::reverse_adjacency_map(&T::relations());
    graph::reachable(entity, &adj)
}

/// Inherit a quality from an ancestor: if the entity doesn't have the quality directly,
/// walk up the taxonomy until an ancestor has it.
pub fn inherit_quality<T, Q>(entity: &T::Entity, quality: &Q) -> Option<Q::Value>
where
    T: TaxonomyDef,
    Q: Quality<Individual = T::Entity>,
{
    if let Some(v) = quality.get(entity) {
        return Some(v);
    }
    for ancestor in ancestors::<T>(entity) {
        if let Some(v) = quality.get(&ancestor) {
            return Some(v);
        }
    }
    None
}

// ---- Axioms ----

/// Axiom: the taxonomy has no cycles (it is a DAG).
pub struct NoCycles<T: TaxonomyDef> {
    _marker: PhantomData<T>,
}

impl<T: TaxonomyDef> NoCycles<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: TaxonomyDef> Default for NoCycles<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TaxonomyDef> crate::logic::Axiom for NoCycles<T> {
    fn description(&self) -> &str {
        "taxonomy has no cycles (is a DAG)"
    }

    fn holds(&self) -> bool {
        let adj = graph::adjacency_map(&T::relations());
        T::Entity::variants()
            .iter()
            .all(|entity| !graph::has_cycle(entity, &adj))
    }
}

/// Axiom: antisymmetry — if A is-a B (and A != B), then B is NOT a A.
pub struct Antisymmetric<T: TaxonomyDef> {
    _marker: PhantomData<T>,
}

impl<T: TaxonomyDef> Antisymmetric<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: TaxonomyDef> Default for Antisymmetric<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TaxonomyDef> crate::logic::Axiom for Antisymmetric<T> {
    fn description(&self) -> &str {
        "taxonomy is antisymmetric: if A is-a B then B is not a A"
    }

    fn holds(&self) -> bool {
        let direct = T::relations();
        for (child, parent) in &direct {
            if child != parent && is_a::<T>(parent, child) {
                return false;
            }
        }
        true
    }
}

// ---- Algebraic structure integrations ----

/// Galois connection for taxonomy: descendants ⊣ ancestors.
///
/// The pair (descendants, ancestors) forms a Galois connection where:
///   descendants(a) ≤ b ⟺ a ≤ ancestors(b)
///
/// In ontological terms: "everything below A is below B" iff "A is below everything above B".
///
/// Reference: Ore, "Galois Connexions" (1944, Trans. AMS)
pub fn galois_connection<T: TaxonomyDef + 'static>()
-> crate::category::galois::GaloisConnection<T::Entity, Vec<T::Entity>>
where
    T::Entity: PartialOrd,
{
    crate::category::galois::GaloisConnection::new(
        |entity: &T::Entity| descendants::<T>(entity),
        |desc: &Vec<T::Entity>| {
            // Upper adjoint: the most specific common ancestor of all descendants
            // Simplified: return the first entity (if any)
            desc.first()
                .map(|e| ancestors::<T>(e))
                .unwrap_or_default()
                .into_iter()
                .next()
                .unwrap_or_else(|| T::Entity::variants()[0].clone())
        },
    )
}

/// Unfold a taxonomy tree from a root entity using an anamorphism.
///
/// Produces a Cofree tree where each node carries an entity
/// and its children are its direct subtypes.
///
/// Reference: Meijer, Fokkinga & Paterson (1991) — anamorphism
pub fn unfold_taxonomy<T: TaxonomyDef + 'static>()
-> crate::category::algebra::Coalgebra<T::Entity, T::Entity>
where
    T::Entity: Clone + std::fmt::Debug,
{
    let relations = T::relations();
    crate::category::algebra::Coalgebra::new(move |entity: &T::Entity| {
        let children: Vec<T::Entity> = relations
            .iter()
            .filter(|(_, parent)| parent == entity)
            .map(|(child, _)| child.clone())
            .collect();
        (entity.clone(), children)
    })
}

/// Compute the Yoneda profile of an entity in the taxonomy category.
///
/// Shows all is-a relationships from and to this entity — the entity's
/// complete taxonomic identity (Yoneda lemma: an entity IS its relationships).
///
/// Reference: Yoneda (1954)
pub fn yoneda_profile<T: TaxonomyDef>(
    entity: &T::Entity,
) -> crate::category::yoneda::YonedaProfile<TaxonomyCategory<T>> {
    crate::category::yoneda::YonedaProfile::of(entity)
}
