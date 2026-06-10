//! [`ConnectionGenerators`] — the finite action-on-generators of a connection
//! (functor / adjunction / natural transformation), extracted from a compiled
//! arrow at registration time.
//!
//! # The finite-presentation theorem made concrete
//!
//! By Lawvere functorial semantics (Lawvere 1963; Mac Lane 1971 CWM Ch. V), a
//! structure-preserving map out of a finitely-presented category is *fully
//! determined by its action on generators*. The generators of an `ontology!`
//! category are its concept variants ([`FinitelyGenerated::variants`]) and its
//! relation-kinds; a [`Functor`] is therefore
//! recoverable from the finite table `(source generator → target image)` it
//! induces. This struct IS that table, computed once per registered arrow.
//!
//! # Why `pr4xis`-native (not the runtime `Connection`)
//!
//! The serialized, content-addressed `Connection` lives in `pr4xis-runtime`,
//! which depends on `pr4xis` — so `pr4xis` cannot name it without a cycle. The
//! registry (here, in `pr4xis`) instead emits this language: the typed
//! source/target [`OntologyName`]s plus the finite generator tables. The
//! `pr4xis-runtime::emit` projection (the only place with both crates in scope)
//! translates [`ConnectionGenerators`] into the wire `Connection` and
//! content-addresses it.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use super::adjunction::Adjunction;
use super::arrow::Arrow;
use super::category::Category;
use super::entity::{Concept, FinitelyGenerated};
use super::functor::Functor;
use super::named::NamedCategory;
use super::transformation::NaturalTransformation;
use crate::ontology::meta::OntologyName;

/// Which categorical family an extracted connection belongs to. The family
/// fixes which generator tables are populated; the `kind` name (a refinement,
/// e.g. `FullyFaithful`) travels alongside in [`ConnectionGenerators`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionFamily {
    /// A functor `F: C → D` — object and morphism-kind tables.
    Functor {
        /// `(source object name, target object name)`, one row per generator.
        map_object: Vec<(String, String)>,
        /// `(source relation-kind name, target relation-kind name)`.
        map_morphism: Vec<(String, String)>,
    },
    /// A natural transformation `η: F ⇒ G` — one component per source object.
    NaturalTransformation {
        /// `(object name, component-morphism rendering)`.
        components: Vec<(String, String)>,
    },
    /// An adjunction `F ⊣ G` — both functors' object tables plus the unit `η`
    /// and counit `ε` component families over the respective objects.
    Adjunction {
        /// Left adjoint `F`'s object table `(C-object, D-object)`.
        left_map_object: Vec<(String, String)>,
        /// Right adjoint `G`'s object table `(D-object, C-object)`.
        right_map_object: Vec<(String, String)>,
        /// Unit `η_A : A → G(F(A))` rendered per source object A.
        unit: Vec<(String, String)>,
        /// Counit `ε_B : F(G(B)) → B` rendered per target object B.
        counit: Vec<(String, String)>,
    },
}

/// The complete finite presentation of one registered connection: its typed
/// source/target ontology names, its kind refinement, the laws it must satisfy,
/// and the family-specific action-on-generators.
///
/// This is what a `FUNCTOR_CONSTRUCTORS` / `ADJUNCTION_CONSTRUCTORS` /
/// `NATURAL_TRANSFORMATION_CONSTRUCTORS` slice entry reconstructs — the mirror
/// of `AXIOM_CONSTRUCTORS` reconstructing a runnable axiom, but here the
/// reconstruction is the *extracted finite action* a projection serializes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionGenerators {
    /// The connection's own stable name (the functor/adjunction/nat-trans
    /// identifier, from its `meta().name`).
    pub name: OntologyName,
    /// The source ontology name — the category the arrow maps *out of*.
    pub source: OntologyName,
    /// The target ontology name — the category the arrow maps *into*.
    pub target: OntologyName,
    /// The kind refinement (e.g. `Faithful`, `FullyFaithful`, `FreeForgetful`).
    pub kind: String,
    /// The laws this connection must satisfy, by name (functor laws, the
    /// triangle identities, …).
    pub laws: Vec<String>,
    /// The family + its finite action-on-generators tables.
    pub family: ConnectionFamily,
}

/// The finite `(source relation-kind → target relation-kind)` table a functor
/// induces, by mapping one representative morphism of each distinct source kind
/// and reading the image's kind. Kind names are rendered from the typed
/// `Arrow::Kind` via `Debug` — the same OBO-RO "relation is its variant
/// identifier" convention `Vocabulary::from_static` and the emit projection use,
/// never a hand-written string.
fn functor_map_morphism_table<F>() -> Vec<(String, String)>
where
    F: Functor,
    <F::Source as Category>::Morphism: PartialEq,
{
    let mut seen: Vec<<<F as Functor>::Source as Category>::Morphism> = Vec::new();
    let mut table: Vec<(String, String)> = Vec::new();
    for m in <F::Source as Category>::morphisms() {
        let src_kind = format!("{:?}", m.kind());
        // One representative per distinct source kind — the action on a kind is
        // determined by the action on any of its morphisms (a kinded functor
        // maps every morphism of a kind uniformly, per the macro-emitted
        // `map_morphism` by-kind translation).
        if seen.iter().any(|s| format!("{:?}", s.kind()) == src_kind) {
            continue;
        }
        seen.push(m.clone());
        let image = F::map_morphism(&m);
        let tgt_kind = format!("{:?}", image.kind());
        table.push((src_kind, tgt_kind));
    }
    table
}

/// Extract a functor's finite action-on-generators into a
/// [`ConnectionGenerators`]. The object table maps every source concept variant
/// to its image's name; the morphism table maps every source relation-kind to
/// its image kind. The laws are the functor laws (refined to the
/// faithful/full-on-image pair when the kind declares a fully-faithful
/// embedding), named to match the runnable axioms in
/// [`crate::category::laws`].
pub fn extract_functor<F>() -> ConnectionGenerators
where
    F: Functor,
    F::Source: NamedCategory,
    F::Target: NamedCategory,
    <F::Source as Category>::Object: FinitelyGenerated,
    <F::Source as Category>::Morphism: PartialEq,
{
    use super::kinds::FunctorKind;

    let map_object: Vec<(String, String)> =
        <<F::Source as Category>::Object as FinitelyGenerated>::variants()
            .iter()
            .map(|obj| {
                (
                    obj.name().to_string(),
                    F::map_object(obj).name().to_string(),
                )
            })
            .collect();
    let map_morphism = functor_map_morphism_table::<F>();

    // Functor laws (Mac Lane 1971 CWM Ch. II §1) always apply; a fully-faithful
    // embedding additionally witnesses the faithful + full-onto-image pair
    // (Ch. I §4). Names match the `Axiom` impls so a loader can resolve them.
    let mut laws = vec![
        "FunctorIdentityLaw".to_string(),
        "FunctorCompositionLaw".to_string(),
    ];
    if matches!(F::KIND, FunctorKind::FullyFaithful) {
        laws.push("FunctorFaithfulLaw".to_string());
        laws.push("FunctorFullOnImageLaw".to_string());
    } else if matches!(F::KIND, FunctorKind::Faithful) {
        laws.push("FunctorFaithfulLaw".to_string());
    }

    ConnectionGenerators {
        name: F::meta().name,
        source: <F::Source as NamedCategory>::ontology_name(),
        target: <F::Target as NamedCategory>::ontology_name(),
        kind: format!("{:?}", F::KIND),
        laws,
        family: ConnectionFamily::Functor {
            map_object,
            map_morphism,
        },
    }
}

/// Extract a natural transformation's finite component family `η_A : F(A) → G(A)`
/// — one component per source object, rendered from the component morphism's
/// own `meta().name` (its directed-arrow identity). Source/target are the two
/// functors' shared endpoint categories (`F, G : C → D`); the connection runs
/// `C → D` like the functors it transforms.
pub fn extract_natural_transformation<N>() -> ConnectionGenerators
where
    N: NaturalTransformation,
    <N::SourceFunctor as Functor>::Source: NamedCategory,
    <N::SourceFunctor as Functor>::Target: NamedCategory,
    <<N::SourceFunctor as Functor>::Source as Category>::Object: FinitelyGenerated,
{
    let components: Vec<(String, String)> = <<<N::SourceFunctor as Functor>::Source as Category>::Object as FinitelyGenerated>::variants()
        .iter()
        .map(|obj| {
            let comp = N::component(obj);
            (obj.name().to_string(), comp.meta().name.as_str().to_string())
        })
        .collect();

    ConnectionGenerators {
        name: N::meta().name,
        source: <<N::SourceFunctor as Functor>::Source as NamedCategory>::ontology_name(),
        target: <<N::SourceFunctor as Functor>::Target as NamedCategory>::ontology_name(),
        kind: format!("{:?}", N::KIND),
        laws: vec!["NaturalityLaw".to_string()],
        family: ConnectionFamily::NaturalTransformation { components },
    }
}

/// Extract an adjunction `F ⊣ G` into its four finite tables: the left and
/// right functors' object maps and the unit/counit component families. The
/// connection's `source`/`target` are C and D (the categories `F : C → D`
/// bridges); a peer reconstructs the round-trip from the four tables.
pub fn extract_adjunction<A>() -> ConnectionGenerators
where
    A: Adjunction,
    <A::Left as Functor>::Source: NamedCategory,
    <A::Left as Functor>::Target: NamedCategory,
    <<A::Left as Functor>::Source as Category>::Object: FinitelyGenerated,
    <<A::Left as Functor>::Target as Category>::Object: FinitelyGenerated,
{
    type C<A> = <<A as Adjunction>::Left as Functor>::Source;
    type D<A> = <<A as Adjunction>::Left as Functor>::Target;

    let left_map_object: Vec<(String, String)> =
        <<C<A> as Category>::Object as FinitelyGenerated>::variants()
            .iter()
            .map(|a| {
                (
                    a.name().to_string(),
                    <A::Left as Functor>::map_object(a).name().to_string(),
                )
            })
            .collect();
    let right_map_object: Vec<(String, String)> =
        <<D<A> as Category>::Object as FinitelyGenerated>::variants()
            .iter()
            .map(|b| {
                (
                    b.name().to_string(),
                    <A::Right as Functor>::map_object(b).name().to_string(),
                )
            })
            .collect();
    let unit: Vec<(String, String)> = <<C<A> as Category>::Object as FinitelyGenerated>::variants()
        .iter()
        .map(|a| {
            (
                a.name().to_string(),
                A::unit(a).meta().name.as_str().to_string(),
            )
        })
        .collect();
    let counit: Vec<(String, String)> =
        <<D<A> as Category>::Object as FinitelyGenerated>::variants()
            .iter()
            .map(|b| {
                (
                    b.name().to_string(),
                    A::counit(b).meta().name.as_str().to_string(),
                )
            })
            .collect();

    ConnectionGenerators {
        name: A::meta().name,
        source: <C<A> as NamedCategory>::ontology_name(),
        target: <D<A> as NamedCategory>::ontology_name(),
        kind: format!("{:?}", A::KIND),
        laws: vec!["AdjunctionTriangleLaw".to_string()],
        family: ConnectionFamily::Adjunction {
            left_map_object,
            right_map_object,
            unit,
            counit,
        },
    }
}
