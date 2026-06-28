//! Adjunctions between hearing-tree domains.
//!
//! An adjunction F ⊣ G captures the optimally-inverse relationship between
//! two domain functors. The unit η embeds an object into the round-trip
//! G(F(-)), and the counit ε projects the round-trip F(G(-)) back.
//!
//! Four canonical hearing-science adjunctions:
//!
//! 1. **Analysis ⊣ Synthesis** (Acoustics ↔ Signal Processing) —
//!    Helmholtz (1863) *On the Sensations of Tone* / Oppenheim & Schafer
//!    (2010) §10 STFT — decomposing sound into spectral components vs
//!    reconstructing sound from spectral components. Lossy: phase
//!    relationships and transient details are dropped.
//!
//! 2. **Health ⊣ Disease** (Anatomy ↔ Pathology) — Moller (2006)
//!    *Hearing: Anatomy, Physiology, and Disorders* — normal structure
//!    vs pathological state.
//!
//! 3. **Bottom-up ⊣ Top-down** (Psychoacoustics ↔ Music Perception) —
//!    Helmholtz (1863) bottom-up sensation vs Huron (2006) *Sweet
//!    Anticipation* top-down expectation; Gregory (1970) "The
//!    Intelligent Eye" perceptual-inference principle.
//!
//! 4. **Diagnosis ⊣ Treatment** (Pathology ↔ Devices) — Dillon (2012)
//!    *Hearing Aids*; Zeng et al. (2008) *Cochlear Implants* — disorder
//!    identification vs intervention application.
//!
//! # Literature
//!
//! - **Mac Lane (1971)** *Categories for the Working Mathematician*, Ch. IV
//!   — adjunctions, units, counits.
//! - **Awodey (2010)** *Category Theory* (2nd ed.), Ch. 9.
//! - **Lambek & Scott (1986)** *Introduction to Higher Order Categorical
//!   Logic*.
//!
//! # Design
//!
//! Per #166 `Composed` kind removed: heterogeneous compositions become
//! partial. Unit/counit components emit Identity at the source object
//! when the round-trip is not an isomorphism, preserving functor laws
//! while making round-trip loss explicit through `map_object` divergence.

use pr4xis::category::{Adjunction, Functor};

// =============================================================================
// 1. Analysis ⊣ Synthesis (Acoustics ↔ Signal Processing)
// =============================================================================

use crate::natural::hearing::acoustics::ontology::*;
use crate::natural::hearing::acoustics::signal_functor::AcousticsToSignalProcessing;
use crate::natural::hearing::signal_processing::ontology::*;

/// Right adjoint: Signal Processing → Acoustics (synthesis / reconstruction).
pub struct SignalProcessingToAcoustics;

impl Functor for SignalProcessingToAcoustics {
    type Source = SignalProcessingCategory;
    type Target = AcousticsCategory;

    fn map_object(obj: &SignalEntity) -> AcousticEntity {
        use AcousticEntity::*;
        use SignalEntity as S;
        match obj {
            S::FourierTransform | S::FFT | S::InverseFFT | S::Transform => SoundWave,
            S::ShortTimeFourierTransform | S::Spectrogram => SoundWave,
            S::WaveletTransform | S::HilbertTransform | S::CepstralAnalysis => SoundWave,
            S::PowerSpectralDensity => Intensity,
            S::Autocorrelation | S::Correlation => Phase,
            S::Cepstrum | S::MelFrequencyCepstrum | S::Representation => Frequency,
            S::LowPassFilter
            | S::HighPassFilter
            | S::BandPassFilter
            | S::BandStopFilter
            | S::FIRFilter
            | S::IIRFilter
            | S::GammatoneFilter
            | S::Filter => Resonance,
            S::Sampling
            | S::NyquistFrequency
            | S::Aliasing
            | S::Quantization
            | S::SamplingConcept => Air,
            S::WindowFunction
            | S::HannWindow
            | S::HammingWindow
            | S::BlackmanWindow
            | S::RectangularWindow => Amplitude,
            S::Convolution | S::SignalOperation => Resonance,
            S::Decimation | S::Interpolation => Attenuation,
            S::TimeDomain => SoundWave,
            S::FrequencyDomain => Frequency,
            S::AnalysisDomain => Wave,
            // Signal events → acoustic events.
            S::RawSignal => SourceVibration,
            S::AntiAliasFiltering => MediumCoupling,
            S::Digitization => WavePropagation,
            S::WindowApplication => BoundaryEncounter,
            S::SpectralTransform => ImpedanceTransition,
            S::SpectralSmoothing => EnergyAbsorption,
            S::FeatureExtraction => EnergyTransmission,
            S::PatternClassification => ReceiverExcitation,
            S::SignalEvent => AcousticEvent,
        }
    }

    fn map_morphism(m: &SignalRelation) -> AcousticRelation {
        use AcousticsCategoryRelationKind as Tk;
        use SignalProcessingCategoryRelationKind as Sk;
        use pr4xis::category::Arrow;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        AcousticRelation { from, to, kind }
    }
}
pr4xis::register_functor!(SignalProcessingToAcoustics);

/// Analysis ⊣ Synthesis adjunction.
///
/// Unit η_A: A → G(F(A)) — sound → spectrum → reconstructed sound.
/// Counit ε_B: F(G(B)) → B — spectrum → sound → re-analysed spectrum.
pub struct AnalysisSynthesis;

impl Adjunction for AnalysisSynthesis {
    type Left = AcousticsToSignalProcessing;
    type Right = SignalProcessingToAcoustics;

    fn unit(obj: &AcousticEntity) -> AcousticRelation {
        use pr4xis::category::Category;
        let analyzed = AcousticsToSignalProcessing::map_object(obj);
        let reconstructed = SignalProcessingToAcoustics::map_object(&analyzed);
        if reconstructed == *obj {
            AcousticRelation {
                from: *obj,
                to: *obj,
                kind: AcousticsCategoryRelationKind::Identity,
            }
        } else {
            // Heterogeneous round trip — emit identity at source.
            AcousticsCategory::identity(obj)
        }
    }

    fn counit(obj: &SignalEntity) -> SignalRelation {
        use pr4xis::category::Category;
        let synthesized = SignalProcessingToAcoustics::map_object(obj);
        let reanalyzed = AcousticsToSignalProcessing::map_object(&synthesized);
        if reanalyzed == *obj {
            SignalRelation {
                from: *obj,
                to: *obj,
                kind: SignalProcessingCategoryRelationKind::Identity,
            }
        } else {
            SignalProcessingCategory::identity(obj)
        }
    }

    fn meta() -> pr4xis::ontology::meta::Provenance {
        pr4xis::ontology::meta::Provenance {
            name: pr4xis::ontology::meta::OntologyName::new_static("AnalysisSynthesis"),
            description: pr4xis::ontology::meta::Label::new_static(
                "Acoustics ⊣ Signal Processing — analysis vs synthesis duality",
            ),
            citation: pr4xis::ontology::meta::Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician Ch. IV; Helmholtz (1863) On the Sensations of Tone; Oppenheim & Schafer (2010) Discrete-Time Signal Processing §10",
            ),
            module_path: pr4xis::ontology::meta::ModulePath::new_static(module_path!()),
        }
    }
}
pr4xis::register_adjunction!(AnalysisSynthesis);

// =============================================================================
// 2. Health ⊣ Disease (Anatomy ↔ Pathology)
// =============================================================================

use crate::natural::hearing::anatomy::ontology::*;
use crate::natural::hearing::pathology::ontology::*;

/// Left adjoint: Anatomy → Pathology (what can go wrong with this structure).
pub struct AnatomyToPathology;

impl Functor for AnatomyToPathology {
    type Source = AnatomyCategory;
    type Target = PathologyCategory;

    fn map_object(obj: &AnatomyConcept) -> PathologyEntity {
        use AnatomyConcept as A;
        use PathologyEntity::*;
        match obj {
            A::Pinna | A::EarCanal => ConductiveHearingLoss,
            A::TympanicMembrane => TympanicPerforation,
            A::Malleus | A::Incus | A::Stapes | A::Ossicle => Otosclerosis,
            A::OvalWindow | A::RoundWindow => OssicularFixation,
            A::EustachianTube | A::TensorTympani | A::Stapedius => OtitisMedia,
            A::InnerHairCell | A::OuterHairCell | A::HairCell => HairCellLoss,
            A::OrganOfCorti | A::BasilarMembrane | A::TectorialMembrane | A::CochlearMembrane => {
                StereociliaDamage
            }
            A::Endolymph | A::ScalaMedia | A::CochlearFluid => MenieresDisease,
            A::Perilymph | A::ScalaVestibuli | A::ScalaTympani => EndolymphaticHydrops,
            A::StriVascularis => StriaDysfunction,
            A::ReissnersMembrane | A::Cochlea => SensorineuralHearingLoss,
            A::SpiralGanglionNeuron | A::AuditoryNerve => AuditoryNeuropathy,
            A::CochlearNucleus
            | A::SuperiorOlivaryComplex
            | A::InferiorColliculus
            | A::MedialGeniculateBody
            | A::AuditoryCortex
            | A::AuditoryNucleus => CentralAuditoryProcessingDisorder,
            A::Vestibule | A::SemicircularCanals => MenieresDisease,
            A::SupportingCell => SynapticRibbonLoss,
            A::Ear | A::OuterEar | A::MiddleEar => ConductiveHearingLoss,
            A::InnerEar => SensorineuralHearingLoss,
        }
    }

    fn map_morphism(m: &AnatomyRelation) -> PathologyRelation {
        use AnatomyRelationKind as Sk;
        use PathologyRelationKind as Tk;
        use pr4xis::category::Arrow;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        PathologyRelation { from, to, kind }
    }
}
pr4xis::register_functor!(AnatomyToPathology);

/// Right adjoint: Pathology → Anatomy (which structure is affected).
pub struct PathologyToAnatomy;

impl Functor for PathologyToAnatomy {
    type Source = PathologyCategory;
    type Target = AnatomyCategory;

    fn map_object(obj: &PathologyEntity) -> AnatomyConcept {
        use AnatomyConcept::*;
        use PathologyEntity as P;
        match obj {
            P::ConductiveHearingLoss | P::OtitisMedia | P::Cholesteatoma => MiddleEar,
            P::SensorineuralHearingLoss
            | P::Presbycusis
            | P::NoiseInducedHearingLoss
            | P::SuddenSensorineuralLoss => Cochlea,
            P::MixedHearingLoss => Ear,
            P::AuditoryNeuropathy | P::DemyelinationVIII => AuditoryNerve,
            P::CentralAuditoryProcessingDisorder => AuditoryCortex,
            P::Otosclerosis | P::OssicularFixation => Stapes,
            P::MenieresDisease | P::EndolymphaticHydrops => Endolymph,
            P::AcousticNeuroma => AuditoryNerve,
            P::Tinnitus | P::Hyperacusis | P::PhantomPercept => Cochlea,
            P::TympanicPerforation => TympanicMembrane,
            P::HairCellLoss | P::StereociliaDamage => OuterHairCell,
            P::SynapticRibbonLoss | P::Excitotoxicity => InnerHairCell,
            P::StriaDysfunction => StriVascularis,
            P::OxidativeStress => OrganOfCorti,
            P::ElevatedThreshold | P::ReducedFrequencySelectivity | P::LoudnessRecruitment => {
                Cochlea
            }
            P::PoorSpeechInNoise
            | P::ReducedTemporalResolution
            | P::AbnormalBinauralProcessing
            | P::PerceptualDeficit => AuditoryCortex,
            P::Audiogram
            | P::PureToneAverage
            | P::SpeechReceptionThreshold
            | P::OtoacousticEmission
            | P::AuditoryBrainstemResponse
            | P::ClinicalMeasure => Ear,
            P::HearingLoss | P::PeripheralPathology => Cochlea,
            P::CentralPathology => AuditoryCortex,
            P::DamageMechanism => OrganOfCorti,
            P::NoiseExposure
            | P::AgingDegeneration
            | P::Infection
            | P::Autoimmune
            | P::GeneticMutation => Ear,
            P::OHCDamage => OuterHairCell,
            P::IHCDamage => InnerHairCell,
            P::SynapseLoss => SpiralGanglionNeuron,
            P::StriDegeneration => StriVascularis,
            P::MiddleEarDysfunction => MiddleEar,
            P::NeuralDegeneration => AuditoryNerve,
            P::ThresholdShift => Cochlea,
            P::FrequencyResolutionLoss => OrganOfCorti,
            P::TemporalSmearing => SpiralGanglionNeuron,
            P::TinnitusGeneration => Cochlea,
            P::CommunicationDifficulty => AuditoryCortex,
            P::PathologyEvent => Ear,
        }
    }

    fn map_morphism(m: &PathologyRelation) -> AnatomyRelation {
        use AnatomyRelationKind as Tk;
        use PathologyCategoryRelationKind as Sk;
        use pr4xis::category::Arrow;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        AnatomyRelation { from, to, kind }
    }
}
pr4xis::register_functor!(PathologyToAnatomy);

pub struct HealthDisease;

impl Adjunction for HealthDisease {
    type Left = AnatomyToPathology;
    type Right = PathologyToAnatomy;

    fn unit(obj: &AnatomyConcept) -> AnatomyRelation {
        use pr4xis::category::Category;
        let diseased = AnatomyToPathology::map_object(obj);
        let recovered = PathologyToAnatomy::map_object(&diseased);
        if recovered == *obj {
            AnatomyRelation {
                from: *obj,
                to: *obj,
                kind: AnatomyRelationKind::Identity,
            }
        } else {
            AnatomyCategory::identity(obj)
        }
    }

    fn counit(obj: &PathologyEntity) -> PathologyRelation {
        use pr4xis::category::Category;
        let structure = PathologyToAnatomy::map_object(obj);
        let re_diseased = AnatomyToPathology::map_object(&structure);
        if re_diseased == *obj {
            PathologyRelation {
                from: *obj,
                to: *obj,
                kind: PathologyCategoryRelationKind::Identity,
            }
        } else {
            PathologyCategory::identity(obj)
        }
    }

    fn meta() -> pr4xis::ontology::meta::Provenance {
        pr4xis::ontology::meta::Provenance {
            name: pr4xis::ontology::meta::OntologyName::new_static("HealthDisease"),
            description: pr4xis::ontology::meta::Label::new_static(
                "Anatomy ⊣ Pathology — structure vs disease duality",
            ),
            citation: pr4xis::ontology::meta::Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician Ch. IV; Moller (2006) Hearing: Anatomy, Physiology, and Disorders",
            ),
            module_path: pr4xis::ontology::meta::ModulePath::new_static(module_path!()),
        }
    }
}
pr4xis::register_adjunction!(HealthDisease);

// =============================================================================
// 3. Bottom-up ⊣ Top-down (Psychoacoustics ↔ Music Perception)
// =============================================================================

use crate::natural::hearing::music_perception::ontology::*;
use crate::natural::hearing::psychoacoustics::music_functor::PsychoacousticsToMusic;
use crate::natural::hearing::psychoacoustics::ontology::*;

/// Right adjoint: Music Perception → Psychoacoustics (top-down influence).
pub struct MusicToPsychoacoustics;

impl Functor for MusicToPsychoacoustics {
    type Source = MusicPerceptionCategory;
    type Target = PsychoacousticsCategory;

    fn map_object(obj: &MusicEntity) -> PsychoacousticEntity {
        use MusicEntity as M;
        use PsychoacousticEntity::*;
        match obj {
            M::PitchHeight
            | M::PitchChroma
            | M::OctaveEquivalence
            | M::AbsolutePitch
            | M::RelativePitch
            | M::MelodicContour
            | M::IntervalPerception
            | M::PitchPercept => Pitch,
            M::Consonance
            | M::Dissonance
            | M::RoughnessModel
            | M::HarmonicSeries
            | M::VirtualPitchPercept
            | M::MissingFundamental
            | M::Chord
            | M::Tonality
            | M::KeySense
            | M::HarmonicPercept => FrequencySelectivity,
            M::Beat
            | M::Meter
            | M::Tempo
            | M::Syncopation
            | M::Groove
            | M::Entrainment
            | M::TemporalExpectation
            | M::RhythmicPercept => TemporalResolution,
            M::SpectralCentroid
            | M::AttackTime
            | M::SpectralFlux
            | M::InstrumentIdentification
            | M::TimbrePercept => Timbre,
            M::MusicalExpectation
            | M::Surprise
            | M::Tension
            | M::Resolution
            | M::MusicalEmotion
            | M::AffectiveResponse => Loudness,
            M::EarWorm | M::MusicalMemory | M::TonalSchemaMemory => Pitch,
            // Music events → psychoacoustic events.
            M::AuditoryInput => AcousticStimulus,
            M::PitchExtraction => PitchExtraction,
            M::OnsetDetection => NeuralTransduction,
            M::HarmonicGrouping => FrequencyAnalysis,
            M::MelodicTracking => CorticalAnalysis,
            M::BeatInduction => BrainstemProcessing,
            M::MetricFraming => CorticalAnalysis,
            M::TonalInterpretation => PerceptFormation,
            M::MusicalExpectationFormation => PerceptFormation,
            M::GroovePerception => CorticalAnalysis,
            M::EmotionalResponse => AwareExperience,
            M::MusicEvent => PsychoacousticEvent,
        }
    }

    fn map_morphism(m: &MusicRelation) -> PsychoacousticRelation {
        use MusicPerceptionCategoryRelationKind as Sk;
        use PsychoacousticRelationKind as Tk;
        use pr4xis::category::Arrow;
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
pr4xis::register_functor!(MusicToPsychoacoustics);

pub struct BottomUpTopDown;

impl Adjunction for BottomUpTopDown {
    type Left = PsychoacousticsToMusic;
    type Right = MusicToPsychoacoustics;

    fn unit(obj: &PsychoacousticEntity) -> PsychoacousticRelation {
        use pr4xis::category::Category;
        let musical = PsychoacousticsToMusic::map_object(obj);
        let feedback = MusicToPsychoacoustics::map_object(&musical);
        if feedback == *obj {
            PsychoacousticRelation {
                from: *obj,
                to: *obj,
                kind: PsychoacousticsCategoryRelationKind::Identity,
            }
        } else {
            PsychoacousticsCategory::identity(obj)
        }
    }

    fn counit(obj: &MusicEntity) -> MusicRelation {
        use pr4xis::category::Category;
        let percept = MusicToPsychoacoustics::map_object(obj);
        let re_musical = PsychoacousticsToMusic::map_object(&percept);
        if re_musical == *obj {
            MusicRelation {
                from: *obj,
                to: *obj,
                kind: MusicPerceptionCategoryRelationKind::Identity,
            }
        } else {
            MusicPerceptionCategory::identity(obj)
        }
    }

    fn meta() -> pr4xis::ontology::meta::Provenance {
        pr4xis::ontology::meta::Provenance {
            name: pr4xis::ontology::meta::OntologyName::new_static("BottomUpTopDown"),
            description: pr4xis::ontology::meta::Label::new_static(
                "Psychoacoustics ⊣ Music Perception — stimulus vs expectation duality",
            ),
            citation: pr4xis::ontology::meta::Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician Ch. IV; Helmholtz (1863) On the Sensations of Tone; Huron (2006) Sweet Anticipation; Gregory (1970) The Intelligent Eye",
            ),
            module_path: pr4xis::ontology::meta::ModulePath::new_static(module_path!()),
        }
    }
}
pr4xis::register_adjunction!(BottomUpTopDown);

// =============================================================================
// 4. Diagnosis ⊣ Treatment (Pathology ↔ Devices)
// =============================================================================

use crate::natural::hearing::devices::ontology::*;
use crate::natural::hearing::pathology::devices_functor::PathologyToDevices;

/// Right adjoint: Devices → Pathology (what condition this device treats).
pub struct DevicesToPathology;

impl Functor for DevicesToPathology {
    type Source = DeviceCategory;
    type Target = PathologyCategory;

    fn map_object(obj: &DeviceEntity) -> PathologyEntity {
        use DeviceEntity as D;
        use PathologyEntity::*;
        match obj {
            D::BehindTheEar
            | D::InTheEar
            | D::CompletelyInCanal
            | D::ReceiverInCanal
            | D::HearingAid => SensorineuralHearingLoss,
            D::CROS | D::BiCROS => AbnormalBinauralProcessing,
            D::CochlearImplant => SensorineuralHearingLoss,
            D::BoneAnchoredHearingAid => ConductiveHearingLoss,
            D::MiddleEarImplant => Otosclerosis,
            D::AuditoryBrainstemImplant => AcousticNeuroma,
            D::ImplantableDevice => SensorineuralHearingLoss,
            D::BoneConductionHeadphone | D::SoftbandBAHA | D::AdhesiveBC | D::BCDevice => {
                ConductiveHearingLoss
            }
            D::DirectionalMicrophone => PoorSpeechInNoise,
            D::NoiseSuppression => Tinnitus,
            D::FeedbackCancellation => ElevatedThreshold,
            D::FrequencyCompression => ReducedFrequencySelectivity,
            D::WideAdaptiveDynamicRange => LoudnessRecruitment,
            D::Telecoil | D::BluetoothStreaming | D::SignalProcessingFeature => PoorSpeechInNoise,
            D::Audiometer => Audiogram,
            D::Tympanometer => OtoacousticEmission,
            D::OAEProbe => OtoacousticEmission,
            D::ABRSystem => AuditoryBrainstemResponse,
            D::RealEarMeasurement | D::DiagnosticEquipment => Audiogram,
            D::Microphone
            | D::Amplifier
            | D::Receiver
            | D::ElectrodeArray
            | D::SpeechProcessor
            | D::DeviceComponent => HearingLoss,
            // Device events → pathology events.
            D::HearingLossDiagnosis => NoiseExposure,
            D::DeviceSelection => OHCDamage,
            D::CustomMolding => StriDegeneration,
            D::InitialFitting => ThresholdShift,
            D::RealEarVerificationEvent => TemporalSmearing,
            D::FineTuning => FrequencyResolutionLoss,
            D::OutcomeImprovement => CommunicationDifficulty,
            D::DeviceEvent => PathologyEvent,
        }
    }

    fn map_morphism(m: &DeviceRelation) -> PathologyRelation {
        use DeviceCategoryRelationKind as Sk;
        use PathologyRelationKind as Tk;
        use pr4xis::category::Arrow;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        PathologyRelation { from, to, kind }
    }
}
pr4xis::register_functor!(DevicesToPathology);

pub struct DiagnosisTreatment;

impl Adjunction for DiagnosisTreatment {
    type Left = PathologyToDevices;
    type Right = DevicesToPathology;

    fn unit(obj: &PathologyEntity) -> PathologyRelation {
        use pr4xis::category::Category;
        let device = PathologyToDevices::map_object(obj);
        let condition = DevicesToPathology::map_object(&device);
        if condition == *obj {
            PathologyRelation {
                from: *obj,
                to: *obj,
                kind: PathologyCategoryRelationKind::Identity,
            }
        } else {
            PathologyCategory::identity(obj)
        }
    }

    fn counit(obj: &DeviceEntity) -> DeviceRelation {
        use pr4xis::category::Category;
        let condition = DevicesToPathology::map_object(obj);
        let device = PathologyToDevices::map_object(&condition);
        if device == *obj {
            DeviceRelation {
                from: *obj,
                to: *obj,
                kind: DeviceCategoryRelationKind::Identity,
            }
        } else {
            DeviceCategory::identity(obj)
        }
    }

    fn meta() -> pr4xis::ontology::meta::Provenance {
        pr4xis::ontology::meta::Provenance {
            name: pr4xis::ontology::meta::OntologyName::new_static("DiagnosisTreatment"),
            description: pr4xis::ontology::meta::Label::new_static(
                "Pathology ⊣ Devices — disorder vs intervention duality",
            ),
            citation: pr4xis::ontology::meta::Citation::parse_static(
                "Mac Lane (1971) Categories for the Working Mathematician Ch. IV; Dillon (2012) Hearing Aids 2nd ed.; Zeng et al. (2008) Cochlear Implants",
            ),
            module_path: pr4xis::ontology::meta::ModulePath::new_static(module_path!()),
        }
    }
}
pr4xis::register_adjunction!(DiagnosisTreatment);

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn signal_to_acoustics_functor_laws() {
        assert_functor_laws::<SignalProcessingToAcoustics>();
    }
    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn pathology_to_anatomy_functor_laws() {
        assert_functor_laws::<PathologyToAnatomy>();
    }
    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn music_to_psychoacoustics_functor_laws() {
        assert_functor_laws::<MusicToPsychoacoustics>();
    }
    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn devices_to_pathology_functor_laws() {
        assert_functor_laws::<DevicesToPathology>();
    }
    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn anatomy_to_pathology_functor_laws() {
        assert_functor_laws::<AnatomyToPathology>();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn analysis_synthesis_unit_wellformed() {
        for obj in AcousticEntity::variants() {
            let m = AnalysisSynthesis::unit(&obj);
            assert_eq!(m.from, obj);
            assert!(AcousticEntity::variants().contains(&m.to));
        }
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn health_disease_unit_wellformed() {
        for obj in AnatomyConcept::variants() {
            let m = HealthDisease::unit(&obj);
            assert_eq!(m.from, obj);
            assert!(AnatomyConcept::variants().contains(&m.to));
        }
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn bottom_up_top_down_unit_wellformed() {
        for obj in PsychoacousticEntity::variants() {
            let m = BottomUpTopDown::unit(&obj);
            assert_eq!(m.from, obj);
            assert!(PsychoacousticEntity::variants().contains(&m.to));
        }
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn diagnosis_treatment_unit_wellformed() {
        for obj in PathologyEntity::variants() {
            let m = DiagnosisTreatment::unit(&obj);
            assert_eq!(m.from, obj);
            assert!(PathologyEntity::variants().contains(&m.to));
        }
    }
}
