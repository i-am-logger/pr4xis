//! Functor: AnatomyCategory → TransductionCategory.
//!
//! Maps anatomical structures to their functional roles in
//! mechanotransduction.
//!
//! Citation: Pickles (2012) *Physiology of Hearing*; Hudspeth (2014)
//! *Nat. Rev. Neurosci.* 15(9):600 — anatomical-to-functional
//! correspondences.

use pr4xis::category::{Arrow, Functor};

use crate::natural::hearing::anatomy::ontology::{
    AnatomyCategory, AnatomyConcept, AnatomyRelation, AnatomyRelationKind,
};
use crate::natural::hearing::transduction::ontology::{
    TransductionCategory, TransductionEntity, TransductionRelation, TransductionRelationKind,
};

pub struct AnatomyToTransduction;

impl Functor for AnatomyToTransduction {
    type Source = AnatomyCategory;
    type Target = TransductionCategory;

    fn map_object(obj: &AnatomyConcept) -> TransductionEntity {
        use AnatomyConcept as A;
        use TransductionEntity::*;
        match obj {
            A::InnerHairCell => GlutamateRelease,
            A::OuterHairCell => Electromotility,
            A::BasilarMembrane => StereociliaDeflection,
            A::TectorialMembrane => TipLinkTension,
            A::OrganOfCorti => METChannel,
            A::Endolymph => EndocochlearPotential,
            A::Perilymph => Potassium,
            A::ScalaMedia => EndocochlearPotential,
            A::StriVascularis => EndocochlearPotential,
            A::SpiralGanglionNeuron => ActionPotential,
            A::AuditoryNerve => ActionPotential,
            A::CochlearNucleus => ActionPotential,
            A::SuperiorOlivaryComplex => ActionPotential,
            A::InferiorColliculus => ActionPotential,
            A::MedialGeniculateBody => ActionPotential,
            A::AuditoryCortex => ActionPotential,
            A::Cochlea => StereociliaBundle,
            A::Stapes | A::OvalWindow => StereociliaDeflection,
            A::Malleus | A::Incus => StereociliaDeflection,
            A::Pinna
            | A::EarCanal
            | A::TympanicMembrane
            | A::RoundWindow
            | A::EustachianTube
            | A::TensorTympani
            | A::Stapedius
            | A::ScalaVestibuli
            | A::ScalaTympani
            | A::ReissnersMembrane
            | A::Vestibule
            | A::SemicircularCanals
            | A::SupportingCell => StereociliaBundle,
            A::Ear
            | A::OuterEar
            | A::MiddleEar
            | A::InnerEar
            | A::Ossicle
            | A::HairCell
            | A::CochlearFluid
            | A::CochlearMembrane
            | A::AuditoryNucleus => StereociliaBundle,
        }
    }

    fn map_morphism(m: &AnatomyRelation) -> TransductionRelation {
        use AnatomyRelationKind as Sk;
        use TransductionRelationKind as Tk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Opposition => Tk::Opposition,
            // anatomy has no causes; canonical kind always present.
            Sk::Causation => Tk::Causation,
        };
        TransductionRelation { from, to, kind }
    }
}
pr4xis::register_functor!(AnatomyToTransduction);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[test]
    fn functor_laws() {
        assert_functor_laws::<AnatomyToTransduction>();
    }
    #[test]
    fn ihc_maps_to_glutamate_release() {
        assert_eq!(
            AnatomyToTransduction::map_object(&AnatomyConcept::InnerHairCell),
            TransductionEntity::GlutamateRelease
        );
    }
    #[test]
    fn ohc_maps_to_electromotility() {
        assert_eq!(
            AnatomyToTransduction::map_object(&AnatomyConcept::OuterHairCell),
            TransductionEntity::Electromotility
        );
    }
    #[test]
    fn every_entity_maps_valid() {
        let targets = TransductionEntity::variants();
        for obj in AnatomyConcept::variants() {
            assert!(targets.contains(&AnatomyToTransduction::map_object(&obj)));
        }
    }
}
