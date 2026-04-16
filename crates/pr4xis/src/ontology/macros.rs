// Declarative macro for defining ontologies.
//
// The user declares domain knowledge. The macro generates structure.
// Two API styles:
//   - Ontological: concepts, is_a, has_a, causes, opposes
//   - Named: entity, relation + taxonomy, mereology, causation, opposition
//
// Both generate: Category, reasoning systems, structural axioms, OntologyMeta.
// Domain-specific axioms and qualities stay hand-written.

/// Define an ontology from domain knowledge.
///
/// # Ontological style (preferred)
///
/// ```ignore
/// define_ontology! {
///     pub Biology for BiologyCategory {
///         concepts: BiologicalEntity,
///         relation: BiologicalRelation,
///         being: AbstractObject,    // optional — DOLCE upper-ontology classification
///
///         is_a: BiologicalTaxonomy [
///             (Cell, Tissue),
///         ],
///
///         has_a: BiologicalMereology [
///             (Organism, Organ),
///         ],
///
///         causes: BiologicalCausalGraph for BiologicalCausalEvent [
///             (Injury, Repair),
///         ],
///
///         opposes: BiologicalOpposition [
///             (Growth, Decay),
///         ],
///     }
/// }
/// ```
///
/// The `being:` clause generates `impl Classified for <Category>` so each
/// ontology classifies itself per DOLCE (Masolo et al., WonderWeb D18, 2003).
/// Variants: `AbstractObject`, `SocialObject`, `MentalObject`,
/// `PhysicalEndurant`, `Process`, `Event`, `Quality`.
#[macro_export]
macro_rules! define_ontology {
    // =========================================================================
    // Ontological style: concepts + is_a/has_a/causes/opposes (dense category)
    // =========================================================================
    (
        $(#[$ont_meta:meta])*
        pub $ont_name:ident for $cat_name:ident {
            concepts: $entity:ident,
            relation: $relation:ident,

            $(being: $being:ident,)?

            $(is_a: $tax_name:ident [
                $(($tax_child:ident, $tax_parent:ident)),* $(,)?
            ],)?

            $(has_a: $mer_name:ident [
                $(($mer_whole:ident, $mer_part:ident)),* $(,)?
            ],)?

            $(causes: $caus_name:ident for $caus_entity:ident [
                $(($caus_cause:ident, $caus_effect:ident)),* $(,)?
            ],)?

            $(opposes: $opp_name:ident [
                $(($opp_a:ident, $opp_b:ident)),* $(,)?
            ],)?
        }
    ) => {
        $crate::define_dense_category! {
            $(#[$ont_meta])*
            pub $cat_name {
                entity: $entity,
                relation: $relation,
            }
        }

        define_ontology!(@reasoning $ont_name, $cat_name, $entity,
            $(being: $being,)?
            $(is_a: $tax_name [ $(($tax_child, $tax_parent)),* ],)?
            $(has_a: $mer_name [ $(($mer_whole, $mer_part)),* ],)?
            $(causes: $caus_name for $caus_entity [ $(($caus_cause, $caus_effect)),* ],)?
            $(opposes: $opp_name [ $(($opp_a, $opp_b)),* ],)?
        );
    };

    // =========================================================================
    // Legacy named style: entity + taxonomy/mereology/causation/opposition (dense)
    // =========================================================================
    (
        $(#[$ont_meta:meta])*
        pub $ont_name:ident for $cat_name:ident {
            entity: $entity:ident,
            relation: $relation:ident,

            $(being: $being:ident,)?

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
        $crate::define_dense_category! {
            $(#[$ont_meta])*
            pub $cat_name {
                entity: $entity,
                relation: $relation,
            }
        }

        define_ontology!(@reasoning $ont_name, $cat_name, $entity,
            $(being: $being,)?
            $(is_a: $tax_name [ $(($tax_child, $tax_parent)),* ],)?
            $(has_a: $mer_name [ $(($mer_whole, $mer_part)),* ],)?
            $(causes: $caus_name for $caus_entity [ $(($caus_cause, $caus_effect)),* ],)?
            $(opposes: $opp_name [ $(($opp_a, $opp_b)),* ],)?
        );
    };

    // =========================================================================
    // Kinded style: explicit relation types (communication/information)
    // =========================================================================
    (
        $(#[$ont_meta:meta])*
        pub $ont_name:ident for $cat_name:ident {
            concepts: $entity:ident,
            relation: $relation:ident,
            kind: $kind:ident,
            kinds: [$($(#[$kind_meta:meta])* $domain_kind:ident),* $(,)?],
            edges: [$(($e_from:ident, $e_to:ident, $e_kind:ident)),* $(,)?],
            composed: [$(($c_from:ident, $c_to:ident)),* $(,)?],

            $(being: $being:ident,)?

            $(is_a: $tax_name:ident [
                $(($tax_child:ident, $tax_parent:ident)),* $(,)?
            ],)?

            $(has_a: $mer_name:ident [
                $(($mer_whole:ident, $mer_part:ident)),* $(,)?
            ],)?

            $(causes: $caus_name:ident for $caus_entity:ident [
                $(($caus_cause:ident, $caus_effect:ident)),* $(,)?
            ],)?

            $(opposes: $opp_name:ident [
                $(($opp_a:ident, $opp_b:ident)),* $(,)?
            ],)?
        }
    ) => {
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
            $(being: $being,)?
            $(is_a: $tax_name [ $(($tax_child, $tax_parent)),* ],)?
            $(has_a: $mer_name [ $(($mer_whole, $mer_part)),* ],)?
            $(causes: $caus_name for $caus_entity [ $(($caus_cause, $caus_effect)),* ],)?
            $(opposes: $opp_name [ $(($opp_a, $opp_b)),* ],)?
        );
    };

    // =========================================================================
    // Internal: generate reasoning systems + structural axioms + meta
    // =========================================================================
    (@reasoning $ont_name:ident, $cat_name:ident, $entity:ident,
        $(being: $being:ident,)?
        $(is_a: $tax_name:ident [ $(($tax_child:ident, $tax_parent:ident)),* ],)?
        $(has_a: $mer_name:ident [ $(($mer_whole:ident, $mer_part:ident)),* ],)?
        $(causes: $caus_name:ident for $caus_entity:ident [ $(($caus_cause:ident, $caus_effect:ident)),* ],)?
        $(opposes: $opp_name:ident [ $(($opp_a:ident, $opp_b:ident)),* ],)?
    ) => {
        // --- Taxonomy (is-a) ---
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

        // --- Mereology (has-a) ---
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

        // --- Causation (causes) ---
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

        // --- Opposition (opposes) ---
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

        // --- DOLCE classification (when `being:` is declared) ---
        $(
            impl $crate::ontology::upper::classify::Classified for $cat_name {
                fn being() -> $crate::ontology::upper::being::Being {
                    $crate::ontology::upper::being::Being::$being
                }
                fn classification_reason() -> &'static str {
                    concat!("DOLCE D18 ", stringify!($being), "; ", module_path!())
                }
            }
        )?

        // --- Ontology struct ---
        pub struct $ont_name;

        impl $ont_name {
            /// Structural axioms — auto-generated. Used internally by Ontology trait.
            /// Override `structural_axioms()` in your `impl Ontology` to wire this in.
            #[doc(hidden)]
            pub fn generated_structural_axioms() -> Vec<Box<dyn $crate::ontology::Axiom>> {
                let mut axioms: Vec<Box<dyn $crate::ontology::Axiom>> = Vec::new();

                $(
                    axioms.push(Box::new(
                        $crate::ontology::reasoning::taxonomy::NoCycles::<$tax_name>::new()
                    ));
                    axioms.push(Box::new(
                        $crate::ontology::reasoning::taxonomy::Antisymmetric::<$tax_name>::new()
                    ));
                )?

                $(
                    axioms.push(Box::new(
                        $crate::ontology::reasoning::mereology::NoCycles::<$mer_name>::new()
                    ));
                    // WeakSupplementation not auto-included — not all mereologies satisfy it.
                    // Add manually in domain_axioms() if your domain requires it.
                )?

                $(
                    axioms.push(Box::new(
                        $crate::ontology::reasoning::causation::Asymmetric::<$caus_name>::new()
                    ));
                    axioms.push(Box::new(
                        $crate::ontology::reasoning::causation::NoSelfCausation::<$caus_name>::new()
                    ));
                )?

                $(
                    axioms.push(Box::new(
                        $crate::ontology::reasoning::opposition::Symmetric::<$opp_name>::new()
                    ));
                    axioms.push(Box::new(
                        $crate::ontology::reasoning::opposition::Irreflexive::<$opp_name>::new()
                    ));
                )?

                axioms
            }

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
