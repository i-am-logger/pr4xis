//! Functor: TransductionCategory → VestibularCategory.
//!
//! Maps hair cell transduction to vestibular function — same molecular
//! machinery, different sensory modality.
//!
//! Citation: Hudspeth & Corey (1977) *PNAS* 74(6):2407 — shared
//! hair-cell transduction across auditory and vestibular hair cells.

use crate::natural::hearing::transduction::ontology::*;
use crate::natural::hearing::vestibular::ontology::*;
use pr4xis::category::{Arrow, Functor};

pub struct TransductionToVestibular;

impl Functor for TransductionToVestibular {
    type Source = TransductionCategory;
    type Target = VestibularCategory;

    fn map_object(obj: &TransductionEntity) -> VestibularEntity {
        use TransductionEntity as T;
        use VestibularEntity::*;
        match obj {
            T::Stereocilium
            | T::StereociliaBundle
            | T::CuticularPlate
            | T::Kinocilium
            | T::StereociliaDeflection => CrisaAmpullaris,
            T::TipLink
            | T::Cadherin23
            | T::Protocadherin15
            | T::TipLinkTension
            | T::TipLinkProtein => TypeIHairCell,
            T::METChannel
            | T::TMC1
            | T::TMC2
            | T::TMIE
            | T::LHFPL5
            | T::METChannelOpening
            | T::METComponent => TypeIHairCell,
            T::PotassiumInflux
            | T::Potassium
            | T::CalciumInflux
            | T::Calcium
            | T::Depolarization
            | T::ReceptorPotential
            | T::EndocochlearPotential => TypeIHairCell,
            T::KCNQ4 | T::CaV1_3 | T::BKChannel | T::IonChannel => TypeIIHairCell,
            T::GlutamateRelease | T::Glutamate => ScarpaGanglion,
            T::ActionPotential => VestibularNerve,
            T::Prestin | T::Electromotility | T::CochlearAmplification => Cupula,
            T::CellularSignal => VestibularNuclei,
            // Transduction events → vestibular events.
            T::BasilarMembraneMotion => HeadRotation,
            T::StereociliaBundleDeflection | T::TipLinkStretch => CupulaDeflection,
            T::METChannelGating | T::PotassiumEntry => CanalHairCellActivation,
            T::CellDepolarization => CanalHairCellActivation,
            T::CalciumEntry => MaculaHairCellActivation,
            T::VesicleRelease => VestibularAfferentFiring,
            T::NerveActivation => VestibularNucleiProcessing,
            T::PrestiConformationChange | T::CellLengthChange | T::BasilarMembraneAmplification => {
                CupulaDeflection
            }
            T::SlowAdaptation | T::FastAdaptation => VestibularAfferentFiring,
            T::TransductionEvent => VestibularEvent,
        }
    }

    fn map_morphism(m: &TransductionRelation) -> VestibularRelation {
        use TransductionCategoryRelationKind as Sk;
        use VestibularRelationKind as Tk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        VestibularRelation { from, to, kind }
    }
}
pr4xis::register_functor!(TransductionToVestibular);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[test]
    fn functor_laws() {
        assert_functor_laws::<TransductionToVestibular>();
    }
    #[test]
    fn action_potential_maps_to_nerve() {
        assert_eq!(
            TransductionToVestibular::map_object(&TransductionEntity::ActionPotential),
            VestibularEntity::VestibularNerve
        );
    }
    #[test]
    fn every_entity_maps_valid() {
        let targets = VestibularEntity::variants();
        for obj in TransductionEntity::variants() {
            assert!(targets.contains(&TransductionToVestibular::map_object(&obj)));
        }
    }
}
