//! Functor: MolecularCategory -> PharmacologyCategory (right adjoint).
//!
//! Maps molecular entities back to the drugs that target them.
//! This is the RIGHT adjoint to PharmacologyToMolecular: it "forgets" the
//! molecular detail and returns the canonical pharmacological intervention.
//!
//! - Nav → Decamethonium (depolarizing blocker targeting Na+ channels)
//! - Kv → Minoxidil (KATP opener)
//! - GlyR → Ivermectin (Levin's primary GlyR agonist)
//! - Proton → Omeprazole (proton pump inhibitor)
//! - Cx43 → GapJunctionModulator (gap junction drugs)
//!
//! Functor laws verified by `check_functor_laws`.

use pr4xis::category::{Arrow, Category, Functor};

use crate::natural::biomedical::molecular::ontology::{
    MolecularCategory, MolecularConcept, MolecularRelation, MolecularRelationKind,
};
use crate::natural::biomedical::pharmacology::ontology::{
    PharmacologyCategory, PharmacologyConcept, PharmacologyRelation, PharmacologyRelationKind,
};

/// Structure-preserving map from molecular entities to their pharmacological targeting.
pub struct MolecularToPharmacology;

impl Functor for MolecularToPharmacology {
    type Source = MolecularCategory;
    type Target = PharmacologyCategory;

    fn map_object(obj: &MolecularConcept) -> PharmacologyConcept {
        use MolecularConcept::*;
        use PharmacologyConcept as P;
        match obj {
            // Ions -> drugs that target ion channels for that ion
            Sodium => P::Decamethonium, // depolarizing Na+ channel agent
            Potassium => P::Minoxidil,  // KATP opener
            Calcium => P::IonChannelModulator, // Ca2+ channel modulators
            Chloride => P::IonChannelModulator, // Cl- channel modulators
            Proton => P::Omeprazole,    // proton pump inhibitor

            // Voltage-gated channels
            Nav => P::Decamethonium,       // Na+ channel blocker
            Kv => P::Minoxidil,            // KATP opener
            Cav => P::IonChannelModulator, // Ca2+ channel modulator

            // Mechanosensitive channels
            Piezo1 => P::MechanosensitiveModulator,
            Piezo2 => P::MechanosensitiveModulator,
            TRPV4 => P::MechanosensitiveModulator,

            // Ligand-gated channels
            GlyR => P::Ivermectin,            // GlyR agonist
            GABA_A => P::IonChannelModulator, // GABA_A modulators

            // Gap junctions
            Cx26 => P::GapJunctionModulator, // GJ opener drugs
            Cx43 => P::GapJunctionModulator, // GJ opener drugs

            // Structural proteins
            Collagen => P::Morphoceutical, // structural target
            Mucin => P::Morphoceutical,    // structural target

            // Signaling molecules
            CalciumSignal => P::Morphoceutical, // Ca2+ signaling → anatomical outcomes
            NitricOxide => P::Morphoceutical,   // NO → anatomical outcomes

            // Abstract categories
            Ion => P::Agent,                       // ions targeted by agents
            IonChannel => P::IonChannelModulator,  // ion channels → modulators
            VoltageGated => P::VoltageGatedOpener, // voltage-gated → openers
            Mechanosensitive => P::MechanosensitiveModulator,
            LigandGated => P::IonChannelModulator, // ligand-gated → modulators
            GapJunction => P::GapJunctionModulator, // gap junctions → modulators
            Protein => P::Morphoceutical,          // proteins → morphoceuticals
            SignalingMolecule => P::Morphoceutical, // signaling → morphoceuticals

            // Mechanotransduction events (umbrella + Piezo1Opening etc.) —
            // the pharmacological view of these events is via the agents
            // that modulate them; default to IonChannelModulator.
            _ => P::IonChannelModulator,
        }
    }

    fn map_morphism(m: &MolecularRelation) -> PharmacologyRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity preserved; all other kinds collapse onto Subsumption in the
        // pharmacology target (the dense, always-present typed kind in this
        // ontology). Identity-vs-non-Identity is what the functor laws check.
        match m.kind {
            MolecularRelationKind::Identity => PharmacologyCategory::identity(&from),
            _ => PharmacologyRelation {
                from,
                to,
                kind: PharmacologyRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(MolecularToPharmacology);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::natural::biomedical::molecular::ontology::MolecularEntity;
    use crate::natural::biomedical::pharmacology::ontology::PharmacologyEntity;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_functor_laws() {
        assert_functor_laws::<MolecularToPharmacology>();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_analogy_validates() {
        Analogy::<MolecularToPharmacology>::validate().unwrap();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_identity_preservation() {
        for obj in MolecularEntity::variants() {
            let id_src = MolecularCategory::identity(&obj);
            let mapped_id = MolecularToPharmacology::map_morphism(&id_src);
            let id_tgt = PharmacologyCategory::identity(&MolecularToPharmacology::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = PharmacologyEntity::variants();
        for obj in MolecularEntity::variants() {
            let mapped = MolecularToPharmacology::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid PharmacologyEntity",
                obj,
                mapped
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_glyr_maps_to_ivermectin() {
        assert_eq!(
            MolecularToPharmacology::map_object(&MolecularEntity::GlyR),
            PharmacologyEntity::Ivermectin,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_proton_maps_to_omeprazole() {
        assert_eq!(
            MolecularToPharmacology::map_object(&MolecularEntity::Proton),
            PharmacologyEntity::Omeprazole,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_kv_maps_to_minoxidil() {
        assert_eq!(
            MolecularToPharmacology::map_object(&MolecularEntity::Kv),
            PharmacologyEntity::Minoxidil,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_cx43_maps_to_gap_junction_modulator() {
        assert_eq!(
            MolecularToPharmacology::map_object(&MolecularEntity::Cx43),
            PharmacologyEntity::GapJunctionModulator,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_piezo1_maps_to_mechanosensitive_modulator() {
        assert_eq!(
            MolecularToPharmacology::map_object(&MolecularEntity::Piezo1),
            PharmacologyEntity::MechanosensitiveModulator,
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn test_collagen_maps_to_morphoceutical() {
        assert_eq!(
            MolecularToPharmacology::map_object(&MolecularEntity::Collagen),
            PharmacologyEntity::Morphoceutical,
        );
    }
}
