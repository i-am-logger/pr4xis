//! Functor: RegenerationCategory -> BioelectricCategory.
//!
//! The REVERSE of the bioelectricity->regeneration functor. Maps regeneration
//! concepts back to the bioelectric framework: target morphology maps directly,
//! anatomical polarity maps to membrane potential (polarity IS Vmem pattern),
//! body axes map to voltage gradients, pattern memory maps to gap junction
//! networks (pattern stored in GJ network), etc.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Arrow, Category, Functor};

use crate::natural::biomedical::bioelectricity::ontology::{
    BioelectricCategory, BioelectricEntity, BioelectricRelation, BioelectricRelationKind,
};
use crate::natural::biomedical::regeneration::ontology::{
    RegenerationCategory, RegenerationEntity, RegenerationRelation, RegenerationRelationKind,
};

/// Structure-preserving map from regeneration concepts to bioelectric entities.
pub struct RegenerationToBioelectric;

impl Functor for RegenerationToBioelectric {
    type Source = RegenerationCategory;
    type Target = BioelectricCategory;

    fn map_object(obj: &RegenerationEntity) -> BioelectricEntity {
        use BioelectricEntity as BE;
        use RegenerationEntity as R;
        match obj {
            // Pattern concepts -> bioelectric signals/networks
            R::TargetMorphology => BE::TargetMorphology, // direct mapping
            R::AnatomicalPolarity => BE::MembranePotential, // polarity IS Vmem pattern
            R::AnteriorPosteriorAxis => BE::VoltageGradient, // axes are voltage gradients
            R::DorsalVentralAxis => BE::VoltageGradient, // axes are voltage gradients
            R::LeftRightAxis => BE::VoltageGradient,     // axes are voltage gradients
            R::PatternMemory => BE::GapJunctionNetwork,  // pattern stored in GJ network
            R::Bistability => BE::MorphogeneticField,    // multiple stable states = field

            // Regeneration types -> current morphology (all are morphospace states)
            R::Epimorphic => BE::CurrentMorphology,
            R::Morphallactic => BE::CurrentMorphology,
            R::Compensatory => BE::CurrentMorphology,
            R::StemCellMediated => BE::CurrentMorphology,
            R::EpithelialRestitution => BE::CurrentMorphology,

            // Structures
            R::Blastema => BE::CurrentMorphology, // proliferative structure = current state
            R::WoundEpithelium => BE::CurrentMorphology, // wound state = current morphology
            R::NerveSupply => BE::BioelectricCircuit, // nerves are bioelectric circuits

            // Abstract categories
            R::RegenerationType => BE::Morphospace,
            R::BodyAxis => BE::Signal,
            R::PatternConcept => BE::Network,
            R::Structure => BE::Morphospace,
            R::RegenerationEvent => BE::Signal,

            // Causal events (merged into the concept enum): each
            // regeneration step maps to its bioelectric correlate.
            R::Injury => BE::Signal,
            R::WoundClosure => BE::Signal,
            R::BlastemaFormation => BE::CurrentMorphology,
            R::PatternSpecification => BE::MorphogeneticField,
            R::Differentiation => BE::CurrentMorphology,
            R::MorphologicalRestoration => BE::TargetMorphology,
            R::BioelectricSignal => BE::Signal,
            R::PolarityDetermination => BE::VoltageGradient,
            R::GapJunctionCommunication => BE::GapJunctionNetwork,
            R::CollectiveDecision => BE::MorphogeneticField,
            R::NerveSignaling => BE::BioelectricCircuit,
        }
    }

    fn map_morphism(m: &RegenerationRelation) -> BioelectricRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity preserved; non-Identity kinds collapse to Subsumption in
        // the (migrated) bioelectricity target so functor laws hold under
        // same-kind transitive composition (#166).
        match m.kind {
            RegenerationRelationKind::Identity => BioelectricCategory::identity(&from),
            _ => BioelectricRelation {
                from,
                to,
                kind: BioelectricRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(RegenerationToBioelectric);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        assert_functor_laws::<RegenerationToBioelectric>();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<RegenerationToBioelectric>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in RegenerationEntity::variants() {
            let id_src = RegenerationCategory::identity(&obj);
            let mapped_id = RegenerationToBioelectric::map_morphism(&id_src);
            let id_tgt =
                BioelectricCategory::identity(&RegenerationToBioelectric::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    /// Composition preservation over a Causation chain that composes in
    /// the source: Injury -> WoundClosure -> BlastemaFormation compose
    /// under Causation-transitivity. After mapping, both become Subsumption
    /// arrows in the target; same-kind transitive composition keeps
    /// F(g∘f) == F(g)∘F(f).
    #[test]
    fn test_composition_preservation_causation_chain() {
        let f = RegenerationRelation {
            from: RegenerationEntity::Injury,
            to: RegenerationEntity::WoundClosure,
            kind: RegenerationRelationKind::Causation,
        };
        let g = RegenerationRelation {
            from: RegenerationEntity::WoundClosure,
            to: RegenerationEntity::BlastemaFormation,
            kind: RegenerationRelationKind::Causation,
        };
        let composed = RegenerationCategory::compose(&f, &g)
            .expect("Causation chain must compose under transitive same-kind inheritance");
        let mapped_composed = RegenerationToBioelectric::map_morphism(&composed);
        let f_mapped = RegenerationToBioelectric::map_morphism(&f);
        let g_mapped = RegenerationToBioelectric::map_morphism(&g);
        if f_mapped.target() == g_mapped.source() {
            let composed_mapped = BioelectricCategory::compose(&f_mapped, &g_mapped)
                .expect("Subsumption-on-Subsumption composes in the target");
            assert_eq!(mapped_composed, composed_mapped);
        } else {
            assert_eq!(
                mapped_composed.source(),
                RegenerationToBioelectric::map_object(&RegenerationEntity::Injury)
            );
        }
    }

    #[test]
    fn test_target_morphology_maps_directly() {
        assert_eq!(
            RegenerationToBioelectric::map_object(&RegenerationEntity::TargetMorphology),
            BioelectricEntity::TargetMorphology,
        );
    }

    #[test]
    fn test_anatomical_polarity_maps_to_membrane_potential() {
        assert_eq!(
            RegenerationToBioelectric::map_object(&RegenerationEntity::AnatomicalPolarity),
            BioelectricEntity::MembranePotential,
        );
    }

    #[test]
    fn test_ap_axis_maps_to_voltage_gradient() {
        assert_eq!(
            RegenerationToBioelectric::map_object(&RegenerationEntity::AnteriorPosteriorAxis),
            BioelectricEntity::VoltageGradient,
        );
    }

    #[test]
    fn test_pattern_memory_maps_to_gap_junction_network() {
        assert_eq!(
            RegenerationToBioelectric::map_object(&RegenerationEntity::PatternMemory),
            BioelectricEntity::GapJunctionNetwork,
        );
    }

    #[test]
    fn test_bistability_maps_to_morphogenetic_field() {
        assert_eq!(
            RegenerationToBioelectric::map_object(&RegenerationEntity::Bistability),
            BioelectricEntity::MorphogeneticField,
        );
    }

    #[test]
    fn test_nerve_supply_maps_to_bioelectric_circuit() {
        assert_eq!(
            RegenerationToBioelectric::map_object(&RegenerationEntity::NerveSupply),
            BioelectricEntity::BioelectricCircuit,
        );
    }

    #[test]
    fn test_epimorphic_maps_to_current_morphology() {
        assert_eq!(
            RegenerationToBioelectric::map_object(&RegenerationEntity::Epimorphic),
            BioelectricEntity::CurrentMorphology,
        );
    }

    #[test]
    fn test_regeneration_type_abstract_maps_to_morphospace() {
        assert_eq!(
            RegenerationToBioelectric::map_object(&RegenerationEntity::RegenerationType),
            BioelectricEntity::Morphospace,
        );
    }

    #[test]
    fn test_body_axis_abstract_maps_to_signal() {
        assert_eq!(
            RegenerationToBioelectric::map_object(&RegenerationEntity::BodyAxis),
            BioelectricEntity::Signal,
        );
    }

    #[test]
    fn test_pattern_concept_abstract_maps_to_network() {
        assert_eq!(
            RegenerationToBioelectric::map_object(&RegenerationEntity::PatternConcept),
            BioelectricEntity::Network,
        );
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = BioelectricEntity::variants();
        for obj in RegenerationEntity::variants() {
            let mapped = RegenerationToBioelectric::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BioelectricEntity",
                obj,
                mapped
            );
        }
    }
}
