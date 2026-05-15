// Declarative macro for defining ontology categories with minimal boilerplate.
//
// `define_category!` — kinded categories with explicit relation types.
// Dense (anonymous-morphism) categories are no longer supported — per Gruber
// (1993) / OBO-RO (Smith et al. 2005), every morphism carries a canonical
// relation-kind tag. Callers declaring sugar clauses (is_a / has_a / causes /
// opposes) go through `ontology!` which synthesises
// kinded edges automatically.

/// Define a kinded category with explicit relation types.
///
/// Generates: RelationKind enum, Relation struct, Arrow impl,
/// Category struct, and full Category impl (identity, compose, 4-phase morphisms).
///
/// # Example
///
/// ```text
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
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $relation {
            pub from: $entity,
            pub to: $entity,
            pub kind: $kind,
        }

        impl $crate::category::Arrow for $relation {
            type Object = $entity;
            type Kind = $kind;
            fn source(&self) -> $entity { self.from }
            fn target(&self) -> $entity { self.to }
            fn kind(&self) -> $kind { self.kind }

            fn meta(&self) -> $crate::ontology::meta::Provenance {
                $crate::ontology::meta::Provenance {
                    name: $crate::ontology::meta::OntologyName::new(
                        format!("{:?}-[{:?}]-{:?}", self.from, self.kind, self.to)
                    ),
                    description: $crate::ontology::meta::Label::new(
                        format!("{:?} -[{:?}]-> {:?}", self.from, self.kind, self.to)
                    ),
                    citation: $crate::ontology::meta::Citation::EMPTY,
                    module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
                }
            }
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
                // Partial composition (#166): heterogeneous / non-transitive
                // compositions return None per OBO-RO / Spivak. Same-kind
                // transitive inheritance is encoded by the proc-macro
                // `ontology!` path; this declarative `define_category!`
                // path is legacy and does not declare kind transitivity,
                // so all non-identity compositions are partial.
                None
            }

            fn morphisms() -> Vec<$relation> {
                use $crate::category::Concept;

                let mut m = Vec::new();

                // Phase 1: Identity morphisms
                for c in $entity::variants() {
                    m.push($relation { from: c, to: c, kind: $kind::Identity });
                }

                // Phase 2: Domain edges — qualify variant names so struct-based
                // entities (tuple structs, newtypes) also work.
                $(m.push($relation { from: $entity::$e_from, to: $entity::$e_to, kind: $kind::$e_kind });)*

                // No transitive-closure edges (#166). The legacy `composed:`
                // clause is silently ignored — per OBO-RO partial-category
                // semantics, composed morphisms aren't typed absent a rule.

                m
            }
        }
    };
}
