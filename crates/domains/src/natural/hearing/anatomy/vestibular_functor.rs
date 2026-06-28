//! Functor: AnatomyCategory → VestibularCategory.
//!
//! Maps auditory anatomy to vestibular structures (shared inner ear).
//!
//! Citation: Pickles (2012) *Physiology of Hearing* — shared inner-ear
//! anatomy; Goldberg et al. (2012) *The Vestibular System*.

use crate::natural::hearing::anatomy::ontology::*;
use crate::natural::hearing::vestibular::ontology::*;
use pr4xis::category::{Arrow, Functor};

pub struct AnatomyToVestibular;

impl Functor for AnatomyToVestibular {
    type Source = AnatomyCategory;
    type Target = VestibularCategory;

    fn map_object(obj: &AnatomyConcept) -> VestibularEntity {
        use AnatomyConcept as A;
        use VestibularEntity::*;
        match obj {
            A::Vestibule => Utricle,
            A::SemicircularCanals => LateralCanal,
            A::Endolymph => Cupula,
            A::Perilymph => OtolithMembrane,
            A::InnerHairCell => TypeIHairCell,
            A::OuterHairCell => TypeIIHairCell,
            A::SupportingCell => CrisaAmpullaris,
            A::SpiralGanglionNeuron => ScarpaGanglion,
            A::AuditoryNerve => VestibularNerve,
            A::CochlearNucleus
            | A::SuperiorOlivaryComplex
            | A::InferiorColliculus
            | A::MedialGeniculateBody => VestibularNuclei,
            A::AuditoryCortex => CerebellumVestibular,
            A::Cochlea | A::BasilarMembrane | A::OrganOfCorti | A::TectorialMembrane => Macula,
            A::ScalaVestibuli | A::ScalaMedia | A::ScalaTympani => Ampulla,
            A::StriVascularis | A::ReissnersMembrane => StriolarRegion,
            A::Pinna | A::EarCanal | A::TympanicMembrane => Cupula,
            A::Malleus
            | A::Incus
            | A::Stapes
            | A::OvalWindow
            | A::RoundWindow
            | A::EustachianTube
            | A::TensorTympani
            | A::Stapedius => Otoconia,
            A::Ear
            | A::OuterEar
            | A::MiddleEar
            | A::InnerEar
            | A::Ossicle
            | A::HairCell
            | A::CochlearFluid
            | A::CochlearMembrane
            | A::AuditoryNucleus => VestibularNuclei,
        }
    }

    fn map_morphism(m: &AnatomyRelation) -> VestibularRelation {
        use AnatomyRelationKind as Sk;
        use VestibularRelationKind as Tk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Opposition => Tk::Opposition,
            // anatomy has no Causation edges; with the canonical-kind
            // emission rule, the variant exists but no morphisms use it.
            Sk::Causation => Tk::Causation,
        };
        VestibularRelation { from, to, kind }
    }
}
pr4xis::register_functor!(AnatomyToVestibular);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws() {
        assert_functor_laws::<AnatomyToVestibular>();
    }
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn ihc_maps_to_type_i() {
        assert_eq!(
            AnatomyToVestibular::map_object(&AnatomyConcept::InnerHairCell),
            VestibularEntity::TypeIHairCell
        );
    }
    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn every_entity_maps_valid() {
        let targets = VestibularEntity::variants();
        for obj in AnatomyConcept::variants() {
            assert!(targets.contains(&AnatomyToVestibular::map_object(&obj)));
        }
    }
}
