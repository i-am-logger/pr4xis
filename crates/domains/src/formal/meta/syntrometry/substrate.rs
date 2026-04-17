//! Minimal pr4xis-core substrate ontology — the target category for the
//! syntrometric lineage functor.
//!
//! Encodes just enough of pr4xis's categorical primitives (Entity, Morphism,
//! Category, Functor, Endofunctor, Ontology) to be a valid functor target
//! for [`super::lineage_functor`]. Dense category — no kinded morphism
//! structure — so the kinded Syntrometry source can map into it without
//! the kind-alignment problems documented in the #98 research note.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Pr4xisSubstrate",
    source: "pr4xis/src/category/*.rs — Entity, Relationship, Category, Functor, Endofunctor, Ontology trait definitions",
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

        // Refined sub-kinds of SubCategory / SubMorphism / SubEntity —
        // each grounded in a literature-named construction (Mac Lane §II.3
        // product categories, CEM mereology, staging level).
        //
        // The former SubOppositionCategory is removed: opposition-structure
        // is the subject of a dedicated Dialectics ontology (Hegel /
        // Aristotle / Priest) that the `Syntrometry → Dialectics`
        // cross-functor maps `Dialektik` into directly. The primary
        // substrate now leaves `Dialektik` to collapse to `SubCategory`
        // (≈7% unit loss) — an honest statement that the core substrate
        // doesn't carry opposition structure; Dialectics does.
        SubProductCategory,
        SubLeveledEntity,
        SubMereologicalMorphism,
    ],

    labels: {
        SubEntity: ("en", "Entity", "The pr4xis::category::Entity trait — an enum of the objects of a category."),
        SubMorphism: ("en", "Morphism", "The pr4xis::category::Relationship trait — an arrow between two Entities."),
        SubCategory: ("en", "Category", "The pr4xis::category::Category trait — an Entity type plus a Morphism type satisfying identity + composition laws."),
        SubFunctor: ("en", "Functor", "The pr4xis::category::Functor trait — a structure-preserving map Source → Target."),
        SubEndofunctor: ("en", "Endofunctor", "The pr4xis::category::Endofunctor trait — a Functor specialised to Source = Target."),
        SubOntology: ("en", "Ontology", "The pr4xis::ontology::Ontology trait — a Category + reasoning systems (taxonomy, mereology, causation, opposition) + axioms."),

        SubEigenform: ("en", "Eigenform", "Von Foerster's X = F(X) — a fixed-point / goal-attractor in the substrate. Maps to the self_model ontology's eigenform + any CommunicativeGoal / Colimit construction."),
        SubIntention: ("en", "Intention", "The BDI (Bratman 1987) commitment-to-plan + the C1 attention selection (Dehaene GWT). The extremal-selection primitive in the substrate."),
        SubStagingLevel: ("en", "StagingLevel", "A single grade of the Futamura-projection staging hierarchy / C1 vs C2 consciousness split. The transcendence-level primitive."),
        SubSystemOfSystems: ("en", "SystemOfSystems", "The hierarchical composition primitive — what system-of-systems ontologies formalise. Graded composition of sub-systems."),

        SubProductCategory: ("en", "ProductCategory", "A Category that is the product of two or more component categories (Mac Lane Ch. II §3). Receives Heim's Aspekt = [D × K × P]."),
        SubLeveledEntity: ("en", "LeveledEntity", "An Entity that carries a grade/level index within a leveled-category tower. Receives Heim's SyntrixLevel."),
        SubMereologicalMorphism: ("en", "MereologicalMorphism", "A Morphism that additionally satisfies CEM mereological axioms (Weak Supplementation etc.). Receives Heim's Part."),
    },

    is_a: [
        // True subsumption only: every Endofunctor is a Functor specialised
        // to Source = Target (Mac Lane Ch. II §1).
        (SubEndofunctor, SubFunctor),
        // Sub-kinds of the core primitives.
        (SubProductCategory, SubCategory),
        (SubLeveledEntity, SubEntity),
        (SubMereologicalMorphism, SubMorphism),
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
            P::SubLeveledEntity => "formal::meta::staging (grade-indexed entities)",
            P::SubMereologicalMorphism => "pr4xis::ontology::reasoning::mereology",
        })
    }
}

impl Ontology for Pr4xisSubstrateOntology {
    type Cat = Pr4xisSubstrateCategory;
    type Qual = SubstrateLocation;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Pr4xisSubstrateOntology::generated_structural_axioms()
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
    fn ontology_validates() {
        Pr4xisSubstrateOntology::validate().unwrap();
    }
}
