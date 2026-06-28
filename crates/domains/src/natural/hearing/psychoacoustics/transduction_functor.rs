//! Functor: TransductionCategory → PsychoacousticsCategory.
//!
//! Maps molecular/cellular transduction entities to their perceptual role.
//!
//! Citation: Hudspeth (2014) *Nat. Rev. Neurosci.* 15(9):600 — the
//! transduction-to-percept correspondences; Moore (2012) *Psychology of
//! Hearing*.

use pr4xis::category::{Arrow, Functor};

use crate::natural::hearing::psychoacoustics::ontology::{
    PsychoacousticEntity, PsychoacousticRelation, PsychoacousticRelationKind,
    PsychoacousticsCategory,
};
use crate::natural::hearing::transduction::ontology::{
    TransductionCategory, TransductionCategoryRelationKind, TransductionEntity,
    TransductionRelation,
};

pub struct TransductionToPsychoacoustics;

impl Functor for TransductionToPsychoacoustics {
    type Source = TransductionCategory;
    type Target = PsychoacousticsCategory;

    fn map_object(obj: &TransductionEntity) -> PsychoacousticEntity {
        use PsychoacousticEntity::*;
        use TransductionEntity as T;
        match obj {
            T::Stereocilium
            | T::StereociliaBundle
            | T::CuticularPlate
            | T::Kinocilium
            | T::StereociliaDeflection => FrequencySelectivity,
            T::TipLink | T::Cadherin23 | T::Protocadherin15 | T::TipLinkTension => AuditoryFilter,
            T::METChannel
            | T::TMC1
            | T::TMC2
            | T::TMIE
            | T::LHFPL5
            | T::METChannelOpening
            | T::METComponent => CriticalBand,
            T::PotassiumInflux | T::Potassium => Loudness,
            T::CalciumInflux | T::Calcium => Loudness,
            T::Depolarization | T::ReceptorPotential | T::EndocochlearPotential => Loudness,
            T::GlutamateRelease | T::Glutamate => TemporalResolution,
            T::ActionPotential => Pitch,
            T::Prestin | T::Electromotility | T::CochlearAmplification => FrequencySelectivity,
            T::KCNQ4 | T::CaV1_3 | T::BKChannel | T::IonChannel => Loudness,
            T::CellularSignal | T::TipLinkProtein => AuditoryFilter,
            // Transduction events → psychoacoustic events.
            T::BasilarMembraneMotion => AcousticStimulus,
            T::StereociliaBundleDeflection | T::TipLinkStretch => CochlearFiltering,
            T::METChannelGating | T::PotassiumEntry => NeuralTransduction,
            T::CellDepolarization => BrainstemProcessing,
            T::CalciumEntry => BrainstemProcessing,
            T::VesicleRelease => CorticalAnalysis,
            T::NerveActivation => PerceptFormation,
            T::PrestiConformationChange | T::CellLengthChange | T::BasilarMembraneAmplification => {
                FrequencyAnalysis
            }
            T::SlowAdaptation | T::FastAdaptation => CochlearFiltering,
            T::TransductionEvent => PsychoacousticEvent,
        }
    }

    fn map_morphism(m: &TransductionRelation) -> PsychoacousticRelation {
        use PsychoacousticRelationKind as Tk;
        use TransductionCategoryRelationKind as Sk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        PsychoacousticRelation { from, to, kind }
    }
}
pr4xis::register_functor!(TransductionToPsychoacoustics);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws() {
        assert_functor_laws::<TransductionToPsychoacoustics>();
    }
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn action_potential_maps_to_pitch() {
        assert_eq!(
            TransductionToPsychoacoustics::map_object(&TransductionEntity::ActionPotential),
            PsychoacousticEntity::Pitch
        );
    }
    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn every_entity_maps_valid() {
        let targets = PsychoacousticEntity::variants();
        for obj in TransductionEntity::variants() {
            assert!(targets.contains(&TransductionToPsychoacoustics::map_object(&obj)));
        }
    }
}
