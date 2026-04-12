//! Functor: ChemistryCategory -> MolecularCategory.
//!
//! Proves that foundational chemistry has a structure-preserving map into
//! molecular biology. Electrolytes and ions map to Ion, bonds map to Protein
//! (covalent/hydrogen/VanDerWaals/metallic) or Ion (ionic), states of matter
//! map to their biological molecular constituents, and physical properties
//! map to Ion (electrochemical basis).
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use praxis::category::{Functor, Relationship};

use crate::science::biomedical::chemistry::ontology::{
    ChemistryCategory, ChemistryEntity, ChemistryRelation,
};
use crate::science::biomedical::molecular::ontology::{
    MolecularCategory, MolecularEntity, MolecularRelation,
};

/// Structure-preserving map from chemistry entities to molecular components.
pub struct ChemistryToMolecular;

impl Functor for ChemistryToMolecular {
    type Source = ChemistryCategory;
    type Target = MolecularCategory;

    fn map_object(obj: &ChemistryEntity) -> MolecularEntity {
        use ChemistryEntity as C;
        use MolecularEntity as M;
        match obj {
            // Solution components -> Ion (electrolyte/ionic basis)
            C::Electrolyte => M::Ion,
            C::Buffer => M::Ion,
            C::Solvent => M::Sodium, // water is the ionic medium; Na+ is primary osmolyte
            C::Solute => M::Ion,

            // Bonding -> Protein or Ion
            C::IonicBond => M::Ion,
            C::CovalentBond => M::Protein,
            C::HydrogenBond => M::Protein, // protein folding
            C::VanDerWaals => M::Protein,
            C::Metallic => M::Protein,

            // Physical properties -> Ion (electrochemical basis)
            C::PH => M::Proton,
            C::Concentration => M::Ion,
            C::Osmolarity => M::Sodium, // Na+ is the primary osmolyte
            C::Temperature => M::Ion,
            C::Pressure => M::Ion,

            // States of matter -> biological molecular constituents
            C::Solid => M::Collagen, // structural solid
            C::Liquid => M::Mucin,   // biological fluids
            C::Gel => M::Mucin,      // biological gel
            C::Colloid => M::Mucin,  // biological colloid
            C::Gas => M::Ion,        // dissolved gases as ions
            C::Plasma => M::Calcium, // ionized plasma; Ca2+ is key plasma ion

            // Abstract categories
            C::StateOfMatter => M::Ion,
            C::ChemicalBond => M::Protein,
            C::PhysicalProperty => M::Ion,
            C::SolutionComponent => M::Ion,
        }
    }

    fn map_morphism(m: &ChemistryRelation) -> MolecularRelation {
        MolecularRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::validate::check_functor_laws;
    use praxis::category::{Category, Entity};
    use praxis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<ChemistryToMolecular>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<ChemistryToMolecular>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in ChemistryEntity::variants() {
            let id_src = ChemistryCategory::identity(&obj);
            let mapped_id = ChemistryToMolecular::map_morphism(&id_src);
            let id_tgt = MolecularCategory::identity(&ChemistryToMolecular::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        let objs = ChemistryEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = ChemistryRelation { from: a, to: b };
                    let g = ChemistryRelation { from: b, to: c };
                    let composed = ChemistryCategory::compose(&f, &g).unwrap();
                    let mapped_composed = ChemistryToMolecular::map_morphism(&composed);
                    let composed_mapped = MolecularCategory::compose(
                        &ChemistryToMolecular::map_morphism(&f),
                        &ChemistryToMolecular::map_morphism(&g),
                    )
                    .unwrap();
                    assert_eq!(
                        mapped_composed, composed_mapped,
                        "composition law failed for {:?} -> {:?} -> {:?}",
                        a, b, c
                    );
                }
            }
        }
    }

    // -- Specific mapping tests --

    #[test]
    fn test_electrolyte_maps_to_ion() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::Electrolyte),
            MolecularEntity::Ion,
        );
    }

    #[test]
    fn test_buffer_maps_to_ion() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::Buffer),
            MolecularEntity::Ion,
        );
    }

    #[test]
    fn test_solvent_maps_to_sodium() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::Solvent),
            MolecularEntity::Sodium,
        );
    }

    #[test]
    fn test_ionic_bond_maps_to_ion() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::IonicBond),
            MolecularEntity::Ion,
        );
    }

    #[test]
    fn test_covalent_bond_maps_to_protein() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::CovalentBond),
            MolecularEntity::Protein,
        );
    }

    #[test]
    fn test_hydrogen_bond_maps_to_protein() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::HydrogenBond),
            MolecularEntity::Protein,
        );
    }

    #[test]
    fn test_ph_maps_to_proton() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::PH),
            MolecularEntity::Proton,
        );
    }

    #[test]
    fn test_osmolarity_maps_to_sodium() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::Osmolarity),
            MolecularEntity::Sodium,
        );
    }

    #[test]
    fn test_solid_maps_to_collagen() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::Solid),
            MolecularEntity::Collagen,
        );
    }

    #[test]
    fn test_liquid_maps_to_mucin() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::Liquid),
            MolecularEntity::Mucin,
        );
    }

    #[test]
    fn test_gel_maps_to_mucin() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::Gel),
            MolecularEntity::Mucin,
        );
    }

    #[test]
    fn test_plasma_maps_to_calcium() {
        assert_eq!(
            ChemistryToMolecular::map_object(&ChemistryEntity::Plasma),
            MolecularEntity::Calcium,
        );
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = MolecularEntity::variants();
        for obj in ChemistryEntity::variants() {
            let mapped = ChemistryToMolecular::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid MolecularEntity",
                obj,
                mapped
            );
        }
    }
}
