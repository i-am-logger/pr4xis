//! Minimal pr4xis-core substrate ontology — the target category for the
//! syntrometric lineage functor.
//!
//! Encodes just enough of pr4xis's categorical primitives (Entity, Morphism,
//! Category, Functor, Endofunctor, Ontology) to be a valid functor target
//! for [`super::lineage_functor`]. Dense category — no kinded morphism
//! structure — so the kinded Syntrometry source can map into it without
//! the kind-alignment problems documented in the #98 research note.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Pr4xisSubstrate",
    source: "pr4xis/src/category/*.rs — Concept, Relationship, Category, Functor, Endofunctor, Ontology trait definitions",
    being: AbstractObject,

    concepts: [
        // Categorical core primitives.
        SubEntity,
        SubMorphism,
        SubCategory,
        SubFunctor,
        SubEndofunctor,
        SubOntology,

        // Architectural primitives — already present in pr4xis elsewhere in
        // the workspace; mirrored here as the functor target.
        SubEigenform,
        SubIntention,
        SubStagingLevel,
        SubSystemOfSystems,
        // NaturalTransformation — Mac Lane Ch. II §4. pr4xis has it as a
        // first-class trait in category/transformation.rs; mirrored here
        // as the target for Heim's Reflexivity ρ.
        SubNaturalTransformation,

        // Refined sub-kinds of SubCategory / SubMorphism / SubEntity —
        // each grounded in a literature-named construction:
        //
        // - `SubProductCategory`: Mac Lane, *Categories for the Working
        //   Mathematician* (1971) Ch. II §3.
        // - `SubGradedObject`: Mac Lane, *Homology* (1963) Ch. II §2 on
        //   graded modules; also Stanley, *Enumerative Combinatorics*
        //   (1986) Ch. 3 for graded posets / ranked structures; standard
        //   "graded object" terminology across homological algebra and
        //   category theory.
        // - `SubObject`: Mac Lane, *CWM* Ch. V §1 "Subobjects and quotient
        //   objects" — a subobject is an equivalence class of monos
        //   A ↪ B. Awodey, *Category Theory* (2010) Ch. 5 covers the
        //   same in modern notation.
        //
        // Opposition-structure is not in the substrate — it lives in the
        // dedicated Dialectics ontology (Hegel / Aristotle / Priest) that
        // `Syntrometry → Dialectics` maps `Dialektik` into directly.
        SubProductCategory,
        SubGradedObject,
        SubObject,
    ],

    labels: {
        SubEntity: ("en", "Concept", "The pr4xis::category::Concept trait — an enum of the objects of a category."),
        SubMorphism: ("en", "Morphism", "The pr4xis::category::Relationship trait — an arrow between two Entities."),
        SubCategory: ("en", "Category", "The pr4xis::category::Category trait — an Concept type plus a Morphism type satisfying identity + composition laws."),
        SubFunctor: ("en", "Functor", "The pr4xis::category::Functor trait — a structure-preserving map Source → Target."),
        SubEndofunctor: ("en", "Endofunctor", "The pr4xis::category::Endofunctor trait — a Functor specialised to Source = Target."),
        SubOntology: ("en", "Ontology", "The pr4xis::ontology::Ontology trait — a Category + reasoning systems (taxonomy, mereology, causation, opposition) + axioms."),

        SubEigenform: ("en", "Eigenform", "Von Foerster's X = F(X) — a fixed-point / goal-attractor in the substrate. Maps to the self_model ontology's eigenform + any CommunicativeGoal / Colimit construction."),
        SubIntention: ("en", "Intention", "The BDI (Bratman 1987) commitment-to-plan + the C1 attention selection (Dehaene GWT). The extremal-selection primitive in the substrate."),
        SubStagingLevel: ("en", "StagingLevel", "A single grade of the Futamura-projection staging hierarchy / C1 vs C2 consciousness split. The transcendence-level primitive."),
        SubSystemOfSystems: ("en", "SystemOfSystems", "The hierarchical composition primitive — what system-of-systems ontologies formalise. Graded composition of sub-systems."),
        SubNaturalTransformation: ("en", "NaturalTransformation", "A natural transformation between two parallel functors (Mac Lane Ch. II §4). pr4xis has it as a first-class trait in category/transformation.rs."),

        SubProductCategory: ("en", "ProductCategory", "A category that is the product of two or more component categories. Mac Lane, Categories for the Working Mathematician (1971), Ch. II §3."),
        SubGradedObject: ("en", "GradedObject", "An object equipped with a grading by an index set (typically ℕ). Mac Lane, Homology (1963), Ch. II §2 on graded modules; Stanley, Enumerative Combinatorics (1986), Ch. 3 on graded posets. Receives Heim's SyntrixLevel."),
        SubObject: ("en", "Subobject", "An equivalence class of monomorphisms A ↪ B. Mac Lane, CWM (1971), Ch. V §1 'Subobjects and quotient objects'; Awodey, Category Theory (2010), Ch. 5. Receives Heim's Part — the categorical formalisation of the part-of relation."),
    },

    is_a: [
        // True subsumption only: every Endofunctor is a Functor specialised
        // to Source = Target (Mac Lane Ch. II §1).
        (SubEndofunctor, SubFunctor),
        // Sub-kinds of the core primitives. Each sub-kind is a literature-
        // grounded specialisation:
        // - ProductCategory ⊂ Category (Mac Lane Ch. II §3)
        // - GradedObject ⊂ Entity (Mac Lane Homology Ch. II §2; Stanley 1986)
        // - Subobject ⊂ Morphism (Mac Lane Ch. V §1; Awodey 2010 Ch. 5)
        (SubProductCategory, SubCategory),
        (SubGradedObject, SubEntity),
        (SubObject, SubMorphism),
    ],
}

/// Quality: which pr4xis crate-section each substrate primitive lives in.
#[derive(Debug, Clone)]
pub struct SubstrateLocation;

impl Quality for SubstrateLocation {
    type Individual = Pr4xisSubstrateConcept;
    type Value = &'static str;

    fn get(&self, c: &Pr4xisSubstrateConcept) -> Option<&'static str> {
        use Pr4xisSubstrateConcept as P;
        Some(match c {
            P::SubEntity => "pr4xis::category::entity",
            P::SubMorphism => "pr4xis::category::relationship",
            P::SubCategory => "pr4xis::category::category",
            P::SubFunctor => "pr4xis::category::functor",
            P::SubEndofunctor => "pr4xis::category::endofunctor",
            P::SubOntology => "pr4xis::ontology",
            P::SubEigenform => "cognitive::cognition::self_model (+ algebra::Colimit)",
            P::SubIntention => {
                "cognitive::linguistics::pragmatics + cognitive::cognition::consciousness (C1)"
            }
            P::SubStagingLevel => {
                "formal::meta::staging + cognitive::cognition::consciousness (C1/C2)"
            }
            P::SubSystemOfSystems => "system-of-systems composition (tracked as issue #94)",
            P::SubProductCategory => "pr4xis::category::monoidal::Product",
            P::SubGradedObject => "Mac Lane 1963 Ch. II §2 / Stanley 1986 Ch. 3",
            P::SubObject => "Mac Lane 1971 Ch. V §1 / Awodey 2010 Ch. 5",
            P::SubNaturalTransformation => "pr4xis::category::transformation",
        })
    }
}

// ---------------------------------------------------------------------------
// Domain axioms — literature-grounded structural claims about the refined
// sub-kinds of the substrate.
// ---------------------------------------------------------------------------

fn direct_children_of(parent: Pr4xisSubstrateConcept) -> Vec<Pr4xisSubstrateConcept> {
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
    Pr4xisSubstrateTaxonomy::relations()
        .into_iter()
        .filter_map(|(child, p)| if p == parent { Some(child) } else { None })
        .collect()
}

/// Axiom: `SubEndofunctor` is a sub-kind of `SubFunctor`. Mac Lane,
/// *Categories for the Working Mathematician* (1971) Ch. II §1 — an
/// endofunctor F: C → C is a functor specialised to Source = Target.
pub struct EndofunctorIsFunctor;

impl Axiom for EndofunctorIsFunctor {
    fn description(&self) -> &str {
        "SubEndofunctor is a direct taxonomic sub-kind of SubFunctor (Mac Lane 1971 Ch. II §1)"
    }
    fn holds(&self) -> bool {
        direct_children_of(Pr4xisSubstrateConcept::SubFunctor)
            .contains(&Pr4xisSubstrateConcept::SubEndofunctor)
    }
}
pr4xis::register_axiom!(EndofunctorIsFunctor);

/// Axiom: `SubProductCategory` is a sub-kind of `SubCategory`. Mac Lane,
/// *CWM* (1971) Ch. II §3 — the product of two categories is itself a
/// category, with objects pairs (a, b) and morphisms pairs (f, g).
pub struct ProductCategoryIsCategory;

impl Axiom for ProductCategoryIsCategory {
    fn description(&self) -> &str {
        "SubProductCategory is a direct taxonomic sub-kind of SubCategory (Mac Lane 1971 Ch. II §3)"
    }
    fn holds(&self) -> bool {
        direct_children_of(Pr4xisSubstrateConcept::SubCategory)
            .contains(&Pr4xisSubstrateConcept::SubProductCategory)
    }
}
pr4xis::register_axiom!(ProductCategoryIsCategory);

/// Axiom: `SubGradedObject` is a sub-kind of `SubEntity`. A graded
/// object is an entity equipped with a decomposition indexed by an index
/// set (usually ℕ). Mac Lane, *Homology* (1963) Ch. II §2 on graded
/// modules; Stanley, *Enumerative Combinatorics* (1986) Ch. 3 on graded
/// posets / ranked structures.
pub struct GradedObjectIsEntity;

impl Axiom for GradedObjectIsEntity {
    fn description(&self) -> &str {
        "SubGradedObject is a direct taxonomic sub-kind of SubEntity (Mac Lane, Homology 1963 Ch. II §2; Stanley, Enumerative Combinatorics 1986 Ch. 3)"
    }
    fn holds(&self) -> bool {
        direct_children_of(Pr4xisSubstrateConcept::SubEntity)
            .contains(&Pr4xisSubstrateConcept::SubGradedObject)
    }
}
pr4xis::register_axiom!(GradedObjectIsEntity);

/// Axiom: `SubObject` is a sub-kind of `SubMorphism`. A subobject is
/// an equivalence class of monomorphisms A ↪ B — the formal categorical
/// account of the part-of relation. Mac Lane, *CWM* (1971) Ch. V §1
/// "Subobjects and quotient objects"; Awodey, *Category Theory* (2010)
/// Ch. 5.
pub struct SubobjectIsMorphism;

impl Axiom for SubobjectIsMorphism {
    fn description(&self) -> &str {
        "SubObject is a direct taxonomic sub-kind of SubMorphism (Mac Lane 1971 Ch. V §1; Awodey 2010 Ch. 5)"
    }
    fn holds(&self) -> bool {
        direct_children_of(Pr4xisSubstrateConcept::SubMorphism)
            .contains(&Pr4xisSubstrateConcept::SubObject)
    }
}
pr4xis::register_axiom!(SubobjectIsMorphism);

impl Ontology for Pr4xisSubstrateOntology {
    type Cat = Pr4xisSubstrateCategory;
    type Qual = SubstrateLocation;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Pr4xisSubstrateOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(EndofunctorIsFunctor),
            Box::new(ProductCategoryIsCategory),
            Box::new(GradedObjectIsEntity),
            Box::new(SubobjectIsMorphism),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<Pr4xisSubstrateCategory>().unwrap();
    }

    #[test]
    fn endofunctor_is_functor_holds() {
        assert!(
            EndofunctorIsFunctor.holds(),
            "{}",
            EndofunctorIsFunctor.description()
        );
    }

    #[test]
    fn product_category_is_category_holds() {
        assert!(
            ProductCategoryIsCategory.holds(),
            "{}",
            ProductCategoryIsCategory.description()
        );
    }

    #[test]
    fn graded_object_is_entity_holds() {
        assert!(
            GradedObjectIsEntity.holds(),
            "{}",
            GradedObjectIsEntity.description()
        );
    }

    #[test]
    fn subobject_is_morphism_holds() {
        assert!(
            SubobjectIsMorphism.holds(),
            "{}",
            SubobjectIsMorphism.description()
        );
    }

    #[test]
    fn ontology_validates() {
        Pr4xisSubstrateOntology::validate().unwrap();
    }
}
