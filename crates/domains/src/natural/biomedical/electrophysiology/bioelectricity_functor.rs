//! Functor: ElectrophysiologyCategory -> BioelectricCategory.
//!
//! Proves that electrophysiology measurement techniques have a structure-preserving
//! map into the bioelectric framework. Each measurement technique maps to the
//! bioelectric concept it measures: PatchClamp -> MembranePotential (directly
//! measures Vmem), VoltageSensitiveDye -> VoltageGradient (spatial Vmem mapping),
//! OpticalMapping -> BioelectricPrepattern (spatial pattern), etc.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Category, Functor, Relationship};

use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation, BioelectricRelationKind,
};
use crate::natural::biomedical::electrophysiology::ontology::{
    ElectrophysiologyCategory, ElectrophysiologyCategoryRelationKind, ElectrophysiologyEntity,
    ElectrophysiologyRelation,
};

/// Structure-preserving map from electrophysiology entities to their bioelectric role.
pub struct ElectrophysiologyToBioelectric;

impl Functor for ElectrophysiologyToBioelectric {
    type Source = ElectrophysiologyCategory;
    type Target = BioelectricCategory;

    fn map_object(obj: &ElectrophysiologyEntity) -> BioelectricEntity {
        use BioelectricEntity::*;
        use ElectrophysiologyEntity as E;
        match obj {
            // PatchClamp and SharpElectrode directly measure Vmem
            E::PatchClamp | E::SharpElectrode => MembranePotential,
            // Voltage-sensitive dye maps spatial Vmem = VoltageGradient
            E::VoltageSensitiveDye => VoltageGradient,
            // Calcium imaging measures calcium as a bioelectric signal
            E::CalciumImaging => Signal,
            // Bioimpedance measures tissue-level potential
            E::Bioimpedance => TransepithelialPotential,
            // Extracellular recording and MEA measure voltage gradients
            E::ExtracellularRecording | E::MultiElectrodeArray => VoltageGradient,
            // Optical mapping reveals spatial bioelectric prepatterns
            E::OpticalMapping => BioelectricPrepattern,

            // Measured quantities -> Signal
            E::RestingPotential
            | E::ActionPotential
            | E::TransepithelialPotential
            | E::FieldPotential
            | E::Impedance
            | E::IntracellularCalcium => Signal,

            // Recording modes -> Signal
            E::WholeCell
            | E::CellAttached
            | E::InsideOut
            | E::OutsideOut
            | E::CurrentClamp
            | E::VoltageClamp => Signal,

            // Abstract categories -> Signal
            E::MeasurementTechnique | E::MeasuredQuantity | E::RecordingMode => Signal,
        }
    }

    fn map_morphism(m: &ElectrophysiologyRelation) -> BioelectricRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity morphisms must map to identity (functor law). Other kinds
        // collapse to Composed in the target — matching how the target's
        // compose produces Composed morphisms for non-Identity inputs (so
        // F(g∘f) == F(g)∘F(f) holds under collapse).
        match m.kind {
            ElectrophysiologyCategoryRelationKind::Identity => BioelectricCategory::identity(&from),
            _ => BioelectricRelation {
                from,
                to,
                kind: BioelectricRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(ElectrophysiologyToBioelectric);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<ElectrophysiologyToBioelectric>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<ElectrophysiologyToBioelectric>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in ElectrophysiologyEntity::variants() {
            let id_src = ElectrophysiologyCategory::identity(&obj);
            let mapped_id = ElectrophysiologyToBioelectric::map_morphism(&id_src);
            let id_tgt =
                BioelectricCategory::identity(&ElectrophysiologyToBioelectric::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        let objs = ElectrophysiologyEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = ElectrophysiologyRelation {
                        from: a,
                        to: b,
                        kind: ElectrophysiologyCategoryRelationKind::Composed,
                    };
                    let g = ElectrophysiologyRelation {
                        from: b,
                        to: c,
                        kind: ElectrophysiologyCategoryRelationKind::Composed,
                    };
                    let composed = ElectrophysiologyCategory::compose(&f, &g).unwrap();
                    let mapped_composed = ElectrophysiologyToBioelectric::map_morphism(&composed);
                    let composed_mapped = BioelectricCategory::compose(
                        &ElectrophysiologyToBioelectric::map_morphism(&f),
                        &ElectrophysiologyToBioelectric::map_morphism(&g),
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
        let target_variants = BioelectricEntity::variants();
        for obj in ElectrophysiologyEntity::variants() {
            let mapped = ElectrophysiologyToBioelectric::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BioelectricEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_patch_clamp_maps_to_membrane_potential() {
        assert_eq!(
            ElectrophysiologyToBioelectric::map_object(&ElectrophysiologyEntity::PatchClamp),
            BioelectricEntity::MembranePotential,
        );
    }

    #[test]
    fn test_sharp_electrode_maps_to_membrane_potential() {
        assert_eq!(
            ElectrophysiologyToBioelectric::map_object(&ElectrophysiologyEntity::SharpElectrode),
            BioelectricEntity::MembranePotential,
        );
    }

    #[test]
    fn test_voltage_sensitive_dye_maps_to_voltage_gradient() {
        assert_eq!(
            ElectrophysiologyToBioelectric::map_object(
                &ElectrophysiologyEntity::VoltageSensitiveDye
            ),
            BioelectricEntity::VoltageGradient,
        );
    }

    #[test]
    fn test_calcium_imaging_maps_to_signal() {
        assert_eq!(
            ElectrophysiologyToBioelectric::map_object(&ElectrophysiologyEntity::CalciumImaging),
            BioelectricEntity::Signal,
        );
    }

    #[test]
    fn test_bioimpedance_maps_to_transepithelial_potential() {
        assert_eq!(
            ElectrophysiologyToBioelectric::map_object(&ElectrophysiologyEntity::Bioimpedance),
            BioelectricEntity::TransepithelialPotential,
        );
    }

    #[test]
    fn test_extracellular_recording_maps_to_voltage_gradient() {
        assert_eq!(
            ElectrophysiologyToBioelectric::map_object(
                &ElectrophysiologyEntity::ExtracellularRecording
            ),
            BioelectricEntity::VoltageGradient,
        );
    }

    #[test]
    fn test_multi_electrode_array_maps_to_voltage_gradient() {
        assert_eq!(
            ElectrophysiologyToBioelectric::map_object(
                &ElectrophysiologyEntity::MultiElectrodeArray
            ),
            BioelectricEntity::VoltageGradient,
        );
    }

    #[test]
    fn test_optical_mapping_maps_to_bioelectric_prepattern() {
        assert_eq!(
            ElectrophysiologyToBioelectric::map_object(&ElectrophysiologyEntity::OpticalMapping),
            BioelectricEntity::BioelectricPrepattern,
        );
    }

    #[test]
    fn test_analogy_translates_patch_clamp() {
        assert_eq!(
            Analogy::<ElectrophysiologyToBioelectric>::translate(
                &ElectrophysiologyEntity::PatchClamp
            ),
            BioelectricEntity::MembranePotential,
        );
    }
}
