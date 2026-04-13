use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Ontology, Quality};

// Schema ontology — ontology structure as data (the M2 level).
//
// A Schema is a category whose objects are entity types and whose
// morphisms are relationships between them. This is Spivak's
// "schema as category" from functorial data migration.
//
// The key insight: an ontology's STRUCTURE can itself be represented
// as data — enabling introspection, migration, comparison, and
// generic persistence across stores.
//
// References:
// - Spivak, "Functorial Data Migration" (2012, Information and Computation)
// - Spivak, "Simplicial Databases" (2009, arXiv:0904.2012)
// - Spivak & Wisnesky, "Relational Foundations for Functorial Data Migration" (2015)
// - Wisnesky et al., "Algebraic Databases" (2017)
// - Baader et al., "The Description Logic Handbook" (2003) — TBox/ABox
// - OMG MDA Guide v2.0 (2014) — M0/M1/M2/M3 levels

/// Concepts in the Schema ontology — what an ontology IS made of.
///
/// These are the meta-level building blocks. Every praxis ontology
/// (chess, linguistics, systems, ...) is an instance of this schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum SchemaConcept {
    /// A category used as a schema — the TBox (Baader 2003).
    /// Spivak (2012): "A schema is a small category C."
    Schema,

    /// An object in the schema category — an entity type.
    /// Spivak: objects of C. DL: concept names in the TBox.
    EntityType,

    /// A morphism in the schema category — a typed relationship.
    /// Spivak: morphisms of C. DL: role names in the TBox.
    MorphismType,

    /// A path equation — composition constraint on morphisms.
    /// Spivak (2012): "path equations" enforce that two paths
    /// through the schema must yield the same result.
    /// CQL: the "equations" in a schema presentation.
    PathEquation,

    /// An axiom — a constraint that instances must satisfy.
    /// DL: TBox axioms (subsumption, disjointness, etc.).
    Axiom,

    /// An instance — a functor from Schema to Set.
    /// Spivak (2012): "An instance on C is a functor I: C → Set."
    /// DL: ABox (the assertional component).
    Instance,

    /// A population — the set of individuals for an entity type.
    /// Spivak: I(c) for object c — the set assigned to entity type c.
    Population,

    /// A mapping between schemas — a functor F: C → D.
    /// Spivak: induces migration functors ΔF, ΣF, ΠF.
    SchemaMapping,

    /// A transform — a natural transformation between instances.
    /// CQL: a morphism in the category of instances on a fixed schema.
    Transform,

    /// A presentation — the syntactic form of a schema or instance.
    /// CQL: generators + equations before evaluation.
    /// This is the serialized/stored form.
    Presentation,

    /// An algebra — the semantic (evaluated) form.
    /// CQL: the initial algebra of a presentation.
    /// This is the live, in-memory form.
    Algebra,
}

define_ontology! {
    pub SchemaOntology for SchemaCategory {
        concepts: SchemaConcept,
        relation: SchemaRelation,
        kind: SchemaRelationKind,
        kinds: [
            /// Schema contains EntityTypes — "has object" in categorical terms.
            ContainsEntity,
            /// Schema contains MorphismTypes — "has morphism".
            ContainsMorphism,
            /// Schema contains PathEquations — "has equation".
            ContainsEquation,
            /// Schema contains Axioms — "has constraint".
            ContainsAxiom,
            /// EntityType participates in MorphismType (as source or target).
            Participates,
            /// Instance is a functor FROM Schema (Spivak: I: C → Set).
            InstantiatedFrom,
            /// Instance assigns Population to EntityType (Spivak: I(c)).
            Assigns,
            /// SchemaMapping connects two Schemas (functor F: C → D).
            Maps,
            /// Transform connects two Instances (natural transformation).
            Transforms,
            /// Presentation evaluates to Algebra (CQL: initial algebra).
            Evaluates,
            /// Algebra is presented by Presentation (CQL: generators+equations).
            Presents,
        ],
        edges: [
            // Schema contains its structural components
            (Schema, EntityType, ContainsEntity),
            (Schema, MorphismType, ContainsMorphism),
            (Schema, PathEquation, ContainsEquation),
            (Schema, Axiom, ContainsAxiom),
            // EntityType participates in MorphismType
            (EntityType, MorphismType, Participates),
            // Instance is a functor from Schema
            (Instance, Schema, InstantiatedFrom),
            // Instance assigns Population to EntityType
            (Instance, Population, Assigns),
            (Population, EntityType, Participates),
            // SchemaMapping connects two Schemas
            (SchemaMapping, Schema, Maps),
            // Transform connects two Instances
            (Transform, Instance, Transforms),
            // Presentation ↔ Algebra (CQL evaluation/presentation adjunction)
            (Presentation, Algebra, Evaluates),
            (Algebra, Presentation, Presents),
        ],
        composed: [
            // Schema → Instance → Population (through schema and instance)
            (Schema, Instance),
            (Schema, Population),
            (SchemaMapping, Instance),
        ],
    }
}

/// MDA level (M0..M3) for each schema concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MdaLevel {
    /// M0: runtime instances (data).
    M0,
    /// M1: models (schemas, instances).
    M1,
    /// M2: meta-models (schema of schemas).
    M2,
}

/// Which MDA level does each schema concept live at?
#[derive(Debug, Clone)]
pub struct MdaLevelQuality;

impl Quality for MdaLevelQuality {
    type Individual = SchemaConcept;
    type Value = MdaLevel;

    fn get(&self, individual: &SchemaConcept) -> Option<MdaLevel> {
        match individual {
            SchemaConcept::Schema => Some(MdaLevel::M2),
            SchemaConcept::EntityType => Some(MdaLevel::M2),
            SchemaConcept::MorphismType => Some(MdaLevel::M2),
            SchemaConcept::PathEquation => Some(MdaLevel::M2),
            SchemaConcept::Axiom => Some(MdaLevel::M2),
            SchemaConcept::Instance => Some(MdaLevel::M1),
            SchemaConcept::Population => Some(MdaLevel::M0),
            SchemaConcept::SchemaMapping => Some(MdaLevel::M2),
            SchemaConcept::Transform => Some(MdaLevel::M1),
            SchemaConcept::Presentation => Some(MdaLevel::M1),
            SchemaConcept::Algebra => Some(MdaLevel::M1),
        }
    }
}

impl Ontology for SchemaOntology {
    type Cat = SchemaCategory;
    type Qual = MdaLevelQuality;

    fn structural_axioms() -> Vec<Box<dyn pr4xis::ontology::Axiom>> {
        Self::generated_structural_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<SchemaCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        SchemaOntology::validate().unwrap();
    }

    #[test]
    fn has_eleven_concepts() {
        assert_eq!(SchemaConcept::variants().len(), 11);
    }

    // --- Spivak (2012): "A schema is a small category C" ---

    #[test]
    fn schema_contains_entity_types() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.from == SchemaConcept::Schema
            && r.to == SchemaConcept::EntityType
            && r.kind == SchemaRelationKind::ContainsEntity));
    }

    #[test]
    fn schema_contains_morphism_types() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.from == SchemaConcept::Schema
            && r.to == SchemaConcept::MorphismType
            && r.kind == SchemaRelationKind::ContainsMorphism));
    }

    #[test]
    fn schema_contains_path_equations() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.from == SchemaConcept::Schema
            && r.to == SchemaConcept::PathEquation
            && r.kind == SchemaRelationKind::ContainsEquation));
    }

    // --- Spivak (2012): "An instance on C is a functor I: C → Set" ---

    #[test]
    fn instance_is_functor_from_schema() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.from == SchemaConcept::Instance
            && r.to == SchemaConcept::Schema
            && r.kind == SchemaRelationKind::InstantiatedFrom));
    }

    #[test]
    fn instance_assigns_population_to_entity_type() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.from == SchemaConcept::Instance
            && r.to == SchemaConcept::Population
            && r.kind == SchemaRelationKind::Assigns));
    }

    // --- CQL: Presentation → Algebra (evaluation) ---

    #[test]
    fn presentation_evaluates_to_algebra() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.from == SchemaConcept::Presentation
            && r.to == SchemaConcept::Algebra
            && r.kind == SchemaRelationKind::Evaluates));
    }

    #[test]
    fn algebra_is_presented_by_presentation() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.from == SchemaConcept::Algebra
            && r.to == SchemaConcept::Presentation
            && r.kind == SchemaRelationKind::Presents));
    }

    // --- Spivak: SchemaMapping induces migration functors ---

    #[test]
    fn schema_mapping_connects_schemas() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.from == SchemaConcept::SchemaMapping
            && r.to == SchemaConcept::Schema
            && r.kind == SchemaRelationKind::Maps));
    }

    // --- Baader (2003): TBox (Schema) vs ABox (Instance) ---

    #[test]
    fn tbox_abox_distinction() {
        // Schema = TBox, Instance = ABox. They are separate concepts.
        assert_ne!(SchemaConcept::Schema, SchemaConcept::Instance);
        // But Instance refers to Schema (instantiated from).
        let m = SchemaCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.from == SchemaConcept::Instance && r.to == SchemaConcept::Schema)
        );
    }

    // --- Composition: Schema → Instance → Population ---

    #[test]
    fn schema_reaches_population() {
        let m = SchemaCategory::morphisms();
        assert!(m.iter().any(|r| r.from == SchemaConcept::Schema
            && r.to == SchemaConcept::Population
            && r.kind == SchemaRelationKind::Composed));
    }

    #[test]
    fn composition_with_identity() {
        for morph in &SchemaCategory::morphisms() {
            let left =
                SchemaCategory::compose(&SchemaCategory::identity(&morph.from), morph).unwrap();
            assert_eq!(left.from, morph.from);
            assert_eq!(left.to, morph.to);
        }
    }
}
