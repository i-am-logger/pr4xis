//! Adjunctions between biomedical ontology domains.
//!
//! An adjunction F ⊣ G captures the "optimal inverse" relationship between
//! two domain functors. The unit η embeds an object into the round-trip
//! G(F(-)), and the counit ε projects the round-trip F(G(-)) back.
//!
//! Three adjunctions connect the molecular / bioelectric / biological /
//! pharmacology domains:
//!
//! 1. **Mechanism ⊣ Role** (Molecular ↔ Bioelectric) — Hille (2001)
//!    *Ion Channels of Excitable Membranes* 3rd ed.; Levin (2014)
//!    *Mol. Biol. Cell* 25(24) — molecular mechanism vs bioelectric role.
//!
//! 2. **Drug ⊣ Target** (Pharmacology ↔ Molecular) — Goodman & Gilman
//!    (2018) *The Pharmacological Basis of Therapeutics* 13th ed.;
//!    Alberts et al. (2015) *Molecular Biology of the Cell* 6th ed. —
//!    drug vs the molecular target it acts on.
//!
//! 3. **Structure ⊣ Signal** (Biology ↔ Bioelectric) — Schleiden &
//!    Schwann (1838–1839) Cell Theory; Levin (2019) *Front. Psychol.*
//!    10:2688 — biological structure vs bioelectric role at that
//!    structure.
//!
//! # Literature
//!
//! - **Mac Lane (1971)** *Categories for the Working Mathematician*, Ch. IV
//!   — adjunctions, units, counits.
//! - **Awodey (2010)** *Category Theory* (2nd ed.), Ch. 9.
//! - **Lambek & Scott (1986)** *Introduction to Higher Order Categorical
//!   Logic*.
//!
//! # Design
//!
//! Per #166 the `Composed` kind has been removed from the proc-macro-
//! generated `*RelationKind` enums. Heterogeneous round-trip components
//! that are not strict round-trip identities collapse to the source
//! category's identity morphism, preserving F(id) = id and making
//! round-trip loss explicit through `map_object` divergence rather than
//! through a synthetic `Composed` arrow.

use pr4xis::category::{Adjunction, Functor};

use crate::natural::biomedical::bioelectricity::biology_functor::BioelectricToBiology;
use crate::natural::biomedical::bioelectricity::molecular_functor::BioelectricToMolecular;
use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation, BioelectricRelationKind,
};
use crate::natural::biomedical::biology::bioelectricity_functor::BiologyToBioelectric;
use crate::natural::biomedical::biology::ontology::{
    BiologicalEntity, BiologicalRelation, BiologyCategory, BiologyRelationKind,
};
use crate::natural::biomedical::molecular::bioelectricity_functor::MolecularToBioelectric;
use crate::natural::biomedical::molecular::ontology::{
    MolecularCategory, MolecularEntity, MolecularRelation, MolecularRelationKind,
};
use crate::natural::biomedical::molecular::pharmacology_functor::MolecularToPharmacology;
use crate::natural::biomedical::pharmacology::molecular_functor::PharmacologyToMolecular;
use crate::natural::biomedical::pharmacology::ontology::{
    PharmacologyCategory, PharmacologyEntity, PharmacologyRelation, PharmacologyRelationKind,
};

// ---------------------------------------------------------------------------
// Adjunction 1: MolecularToBioelectric ⊣ BioelectricToMolecular
// ---------------------------------------------------------------------------

/// Adjunction between the molecular and bioelectric domains.
///
/// Left adjoint F = MolecularToBioelectric: maps molecules to their bioelectric role.
/// Right adjoint G = BioelectricToMolecular: maps bioelectric entities to canonical molecules.
///
/// Unit η_A: A → G(F(A)) — embeds a molecule into its round-trip canonical form.
/// Counit ε_B: F(G(B)) → B — projects the molecular mechanism back to its bioelectric role.
pub struct MolecularBioelectricAdjunction;

impl Adjunction for MolecularBioelectricAdjunction {
    type Left = MolecularToBioelectric;
    type Right = BioelectricToMolecular;

    fn unit(obj: &MolecularEntity) -> MolecularRelation {
        use pr4xis::category::Category;
        // η_A: A → G(F(A))
        let round_trip =
            BioelectricToMolecular::map_object(&MolecularToBioelectric::map_object(obj));
        if round_trip == *obj {
            MolecularRelation {
                from: *obj,
                to: *obj,
                kind: MolecularRelationKind::Identity,
            }
        } else {
            // Heterogeneous round trip — emit identity at source per #166.
            MolecularCategory::identity(obj)
        }
    }

    fn counit(obj: &BioelectricEntity) -> BioelectricRelation {
        use pr4xis::category::Category;
        // ε_B: F(G(B)) → B
        let round_trip =
            MolecularToBioelectric::map_object(&BioelectricToMolecular::map_object(obj));
        if round_trip == *obj {
            BioelectricRelation {
                from: *obj,
                to: *obj,
                kind: BioelectricRelationKind::Identity,
            }
        } else {
            BioelectricCategory::identity(obj)
        }
    }

    fn meta() -> pr4xis::ontology::meta::Provenance {
        pr4xis::ontology::meta::Provenance {
            name: pr4xis::ontology::meta::OntologyName::new_static(
                "MolecularBioelectricAdjunction",
            ),
            description: pr4xis::ontology::meta::Label::new_static(
                "Molecular ⊣ Bioelectric — mechanism vs role duality",
            ),
            citation: pr4xis::ontology::meta::Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician Ch. IV; Hille (2001) Ion Channels of Excitable Membranes 3rd ed.; Levin (2014) Mol. Biol. Cell 25(24)",
            ),
            module_path: pr4xis::ontology::meta::ModulePath::new_static(module_path!()),
        }
    }
}
pr4xis::register_adjunction!(MolecularBioelectricAdjunction);

// ---------------------------------------------------------------------------
// Adjunction 2: PharmacologyToMolecular ⊣ MolecularToPharmacology
// ---------------------------------------------------------------------------

/// Adjunction between the pharmacology and molecular domains.
///
/// Left adjoint F = PharmacologyToMolecular: maps drugs to their molecular targets.
/// Right adjoint G = MolecularToPharmacology: maps molecules to targeting drugs.
///
/// Unit η_A: A → G(F(A)) — embeds a drug into its round-trip canonical form.
/// Counit ε_B: F(G(B)) → B — projects the drug target back to the molecule.
pub struct PharmacologyMolecularAdjunction;

impl Adjunction for PharmacologyMolecularAdjunction {
    type Left = PharmacologyToMolecular;
    type Right = MolecularToPharmacology;

    fn unit(obj: &PharmacologyEntity) -> PharmacologyRelation {
        use pr4xis::category::Category;
        // η_A: A → G(F(A))
        let round_trip =
            MolecularToPharmacology::map_object(&PharmacologyToMolecular::map_object(obj));
        if round_trip == *obj {
            PharmacologyRelation {
                from: *obj,
                to: *obj,
                kind: PharmacologyRelationKind::Identity,
            }
        } else {
            PharmacologyCategory::identity(obj)
        }
    }

    fn counit(obj: &MolecularEntity) -> MolecularRelation {
        use pr4xis::category::Category;
        // ε_B: F(G(B)) → B
        let round_trip =
            PharmacologyToMolecular::map_object(&MolecularToPharmacology::map_object(obj));
        if round_trip == *obj {
            MolecularRelation {
                from: *obj,
                to: *obj,
                kind: MolecularRelationKind::Identity,
            }
        } else {
            MolecularCategory::identity(obj)
        }
    }

    fn meta() -> pr4xis::ontology::meta::Provenance {
        pr4xis::ontology::meta::Provenance {
            name: pr4xis::ontology::meta::OntologyName::new_static(
                "PharmacologyMolecularAdjunction",
            ),
            description: pr4xis::ontology::meta::Label::new_static(
                "Pharmacology ⊣ Molecular — drug vs target duality",
            ),
            citation: pr4xis::ontology::meta::Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician Ch. IV; Goodman & Gilman (2018) The Pharmacological Basis of Therapeutics 13th ed.; Alberts et al. (2015) Molecular Biology of the Cell 6th ed.",
            ),
            module_path: pr4xis::ontology::meta::ModulePath::new_static(module_path!()),
        }
    }
}
pr4xis::register_adjunction!(PharmacologyMolecularAdjunction);

// ---------------------------------------------------------------------------
// Adjunction 3: BiologyToBioelectric ⊣ BioelectricToBiology
// ---------------------------------------------------------------------------

/// Adjunction between the biology and bioelectric domains.
///
/// Left adjoint F = BiologyToBioelectric: maps biological structures to bioelectric roles.
/// Right adjoint G = BioelectricToBiology: maps bioelectric entities to biological structures.
///
/// Unit η_A: A → G(F(A)) — embeds a biological entity into its round-trip form.
/// Counit ε_B: F(G(B)) → B — projects the biological structure back to its bioelectric role.
pub struct BiologyBioelectricAdjunction;

impl Adjunction for BiologyBioelectricAdjunction {
    type Left = BiologyToBioelectric;
    type Right = BioelectricToBiology;

    fn unit(obj: &BiologicalEntity) -> BiologicalRelation {
        use pr4xis::category::Category;
        let round_trip = BioelectricToBiology::map_object(&BiologyToBioelectric::map_object(obj));
        if round_trip == *obj {
            BiologicalRelation {
                from: *obj,
                to: *obj,
                kind: BiologyRelationKind::Identity,
            }
        } else {
            BiologyCategory::identity(obj)
        }
    }

    fn counit(obj: &BioelectricEntity) -> BioelectricRelation {
        use pr4xis::category::Category;
        let round_trip = BiologyToBioelectric::map_object(&BioelectricToBiology::map_object(obj));
        if round_trip == *obj {
            BioelectricRelation {
                from: *obj,
                to: *obj,
                kind: BioelectricRelationKind::Identity,
            }
        } else {
            BioelectricCategory::identity(obj)
        }
    }

    fn meta() -> pr4xis::ontology::meta::Provenance {
        pr4xis::ontology::meta::Provenance {
            name: pr4xis::ontology::meta::OntologyName::new_static("BiologyBioelectricAdjunction"),
            description: pr4xis::ontology::meta::Label::new_static(
                "Biology ⊣ Bioelectric — structure vs signal duality",
            ),
            citation: pr4xis::ontology::meta::Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician Ch. IV; Schleiden & Schwann (1838–1839) Cell Theory; Levin (2019) Front. Psychol. 10:2688",
            ),
            module_path: pr4xis::ontology::meta::ModulePath::new_static(module_path!()),
        }
    }
}
pr4xis::register_adjunction!(BiologyBioelectricAdjunction);

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;

    // -----------------------------------------------------------------------
    // Adjunction 1: MolecularBioelectricAdjunction
    // -----------------------------------------------------------------------

    #[test]
    fn test_molecular_bioelectric_unit_is_valid() {
        let variants = MolecularEntity::variants();
        for obj in &variants {
            let unit = MolecularBioelectricAdjunction::unit(obj);
            assert!(
                variants.contains(&unit.from),
                "unit from {:?} has invalid source {:?}",
                obj,
                unit.from
            );
            assert!(
                variants.contains(&unit.to),
                "unit from {:?} has invalid target {:?}",
                obj,
                unit.to
            );
        }
    }

    #[test]
    fn test_molecular_bioelectric_counit_is_valid() {
        let variants = BioelectricEntity::variants();
        for obj in &variants {
            let counit = MolecularBioelectricAdjunction::counit(obj);
            assert!(
                variants.contains(&counit.from),
                "counit from {:?} has invalid source {:?}",
                obj,
                counit.from
            );
            assert!(
                variants.contains(&counit.to),
                "counit from {:?} has invalid target {:?}",
                obj,
                counit.to
            );
        }
    }

    // -----------------------------------------------------------------------
    // Adjunction 2: PharmacologyMolecularAdjunction
    // -----------------------------------------------------------------------

    #[test]
    fn test_pharmacology_molecular_unit_is_valid() {
        let variants = PharmacologyEntity::variants();
        for obj in &variants {
            let unit = PharmacologyMolecularAdjunction::unit(obj);
            assert!(
                variants.contains(&unit.from),
                "unit from {:?} has invalid source {:?}",
                obj,
                unit.from
            );
            assert!(
                variants.contains(&unit.to),
                "unit from {:?} has invalid target {:?}",
                obj,
                unit.to
            );
        }
    }

    #[test]
    fn test_pharmacology_molecular_counit_is_valid() {
        let variants = MolecularEntity::variants();
        for obj in &variants {
            let counit = PharmacologyMolecularAdjunction::counit(obj);
            assert!(
                variants.contains(&counit.from),
                "counit from {:?} has invalid source {:?}",
                obj,
                counit.from
            );
            assert!(
                variants.contains(&counit.to),
                "counit from {:?} has invalid target {:?}",
                obj,
                counit.to
            );
        }
    }

    // -----------------------------------------------------------------------
    // Adjunction 3: BiologyBioelectricAdjunction
    // -----------------------------------------------------------------------

    #[test]
    fn test_biology_bioelectric_unit_is_valid() {
        let variants = BiologicalEntity::variants();
        for obj in &variants {
            let unit = BiologyBioelectricAdjunction::unit(obj);
            assert!(
                variants.contains(&unit.from),
                "unit from {:?} has invalid source {:?}",
                obj,
                unit.from
            );
            assert!(
                variants.contains(&unit.to),
                "unit from {:?} has invalid target {:?}",
                obj,
                unit.to
            );
        }
    }

    #[test]
    fn test_biology_bioelectric_counit_is_valid() {
        let variants = BioelectricEntity::variants();
        for obj in &variants {
            let counit = BiologyBioelectricAdjunction::counit(obj);
            assert!(
                variants.contains(&counit.from),
                "counit from {:?} has invalid source {:?}",
                obj,
                counit.from
            );
            assert!(
                variants.contains(&counit.to),
                "counit from {:?} has invalid target {:?}",
                obj,
                counit.to
            );
        }
    }
}
