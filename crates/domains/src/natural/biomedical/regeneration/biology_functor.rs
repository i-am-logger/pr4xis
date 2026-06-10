//! Functor: RegenerationCategory -> BiologyCategory.
//!
//! Proves that the regeneration domain has a structure-preserving map into
//! biological organization. Regeneration types map to Tissue (regeneration
//! happens at the tissue level), WoundEpithelium -> SquamousEpithelium,
//! NerveSupply -> NeuralTissue, TargetMorphology -> Organism (target is
//! whole-organism form), body axes -> Organism.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping is
//! mathematically valid -- verified by `check_functor_laws`.

use pr4xis::category::{Arrow, Category, Functor};

use crate::natural::biomedical::biology::ontology::{
    BiologicalEntity, BiologicalRelation, BiologyCategory, BiologyRelationKind,
};
use crate::natural::biomedical::regeneration::ontology::{
    RegenerationCategory, RegenerationEntity, RegenerationRelation, RegenerationRelationKind,
};

/// Structure-preserving map from regeneration entities to biological organization.
pub struct RegenerationToBiology;

impl Functor for RegenerationToBiology {
    type Source = RegenerationCategory;
    type Target = BiologyCategory;

    fn map_object(obj: &RegenerationEntity) -> BiologicalEntity {
        use BiologicalEntity as B;
        use RegenerationEntity as R;
        match obj {
            // Regeneration types happen at the tissue level
            R::Epimorphic
            | R::Morphallactic
            | R::Compensatory
            | R::StemCellMediated
            | R::EpithelialRestitution => B::Tissue,

            // Blastema is a tissue-level structure
            R::Blastema => B::Tissue,

            // Wound epithelium maps to squamous epithelium
            R::WoundEpithelium => B::SquamousEpithelium,

            // Nerve supply maps to neural tissue
            R::NerveSupply => B::NeuralTissue,

            // Target morphology = whole-organism form
            R::TargetMorphology => B::Organism,

            // Pattern concepts -> Tissue (tissue-level patterning)
            R::AnatomicalPolarity | R::PatternMemory | R::Bistability => B::Tissue,

            // Body axes describe organism-level organization
            R::AnteriorPosteriorAxis | R::DorsalVentralAxis | R::LeftRightAxis => B::Organism,

            // Abstract categories
            R::RegenerationType => B::Tissue,
            R::BodyAxis => B::Organism,
            R::PatternConcept => B::Tissue,
            R::Structure => B::Tissue,
            R::RegenerationEvent => B::Tissue,

            // Causal events (merged into the concept enum): every step in
            // the regeneration cascade occurs at the tissue level.
            R::Injury => B::Tissue,
            R::WoundClosure => B::Tissue,
            R::BlastemaFormation => B::Tissue,
            R::PatternSpecification => B::Tissue,
            R::Differentiation => B::Tissue,
            R::MorphologicalRestoration => B::Organism,
            R::BioelectricSignal => B::Tissue,
            R::PolarityDetermination => B::Tissue,
            R::GapJunctionCommunication => B::Tissue,
            R::CollectiveDecision => B::Tissue,
            R::NerveSignaling => B::NeuralTissue,
        }
    }

    fn map_morphism(m: &RegenerationRelation) -> BiologicalRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        // Identity preserved; non-Identity kinds collapse to Subsumption in
        // the (migrated) biology target so functor laws hold under same-kind
        // transitive composition (#166).
        match m.kind {
            RegenerationRelationKind::Identity => BiologyCategory::identity(&from),
            _ => BiologicalRelation {
                from,
                to,
                kind: BiologyRelationKind::Subsumption,
            },
        }
    }
}
pr4xis::register_functor!(RegenerationToBiology);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;
    use pr4xis::category::{Category, FinitelyGenerated};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        assert_functor_laws::<RegenerationToBiology>();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<RegenerationToBiology>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in RegenerationEntity::variants() {
            let id_src = RegenerationCategory::identity(&obj);
            let mapped_id = RegenerationToBiology::map_morphism(&id_src);
            let id_tgt = BiologyCategory::identity(&RegenerationToBiology::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    /// Composition preservation over a Subsumption chain that actually
    /// composes: AnteriorPosteriorAxis -> BodyAxis -> PatternConcept
    /// compose under Subsumption-transitivity.
    #[test]
    fn test_composition_preservation_subsumption_chain() {
        let f = RegenerationRelation {
            from: RegenerationEntity::AnteriorPosteriorAxis,
            to: RegenerationEntity::BodyAxis,
            kind: RegenerationRelationKind::Subsumption,
        };
        let g = RegenerationRelation {
            from: RegenerationEntity::BodyAxis,
            to: RegenerationEntity::PatternConcept,
            kind: RegenerationRelationKind::Subsumption,
        };
        let composed = RegenerationCategory::compose(&f, &g)
            .expect("Subsumption chain AP -> BodyAxis -> PatternConcept must compose");
        let mapped_composed = RegenerationToBiology::map_morphism(&composed);
        let composed_mapped = BiologyCategory::compose(
            &RegenerationToBiology::map_morphism(&f),
            &RegenerationToBiology::map_morphism(&g),
        )
        .expect("functor-mapped morphisms must compose in the target category");
        assert_eq!(mapped_composed, composed_mapped);
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = BiologicalEntity::variants();
        for obj in RegenerationEntity::variants() {
            let mapped = RegenerationToBiology::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BiologicalEntity",
                obj,
                mapped
            );
        }
    }

    #[test]
    fn test_epimorphic_maps_to_tissue() {
        assert_eq!(
            RegenerationToBiology::map_object(&RegenerationEntity::Epimorphic),
            BiologicalEntity::Tissue,
        );
    }

    #[test]
    fn test_morphallactic_maps_to_tissue() {
        assert_eq!(
            RegenerationToBiology::map_object(&RegenerationEntity::Morphallactic),
            BiologicalEntity::Tissue,
        );
    }

    #[test]
    fn test_blastema_maps_to_tissue() {
        assert_eq!(
            RegenerationToBiology::map_object(&RegenerationEntity::Blastema),
            BiologicalEntity::Tissue,
        );
    }

    #[test]
    fn test_wound_epithelium_maps_to_squamous_epithelium() {
        assert_eq!(
            RegenerationToBiology::map_object(&RegenerationEntity::WoundEpithelium),
            BiologicalEntity::SquamousEpithelium,
        );
    }

    #[test]
    fn test_nerve_supply_maps_to_neural_tissue() {
        assert_eq!(
            RegenerationToBiology::map_object(&RegenerationEntity::NerveSupply),
            BiologicalEntity::NeuralTissue,
        );
    }

    #[test]
    fn test_target_morphology_maps_to_organism() {
        assert_eq!(
            RegenerationToBiology::map_object(&RegenerationEntity::TargetMorphology),
            BiologicalEntity::Organism,
        );
    }

    #[test]
    fn test_anterior_posterior_axis_maps_to_organism() {
        assert_eq!(
            RegenerationToBiology::map_object(&RegenerationEntity::AnteriorPosteriorAxis),
            BiologicalEntity::Organism,
        );
    }

    #[test]
    fn test_analogy_translates_epimorphic() {
        assert_eq!(
            Analogy::<RegenerationToBiology>::translate(&RegenerationEntity::Epimorphic),
            BiologicalEntity::Tissue,
        );
    }
}
