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
///         being: AbstractObject,
///         source: "Euclid; Hilbert (1899)",
///
///         is_a: BiologicalTaxonomy [
///             (Cell, Tissue),
///         ],
///     }
/// }
/// ```
///
/// `being:` classifies per DOLCE (Masolo et al., WonderWeb D18, 2003).
/// `source:` captures the primary citation.
/// Both are optional. When present, they flow into `fn vocabulary()`.
/// Manually register an ontology's Vocabulary into the global registry.
///
/// Used by ontologies that provide Category/Entity impls manually (not via
/// `define_ontology!` / `ontology!` macro). On native targets, emits a
/// `#[distributed_slice]` entry so the ontology shows up in
/// `describe_knowledge_base()`. On wasm32, this is a no-op (linkme is
/// unsupported there; wasm consumers build the registry via
/// `pr4xis::ontology::registry::collect_all`).
#[macro_export]
macro_rules! register_manual {
    (
        ident: $ident:ident,
        category: $cat:ty,
        entity: $entity:ty,
        name: $name:expr,
        module: $module:expr,
        source: $source:expr,
        being: $being:ident,
    ) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::VOCABULARIES)]
            #[linkme(crate = $crate::linkme)]
            static [<_MANUAL_REGISTER_ $ident>]: fn() -> $crate::ontology::Vocabulary = || {
                $crate::ontology::Vocabulary::from_ontology::<$cat, $entity>(
                    $name,
                    $module,
                    $source,
                    Some($crate::ontology::upper::being::Being::$being),
                )
            };
        }
    };
}

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
            $(source: $source:expr,)?

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
            $(source: $source,)?
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
            $(source: $source:expr,)?

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
            $(source: $source,)?
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
            $(source: $source:expr,)?

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
            $(source: $source,)?
            $(is_a: $tax_name [ $(($tax_child, $tax_parent)),* ],)?
            $(has_a: $mer_name [ $(($mer_whole, $mer_part)),* ],)?
            $(causes: $caus_name for $caus_entity [ $(($caus_cause, $caus_effect)),* ],)?
            $(opposes: $opp_name [ $(($opp_a, $opp_b)),* ],)?
        );
    };

    // =========================================================================
    // Internal: generate reasoning systems + structural axioms + meta + vocabulary
    // =========================================================================
    (@reasoning $ont_name:ident, $cat_name:ident, $entity:ident,
        $(being: $being:ident,)?
        $(source: $source:expr,)?
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

            /// Runtime Vocabulary — instance of Knowledge::Vocabulary (VoID).
            #[allow(dead_code, unused_assignments)]
            pub fn vocabulary() -> $crate::ontology::Vocabulary {
                let mut _being: Option<$crate::ontology::upper::being::Being> = None;
                $(
                    _being = Some($crate::ontology::upper::being::Being::$being);
                )?
                let mut _source: &'static str = "";
                $(
                    _source = $source;
                )?
                $crate::ontology::Vocabulary::from_static::<$cat_name, $entity>(
                    $crate::ontology::OntologyName::new_static(stringify!($ont_name)),
                    $crate::ontology::ModulePath::new_static(module_path!()),
                    $crate::ontology::Citation::parse_static(_source),
                    _being,
                )
            }
        }

        // Auto-register this ontology into the global VOCABULARIES slice.
        // Collected at link time via linkme::distributed_slice.
        // On wasm32, linkme is unsupported — registration is skipped.
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::VOCABULARIES)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_ $ont_name:snake:upper>]: fn() -> $crate::ontology::Vocabulary = $ont_name::vocabulary;
        }
    };
}

/// Declare a functor between two categories, with Lemon-style metadata.
///
/// Issue #148: functors live *between* ontologies, so they get their own
/// macro (sibling to `ontology!`, not a clause inside it). The macro
/// emits:
///
/// - a unit struct with the given name
/// - `impl Functor<Source = ..., Target = ...>` with the user's object
///   and morphism mappings
/// - `FunctorMeta` (name + citation + module_path) wired into the
///   trait's `meta()` override
///
/// # Example
///
/// ```ignore
/// pr4xis::functor! {
///     name: SomeFunctor,
///     source: SourceCategory,
///     target: TargetCategory,
///     citation: "Kephart & Chess (2003); Mac Lane (1971) Ch. II §1",
///     map_object: |obj| -> SomeTargetConcept { /* ... */ },
///     map_morphism: |m| -> SomeTargetRelation { /* ... */ },
/// }
/// ```
///
/// `map_object` and `map_morphism` accept any Rust expression — typically
/// a `|arg| { ... }` closure or a `|arg| expr` shorthand. The macro
/// inlines them in the trait's required methods.
#[macro_export]
macro_rules! functor {
    (
        name: $name:ident,
        source: $source:ty,
        target: $target:ty,
        citation: $citation:literal,
        map_object: $map_obj:expr,
        map_morphism: $map_morph:expr $(,)?
    ) => {
        pub struct $name;

        impl $crate::category::Functor for $name {
            type Source = $source;
            type Target = $target;

            fn map_object(
                obj: &<$source as $crate::category::Category>::Object,
            ) -> <$target as $crate::category::Category>::Object {
                let f: fn(
                    &<$source as $crate::category::Category>::Object,
                ) -> <$target as $crate::category::Category>::Object = $map_obj;
                f(obj)
            }

            fn map_morphism(
                m: &<$source as $crate::category::Category>::Morphism,
            ) -> <$target as $crate::category::Category>::Morphism {
                let f: fn(
                    &<$source as $crate::category::Category>::Morphism,
                ) -> <$target as $crate::category::Category>::Morphism = $map_morph;
                f(m)
            }

            fn meta() -> $crate::category::FunctorMeta {
                $crate::category::FunctorMeta {
                    name: $crate::ontology::meta::OntologyName::new_static(stringify!($name)),
                    citation: $crate::ontology::meta::Citation::parse_static($citation),
                    module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
                }
            }
        }

        // Auto-register into the FUNCTORS distributed slice (native only).
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::FUNCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_FUNCTOR_ $name:snake:upper>]: fn() -> $crate::category::FunctorMeta =
                <$name as $crate::category::Functor>::meta;
        }
    };
}

/// Declare an adjunction F ⊣ G, with Lemon-style metadata.
///
/// Issue #148: adjunctions live *between* two functors — their own
/// structural object. The macro emits:
///
/// - a unit struct with the given name
/// - `impl Adjunction<Left = F, Right = G>` with the user's unit and
///   counit component functions
/// - `AdjunctionMeta` (name + citation + module_path) in the trait's
///   `meta()` override
///
/// # Example
///
/// ```ignore
/// pr4xis::adjunction! {
///     name: ParseGenerate,
///     left: ParseFunctor,
///     right: GenerateFunctor,
///     citation: "de Groote (2001); Lambek & Scott (1986)",
///     unit: |obj| { /* A → G(F(A)) */ },
///     counit: |obj| { /* F(G(B)) → B */ },
/// }
/// ```
#[macro_export]
macro_rules! adjunction {
    (
        name: $name:ident,
        left: $left:ty,
        right: $right:ty,
        citation: $citation:literal,
        unit: $unit:expr,
        counit: $counit:expr $(,)?
    ) => {
        pub struct $name;

        impl $crate::category::Adjunction for $name {
            type Left = $left;
            type Right = $right;

            fn unit(
                obj: &<<$left as $crate::category::Functor>::Source as $crate::category::Category>::Object,
            ) -> <<$left as $crate::category::Functor>::Source as $crate::category::Category>::Morphism {
                let f: fn(
                    &<<$left as $crate::category::Functor>::Source as $crate::category::Category>::Object,
                ) -> <<$left as $crate::category::Functor>::Source as $crate::category::Category>::Morphism = $unit;
                f(obj)
            }

            fn counit(
                obj: &<<$left as $crate::category::Functor>::Target as $crate::category::Category>::Object,
            ) -> <<$left as $crate::category::Functor>::Target as $crate::category::Category>::Morphism {
                let f: fn(
                    &<<$left as $crate::category::Functor>::Target as $crate::category::Category>::Object,
                ) -> <<$left as $crate::category::Functor>::Target as $crate::category::Category>::Morphism = $counit;
                f(obj)
            }

            fn meta() -> $crate::category::AdjunctionMeta {
                $crate::category::AdjunctionMeta {
                    name: $crate::ontology::meta::OntologyName::new_static(stringify!($name)),
                    citation: $crate::ontology::meta::Citation::parse_static($citation),
                    module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
                }
            }
        }

        // Auto-register into the ADJUNCTIONS distributed slice (native only).
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::ADJUNCTIONS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_ADJUNCTION_ $name:snake:upper>]: fn() -> $crate::category::AdjunctionMeta =
                <$name as $crate::category::Adjunction>::meta;
        }
    };
}

/// Declare a natural transformation η: F ⇒ G, with Lemon-style metadata.
///
/// Issue #148: natural transformations live *between* two functors — a
/// distinct structural object. The macro emits a unit struct plus
/// `impl NaturalTransformation` with the user's component function and
/// `NaturalTransformationMeta`.
///
/// # Example
///
/// ```ignore
/// pr4xis::natural_transformation! {
///     name: Reflexivity,
///     from: IdentityFunctor,
///     to:   SyncolatorFunctor,
///     citation: "Heim; von Foerster (1981) eigenform",
///     component: |obj| { /* ... */ },
/// }
/// ```
#[macro_export]
macro_rules! natural_transformation {
    (
        name: $name:ident,
        from: $from:ty,
        to: $to:ty,
        citation: $citation:literal,
        component: $component:expr $(,)?
    ) => {
        pub struct $name;

        impl $crate::category::NaturalTransformation for $name {
            type SourceFunctor = $from;
            type TargetFunctor = $to;

            fn component(
                obj: &<<$from as $crate::category::Functor>::Source as $crate::category::Category>::Object,
            ) -> <<$from as $crate::category::Functor>::Target as $crate::category::Category>::Morphism {
                let f: fn(
                    &<<$from as $crate::category::Functor>::Source as $crate::category::Category>::Object,
                ) -> <<$from as $crate::category::Functor>::Target as $crate::category::Category>::Morphism = $component;
                f(obj)
            }

            fn meta() -> $crate::category::NaturalTransformationMeta {
                $crate::category::NaturalTransformationMeta {
                    name: $crate::ontology::meta::OntologyName::new_static(stringify!($name)),
                    citation: $crate::ontology::meta::Citation::parse_static($citation),
                    module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
                }
            }
        }

        // Auto-register into the NATURAL_TRANSFORMATIONS distributed slice.
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::NATURAL_TRANSFORMATIONS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_NAT_TRANS_ $name:snake:upper>]: fn() -> $crate::category::NaturalTransformationMeta =
                <$name as $crate::category::NaturalTransformation>::meta;
        }
    };
}

/// Register a hand-written `impl Axiom for X` into the global AXIOMS
/// distributed slice so the Lemon lexicon sees it without rewriting the
/// impl block itself. Useful for existing impls that don't yet use the
/// `axioms:` clause of `ontology!`.
///
/// # Example
///
/// ```ignore
/// pub struct MyAxiom;
/// impl Axiom for MyAxiom {
///     fn description(&self) -> &str { "..." }
///     fn holds(&self) -> bool { ... }
///     pr4xis::axiom_meta!("MyAxiom", "Smith (1999)");
/// }
/// pr4xis::register_axiom!(MyAxiom);
/// ```
#[macro_export]
macro_rules! register_axiom {
    // Registration by type-name identity (no instance needed — works for
    // unit structs and structs-with-fields). The registry entry reports
    // the axiom's identity via `std::any::type_name` + an empty citation.
    // Axioms that want their declared citation in the registry should
    // add an explicit `meta()` override via the `axiom_meta!` helper AND
    // register with `register_axiom!(Name, &instance)` instead.
    ($name:ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::AXIOMS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_AXIOM_ $name:snake:upper>]: fn() -> $crate::logic::axiom::AxiomMeta =
                || $crate::logic::axiom::AxiomMeta {
                    name: $crate::ontology::meta::OntologyName::new_static(stringify!($name)),
                    citation: $crate::ontology::meta::Citation::EMPTY,
                    module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
                };
        }
    };
    // Registration with an explicit instance expression — calls the
    // instance's `meta()` method so declared citations propagate.
    ($name:ident, $instance:expr) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::AXIOMS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_AXIOM_ $name:snake:upper>]: fn() -> $crate::logic::axiom::AxiomMeta =
                || <$name as $crate::logic::axiom::Axiom>::meta(&$instance);
        }
    };
}

/// Register a hand-written `impl Functor for X` into the FUNCTORS slice.
#[macro_export]
macro_rules! register_functor {
    ($name:ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::FUNCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_FUNCTOR_ $name:snake:upper>]: fn() -> $crate::category::FunctorMeta =
                <$name as $crate::category::Functor>::meta;
        }
    };
}

/// Register a hand-written `impl Adjunction for X` into the ADJUNCTIONS slice.
#[macro_export]
macro_rules! register_adjunction {
    ($name:ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::ADJUNCTIONS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_ADJUNCTION_ $name:snake:upper>]: fn() -> $crate::category::AdjunctionMeta =
                <$name as $crate::category::Adjunction>::meta;
        }
    };
}

/// Register a hand-written `impl NaturalTransformation for X` into the slice.
#[macro_export]
macro_rules! register_natural_transformation {
    ($name:ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::NATURAL_TRANSFORMATIONS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_NAT_TRANS_ $name:snake:upper>]: fn() -> $crate::category::NaturalTransformationMeta =
                <$name as $crate::category::NaturalTransformation>::meta;
        }
    };
}
