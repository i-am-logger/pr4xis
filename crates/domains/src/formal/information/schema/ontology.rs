//! Schema — ontology structure as data (the M2 level).
//!
//! A Schema is a category whose objects are entity types and whose
//! morphisms are typed relationships between them. Spivak (2012)
//! "schema as category"; an ontology's STRUCTURE can itself be
//! represented as data — enabling introspection, migration, and
//! comparison.
//!
//! # Literature
//!
//! - **Spivak (2012)** "Functorial Data Migration", *Information and
//!   Computation* 217:31-51 — schema as small category C; instance
//!   as functor I: C → Set; the three migration functors Σ ⊣ Δ ⊣ Π.
//! - **Spivak (2009)** "Simplicial Databases", arXiv:0904.2012.
//! - **Spivak & Wisnesky (2015)** "Relational Foundations for
//!   Functorial Data Migration", *DBPL 2015*.
//! - **Wisnesky, Schultz, Spivak (2017)** "Algebraic Databases",
//!   *Theory and Applications of Categories* — CQL: Presentation /
//!   Algebra duality.
//! - **Baader, Calvanese, McGuinness, Nardi, Patel-Schneider (2003)**
//!   *The Description Logic Handbook*, Cambridge UP — TBox / ABox.
//! - **OMG (2014)** *MDA Guide v2.0* — M0 / M1 / M2 / M3 model levels.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Schema",
    source: "Spivak (2012) Functorial Data Migration, Information and Computation 217:31-51; Spivak & Wisnesky (2015) Relational Foundations for Functorial Data Migration, DBPL; Wisnesky, Schultz & Spivak (2017) Algebraic Databases, TAC; Baader et al. (2003) The Description Logic Handbook; OMG (2014) MDA Guide v2.0",

    concepts: [
        Schema,
        EntityType,
        MorphismType,
        PathEquation,
        Axiom,
        Instance,
        Population,
        SchemaMapping,
        Transform,
        Presentation,
        Algebra,
    ],

    labels: {
        Schema: ("en", "Schema",
            "Spivak (2012): a schema is a small category C. Baader (2003) calls this the TBox."),
        EntityType: ("en", "Entity type",
            "Spivak (2012): an object of the schema category. DL concept name."),
        MorphismType: ("en", "Morphism type",
            "Spivak (2012): a morphism of the schema category. DL role name."),
        PathEquation: ("en", "Path equation",
            "Spivak (2012): a composition constraint - two paths through the schema must yield the same result."),
        Axiom: ("en", "Schema axiom",
            "Baader (2003): a TBox axiom - subsumption, disjointness, equivalence."),
        Instance: ("en", "Instance",
            "Spivak (2012): a functor I: C → Set. Baader (2003) calls this the ABox."),
        Population: ("en", "Population",
            "Spivak (2012): I(c) for an entity type c - the set of individuals."),
        SchemaMapping: ("en", "Schema mapping",
            "Spivak (2012): a functor F: C → D inducing the migration functors Σ ⊣ Δ ⊣ Π."),
        Transform: ("en", "Transform",
            "Spivak (2012): a natural transformation between two instances on the same schema."),
        Presentation: ("en", "Presentation",
            "Wisnesky et al. (2017) CQL: the syntactic form of a schema - generators + equations."),
        Algebra: ("en", "Algebra",
            "Wisnesky et al. (2017) CQL: the semantic (evaluated) form - the initial algebra of a presentation."),
    },

    edges: [
        // Spivak (2012): Schema contains its structural components.
        (Schema, EntityType, ContainsEntity),
        (Schema, MorphismType, ContainsMorphism),
        (Schema, PathEquation, ContainsEquation),
        (Schema, Axiom, ContainsAxiom),
        // EntityType participates in MorphismType (as source or target).
        (EntityType, MorphismType, Participates),
        // Spivak (2012): Instance is a functor FROM Schema.
        (Instance, Schema, InstantiatedFrom),
        // Instance assigns a Population to each EntityType.
        (Instance, Population, Assigns),
        (Population, EntityType, Participates),
        // Spivak (2012): SchemaMapping is a functor between schemas.
        (SchemaMapping, Schema, Maps),
        // Transform is a natural transformation between Instances.
        (Transform, Instance, Transforms),
        // Wisnesky et al. (2017) CQL: Presentation evaluates to Algebra.
        (Presentation, Algebra, Evaluates),
        (Algebra, Presentation, Presents),
    ],
}

/// MDA model levels (OMG 2014). M0 = runtime instances; M1 = models;
/// M2 = meta-models (schema of schemas).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MdaLevel {
    M0,
    M1,
    M2,
}

/// Quality: which MDA level each schema concept lives at.
#[derive(Debug, Clone)]
pub struct MdaLevelQuality;

impl Quality for MdaLevelQuality {
    type Individual = SchemaConcept;
    type Value = MdaLevel;

    fn get(&self, c: &SchemaConcept) -> Option<MdaLevel> {
        use SchemaConcept as S;
        Some(match c {
            S::Schema
            | S::EntityType
            | S::MorphismType
            | S::PathEquation
            | S::Axiom
            | S::SchemaMapping => MdaLevel::M2,
            S::Instance | S::Transform | S::Presentation | S::Algebra => MdaLevel::M1,
            S::Population => MdaLevel::M0,
        })
    }
}

impl Ontology for SchemaOntology {
    type Cat = SchemaCategory;
    type Qual = MdaLevelQuality;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<SchemaCategory>();
    }

    #[test]
    fn ontology_validates() {
        SchemaOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn eleven_concepts() {
        assert_eq!(SchemaConcept::variants().len(), 11);
    }

    #[test]
    fn schema_contains_entity_types() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == SchemaConcept::Schema
            && r.target() == SchemaConcept::EntityType
            && r.kind() == SchemaRelationKind::ContainsEntity));
    }

    #[test]
    fn instance_is_functor_from_schema() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == SchemaConcept::Instance
            && r.target() == SchemaConcept::Schema
            && r.kind() == SchemaRelationKind::InstantiatedFrom));
    }

    #[test]
    fn presentation_evaluates_to_algebra() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == SchemaConcept::Presentation
            && r.target() == SchemaConcept::Algebra
            && r.kind() == SchemaRelationKind::Evaluates));
    }

    #[test]
    fn mda_level_total() {
        for c in SchemaConcept::variants() {
            assert!(MdaLevelQuality.get(&c).is_some());
        }
    }

    fn arb_concept() -> impl Strategy<Value = SchemaConcept> {
        proptest::sample::select(SchemaConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in SchemaCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in SchemaOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_mda_level_total(c in arb_concept()) {
            prop_assert!(MdaLevelQuality.get(&c).is_some());
        }
    }
}
