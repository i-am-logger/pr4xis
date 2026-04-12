use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for SchemaConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Schema,
            Self::EntityType,
            Self::MorphismType,
            Self::PathEquation,
            Self::Axiom,
            Self::Instance,
            Self::Population,
            Self::SchemaMapping,
            Self::Transform,
            Self::Presentation,
            Self::Algebra,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaRelation {
    pub from: SchemaConcept,
    pub to: SchemaConcept,
    pub kind: SchemaRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaRelationKind {
    Identity,
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
    Composed,
}

impl Relationship for SchemaRelation {
    type Object = SchemaConcept;
    fn source(&self) -> SchemaConcept {
        self.from
    }
    fn target(&self) -> SchemaConcept {
        self.to
    }
}

pub struct SchemaCategory;

impl Category for SchemaCategory {
    type Object = SchemaConcept;
    type Morphism = SchemaRelation;

    fn identity(obj: &SchemaConcept) -> SchemaRelation {
        SchemaRelation {
            from: *obj,
            to: *obj,
            kind: SchemaRelationKind::Identity,
        }
    }

    fn compose(f: &SchemaRelation, g: &SchemaRelation) -> Option<SchemaRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == SchemaRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == SchemaRelationKind::Identity {
            return Some(f.clone());
        }
        Some(SchemaRelation {
            from: f.from,
            to: g.to,
            kind: SchemaRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<SchemaRelation> {
        use SchemaConcept as C;
        use SchemaRelationKind as R;
        let mut m = Vec::new();

        // Identities
        for c in SchemaConcept::variants() {
            m.push(SchemaRelation {
                from: c,
                to: c,
                kind: R::Identity,
            });
        }

        // Schema contains its structural components
        m.push(SchemaRelation {
            from: C::Schema,
            to: C::EntityType,
            kind: R::ContainsEntity,
        });
        m.push(SchemaRelation {
            from: C::Schema,
            to: C::MorphismType,
            kind: R::ContainsMorphism,
        });
        m.push(SchemaRelation {
            from: C::Schema,
            to: C::PathEquation,
            kind: R::ContainsEquation,
        });
        m.push(SchemaRelation {
            from: C::Schema,
            to: C::Axiom,
            kind: R::ContainsAxiom,
        });

        // EntityType participates in MorphismType
        m.push(SchemaRelation {
            from: C::EntityType,
            to: C::MorphismType,
            kind: R::Participates,
        });

        // Instance is a functor from Schema
        m.push(SchemaRelation {
            from: C::Instance,
            to: C::Schema,
            kind: R::InstantiatedFrom,
        });

        // Instance assigns Population to EntityType
        m.push(SchemaRelation {
            from: C::Instance,
            to: C::Population,
            kind: R::Assigns,
        });
        m.push(SchemaRelation {
            from: C::Population,
            to: C::EntityType,
            kind: R::Participates,
        });

        // SchemaMapping connects two Schemas
        m.push(SchemaRelation {
            from: C::SchemaMapping,
            to: C::Schema,
            kind: R::Maps,
        });

        // Transform connects two Instances
        m.push(SchemaRelation {
            from: C::Transform,
            to: C::Instance,
            kind: R::Transforms,
        });

        // Presentation ↔ Algebra (CQL evaluation/presentation adjunction)
        m.push(SchemaRelation {
            from: C::Presentation,
            to: C::Algebra,
            kind: R::Evaluates,
        });
        m.push(SchemaRelation {
            from: C::Algebra,
            to: C::Presentation,
            kind: R::Presents,
        });

        // Transitive: Schema → Instance → Population (through schema and instance)
        m.push(SchemaRelation {
            from: C::Schema,
            to: C::Instance,
            kind: R::Composed,
        });
        m.push(SchemaRelation {
            from: C::Schema,
            to: C::Population,
            kind: R::Composed,
        });
        m.push(SchemaRelation {
            from: C::SchemaMapping,
            to: C::Instance,
            kind: R::Composed,
        });

        // Self-compositions
        for c in SchemaConcept::variants() {
            m.push(SchemaRelation {
                from: c,
                to: c,
                kind: R::Composed,
            });
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<SchemaCategory>().unwrap();
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
