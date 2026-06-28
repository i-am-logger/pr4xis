//! Functor: BiophysicsCategory -> BioelectricCategory.
//!
//! Proves that biophysics has a structure-preserving map into the bioelectric
//! framework. Mechanotransduction maps to MechanicalStimulation, piezoelectric
//! entities map to MechanicalStimulation, membrane biophysics maps to
//! MembranePotential, wave properties map to Signal, and biological media
//! map to morphospace entities.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Arrow, Category, Functor};

use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation, BioelectricRelationKind,
};
use crate::natural::biomedical::biophysics::ontology::{
    BiophysicsCategory, BiophysicsEntity, BiophysicsRelation, BiophysicsRelationKind,
};

/// Structure-preserving map from biophysics entities to bioelectric concepts.
pub struct BiophysicsToBioelectric;

impl Functor for BiophysicsToBioelectric {
    type Source = BiophysicsCategory;
    type Target = BioelectricCategory;

    fn map_object(obj: &BiophysicsEntity) -> BioelectricEntity {
        use BioelectricEntity as BE;
        use BiophysicsEntity as BP;
        match obj {
            // Mechanical properties -> MechanicalStimulation or Signal
            BP::Viscoelasticity => BE::MechanicalStimulation,
            BP::Elasticity => BE::MechanicalStimulation,
            BP::Viscosity => BE::MechanicalStimulation,
            BP::StiffnessModulus => BE::MechanicalStimulation,
            BP::StrainRate => BE::Signal,
            BP::MechanicalStress => BE::MechanicalStimulation,
            BP::MechanicalStrain => BE::MechanicalStimulation,

            // Wave physics -> Signal
            BP::MechanicalWave => BE::MechanicalStimulation,
            BP::AcousticImpedance => BE::Signal,
            BP::Attenuation => BE::Signal,
            BP::Frequency => BE::Signal,
            BP::Wavelength => BE::Signal,
            BP::ResonanceFrequency => BE::Signal,

            // Piezoelectricity -> MechanicalStimulation
            // (piezoelectric effects generate bioelectric signals from mechanics)
            BP::PiezoelectricEffect => BE::MechanicalStimulation,
            BP::DirectPiezoelectric => BE::MechanicalStimulation,
            BP::ConversePiezoelectric => BE::MechanicalStimulation,
            BP::CollagenPiezoelectricity => BE::MechanicalStimulation,

            // Membrane biophysics -> MembranePotential
            // (strain changes Vmem via mechanosensitive channels)
            BP::MembraneCapacitance => BE::MembranePotential,
            BP::MembraneTension => BE::MembranePotential,
            BP::CellDeformation => BE::MembranePotential,

            // Biological media -> morphospace / bioelectric entities
            BP::BoneMatrix => BE::MechanicalStimulation, // bone conducts vibration
            BP::SoftTissue => BE::CurrentMorphology,
            BP::CellMembrane => BE::MembranePotential,
            BP::FluidMedium => BE::TransepithelialPotential,

            // Abstract categories
            BP::MechanicalProperty => BE::Signal,
            BP::WaveProperty => BE::Signal,
            BP::PiezoelectricProperty => BE::MechanicalStimulation,
            BP::BiologicalMedium => BE::Morphospace,

            // Biophysical events (umbrella + propagation events) — these
            // are dynamic processes; in the bioelectric view they map to
            // Signal (a signal-flow event in the target ontology).
            _ => BE::Signal,
        }
    }

    fn map_morphism(m: &BiophysicsRelation) -> BioelectricRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity preserved; non-Identity kinds collapse to Subsumption in the
        // target so F(g∘f) = F(g)∘F(f) holds for same-kind transitive chains
        // under #166 (no `Composed` variant in proc-macro-generated enums).
        match m.kind {
            BiophysicsRelationKind::Identity => BioelectricCategory::identity(&from),
            _ => BioelectricRelation {
                from,
                to,
                kind: BioelectricRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(BiophysicsToBioelectric);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_functor_laws() {
        assert_functor_laws::<BiophysicsToBioelectric>();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_analogy_validates() {
        Analogy::<BiophysicsToBioelectric>::validate().unwrap();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_identity_preservation() {
        for obj in BiophysicsEntity::variants() {
            let id_src = BiophysicsCategory::identity(&obj);
            let mapped_id = BiophysicsToBioelectric::map_morphism(&id_src);
            let id_tgt = BioelectricCategory::identity(&BiophysicsToBioelectric::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_composition_preservation_on_subsumption() {
        // The migrated Biophysics category is kinded and partial (#166):
        // compose only succeeds for same-kind transitive relations. Exercise
        // composition along Subsumption chains and verify the functor
        // preserves the composite.
        for m in BiophysicsCategory::morphisms() {
            if m.kind() != BiophysicsRelationKind::Subsumption {
                continue;
            }
            for n in BiophysicsCategory::morphisms() {
                if n.kind() != BiophysicsRelationKind::Subsumption {
                    continue;
                }
                if m.target() != n.source() {
                    continue;
                }
                let composed = match BiophysicsCategory::compose(&m, &n) {
                    Some(c) => c,
                    None => continue,
                };
                let mapped_composed = BiophysicsToBioelectric::map_morphism(&composed);
                let composed_mapped = BioelectricCategory::compose(
                    &BiophysicsToBioelectric::map_morphism(&m),
                    &BiophysicsToBioelectric::map_morphism(&n),
                )
                .expect("target composition is total for same-kind");
                assert_eq!(
                    mapped_composed, composed_mapped,
                    "composition law failed for {:?} ∘ {:?}",
                    m, n
                );
            }
        }
    }

    // -- Specific mapping tests --

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_collagen_piezoelectricity_maps_to_mechanical_stimulation() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::CollagenPiezoelectricity),
            BioelectricEntity::MechanicalStimulation,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_cell_membrane_strain_maps_to_membrane_potential() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::MembraneTension),
            BioelectricEntity::MembranePotential,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_cell_deformation_maps_to_membrane_potential() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::CellDeformation),
            BioelectricEntity::MembranePotential,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_mechanical_wave_maps_to_mechanical_stimulation() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::MechanicalWave),
            BioelectricEntity::MechanicalStimulation,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_acoustic_impedance_maps_to_signal() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::AcousticImpedance),
            BioelectricEntity::Signal,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_bone_matrix_maps_to_mechanical_stimulation() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::BoneMatrix),
            BioelectricEntity::MechanicalStimulation,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_soft_tissue_maps_to_current_morphology() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::SoftTissue),
            BioelectricEntity::CurrentMorphology,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_cell_membrane_maps_to_membrane_potential() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::CellMembrane),
            BioelectricEntity::MembranePotential,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_fluid_medium_maps_to_transepithelial_potential() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::FluidMedium),
            BioelectricEntity::TransepithelialPotential,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_membrane_capacitance_maps_to_membrane_potential() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::MembraneCapacitance),
            BioelectricEntity::MembranePotential,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_piezoelectric_effect_maps_to_mechanical_stimulation() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::PiezoelectricEffect),
            BioelectricEntity::MechanicalStimulation,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_biological_medium_maps_to_morphospace() {
        assert_eq!(
            BiophysicsToBioelectric::map_object(&BiophysicsEntity::BiologicalMedium),
            BioelectricEntity::Morphospace,
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = BioelectricEntity::variants();
        for obj in BiophysicsEntity::variants() {
            let mapped = BiophysicsToBioelectric::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BioelectricEntity",
                obj,
                mapped
            );
        }
    }
}
