//! Category Theory — the meta-ontology grounding pr4xis's categorical substrate.
//!
//! # Why this ontology exists
//!
//! pr4xis's Rust core defines trait and struct names (`Category`, `Arrow`,
//! `Morphism`, `Functor`, `NaturalTransformation`, `Adjunction`,
//! `Monad`, `Applicative`, `Kleisli`, `Yoneda`, `Algebra`, …) for the
//! substrate that every domain ontology sits on. Per
//! `feedback_api_ontological_from_day_one` — "ALL code uses typed
//! ontological concepts, never primitives. No exceptions." — those
//! substrate types must themselves be grounded in an ontology.
//!
//! This module IS that ontology. Every name used in `pr4xis::category::*`
//! is an instance of a concept declared here, cited to primary literature.
//!
//! # Synonymy — Morphism and Arrow
//!
//! Mac Lane uses `morphism` and `arrow` interchangeably (CWM Ch. I §1).
//! Awodey (2010) uses `arrow` as primary. In pr4xis:
//! - **One concept**: `Morphism`
//! - **Two labels**: "Morphism" (Mac Lane primary) and "Arrow" (Awodey
//!   primary / Mac Lane intuitive).
//!
//! The Rust trait is named `Arrow`; the Rust struct for a morphism
//! instance is named `Morphism`. Both surface forms point at
//! [`CategoryTheoryConcept::Morphism`].
//!
//! # Literature
//!
//! - Mac Lane (1971) *Categories for the Working Mathematician* — canonical
//! - Awodey (2010) *Category Theory* — "arrow" primary
//! - Bénabou (1967) *Introduction to Bicategories*
//! - Leinster (2004) *Higher Operads, Higher Categories*
//! - Eilenberg & Moore (1965) — monad algebras
//! - Kleisli (1965) — Kleisli category
//! - Moggi (1991) "Notions of computation and monads"
//! - Wadler (1992) "The essence of functional programming"
//! - McBride & Paterson (2008) "Applicative Programming with Effects"
//! - Meijer, Fokkinga & Paterson (1991) "Bananas, Lenses, Envelopes and Barbed Wire"
//! - Liang, Hudak & Jones (1995) "Monad Transformers and Modular Interpreters"
//! - Yoneda (1954) — Yoneda lemma
//! - Ore (1944) "Galois connexions"
//! - Gruber (1993) / Smith et al. (2005) OBO-RO — naming principles

use crate as pr4xis;
use crate::ontology::{Axiom, Ontology};

pr4xis::ontology! {
    name: "CategoryTheory",
    source: "Mac Lane (1971); Awodey (2010); Bénabou (1967); Leinster (2004); Eilenberg & Moore (1965); Kleisli (1965); Moggi (1991); Wadler (1992); McBride & Paterson (2008); Meijer-Fokkinga-Paterson (1991); Liang-Hudak-Jones (1995); Yoneda (1954); Ore (1944)",

    concepts: [
        // === Core cells (Mac Lane 1971 Ch. I) ===
        Object,
        Morphism,
        Composition,
        Identity,
        Source,
        Target,
        Kind,

        // === Specialised morphisms (Mac Lane Ch. I §5) ===
        Endomorphism,
        Isomorphism,
        Automorphism,
        Monomorphism,
        Epimorphism,

        // === Whole structures & higher cells (Mac Lane Ch. II, XII.3) ===
        CategoryStructure,
        Functor,
        Endofunctor,
        NaturalTransformation,
        Adjunction,
        Unit,
        Counit,
        Bicategory,
        TwoCategory,
        HigherCategory,

        // === Typeclass-like structures (Moggi 1991; McBride & Paterson 2008) ===
        Applicative,
        Monad,
        Comonad,
        Pure,
        Multiplication,
        Comultiplication,

        // === Monoidal + algebraic structures (Mac Lane Ch. III, VII) ===
        Semigroup,
        Monoid,
        MonoidalCategory,
        Tensor,
        Product,
        Coproduct,
        Terminal,
        InitialObject,

        // === Derived categories (Mac Lane Ch. II §2; Kleisli 1965; Eilenberg-Moore 1965) ===
        OppositeCategory,
        KleisliCategory,

        // === F-algebras (Meijer-Fokkinga-Paterson 1991) ===
        Algebra,
        Coalgebra,
        StructureMap,

        // === Specific monads (Liang-Hudak-Jones 1995; Wadler 1992) ===
        StateMonad,
        ReaderMonad,
        WriterMonad,
        FreeMonad,
        MonadTransformer,

        // === Yoneda-related (Yoneda 1954) ===
        YonedaEmbedding,
        Representable,

        // === Special adjunctions (Ore 1944) ===
        GaloisConnection,

        // === Interpretation / functor semantics (Spivak 2012 FDM;
        //     Lawvere-Rosebrugh 2003; Goguen-Burstall 1984; Lambek-Scott 1986) ===
        Interpretation,
        InstanceFunctor,
        SchemaCategory,
        Syntactic,
        Semantic,
    ],

    labels: {
        Object: ("en", "Object", "A 0-cell inside a category — the basic entity. Mac Lane (1971) CWM Ch. I §1."),
        Morphism: ("en", "Morphism / Arrow", "A 1-cell — directed structure-preserving map between objects. Mac Lane uses 'morphism' primary, 'arrow' synonym; Awodey (2010) uses 'arrow' primary."),
        Composition: ("en", "Composition", "Given f: A → B and g: B → C, produces g ∘ f: A → C. Mac Lane (1971) Ch. I §1."),
        Identity: ("en", "Identity morphism", "For every object A, id_A: A → A that is left and right neutral for composition. Mac Lane (1971) Ch. I §1."),
        Source: ("en", "Source", "The domain of a morphism — what it comes from."),
        Target: ("en", "Target", "The codomain of a morphism — what it goes to."),
        Kind: ("en", "Relation kind", "The named relation-type tag carried by every morphism per OBO-RO (Smith et al. 2005)."),

        Endomorphism: ("en", "Endomorphism", "A morphism whose source and target are the same object. Mac Lane (1971) Ch. I §5."),
        Isomorphism: ("en", "Isomorphism", "A morphism with a two-sided inverse. Mac Lane (1971) Ch. I §5."),
        Automorphism: ("en", "Automorphism", "An isomorphism that is also an endomorphism. Mac Lane (1971) Ch. I §5."),
        Monomorphism: ("en", "Monomorphism", "A left-cancellative morphism. Mac Lane (1971) Ch. I §5."),
        Epimorphism: ("en", "Epimorphism", "A right-cancellative morphism. Mac Lane (1971) Ch. I §5."),

        CategoryStructure: ("en", "Category", "Objects + morphisms + composition + identity satisfying the category laws. Mac Lane (1971) Ch. I §1."),
        Functor: ("en", "Functor", "A 1-cell in Cat — structure-preserving map between categories. Mac Lane (1971) Ch. II §1."),
        Endofunctor: ("en", "Endofunctor", "A functor F: C → C whose source and target categories coincide. Mac Lane (1971) Ch. II §1; foundation for monads/algebras."),
        NaturalTransformation: ("en", "Natural transformation", "A 2-cell in Cat — a map between parallel functors. Mac Lane (1971) Ch. II §4."),
        Adjunction: ("en", "Adjunction", "A structured pair F ⊣ G with unit and counit satisfying triangle identities. Mac Lane (1971) Ch. IV §1."),
        Unit: ("en", "Unit", "A natural transformation η: 1 ⇒ T (for a monad T) or η: 1 ⇒ G∘F (for an adjunction F ⊣ G). Mac Lane (1971) Ch. IV §1."),
        Counit: ("en", "Counit", "A natural transformation ε: T ⇒ 1 (for a comonad T) or ε: F∘G ⇒ 1 (for an adjunction). Mac Lane (1971) Ch. IV §1."),

        Bicategory: ("en", "Bicategory", "A weak 2-category — associativity/identity up to coherent 2-isomorphism. Bénabou (1967)."),
        TwoCategory: ("en", "2-category", "Cat is a 2-category: 0-cells are categories, 1-cells functors, 2-cells nat-trans. Mac Lane (1971) XII.3."),
        HigherCategory: ("en", "Higher category", "An n-category generalising 2-categories. Leinster (2004)."),

        Applicative: ("en", "Applicative functor", "An endofunctor with `pure` and `ap` — supports context-free effectful composition. McBride & Paterson (2008)."),
        Monad: ("en", "Monad", "A monoid in the category of endofunctors — (T, η, μ) satisfying unit and associativity laws. Moggi (1991); Wadler (1992); Mac Lane (1971) Ch. VI."),
        Comonad: ("en", "Comonad", "Dual of a monad — (T, ε, δ) with counit and comultiplication. Mac Lane (1971) Ch. VI §4."),
        Pure: ("en", "Pure / Return / Unit-morphism", "The lifting A → F(A) provided by an Applicative or Monad."),
        Multiplication: ("en", "Multiplication (μ)", "The monad's natural transformation μ: T∘T ⇒ T (collapse two layers into one). Moggi (1991)."),
        Comultiplication: ("en", "Comultiplication (δ)", "The comonad's natural transformation δ: T ⇒ T∘T (split into two layers)."),

        Semigroup: ("en", "Semigroup", "A set with an associative binary operation. Foundational algebraic structure, predating Monoid."),
        Monoid: ("en", "Monoid", "A semigroup with an identity element. Mac Lane (1971) Ch. III §6; monads are monoids in the endofunctor category."),
        MonoidalCategory: ("en", "Monoidal category", "A category equipped with a tensor product ⊗ and unit I, satisfying coherence laws. Mac Lane (1971) Ch. VII."),
        Tensor: ("en", "Tensor product (⊗)", "The bifunctor ⊗: C × C → C of a monoidal category. Mac Lane (1971) Ch. VII."),
        Product: ("en", "Product (×)", "A limit of a two-object diagram — paired with projections. Mac Lane (1971) Ch. III §4."),
        Coproduct: ("en", "Coproduct (+)", "Dual of product — a colimit with injections. Mac Lane (1971) Ch. III §4."),
        Terminal: ("en", "Terminal object", "An object 1 such that there is exactly one morphism A → 1 from every object. Mac Lane (1971) Ch. III §3."),
        InitialObject: ("en", "Initial object", "Dual of terminal — an object 0 with exactly one morphism 0 → A to every object. Mac Lane (1971) Ch. III §3."),

        OppositeCategory: ("en", "Opposite category (C^op)", "The dual category: same objects, morphisms reversed. Mac Lane (1971) Ch. II §2."),
        KleisliCategory: ("en", "Kleisli category", "For a monad T on C, the category whose morphisms A → B are C-morphisms A → T(B). Kleisli (1965)."),

        Algebra: ("en", "F-algebra", "Pair (A, α: F(A) → A) where F is an endofunctor. Meijer-Fokkinga-Paterson (1991); initial algebras characterise recursive types."),
        Coalgebra: ("en", "F-coalgebra", "Pair (A, α: A → F(A)) — dual of F-algebra. Characterises corecursive / infinite structures."),
        StructureMap: ("en", "Structure map", "The defining morphism of an F-(co)algebra: α: F(A) → A (algebra) or α: A → F(A) (coalgebra)."),

        StateMonad: ("en", "State monad", "S → (A, S) — threads mutable state through pure computations. Wadler (1992); Liang-Hudak-Jones (1995)."),
        ReaderMonad: ("en", "Reader monad", "R → A — reads from a fixed environment. Wadler (1992)."),
        WriterMonad: ("en", "Writer monad", "(A, W) with W a monoid — accumulates output alongside computation. Wadler (1992); pr4xis's `Traced` specialises it."),
        FreeMonad: ("en", "Free monad", "The initial algebra of the functor T(A) = A + F(T(A)) — universal among monads over F."),
        MonadTransformer: ("en", "Monad transformer", "A type constructor T such that if M is a monad then T(M) is also a monad — composes monadic effects. Liang-Hudak-Jones (1995)."),

        YonedaEmbedding: ("en", "Yoneda embedding", "The functor y: C → [C^op, Set] sending A to Hom(-, A). Fully faithful — an object IS its representable. Yoneda (1954)."),
        Representable: ("en", "Representable functor", "A functor naturally isomorphic to Hom(A, -) for some A. Yoneda (1954); Mac Lane (1971) Ch. III §2."),

        GaloisConnection: ("en", "Galois connection", "An adjunction between posets — a pair (f ⊣ g) of monotone maps on partially-ordered sets. Ore (1944); special case of Mac Lane's adjunction."),

        Interpretation: ("en", "Interpretation functor", "A functor that gives semantic meaning to a syntactic structure — maps from a syntactic/theory category to a semantic category. Lambek & Scott (1986); Goguen & Burstall (1984) institutions."),
        InstanceFunctor: ("en", "Instance functor", "Spivak (2012) FDM §3: a functor I: S → Set from a schema category S to Set — each object's image is the set of instances, each morphism's image is the function between those sets. Characterises database-instance semantics of schema categories."),
        SchemaCategory: ("en", "Schema category", "Spivak (2012) FDM: a finitely-presented category encoding a database schema — objects are tables, morphisms are foreign-key paths, path equivalences are schema constraints."),
        Syntactic: ("en", "Syntactic side", "The formal-structure source of an interpretation — schema, theory, type system. Lambek & Scott (1986) — the category where propositions/types live formally."),
        Semantic: ("en", "Semantic side", "The model / meaning-carrying target of an interpretation — Set for instances, a model category for logical semantics. Tarski (1936); Lambek-Scott (1986)."),
    },

    is_a: [
        // Specialised morphisms
        (Endomorphism, Morphism),
        (Isomorphism, Morphism),
        (Automorphism, Endomorphism),
        (Automorphism, Isomorphism),
        (Monomorphism, Morphism),
        (Epimorphism, Morphism),

        // Cells at higher dimension (Mac Lane XII.3)
        (Functor, Morphism),
        (Endofunctor, Functor),
        (NaturalTransformation, Morphism),
        (Unit, NaturalTransformation),
        (Counit, NaturalTransformation),
        (Multiplication, NaturalTransformation),
        (Comultiplication, NaturalTransformation),

        // Higher-category structures
        (TwoCategory, HigherCategory),
        (Bicategory, HigherCategory),

        // Typeclass hierarchy — McBride & Paterson (2008), Moggi (1991)
        (Applicative, Endofunctor),
        (Monad, Applicative),
        (Comonad, Endofunctor),

        // Monoidal structures
        (Monoid, Semigroup),
        (MonoidalCategory, CategoryStructure),

        // Limits and colimits
        (Product, Object),
        (Coproduct, Object),
        (Terminal, Object),
        (InitialObject, Object),

        // Derived categories
        (OppositeCategory, CategoryStructure),
        (KleisliCategory, CategoryStructure),

        // Specific monads
        (StateMonad, Monad),
        (ReaderMonad, Monad),
        (WriterMonad, Monad),
        (FreeMonad, Monad),

        // Yoneda
        (Representable, Functor),
        (YonedaEmbedding, Functor),

        // Special adjunction
        (GaloisConnection, Adjunction),

        // Interpretation / functor semantics — Spivak FDM; Goguen-Burstall
        (Interpretation, Functor),
        (InstanceFunctor, Interpretation),
        (SchemaCategory, CategoryStructure),
    ],

    has_a: [
        // Morphism structure (OBO-RO)
        (Morphism, Source),
        (Morphism, Target),
        (Morphism, Kind),

        // Category pieces
        (CategoryStructure, Object),
        (CategoryStructure, Morphism),
        (CategoryStructure, Composition),
        (CategoryStructure, Identity),

        // Adjunction pieces
        (Adjunction, Unit),
        (Adjunction, Counit),

        // 2-category cells
        (TwoCategory, CategoryStructure),
        (TwoCategory, Functor),
        (TwoCategory, NaturalTransformation),

        // Typeclass structure
        (Applicative, Pure),
        (Monad, Unit),
        (Monad, Multiplication),
        (Comonad, Counit),
        (Comonad, Comultiplication),

        // Monoidal structure
        (MonoidalCategory, Tensor),
        (Monoid, Identity),
        (Monoid, Multiplication),

        // F-(co)algebra structure
        (Algebra, StructureMap),
        (Coalgebra, StructureMap),

        // Interpretation has syntactic and semantic sides
        (Interpretation, Syntactic),
        (Interpretation, Semantic),

        // Instance functor sources a Schema
        (InstanceFunctor, SchemaCategory),
    ],

    opposes: [
        // Algebra / Coalgebra — structural duality (arrow direction reversed)
        (Algebra, Coalgebra),
        (Coalgebra, Algebra),

        // Monad / Comonad — categorical duality
        (Monad, Comonad),
        (Comonad, Monad),

        // Product / Coproduct — limit/colimit duality
        (Product, Coproduct),
        (Coproduct, Product),

        // Terminal / Initial — limit/colimit duality
        (Terminal, InitialObject),
        (InitialObject, Terminal),

        // Unit / Counit — adjunction triangle duality
        (Unit, Counit),
        (Counit, Unit),

        // Syntactic / Semantic — the classical syntax-semantics duality
        // (Tarski 1936; Lambek & Scott 1986). An interpretation bridges them.
        (Syntactic, Semantic),
        (Semantic, Syntactic),
    ],
}

/// This meta-ontology declares structure only; no per-concept quality
/// is attached. Domain ontologies *about* categorical structure
/// (algebra, relation properties) have their own qualities. Here the
/// concepts are pure classifications with no attached attribute.
#[derive(Debug, Clone)]
pub struct NoQuality;

impl crate::ontology::Quality for NoQuality {
    type Individual = CategoryTheoryConcept;
    type Value = ();

    fn get(&self, _: &CategoryTheoryConcept) -> Option<()> {
        None
    }
}

impl Ontology for CategoryTheoryOntology {
    type Cat = CategoryTheoryCategory;
    type Qual = NoQuality;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        crate::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

/// META-ONTOLOGY DISCRIMINATOR — does a connection's `kind` name a GROUNDING
/// (instance) functor: is it [`InstanceFunctor`](CategoryTheoryConcept::InstanceFunctor)
/// or a refinement subsumed by it?
///
/// Spivak (2012) *Functorial Data Migration* §3: an instance functor `I: S → Set`
/// interprets each schema object as its SET of instances — exactly the
/// "this typed node IS an instance of that concept" grounding a `.prx` carries as
/// DATA. A loader uses this to decide, PER CONNECTION, whether that connection is
/// a cross-ontology grounding functor to mint type edges from, or a plain schema
/// relabel to leave to the runtime `apply` step.
///
/// It is a genuine reflexive-transitive reachability query over THIS
/// meta-ontology's `Subsumption` edges (`kind ⊑* InstanceFunctor`), never a
/// `kind == "InstanceFunctor"` string match and never a `source != target` test.
/// A `FullyFaithful` schema-relabel functor (the USC/OWL `apply` relabels) has
/// `source != target` yet is a bare [`Functor`](CategoryTheoryConcept::Functor)
/// that does NOT reach `InstanceFunctor`, so it is correctly excluded — it is an
/// `apply`-relabel, not a grounding. Symmetrically a broad
/// [`Interpretation`](CategoryTheoryConcept::Interpretation) is a SUPERtype of
/// `InstanceFunctor`, so it does not reach it either (wrong direction) — only the
/// instance-grounding refinement qualifies.
pub fn is_grounding_functor_kind(kind: &str) -> bool {
    use crate::category::{Arrow, Category, Concept, FinitelyGenerated};
    let Some(start) = CategoryTheoryConcept::variants()
        .into_iter()
        .find(|c| c.name() == kind)
    else {
        // A kind that is not even a category-theory concept cannot be a grounding
        // functor (fail-closed — never treat an unknown kind as grounding).
        return false;
    };
    let goal = CategoryTheoryConcept::InstanceFunctor;
    // Reflexive-transitive Subsumption reachability: does `start` reach `goal`
    // following child ⊑ parent edges (source ⊑ target)?
    let mut frontier = alloc::vec![start];
    let mut seen: alloc::vec::Vec<CategoryTheoryConcept> = alloc::vec::Vec::new();
    while let Some(c) = frontier.pop() {
        if c == goal {
            return true;
        }
        if seen.contains(&c) {
            continue;
        }
        seen.push(c);
        for m in CategoryTheoryCategory::morphisms() {
            if m.kind() == CategoryTheoryRelationKind::Subsumption && m.source() == c {
                frontier.push(m.target());
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::laws::assert_category_laws;
    use crate::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_theory_ontology_category_laws() {
        assert_category_laws::<CategoryTheoryCategory>();
    }

    #[test]
    fn category_theory_ontology_validates() {
        CategoryTheoryOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn monad_is_applicative_is_endofunctor_is_functor() {
        // McBride & Paterson (2008) hierarchy; Moggi (1991) monad
        // as endofunctor with unit+multiplication.
        let sub: Vec<_> = CategoryTheoryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == CategoryTheoryRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(sub.contains(&(
            CategoryTheoryConcept::Monad,
            CategoryTheoryConcept::Applicative
        )));
        assert!(sub.contains(&(
            CategoryTheoryConcept::Applicative,
            CategoryTheoryConcept::Endofunctor
        )));
        assert!(sub.contains(&(
            CategoryTheoryConcept::Endofunctor,
            CategoryTheoryConcept::Functor
        )));
    }

    #[test]
    fn monad_has_unit_and_multiplication() {
        // Moggi (1991): monad = (T, η, μ)
        let parthood: Vec<_> = CategoryTheoryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == CategoryTheoryRelationKind::Parthood)
            .map(|m| (m.source(), m.target()))
            .collect();
        // Parthood is part→whole (BFO:0000050): Unit/Multiplication are PARTS of
        // the Monad, so the part is the source.
        assert!(parthood.contains(&(CategoryTheoryConcept::Unit, CategoryTheoryConcept::Monad)));
        assert!(parthood.contains(&(
            CategoryTheoryConcept::Multiplication,
            CategoryTheoryConcept::Monad
        )));
    }

    #[test]
    fn specific_monads_are_monads() {
        // Wadler (1992); Liang-Hudak-Jones (1995) — canonical monad examples.
        let sub: Vec<_> = CategoryTheoryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == CategoryTheoryRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for m in [
            CategoryTheoryConcept::StateMonad,
            CategoryTheoryConcept::ReaderMonad,
            CategoryTheoryConcept::WriterMonad,
            CategoryTheoryConcept::FreeMonad,
        ] {
            assert!(
                sub.contains(&(m, CategoryTheoryConcept::Monad)),
                "{:?} should be-a Monad",
                m
            );
        }
    }

    #[test]
    fn algebra_opposes_coalgebra() {
        // Meijer-Fokkinga-Paterson (1991) — algebra and coalgebra are
        // dual (arrows reversed).
        let opp: Vec<_> = CategoryTheoryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == CategoryTheoryRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(
            CategoryTheoryConcept::Algebra,
            CategoryTheoryConcept::Coalgebra
        )));
        assert!(opp.contains(&(
            CategoryTheoryConcept::Coalgebra,
            CategoryTheoryConcept::Algebra
        )));
    }

    #[test]
    fn interpretation_is_functor_instance_specialises_interpretation() {
        // Spivak (2012) FDM: an InstanceFunctor is a specific kind of
        // Interpretation (schema-to-Set); Interpretation is a Functor
        // (Lambek & Scott 1986; Goguen & Burstall 1984).
        let sub: Vec<_> = CategoryTheoryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == CategoryTheoryRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(sub.contains(&(
            CategoryTheoryConcept::Interpretation,
            CategoryTheoryConcept::Functor
        )));
        assert!(sub.contains(&(
            CategoryTheoryConcept::InstanceFunctor,
            CategoryTheoryConcept::Interpretation
        )));
    }

    #[test]
    fn interpretation_bridges_syntactic_and_semantic() {
        // Lambek & Scott (1986): an interpretation HAS a syntactic source
        // and a semantic target; the two are DUAL sides.
        let parthood: Vec<_> = CategoryTheoryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == CategoryTheoryRelationKind::Parthood)
            .map(|m| (m.source(), m.target()))
            .collect();
        // part→whole: Syntactic/Semantic are PARTS of the Interpretation.
        assert!(parthood.contains(&(
            CategoryTheoryConcept::Syntactic,
            CategoryTheoryConcept::Interpretation
        )));
        assert!(parthood.contains(&(
            CategoryTheoryConcept::Semantic,
            CategoryTheoryConcept::Interpretation
        )));
        let opp: Vec<_> = CategoryTheoryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == CategoryTheoryRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(
            CategoryTheoryConcept::Syntactic,
            CategoryTheoryConcept::Semantic
        )));
    }

    #[test]
    fn is_grounding_functor_kind_discriminates_instance_from_relabel() {
        // Spivak (2012) FDM §3: only an InstanceFunctor (or a refinement below it)
        // is a grounding functor. The discriminator is a reachability query, so:
        // - InstanceFunctor itself qualifies (reflexive),
        assert!(is_grounding_functor_kind("InstanceFunctor"));
        // - `FullyFaithful` (the kind the USC/OWL `apply` relabels actually carry)
        //   is not a registered concept, so it is excluded by the fail-closed
        //   unknown-kind branch — a schema-relabel is never grounded,
        assert!(!is_grounding_functor_kind("FullyFaithful"));
        // - and the REACHABILITY exclusion proper: `Functor` IS a registered
        //   concept but is a PARENT of InstanceFunctor, so it does not reach it
        //   (this, not `source != target`, is the discriminating test),
        assert!(!is_grounding_functor_kind("Functor"));
        // - a broad Interpretation is a SUPERtype of InstanceFunctor, so it does
        //   not reach it (wrong direction) — only the instance refinement grounds,
        assert!(!is_grounding_functor_kind("Interpretation"));
        // - and a kind that is not even a category-theory concept is fail-closed.
        assert!(!is_grounding_functor_kind("TypeGrounding"));
        assert!(!is_grounding_functor_kind("NotAConcept"));
    }

    #[test]
    fn galois_connection_is_adjunction() {
        // Ore (1944): Galois connections are adjunctions between posets.
        let sub: Vec<_> = CategoryTheoryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == CategoryTheoryRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(sub.contains(&(
            CategoryTheoryConcept::GaloisConnection,
            CategoryTheoryConcept::Adjunction
        )));
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in CategoryTheoryCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = CategoryTheoryConcept::variants();
            for m in CategoryTheoryCategory::morphisms() {
                if m.kind() == CategoryTheoryRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in CategoryTheoryOntology::axioms() {
                match axiom.verify() {
                    Ok(_) => {}
                    Err(c) => prop_assert!(
                        false,
                        "structural axiom failed: {}",
                        c.meta().name.as_str()
                    ),
                }
            }
        }

        #[test]
        fn prop_opposition_symmetric(_seed in any::<u32>()) {
            // Smith et al. (2005) OBO-RO: Opposition is symmetric.
            let opp: Vec<_> = CategoryTheoryCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == CategoryTheoryRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in &opp {
                prop_assert!(
                    opp.contains(&(*b, *a)),
                    "opposition ({:?}, {:?}) missing symmetric partner",
                    a, b
                );
            }
        }

        #[test]
        fn prop_concept_count_is_sufficient(_seed in any::<u32>()) {
            // Category theory is a rich vocabulary; we expect at least
            // the core 20 concepts Mac Lane develops in Ch. I-III.
            let variants: Vec<_> = CategoryTheoryConcept::variants();
            prop_assert!(variants.len() >= 40,
                "expected >= 40 concepts after extension, got {}", variants.len());
        }
    }
}
