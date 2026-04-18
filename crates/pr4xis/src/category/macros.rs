// Declarative macros for defining ontology categories with minimal boilerplate.
//
// Two patterns:
// - `define_category!` — kinded categories with explicit relation types (Pattern A)
// - `define_dense_category!` — dense categories where all entity pairs are morphisms (Pattern B)

/// Define a kinded category with explicit relation types.
///
/// Generates: RelationKind enum, Relation struct, Relationship impl,
/// Category struct, and full Category impl (identity, compose, 4-phase morphisms).
///
/// # Example
///
/// ```ignore
/// define_category! {
///     /// Communication ontology.
///     pub CommunicationCategory {
///         entity: CommunicationConcept,
///         relation: CommunicationRelation,
///         kind: CommunicationRelationKind,
///         kinds: [Produces, Interprets, Corrupts],
///         edges: [
///             (Sender, Message, Produces),
///             (Receiver, Message, Interprets),
///         ],
///         composed: [
///             (Sender, Receiver),
///         ],
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_category {
    (
        $(#[$cat_meta:meta])*
        pub $cat_name:ident {
            entity: $entity:ident,
            relation: $relation:ident,
            kind: $kind:ident,
            kinds: [$($(#[$kind_meta:meta])* $domain_kind:ident),* $(,)?],
            edges: [$(($e_from:ident, $e_to:ident, $e_kind:ident)),* $(,)?],
            composed: [$(($c_from:ident, $c_to:ident)),* $(,)?],
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[allow(dead_code)]
        pub enum $kind {
            Identity,
            $($(#[$kind_meta])* $domain_kind,)*
            Composed,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $relation {
            pub from: $entity,
            pub to: $entity,
            pub kind: $kind,
        }

        impl $crate::category::Relationship for $relation {
            type Object = $entity;
            type Kind = $kind;
            fn source(&self) -> $entity { self.from }
            fn target(&self) -> $entity { self.to }
            fn kind(&self) -> $kind { self.kind }
        }

        $(#[$cat_meta])*
        pub struct $cat_name;

        impl $crate::category::Category for $cat_name {
            type Object = $entity;
            type Morphism = $relation;

            fn identity(obj: &$entity) -> $relation {
                $relation { from: *obj, to: *obj, kind: $kind::Identity }
            }

            fn compose(f: &$relation, g: &$relation) -> Option<$relation> {
                if f.to != g.from { return None; }
                if f.kind == $kind::Identity { return Some(g.clone()); }
                if g.kind == $kind::Identity { return Some(f.clone()); }
                Some($relation { from: f.from, to: g.to, kind: $kind::Composed })
            }

            fn morphisms() -> Vec<$relation> {
                use $crate::category::Entity;

                let mut m = Vec::new();

                // Phase 1: Identity morphisms
                for c in $entity::variants() {
                    m.push($relation { from: c, to: c, kind: $kind::Identity });
                }

                // Phase 2: Domain edges — qualify variant names so struct-based
                // entities (tuple structs, newtypes) also work.
                $(m.push($relation { from: $entity::$e_from, to: $entity::$e_to, kind: $kind::$e_kind });)*

                // Phase 3: Composed (transitive) edges
                $(m.push($relation { from: $entity::$c_from, to: $entity::$c_to, kind: $kind::Composed });)*

                // Phase 4: Self-composed closure. Each object's identity
                // round-trips to itself as Composed (distinct from the
                // Identity morphism — Mac Lane I.1 closure under compose).
                for c in $entity::variants() {
                    m.push($relation { from: c, to: c, kind: $kind::Composed });
                }

                m
            }
        }
    };
}

/// Define a dense category where all entity pairs are morphisms.
///
/// # Example
///
/// ```ignore
/// define_dense_category! {
///     /// Acoustics category — all entity pairs connected.
///     pub AcousticsCategory {
///         entity: AcousticEntity,
///         relation: AcousticRelation,
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_dense_category {
    (
        $(#[$cat_meta:meta])*
        pub $cat_name:ident {
            entity: $entity:ident,
            relation: $relation:ident,
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $relation {
            pub from: $entity,
            pub to: $entity,
        }

        impl $crate::category::Relationship for $relation {
            type Object = $entity;
            type Kind = ();
            fn source(&self) -> $entity { self.from }
            fn target(&self) -> $entity { self.to }
            fn kind(&self) -> () {}
        }

        $(#[$cat_meta])*
        pub struct $cat_name;

        impl $crate::category::Category for $cat_name {
            type Object = $entity;
            type Morphism = $relation;

            fn identity(obj: &$entity) -> $relation {
                $relation { from: *obj, to: *obj }
            }

            fn compose(f: &$relation, g: &$relation) -> Option<$relation> {
                if f.to != g.from { return None; }
                Some($relation { from: f.from, to: g.to })
            }

            fn morphisms() -> Vec<$relation> {
                use $crate::category::Entity;
                let variants = $entity::variants();
                variants.iter()
                    .flat_map(|&a| variants.iter().map(move |&b| $relation { from: a, to: b }))
                    .collect()
            }
        }
    };
}
