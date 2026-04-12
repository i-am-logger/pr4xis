use pr4xis::category::Entity;
use pr4xis::define_category;

// Instance ontology — the Spivak instance functor formalized.
//
// An Instance is a functor I: Schema → Set — it populates a schema
// with actual data. For each entity type in the schema, the instance
// provides a set of individuals. For each morphism type, a function
// between those sets.
//
// This also models the three Spivak migration functors (Δ, Σ, Π)
// that transform instances along schema mappings, and the
// Presentation/Algebra duality from CQL.
//
// References:
// - Spivak, "Functorial Data Migration" (2012, Information and Computation)
// - Spivak & Wisnesky, "Relational Foundations for Functorial Data Migration" (2015)
// - Wisnesky et al., "Algebraic Databases" (2017) — CQL
// - Baader et al., "The Description Logic Handbook" (2003) — ABox

/// Concepts in the Instance ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum InstanceConcept {
    /// A functor I: Schema → Set — the populated data.
    /// Spivak (2012): "An instance on C is a functor I: C → Set."
    /// DL (Baader 2003): the ABox.
    Instance,

    /// A set of individuals for one entity type.
    /// Spivak: I(c) for object c — a set.
    Population,

    /// A function between populations for one morphism type.
    /// Spivak: I(f) for morphism f — a function between sets.
    Assignment,

    /// A specific individual in a population.
    /// DL: an ABox assertion a:C (individual a is of type C).
    Individual,

    /// A constraint that an instance must satisfy.
    /// Spivak: path equations → commutative diagrams that instances must respect.
    InstanceConstraint,

    /// Pullback migration functor — ΔF.
    /// Spivak (2012): given F: C → D, ΔF: D-Inst → C-Inst.
    /// Restricts/projects data from target schema to source.
    DeltaMigration,

    /// Left pushforward migration functor — ΣF.
    /// Spivak (2012): ΣF: C-Inst → D-Inst.
    /// Pushes data forward via coproduct (union).
    SigmaMigration,

    /// Right pushforward migration functor — ΠF.
    /// Spivak (2012): ΠF: C-Inst → D-Inst.
    /// Pushes data forward via product (universal).
    PiMigration,

    /// The adjunction: ΣF ⊣ ΔF ⊣ ΠF.
    /// Spivak (2012): Σ is left adjoint to Δ, Δ is left adjoint to Π.
    MigrationAdjunction,
}

define_category! {
    pub InstanceCategory {
        entity: InstanceConcept,
        relation: InstanceRelation,
        kind: InstanceRelationKind,
        kinds: [
            /// Instance contains Population (one per entity type).
            Contains,
            /// Instance contains Assignment (one per morphism type).
            ContainsAssignment,
            /// Population contains Individuals.
            ContainsIndividuals,
            /// Assignment maps between Populations (function between sets).
            MapsBetween,
            /// Instance must satisfy InstanceConstraint.
            Satisfies,
            /// DeltaMigration pulls Instance back along a schema mapping.
            PullsBack,
            /// SigmaMigration pushes Instance forward (coproduct).
            PushesForwardLeft,
            /// PiMigration pushes Instance forward (product).
            PushesForwardRight,
            /// The adjunction ΣF ⊣ ΔF ⊣ ΠF.
            AdjointTo,
        ],
        edges: [
            // Instance contains Populations and Assignments
            (Instance, Population, Contains),
            (Instance, Assignment, ContainsAssignment),
            // Population contains Individuals
            (Population, Individual, ContainsIndividuals),
            // Assignment maps between Populations
            (Assignment, Population, MapsBetween),
            // Instance satisfies InstanceConstraint
            (Instance, InstanceConstraint, Satisfies),
            // Migration functors operate on Instances
            (DeltaMigration, Instance, PullsBack),
            (SigmaMigration, Instance, PushesForwardLeft),
            (PiMigration, Instance, PushesForwardRight),
            // The adjunction: ΣF ⊣ ΔF ⊣ ΠF
            (SigmaMigration, MigrationAdjunction, AdjointTo),
            (DeltaMigration, MigrationAdjunction, AdjointTo),
            (PiMigration, MigrationAdjunction, AdjointTo),
        ],
        composed: [
            // Instance → Individual (through Population)
            (Instance, Individual),
            // Migration → Population (through Instance)
            (DeltaMigration, Population),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws_hold() {
        check_category_laws::<InstanceCategory>().unwrap();
    }

    #[test]
    fn has_nine_concepts() {
        assert_eq!(InstanceConcept::variants().len(), 9);
    }

    // --- Spivak (2012): Instance contains Populations ---

    #[test]
    fn instance_contains_populations() {
        let m = InstanceCategory::morphisms();
        assert!(m.iter().any(|r| r.from == InstanceConcept::Instance
            && r.to == InstanceConcept::Population
            && r.kind == InstanceRelationKind::Contains));
    }

    // --- Spivak: Instance contains Assignments (morphism images) ---

    #[test]
    fn instance_contains_assignments() {
        let m = InstanceCategory::morphisms();
        assert!(m.iter().any(|r| r.from == InstanceConcept::Instance
            && r.to == InstanceConcept::Assignment
            && r.kind == InstanceRelationKind::ContainsAssignment));
    }

    // --- Spivak: Assignment maps between Populations ---

    #[test]
    fn assignment_maps_between_populations() {
        let m = InstanceCategory::morphisms();
        assert!(m.iter().any(|r| r.from == InstanceConcept::Assignment
            && r.to == InstanceConcept::Population
            && r.kind == InstanceRelationKind::MapsBetween));
    }

    // --- Spivak (2012): ΣF ⊣ ΔF ⊣ ΠF (three migration functors) ---

    #[test]
    fn three_migration_functors_exist() {
        let m = InstanceCategory::morphisms();
        // Delta pulls back
        assert!(m.iter().any(|r| r.from == InstanceConcept::DeltaMigration
            && r.to == InstanceConcept::Instance
            && r.kind == InstanceRelationKind::PullsBack));
        // Sigma pushes forward (left)
        assert!(m.iter().any(|r| r.from == InstanceConcept::SigmaMigration
            && r.to == InstanceConcept::Instance
            && r.kind == InstanceRelationKind::PushesForwardLeft));
        // Pi pushes forward (right)
        assert!(m.iter().any(|r| r.from == InstanceConcept::PiMigration
            && r.to == InstanceConcept::Instance
            && r.kind == InstanceRelationKind::PushesForwardRight));
    }

    // --- Spivak: all three participate in the adjunction ---

    #[test]
    fn migration_adjunction() {
        let m = InstanceCategory::morphisms();
        for functor in [
            InstanceConcept::SigmaMigration,
            InstanceConcept::DeltaMigration,
            InstanceConcept::PiMigration,
        ] {
            assert!(
                m.iter().any(|r| r.from == functor
                    && r.to == InstanceConcept::MigrationAdjunction
                    && r.kind == InstanceRelationKind::AdjointTo),
                "{functor:?} should participate in the migration adjunction"
            );
        }
    }

    // --- Instance reaches Individual (through Population) ---

    #[test]
    fn instance_reaches_individuals() {
        let m = InstanceCategory::morphisms();
        assert!(m.iter().any(|r| r.from == InstanceConcept::Instance
            && r.to == InstanceConcept::Individual));
    }

    // --- Instance satisfies constraints (path equations) ---

    #[test]
    fn instance_satisfies_constraints() {
        let m = InstanceCategory::morphisms();
        assert!(m.iter().any(|r| r.from == InstanceConcept::Instance
            && r.to == InstanceConcept::InstanceConstraint
            && r.kind == InstanceRelationKind::Satisfies));
    }
}
