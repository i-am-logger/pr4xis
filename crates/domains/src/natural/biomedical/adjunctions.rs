//! Adjunctions between ontology domains.
//!
//! An adjunction F ⊣ G captures the "optimal inverse" relationship between
//! two domain functors. The unit η embeds an object into the round-trip G(F(-)),
//! and the counit ε projects the round-trip F(G(-)) back.
//!
//! Three adjunctions connect the four core ontology domains:
//!
//! 1. MolecularToBioelectric ⊣ BioelectricToMolecular
//!    Molecular ⇄ Bioelectric: molecules ↔ bioelectric roles
//!
//! 2. PharmacologyToMolecular ⊣ MolecularToPharmacology
//!    Pharmacology ⇄ Molecular: drugs ↔ molecular targets
//!
//! 3. BiologyToBioelectric ⊣ BioelectricToBiology
//!    Biology ⇄ Bioelectric: biological structures ↔ bioelectric roles

use pr4xis::category::Adjunction;
use pr4xis::category::Functor;

use crate::natural::biomedical::bioelectricity::biology_functor::BioelectricToBiology;
use crate::natural::biomedical::bioelectricity::molecular_functor::BioelectricToMolecular;
use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricEntity, BioelectricRelation, BioelectricRelationKind,
};
use crate::natural::biomedical::biology::bioelectricity_functor::BiologyToBioelectric;
use crate::natural::biomedical::biology::ontology::{
    BiologicalEntity, BiologicalRelation, BiologyCategoryRelationKind,
};
use crate::natural::biomedical::molecular::bioelectricity_functor::MolecularToBioelectric;
use crate::natural::biomedical::molecular::ontology::{
    MolecularCategoryRelationKind, MolecularEntity, MolecularRelation,
};
use crate::natural::biomedical::molecular::pharmacology_functor::MolecularToPharmacology;
use crate::natural::biomedical::pharmacology::molecular_functor::PharmacologyToMolecular;
use crate::natural::biomedical::pharmacology::ontology::{
    PharmacologyCategoryRelationKind, PharmacologyEntity, PharmacologyRelation,
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
        // η_A: A → G(F(A))
        let round_trip =
            BioelectricToMolecular::map_object(&MolecularToBioelectric::map_object(obj));
        let kind = if round_trip == *obj {
            MolecularCategoryRelationKind::Identity
        } else {
            MolecularCategoryRelationKind::Composed
        };
        MolecularRelation {
            from: *obj,
            to: round_trip,
            kind,
        }
    }

    fn counit(obj: &BioelectricEntity) -> BioelectricRelation {
        // ε_B: F(G(B)) → B
        let round_trip =
            MolecularToBioelectric::map_object(&BioelectricToMolecular::map_object(obj));
        let kind = if round_trip == *obj {
            BioelectricRelationKind::Identity
        } else {
            BioelectricRelationKind::Composed
        };
        BioelectricRelation {
            from: round_trip,
            to: *obj,
            kind,
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
        // η_A: A → G(F(A))
        let round_trip =
            MolecularToPharmacology::map_object(&PharmacologyToMolecular::map_object(obj));
        let kind = if round_trip == *obj {
            PharmacologyCategoryRelationKind::Identity
        } else {
            PharmacologyCategoryRelationKind::Composed
        };
        PharmacologyRelation {
            from: *obj,
            to: round_trip,
            kind,
        }
    }

    fn counit(obj: &MolecularEntity) -> MolecularRelation {
        // ε_B: F(G(B)) → B
        let round_trip =
            PharmacologyToMolecular::map_object(&MolecularToPharmacology::map_object(obj));
        let kind = if round_trip == *obj {
            MolecularCategoryRelationKind::Identity
        } else {
            MolecularCategoryRelationKind::Composed
        };
        MolecularRelation {
            from: round_trip,
            to: *obj,
            kind,
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
        // η_A: A → G(F(A))
        let round_trip = BioelectricToBiology::map_object(&BiologyToBioelectric::map_object(obj));
        let kind = if round_trip == *obj {
            BiologyCategoryRelationKind::Identity
        } else {
            BiologyCategoryRelationKind::Composed
        };
        BiologicalRelation {
            from: *obj,
            to: round_trip,
            kind,
        }
    }

    fn counit(obj: &BioelectricEntity) -> BioelectricRelation {
        // ε_B: F(G(B)) → B
        let round_trip = BiologyToBioelectric::map_object(&BioelectricToBiology::map_object(obj));
        let kind = if round_trip == *obj {
            BioelectricRelationKind::Identity
        } else {
            BioelectricRelationKind::Composed
        };
        BioelectricRelation {
            from: round_trip,
            to: *obj,
            kind,
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
    use pr4xis::category::Entity;

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
