// Declarative macro for defining complete ontologies.
//
// Generates: Category + reasoning systems (Taxonomy, Mereology, Causation,
// Opposition) + Ontology trait impl with automatic axiom collection.
//
// The user declares domain knowledge. The macro generates structure.
// Qualities, custom axioms, and functors stay hand-written.

/// Define a complete ontology from domain knowledge.
///
/// Generates the Category, reasoning system impls (TaxonomyDef, MereologyDef,
/// CausalDef, OppositionDef), and Ontology trait impl with standard axioms.
///
/// # Dense category example (biomedical)
///
/// ```ignore
/// define_ontology! {
///     pub BiologyOntology for BiologyCategory {
///         entity: BiologicalEntity,
///         relation: BiologicalRelation,
///
///         taxonomy: BiologicalTaxonomy [
///             (SquamousEpithelial, Cell),
///             (Fibroblast, Cell),
///         ],
///
///         mereology: BiologicalMereology [
///             (Organism, Esophagus),
///             (Esophagus, SquamousEpithelium),
///         ],
///
///         causation: BiologicalCausalGraph for BiologicalCausalEvent [
///             (StemCellDivision, CellDifferentiation),
///         ],
///
///         opposition: BiologicalOpposition [
///             (MacrophageM1, MacrophageM2),
///         ],
///     }
/// }
/// ```
///
/// # Kinded category example (information)
///
/// ```ignore
/// define_ontology! {
///     pub CommunicationOntology for CommunicationCategory {
///         entity: CommunicationConcept,
///         relation: CommunicationRelation,
///         kind: CommunicationRelationKind,
///         kinds: [Produces, Interprets, Corrupts],
///         edges: [(Sender, Message, Produces)],
///         composed: [(Sender, Receiver)],
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_ontology {
    // =========================================================================
    // Pattern A: Dense category + reasoning systems
    // =========================================================================
    (
        $(#[$ont_meta:meta])*
        pub $ont_name:ident for $cat_name:ident {
            entity: $entity:ident,
            relation: $relation:ident,

            $(taxonomy: $tax_name:ident [
                $(($tax_child:ident, $tax_parent:ident)),* $(,)?
            ],)?

            $(mereology: $mer_name:ident [
                $(($mer_whole:ident, $mer_part:ident)),* $(,)?
            ],)?

            $(causation: $caus_name:ident for $caus_entity:ident [
                $(($caus_cause:ident, $caus_effect:ident)),* $(,)?
            ],)?

            $(opposition: $opp_name:ident [
                $(($opp_a:ident, $opp_b:ident)),* $(,)?
            ],)?
        }
    ) => {
        // Generate dense category
        $crate::define_dense_category! {
            $(#[$ont_meta])*
            pub $cat_name {
                entity: $entity,
                relation: $relation,
            }
        }

        define_ontology!(@reasoning $ont_name, $cat_name, $entity,
            $(taxonomy: $tax_name [ $(($tax_child, $tax_parent)),* ],)?
            $(mereology: $mer_name [ $(($mer_whole, $mer_part)),* ],)?
            $(causation: $caus_name for $caus_entity [ $(($caus_cause, $caus_effect)),* ],)?
            $(opposition: $opp_name [ $(($opp_a, $opp_b)),* ],)?
        );
    };

    // =========================================================================
    // Pattern B: Kinded category + reasoning systems
    // =========================================================================
    (
        $(#[$ont_meta:meta])*
        pub $ont_name:ident for $cat_name:ident {
            entity: $entity:ident,
            relation: $relation:ident,
            kind: $kind:ident,
            kinds: [$($(#[$kind_meta:meta])* $domain_kind:ident),* $(,)?],
            edges: [$(($e_from:ident, $e_to:ident, $e_kind:ident)),* $(,)?],
            composed: [$(($c_from:ident, $c_to:ident)),* $(,)?],

            $(taxonomy: $tax_name:ident [
                $(($tax_child:ident, $tax_parent:ident)),* $(,)?
            ],)?

            $(mereology: $mer_name:ident [
                $(($mer_whole:ident, $mer_part:ident)),* $(,)?
            ],)?

            $(causation: $caus_name:ident for $caus_entity:ident [
                $(($caus_cause:ident, $caus_effect:ident)),* $(,)?
            ],)?

            $(opposition: $opp_name:ident [
                $(($opp_a:ident, $opp_b:ident)),* $(,)?
            ],)?
        }
    ) => {
        // Generate kinded category
        $crate::define_category! {
            $(#[$ont_meta])*
            pub $cat_name {
                entity: $entity,
                relation: $relation,
                kind: $kind,
                kinds: [$($(#[$kind_meta])* $domain_kind),*],
                edges: [$(($e_from, $e_to, $e_kind)),*],
                composed: [$(($c_from, $c_to)),*],
            }
        }

        define_ontology!(@reasoning $ont_name, $cat_name, $entity,
            $(taxonomy: $tax_name [ $(($tax_child, $tax_parent)),* ],)?
            $(mereology: $mer_name [ $(($mer_whole, $mer_part)),* ],)?
            $(causation: $caus_name for $caus_entity [ $(($caus_cause, $caus_effect)),* ],)?
            $(opposition: $opp_name [ $(($opp_a, $opp_b)),* ],)?
        );
    };

    // =========================================================================
    // Internal: generate reasoning systems + Ontology impl
    // =========================================================================
    (@reasoning $ont_name:ident, $cat_name:ident, $entity:ident,
        $(taxonomy: $tax_name:ident [ $(($tax_child:ident, $tax_parent:ident)),* ],)?
        $(mereology: $mer_name:ident [ $(($mer_whole:ident, $mer_part:ident)),* ],)?
        $(causation: $caus_name:ident for $caus_entity:ident [ $(($caus_cause:ident, $caus_effect:ident)),* ],)?
        $(opposition: $opp_name:ident [ $(($opp_a:ident, $opp_b:ident)),* ],)?
    ) => {
        // --- Taxonomy ---
        $(
            pub struct $tax_name;
            impl $crate::ontology::reasoning::taxonomy::TaxonomyDef for $tax_name {
                type Entity = $entity;
                fn relations() -> Vec<($entity, $entity)> {
                    #[allow(unused_imports)]
                    use $entity::*;
                    vec![$(($tax_child, $tax_parent)),*]
                }
            }
        )?

        // --- Mereology ---
        $(
            pub struct $mer_name;
            impl $crate::ontology::reasoning::mereology::MereologyDef for $mer_name {
                type Entity = $entity;
                fn relations() -> Vec<($entity, $entity)> {
                    #[allow(unused_imports)]
                    use $entity::*;
                    vec![$(($mer_whole, $mer_part)),*]
                }
            }
        )?

        // --- Causation ---
        $(
            pub struct $caus_name;
            impl $crate::ontology::reasoning::causation::CausalDef for $caus_name {
                type Entity = $caus_entity;
                fn relations() -> Vec<($caus_entity, $caus_entity)> {
                    #[allow(unused_imports)]
                    use $caus_entity::*;
                    vec![$(($caus_cause, $caus_effect)),*]
                }
            }
        )?

        // --- Opposition ---
        $(
            pub struct $opp_name;
            impl $crate::ontology::reasoning::opposition::OppositionDef for $opp_name {
                type Entity = $entity;
                fn pairs() -> Vec<($entity, $entity)> {
                    #[allow(unused_imports)]
                    use $entity::*;
                    vec![$(($opp_a, $opp_b)),*]
                }
            }
        )?

        // --- Ontology struct + meta ---
        pub struct $ont_name;

        impl $ont_name {
            /// Ontology metadata for tracing and introspection.
            pub const fn meta() -> $crate::ontology::OntologyMeta {
                $crate::ontology::OntologyMeta {
                    name: stringify!($ont_name),
                    module_path: module_path!(),
                }
            }
        }
    };
}
