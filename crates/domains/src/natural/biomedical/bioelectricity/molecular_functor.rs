//! Functor: BioelectricCategory -> MolecularCategory (right adjoint).
//!
//! Maps bioelectric entities back to their underlying molecular mechanisms.
//! This is the RIGHT adjoint to MolecularToBioelectric: it "forgets" the
//! bioelectric interpretation and returns the canonical molecule responsible.
//!
//! - MembranePotential → Kv (resting potential set by K+ leak channels)
//! - VoltageGradient → Cx43 (gradients propagate through gap junctions)
//! - MechanicalStimulation → Piezo1 (the mechanosensitive channel)
//! - GlyR is Levin's primary pharmacological tool for ion channel modulation
//!
//! Functor laws verified by `check_functor_laws`.

use pr4xis::category::{Functor, Relationship};

use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation,
};
use crate::natural::biomedical::molecular::ontology::{
    MolecularCategory, MolecularEntity, MolecularRelation,
};

/// Structure-preserving map from bioelectric entities to their molecular mechanism.
pub struct BioelectricToMolecular;

impl Functor for BioelectricToMolecular {
    type Source = BioelectricCategory;
    type Target = MolecularCategory;

    fn map_object(obj: &BioelectricEntity) -> MolecularEntity {
        use BioelectricEntity::*;
        use MolecularEntity as M;
        match obj {
            // Signals
            MembranePotential => M::Kv, // resting potential set by K+ channels
            VoltageGradient => M::Cx43, // gradients propagate through gap junctions
            BioelectricPrepattern => M::Cx43, // patterns require GJ network
            TransepithelialPotential => M::Nav, // Na+ drives TEP
            // Networks
            GapJunctionNetwork => M::Cx43, // Cx43 is the major connexin
            BioelectricCircuit => M::Kv,   // circuit driven by K+ channels
            CognitiveLightcone => M::Cx43, // GJ network defines lightcone
            // Morphospace
            TargetMorphology => M::CalciumSignal, // Ca2+ signals encode target
            CurrentMorphology => M::Calcium,      // current state reflected in Ca2+
            MorphogeneticField => M::CalciumSignal, // morphogenetic info via Ca2+ waves
            // Interventions
            IonChannelModulation => M::GlyR, // Levin's primary tool (ivermectin/GlyR)
            GapJunctionModulation => M::Cx43, // gap junction modulation via Cx43
            BioelectricCocktail => M::GlyR,  // cocktail centered on GlyR agonists
            MechanicalStimulation => M::Piezo1, // the mechanosensitive channel!
            ProtonPumpInhibition => M::Proton, // PPI targets proton pumps
            // Abstract
            Signal => M::CalciumSignal, // signals mediated by Ca2+
            Network => M::Cx43,         // networks = gap junctions
            Morphospace => M::Calcium,  // morphospace tracked by Ca2+
            Intervention => M::GlyR,    // interventions via GlyR
        }
    }

    fn map_morphism(m: &BioelectricRelation) -> MolecularRelation {
        MolecularRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<BioelectricToMolecular>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<BioelectricToMolecular>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in BioelectricEntity::variants() {
            let id_src = BioelectricCategory::identity(&obj);
            let mapped_id = BioelectricToMolecular::map_morphism(&id_src);
            let id_tgt = MolecularCategory::identity(&BioelectricToMolecular::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = MolecularEntity::variants();
        for obj in BioelectricEntity::variants() {
            let mapped = BioelectricToMolecular::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid MolecularEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_membrane_potential_maps_to_kv() {
        assert_eq!(
            BioelectricToMolecular::map_object(&BioelectricEntity::MembranePotential),
            MolecularEntity::Kv,
        );
    }

    #[test]
    fn test_mechanical_stimulation_maps_to_piezo1() {
        assert_eq!(
            BioelectricToMolecular::map_object(&BioelectricEntity::MechanicalStimulation),
            MolecularEntity::Piezo1,
        );
    }

    #[test]
    fn test_gap_junction_network_maps_to_cx43() {
        assert_eq!(
            BioelectricToMolecular::map_object(&BioelectricEntity::GapJunctionNetwork),
            MolecularEntity::Cx43,
        );
    }

    #[test]
    fn test_ion_channel_modulation_maps_to_glyr() {
        assert_eq!(
            BioelectricToMolecular::map_object(&BioelectricEntity::IonChannelModulation),
            MolecularEntity::GlyR,
        );
    }

    #[test]
    fn test_proton_pump_inhibition_maps_to_proton() {
        assert_eq!(
            BioelectricToMolecular::map_object(&BioelectricEntity::ProtonPumpInhibition),
            MolecularEntity::Proton,
        );
    }
}
