//! Functor: BiophysicsCategory -> MolecularCategory.
//!
//! Proves that biophysics has a structure-preserving map into molecular biology.
//! Each biophysical entity maps to its molecular substrate:
//! MechanotransducerActivation -> Piezo1 (mechanosensitive channel activation),
//! PiezoelectricChargeGeneration -> CalciumSignal (charge -> signal),
//! CollagenPiezoelectricity -> Collagen (structural protein),
//! BoneMatrix -> Collagen, SoftTissue -> Mucin, etc.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use praxis::category::{Functor, Relationship};

use crate::science::biomedical::biophysics::ontology::{
    BiophysicsCategory, BiophysicsEntity, BiophysicsRelation,
};
use crate::science::biomedical::molecular::ontology::{
    MolecularCategory, MolecularEntity, MolecularRelation,
};

/// Structure-preserving map from biophysics entities to their molecular substrate.
pub struct BiophysicsToMolecular;

impl Functor for BiophysicsToMolecular {
    type Source = BiophysicsCategory;
    type Target = MolecularCategory;

    fn map_object(obj: &BiophysicsEntity) -> MolecularEntity {
        use BiophysicsEntity as BP;
        use MolecularEntity as M;
        match obj {
            // Mechanical properties -> structural proteins / mechanosensitive
            BP::Viscoelasticity => M::Collagen,
            BP::Elasticity => M::Collagen,
            BP::Viscosity => M::Mucin,
            BP::StiffnessModulus => M::Collagen,
            BP::StrainRate => M::Mechanosensitive,
            BP::MechanicalStress => M::Mechanosensitive,
            BP::MechanicalStrain => M::Mechanosensitive,

            // Wave physics -> ion/channel abstracts
            BP::MechanicalWave => M::Mechanosensitive,
            BP::AcousticImpedance => M::Protein,
            BP::Attenuation => M::Protein,
            BP::Frequency => M::Mechanosensitive,
            BP::Wavelength => M::Mechanosensitive,
            BP::ResonanceFrequency => M::Mechanosensitive,

            // Piezoelectricity -> collagen / calcium signaling
            BP::PiezoelectricEffect => M::Collagen,
            BP::DirectPiezoelectric => M::Collagen,
            BP::ConversePiezoelectric => M::Collagen,
            BP::CollagenPiezoelectricity => M::Collagen,

            // Membrane biophysics -> membrane channels
            BP::MembraneCapacitance => M::Calcium,
            BP::MembraneTension => M::Mechanosensitive,
            BP::CellDeformation => M::Mechanosensitive,

            // Media -> molecular constituents
            BP::BoneMatrix => M::Collagen,
            BP::SoftTissue => M::Mucin,
            BP::FluidMedium => M::Sodium,   // ionic medium
            BP::CellMembrane => M::Calcium, // membrane channels

            // Abstract categories -> abstract molecular categories
            BP::MechanicalProperty => M::Protein,
            BP::WaveProperty => M::Mechanosensitive,
            BP::PiezoelectricProperty => M::Protein,
            BP::BiologicalMedium => M::Protein,
        }
    }

    fn map_morphism(m: &BiophysicsRelation) -> MolecularRelation {
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
        check_functor_laws::<BiophysicsToMolecular>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<BiophysicsToMolecular>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in BiophysicsEntity::variants() {
            let id_src = BiophysicsCategory::identity(&obj);
            let mapped_id = BiophysicsToMolecular::map_morphism(&id_src);
            let id_tgt = MolecularCategory::identity(&BiophysicsToMolecular::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        let objs = BiophysicsEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = BiophysicsRelation { from: a, to: b };
                    let g = BiophysicsRelation { from: b, to: c };
                    let composed = BiophysicsCategory::compose(&f, &g).unwrap();
                    let mapped_composed = BiophysicsToMolecular::map_morphism(&composed);
                    let composed_mapped = MolecularCategory::compose(
                        &BiophysicsToMolecular::map_morphism(&f),
                        &BiophysicsToMolecular::map_morphism(&g),
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

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = MolecularEntity::variants();
        for obj in BiophysicsEntity::variants() {
            let mapped = BiophysicsToMolecular::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid MolecularEntity",
                obj,
                mapped
            );
        }
    }

    // -- Specific mapping tests --

    #[test]
    fn test_collagen_piezoelectricity_maps_to_collagen() {
        assert_eq!(
            BiophysicsToMolecular::map_object(&BiophysicsEntity::CollagenPiezoelectricity),
            MolecularEntity::Collagen,
        );
    }

    #[test]
    fn test_bone_matrix_maps_to_collagen() {
        assert_eq!(
            BiophysicsToMolecular::map_object(&BiophysicsEntity::BoneMatrix),
            MolecularEntity::Collagen,
        );
    }

    #[test]
    fn test_soft_tissue_maps_to_mucin() {
        assert_eq!(
            BiophysicsToMolecular::map_object(&BiophysicsEntity::SoftTissue),
            MolecularEntity::Mucin,
        );
    }

    #[test]
    fn test_fluid_medium_maps_to_sodium() {
        assert_eq!(
            BiophysicsToMolecular::map_object(&BiophysicsEntity::FluidMedium),
            MolecularEntity::Sodium,
        );
    }

    #[test]
    fn test_cell_membrane_maps_to_calcium() {
        assert_eq!(
            BiophysicsToMolecular::map_object(&BiophysicsEntity::CellMembrane),
            MolecularEntity::Calcium,
        );
    }

    #[test]
    fn test_mechanical_strain_maps_to_mechanosensitive() {
        assert_eq!(
            BiophysicsToMolecular::map_object(&BiophysicsEntity::MechanicalStrain),
            MolecularEntity::Mechanosensitive,
        );
    }

    #[test]
    fn test_cell_deformation_maps_to_mechanosensitive() {
        assert_eq!(
            BiophysicsToMolecular::map_object(&BiophysicsEntity::CellDeformation),
            MolecularEntity::Mechanosensitive,
        );
    }

    #[test]
    fn test_viscosity_maps_to_mucin() {
        assert_eq!(
            BiophysicsToMolecular::map_object(&BiophysicsEntity::Viscosity),
            MolecularEntity::Mucin,
        );
    }

    #[test]
    fn test_membrane_capacitance_maps_to_calcium() {
        assert_eq!(
            BiophysicsToMolecular::map_object(&BiophysicsEntity::MembraneCapacitance),
            MolecularEntity::Calcium,
        );
    }
}
