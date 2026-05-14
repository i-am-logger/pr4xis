//! Instance — the Spivak instance functor and the three migration
//! functors Σ ⊣ Δ ⊣ Π.
//!
//! An Instance is a functor I: Schema → Set: it populates a schema with
//! actual data. For each entity type it provides a set of individuals;
//! for each morphism type, a function between those sets.
//!
//! # Literature
//!
//! - **Spivak (2012)** "Functorial Data Migration", *Information and
//!   Computation* 217:31-51 — Instance as functor; the three migration
//!   functors ΣF ⊣ ΔF ⊣ ΠF.
//! - **Spivak & Wisnesky (2015)** "Relational Foundations for
//!   Functorial Data Migration", *DBPL 2015*.
//! - **Wisnesky, Schultz & Spivak (2017)** "Algebraic Databases",
//!   *Theory and Applications of Categories* — CQL implementation.
//! - **Baader, Calvanese, McGuinness, Nardi, Patel-Schneider (2003)**
//!   *The Description Logic Handbook*, Cambridge UP — ABox.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Instance",
    source: "Spivak (2012) Functorial Data Migration, Information and Computation 217:31-51; Spivak & Wisnesky (2015) Relational Foundations for Functorial Data Migration, DBPL; Wisnesky, Schultz & Spivak (2017) Algebraic Databases, TAC; Baader et al. (2003) The Description Logic Handbook",

    concepts: [
        Instance,
        Population,
        Assignment,
        Individual,
        InstanceConstraint,
        DeltaMigration,
        SigmaMigration,
        PiMigration,
        MigrationAdjunction,
    ],

    labels: {
        Instance: ("en", "Instance",
            "Spivak (2012): a functor I: C → Set. Baader (2003) ABox."),
        Population: ("en", "Population",
            "Spivak (2012): I(c) for an entity type c - a set of individuals."),
        Assignment: ("en", "Assignment",
            "Spivak (2012): I(f) for a morphism f - a function between populations."),
        Individual: ("en", "Individual",
            "Baader (2003): an ABox assertion a:C - a specific individual in a population."),
        InstanceConstraint: ("en", "Instance constraint",
            "Spivak (2012): a path equation - a commutative-diagram law instances must satisfy."),
        DeltaMigration: ("en", "Delta migration",
            "Spivak (2012): ΔF: D-Inst → C-Inst - pullback migration; restricts/projects data."),
        SigmaMigration: ("en", "Sigma migration",
            "Spivak (2012): ΣF: C-Inst → D-Inst - left pushforward; pushes data via coproduct (union)."),
        PiMigration: ("en", "Pi migration",
            "Spivak (2012): ΠF: C-Inst → D-Inst - right pushforward; pushes data via product (universal)."),
        MigrationAdjunction: ("en", "Migration adjunction",
            "Spivak (2012): ΣF ⊣ ΔF ⊣ ΠF - the three migration functors form a chain of adjunctions."),
    },

    edges: [
        // Spivak (2012) instance structure.
        (Instance, Population, Contains),
        (Instance, Assignment, ContainsAssignment),
        (Population, Individual, ContainsIndividuals),
        (Assignment, Population, MapsBetween),
        // Spivak (2012): instance satisfies path equations.
        (Instance, InstanceConstraint, Satisfies),
        // Spivak (2012): migration functors operate on instances.
        (DeltaMigration, Instance, PullsBack),
        (SigmaMigration, Instance, PushesForwardLeft),
        (PiMigration, Instance, PushesForwardRight),
        // The chain of adjunctions ΣF ⊣ ΔF ⊣ ΠF.
        (SigmaMigration, MigrationAdjunction, AdjointTo),
        (DeltaMigration, MigrationAdjunction, AdjointTo),
        (PiMigration, MigrationAdjunction, AdjointTo),
    ],
}

/// Quality: whether a concept is one of the three Spivak migration
/// functors that participate in the ΣF ⊣ ΔF ⊣ ΠF adjunction.
#[derive(Debug, Clone)]
pub struct IsMigrationFunctor;

impl Quality for IsMigrationFunctor {
    type Individual = InstanceConcept;
    type Value = bool;

    fn get(&self, c: &InstanceConcept) -> Option<bool> {
        use InstanceConcept as I;
        Some(matches!(
            c,
            I::DeltaMigration | I::SigmaMigration | I::PiMigration
        ))
    }
}

impl Ontology for InstanceOntology {
    type Cat = InstanceCategory;
    type Qual = IsMigrationFunctor;

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
        assert_category_laws::<InstanceCategory>();
    }

    #[test]
    fn ontology_validates() {
        InstanceOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn nine_concepts() {
        assert_eq!(InstanceConcept::variants().len(), 9);
    }

    #[test]
    fn instance_contains_populations() {
        let m = InstanceCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == InstanceConcept::Instance
            && r.target() == InstanceConcept::Population
            && r.kind() == InstanceRelationKind::Contains));
    }

    #[test]
    fn three_migration_functors() {
        // Spivak (2012) ΣF ⊣ ΔF ⊣ ΠF.
        let m = InstanceCategory::morphisms();
        assert!(
            m.iter()
                .any(|r| r.source() == InstanceConcept::DeltaMigration
                    && r.target() == InstanceConcept::Instance
                    && r.kind() == InstanceRelationKind::PullsBack)
        );
        assert!(
            m.iter()
                .any(|r| r.source() == InstanceConcept::SigmaMigration
                    && r.target() == InstanceConcept::Instance
                    && r.kind() == InstanceRelationKind::PushesForwardLeft)
        );
        assert!(m.iter().any(|r| r.source() == InstanceConcept::PiMigration
            && r.target() == InstanceConcept::Instance
            && r.kind() == InstanceRelationKind::PushesForwardRight));
    }

    #[test]
    fn migration_adjunction_chain() {
        // All three migration functors participate in MigrationAdjunction.
        let m = InstanceCategory::morphisms();
        for functor in [
            InstanceConcept::SigmaMigration,
            InstanceConcept::DeltaMigration,
            InstanceConcept::PiMigration,
        ] {
            assert!(m.iter().any(|r| r.source() == functor
                && r.target() == InstanceConcept::MigrationAdjunction
                && r.kind() == InstanceRelationKind::AdjointTo));
        }
    }

    #[test]
    fn instance_satisfies_constraints() {
        let m = InstanceCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == InstanceConcept::Instance
            && r.target() == InstanceConcept::InstanceConstraint
            && r.kind() == InstanceRelationKind::Satisfies));
    }

    fn arb_concept() -> impl Strategy<Value = InstanceConcept> {
        proptest::sample::select(InstanceConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in InstanceCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in InstanceOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_migration_functor_total(c in arb_concept()) {
            prop_assert!(IsMigrationFunctor.get(&c).is_some());
        }
    }
}
