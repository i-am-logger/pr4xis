// Registration macros for ontologies and related entities.
//
// The `ontology!` proc macro (in `pr4xis-derive`) is the canonical
// surface for defining an ontology. The declarative `define_ontology!`
// macro that used to live here was deleted per #168 (one macro path).
//
// What remains here: registration helpers, axiom-meta emission, and
// per-entity `register_*!` macros that splice metadata into the global
// distributed slices.

/// Manually register an ontology's Vocabulary into the global registry.
///
/// Used by ontologies that provide `Category`/`Concept` impls manually
/// (not via the `ontology!` macro). On native targets, emits a
/// `#[distributed_slice]` entry so the ontology shows up in
/// `describe_knowledge_base()`. On `wasm32`, this is a no-op (linkme
/// is unsupported there; wasm consumers build the registry via
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
                )
            };
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
/// ```text
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

            fn meta() -> $crate::ontology::meta::Provenance {
                $crate::ontology::meta::Provenance {
                    name: $crate::ontology::meta::OntologyName::new_static(stringify!($name)),
                    description: $crate::ontology::meta::Label::new_static(stringify!($name)),
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
            static [<_REGISTER_FUNCTOR_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                <$name as $crate::category::Functor>::meta;
            // Connection constructor — the finite action-on-generators this
            // functor induces, the 1-cell analogue of `register_axiom!`'s
            // `AXIOM_CONSTRUCTORS` arm. A projection reads it to serialize the
            // functor as a content-addressed cross-ontology `Connection`.
            #[$crate::linkme::distributed_slice($crate::ontology::FUNCTOR_CONSTRUCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_FUNCTOR_CTOR_ $name:snake:upper>]: fn() -> $crate::category::ConnectionGenerators =
                $crate::category::extract_functor::<$name>;
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
/// ```text
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

            fn meta() -> $crate::ontology::meta::Provenance {
                $crate::ontology::meta::Provenance {
                    name: $crate::ontology::meta::OntologyName::new_static(stringify!($name)),
                    description: $crate::ontology::meta::Label::new_static(stringify!($name)),
                    citation: $crate::ontology::meta::Citation::parse_static($citation),
                    module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
                }
            }
        }

        // Higher cells (functor, adjunction, nat-trans) are not `Arrow`
        // instances — `Arrow` is the 0-cell-to-0-cell morphism trait.
        // `Adjunction`'s own `fn meta()` (type-level) carries provenance.

        // Auto-register into the ADJUNCTIONS distributed slice (native only).
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::ADJUNCTIONS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_ADJUNCTION_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                <$name as $crate::category::Adjunction>::meta;
            // Connection constructor — the four finite tables (both functors'
            // object maps + unit/counit families) this adjunction induces.
            #[$crate::linkme::distributed_slice($crate::ontology::ADJUNCTION_CONSTRUCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_ADJUNCTION_CTOR_ $name:snake:upper>]: fn() -> $crate::category::ConnectionGenerators =
                $crate::category::extract_adjunction::<$name>;
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
/// ```text
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

            fn meta() -> $crate::ontology::meta::Provenance {
                $crate::ontology::meta::Provenance {
                    name: $crate::ontology::meta::OntologyName::new_static(stringify!($name)),
                    description: $crate::ontology::meta::Label::new_static(stringify!($name)),
                    citation: $crate::ontology::meta::Citation::parse_static($citation),
                    module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
                }
            }
        }

        // Higher cells (functor, nat-trans, adjunction) are not `Arrow`
        // instances — `Arrow` is the 0-cell-to-0-cell morphism trait.
        // `NaturalTransformation`'s own `fn meta()` (type-level) carries provenance.

        // Auto-register into the NATURAL_TRANSFORMATIONS distributed slice.
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::NATURAL_TRANSFORMATIONS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_NAT_TRANS_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                <$name as $crate::category::NaturalTransformation>::meta;
            // Connection constructor — the component family `η_A : F(A) → G(A)`.
            #[$crate::linkme::distributed_slice($crate::ontology::NATURAL_TRANSFORMATION_CONSTRUCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_NAT_TRANS_CTOR_ $name:snake:upper>]: fn() -> $crate::category::ConnectionGenerators =
                $crate::category::extract_natural_transformation::<$name>;
        }
    };
}

/// Register a hand-written `impl Axiom for X` into the global AXIOMS
/// distributed slice so the Lemon lexicon sees it without rewriting the
/// impl block itself.
///
/// Citation-required (#167): the second argument is the literature
/// citation — no zero-citation form exists. If an axiom has no
/// literature backing, collapse it into a parent concept or remove it.
///
/// # Example
///
/// ```text
/// pub struct NoCycles;
/// impl Axiom for NoCycles { ... }
/// pr4xis::register_axiom!(NoCycles, "Guarino (2009); Gruber (1993)");
/// ```
#[macro_export]
macro_rules! register_axiom {
    // Citation-required: axiom's name comes from its type identity;
    // the literal citation is the surrounding file's literature source.
    // Per `feedback_citation_required` (#167), no citation-less form —
    // every axiom is literature-grounded or it's not an axiom.
    ($name:ident, $citation:literal) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::AXIOMS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_AXIOM_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                || $crate::ontology::meta::Provenance {
                    name: $crate::ontology::meta::OntologyName::new_static(stringify!($name)),
                    description: $crate::ontology::meta::Label::new_static(stringify!($name)),
                    citation: $crate::ontology::meta::Citation::parse_static($citation),
                    module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
                };
        }
    };
    // Instance-propagating — calls the instance's `meta()` method so any
    // description / citation declared inside `impl Axiom` propagates.
    ($name:ident, instance: $instance:expr) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::AXIOMS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_AXIOM_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                || <$name as $crate::logic::axiom::Axiom>::meta(&$instance);
        }
    };
    // Constructor arm — registers the metadata (from the axiom's `meta()`)
    // AND a re-bind constructor into AXIOM_CONSTRUCTORS, so a persisted
    // `AxiomNode` can re-bind to a freshly-built predicate by its stable
    // name on load. `$name` must be a unit-struct axiom (constructible from
    // its identifier). The metadata name and the reconstructed axiom's
    // `name()` are the same value, so `axiom_by_name` round-trips.
    ($name:ident, constructor) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::AXIOMS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_AXIOM_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                || <$name as $crate::logic::axiom::Axiom>::meta(&$name);
            #[$crate::linkme::distributed_slice($crate::ontology::AXIOM_CONSTRUCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_AXIOM_CTOR_ $name:snake:upper>]: fn() -> $crate::ontology::BoxedAxiom =
                || $crate::ontology::boxed_axiom($name);
        }
    };
}

/// Register a hand-written `impl Functor for X` into the FUNCTORS slice — and
/// its connection constructor into FUNCTOR_CONSTRUCTORS, so the functor is
/// serialized as a content-addressed `Connection` (mirrors `register_axiom!`'s
/// constructor arm).
#[macro_export]
macro_rules! register_functor {
    ($name:ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::FUNCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_FUNCTOR_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                <$name as $crate::category::Functor>::meta;
            #[$crate::linkme::distributed_slice($crate::ontology::FUNCTOR_CONSTRUCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_FUNCTOR_CTOR_ $name:snake:upper>]: fn() -> $crate::category::ConnectionGenerators =
                $crate::category::extract_functor::<$name>;
        }
    };
    ($name:ident, $citation:literal) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::FUNCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_FUNCTOR_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                || $crate::ontology::meta::Provenance {
                    name: $crate::ontology::meta::OntologyName::new_static(stringify!($name)),
                    description: $crate::ontology::meta::Label::new_static(stringify!($name)),
                    citation: $crate::ontology::meta::Citation::parse_static($citation),
                    module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
                };
            #[$crate::linkme::distributed_slice($crate::ontology::FUNCTOR_CONSTRUCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_FUNCTOR_CTOR_ $name:snake:upper>]: fn() -> $crate::category::ConnectionGenerators =
                $crate::category::extract_functor::<$name>;
        }
    };
}

/// Register a hand-written `impl Adjunction for X` into the ADJUNCTIONS slice —
/// and its connection constructor into ADJUNCTION_CONSTRUCTORS.
#[macro_export]
macro_rules! register_adjunction {
    ($name:ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::ADJUNCTIONS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_ADJUNCTION_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                <$name as $crate::category::Adjunction>::meta;
            #[$crate::linkme::distributed_slice($crate::ontology::ADJUNCTION_CONSTRUCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_ADJUNCTION_CTOR_ $name:snake:upper>]: fn() -> $crate::category::ConnectionGenerators =
                $crate::category::extract_adjunction::<$name>;
        }
    };
    ($name:ident, $citation:literal) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::ADJUNCTIONS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_ADJUNCTION_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                || $crate::ontology::meta::Provenance {
                    name: $crate::ontology::meta::OntologyName::new_static(stringify!($name)),
                    description: $crate::ontology::meta::Label::new_static(stringify!($name)),
                    citation: $crate::ontology::meta::Citation::parse_static($citation),
                    module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
                };
            #[$crate::linkme::distributed_slice($crate::ontology::ADJUNCTION_CONSTRUCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_ADJUNCTION_CTOR_ $name:snake:upper>]: fn() -> $crate::category::ConnectionGenerators =
                $crate::category::extract_adjunction::<$name>;
        }
    };
}

/// Register a hand-written `impl NaturalTransformation for X` into the slice —
/// and its connection constructor into NATURAL_TRANSFORMATION_CONSTRUCTORS.
#[macro_export]
macro_rules! register_natural_transformation {
    ($name:ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::NATURAL_TRANSFORMATIONS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_NAT_TRANS_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                <$name as $crate::category::NaturalTransformation>::meta;
            #[$crate::linkme::distributed_slice($crate::ontology::NATURAL_TRANSFORMATION_CONSTRUCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_NAT_TRANS_CTOR_ $name:snake:upper>]: fn() -> $crate::category::ConnectionGenerators =
                $crate::category::extract_natural_transformation::<$name>;
        }
    };
    ($name:ident, $citation:literal) => {
        #[cfg(not(target_arch = "wasm32"))]
        $crate::paste::paste! {
            #[$crate::linkme::distributed_slice($crate::ontology::NATURAL_TRANSFORMATIONS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_NAT_TRANS_ $name:snake:upper>]: fn() -> $crate::ontology::meta::Provenance =
                || $crate::ontology::meta::Provenance {
                    name: $crate::ontology::meta::OntologyName::new_static(stringify!($name)),
                    description: $crate::ontology::meta::Label::new_static(stringify!($name)),
                    citation: $crate::ontology::meta::Citation::parse_static($citation),
                    module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
                };
            #[$crate::linkme::distributed_slice($crate::ontology::NATURAL_TRANSFORMATION_CONSTRUCTORS)]
            #[linkme(crate = $crate::linkme)]
            static [<_REGISTER_NAT_TRANS_CTOR_ $name:snake:upper>]: fn() -> $crate::category::ConnectionGenerators =
                $crate::category::extract_natural_transformation::<$name>;
        }
    };
}

/// Unified helper: write the `meta()` associated function for a hand-written
/// `impl Functor`, `impl Adjunction`, or `impl NaturalTransformation` with a
/// literature citation in one line. Replaces the three parallel helpers
/// (`functor_meta!`, `adjunction_meta!`, `natural_transformation_meta!`) —
/// all three cell-levels of Cat share one metadata shape now (issue #153).
///
/// # Example
///
/// ```text
/// impl Functor for MyFunctor {
///     type Source = ...;
///     type Target = ...;
///     fn map_object(...) -> ... { ... }
///     fn map_morphism(...) -> ... { ... }
///     pr4xis::relationship_meta!("MyFunctor", "Mac Lane (1971) Ch. II §1");
/// }
/// ```
/// Emit the three `Axiom` override methods — `name`, `description`,
/// `citation` — inside an `impl Axiom for X { ... }` block. The common
/// case is three string literals: name, description, citation.
///
/// # Example
///
/// ```text
/// impl Axiom for NoCycles {
///     fn verify(&self) -> Verdict { ... }
///     pr4xis::axiom_meta!(
///         "NoCycles[Taxonomy]",
///         "taxonomy has no cycles (is a DAG)",
///         "Guarino (2009); Gruber (1993)"
///     );
/// }
/// ```
///
/// The macro expands to:
///
/// ```text
/// fn name(&self) -> OntologyName { OntologyName::new_static(name_literal) }
/// fn description(&self) -> Label { Label::new_static(description_literal) }
/// fn citation(&self) -> Citation { Citation::parse_static(citation_literal) }
/// ```
///
/// Citation is required on every `Axiom` impl — the two-arg form (no
/// description) defaults the description to the name literal; there is
/// no zero-citation form.
#[macro_export]
macro_rules! axiom_meta {
    ($name:literal, $description:literal, $citation:literal) => {
        fn name(&self) -> $crate::ontology::meta::OntologyName {
            $crate::ontology::meta::OntologyName::new_static($name)
        }
        fn description(&self) -> $crate::ontology::meta::Label {
            $crate::ontology::meta::Label::new_static($description)
        }
        fn citation(&self) -> $crate::ontology::meta::Citation {
            $crate::ontology::meta::Citation::parse_static($citation)
        }
    };
    ($name:literal, $citation:literal) => {
        fn name(&self) -> $crate::ontology::meta::OntologyName {
            $crate::ontology::meta::OntologyName::new_static($name)
        }
        fn description(&self) -> $crate::ontology::meta::Label {
            $crate::ontology::meta::Label::new_static($name)
        }
        fn citation(&self) -> $crate::ontology::meta::Citation {
            $crate::ontology::meta::Citation::parse_static($citation)
        }
    };
}

#[macro_export]
macro_rules! relationship_meta {
    ($name:literal, $description:literal, $citation:literal) => {
        fn meta() -> $crate::ontology::meta::Provenance {
            $crate::ontology::meta::Provenance {
                name: $crate::ontology::meta::OntologyName::new_static($name),
                description: $crate::ontology::meta::Label::new_static($description),
                citation: $crate::ontology::meta::Citation::parse_static($citation),
                module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
            }
        }
    };
    ($name:literal, $citation:literal) => {
        fn meta() -> $crate::ontology::meta::Provenance {
            $crate::ontology::meta::Provenance {
                name: $crate::ontology::meta::OntologyName::new_static($name),
                description: $crate::ontology::meta::Label::new_static($name),
                citation: $crate::ontology::meta::Citation::parse_static($citation),
                module_path: $crate::ontology::meta::ModulePath::new_static(module_path!()),
            }
        }
    };
}
