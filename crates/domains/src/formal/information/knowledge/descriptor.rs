use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::ontology::upper::being::Being;

// Vocabulary descriptors — runtime instances of the KnowledgeBase ontology.
//
// Each descriptor is an instance of the Vocabulary concept (VoID).
// The descriptor is causally connected (Smith 1984): computed from
// the actual Category implementation.

/// A runtime description of a loaded ontology — an instance of Vocabulary.
#[derive(Debug, Clone)]
pub struct VocabularyDescriptor {
    pub name: &'static str,
    pub domain: &'static str,
    pub being: Being,
    pub reason: &'static str,
    pub source: &'static str,
    pub concepts: usize,
    pub morphisms: usize,
}

/// Enumerate all ontologies loaded in the pr4xis knowledge base.
///
/// Cached after first call — the descriptor list is computed once from
/// `Category::morphisms()` / `Entity::variants()` and then reused.
pub fn describe_knowledge_base() -> Vec<VocabularyDescriptor> {
    static CACHE: std::sync::OnceLock<Vec<VocabularyDescriptor>> = std::sync::OnceLock::new();
    CACHE.get_or_init(build_descriptors).clone()
}

fn build_descriptors() -> Vec<VocabularyDescriptor> {
    vec![
        // =====================================================================
        // Cognitive — Cognition
        // =====================================================================
        descriptor::<
            crate::cognitive::cognition::distinction::DistinctionCategory,
            crate::cognitive::cognition::distinction::DistinctionElement,
        >("Distinction", "cognitive.cognition", "Spencer-Brown (1969)"),
        descriptor::<
            crate::cognitive::cognition::epistemics::EpistemicCategory,
            crate::cognitive::cognition::epistemics::EpistemicState,
        >("Epistemics", "cognitive.cognition", "von Foerster (1981)"),
        descriptor::<
            crate::cognitive::cognition::metacognition::MetaCognitionCategory,
            crate::cognitive::cognition::metacognition::MetaConcept,
        >("Metacognition", "cognitive.cognition", "von Foerster (1981); Olivares-Alarcos MOI (2023)"),
        descriptor::<
            crate::cognitive::cognition::self_model::SelfModelCategory,
            crate::cognitive::cognition::self_model::SelfModelConcept,
        >("Self-Model", "cognitive.cognition", "von Foerster (1981); IEEE AuR (2021); MAPE-K (2003)"),
        // =====================================================================
        // Cognitive — Linguistics
        // =====================================================================
        descriptor::<
            crate::cognitive::linguistics::lexicon::ontology::LexicalCategory,
            crate::cognitive::linguistics::lexicon::pos::PosTag,
        >("Lexicon", "cognitive.linguistics.lexicon", "Lambek (1958); Chiarcos & Sukhareva OLiA (2015)"),
        descriptor::<
            crate::cognitive::linguistics::morphology::tense::TenseCategory,
            crate::cognitive::linguistics::morphology::tense::TenseAspect,
        >("Tense & Aspect", "cognitive.linguistics.morphology", "Reichenbach (1947); Comrie (1976)"),
        descriptor::<
            crate::cognitive::linguistics::orthography::distance::SpellingErrorCategory,
            crate::cognitive::linguistics::orthography::distance::SpellingErrorConcept,
        >("Spelling Errors", "cognitive.linguistics.orthography", "Damerau (1964); Brill & Moore (2000)"),
        descriptor::<
            crate::cognitive::linguistics::orthography::channel::ChannelCategory,
            crate::cognitive::linguistics::orthography::channel::ChannelConcept,
        >("Noisy Channel", "cognitive.linguistics.orthography", "Shannon (1948); Kernighan et al. (1990)"),
        descriptor::<
            crate::cognitive::linguistics::pragmatics::reference::ReferenceCategory,
            crate::cognitive::linguistics::pragmatics::reference::ReferenceConcept,
        >("Discourse Reference", "cognitive.linguistics.pragmatics", "Kamp (1981); Grosz, Joshi & Weinstein (1995)"),
        descriptor::<
            crate::cognitive::linguistics::pragmatics::generation::ProductionCategory,
            crate::cognitive::linguistics::pragmatics::generation::ProductionConcept,
        >("Speech Production", "cognitive.linguistics.pragmatics", "Levelt (1989); de Groote (2001)"),
        descriptor::<
            crate::cognitive::linguistics::pragmatics::nlg::NlgCategory,
            crate::cognitive::linguistics::pragmatics::nlg::NlgConcept,
        >("NLG Pipeline", "cognitive.linguistics.pragmatics", "Reiter & Dale (2000)"),
        descriptor::<
            crate::cognitive::linguistics::pragmatics::response::ResponseCategory,
            crate::cognitive::linguistics::pragmatics::response::ResponseConcept,
        >("Response Generation", "cognitive.linguistics.pragmatics", "Reiter & Dale (2000); Lambek & Scott (1986)"),
        // =====================================================================
        // Formal — Mathematics
        // =====================================================================
        descriptor::<
            crate::formal::math::ontology::NumberHierarchy,
            crate::formal::math::ontology::MathDomain,
        >("Mathematics", "formal.math", "Landau (1930)"),
        descriptor::<
            crate::formal::math::geometry::ontology::GeometryCategory,
            crate::formal::math::geometry::ontology::GeometricPrimitive,
        >("Geometry", "formal.math.geometry", "Hilbert (1899); Avigad et al. (2009)"),
        descriptor::<
            crate::formal::math::linear_algebra::ontology::LinearAlgebraCategory,
            crate::formal::math::linear_algebra::ontology::AlgebraicStructure,
        >("Linear Algebra", "formal.math.linear_algebra", "Strang; Golub & Van Loan"),
        descriptor::<
            crate::formal::math::probability::ontology::ProbabilityCategory,
            crate::formal::math::probability::ontology::ProbabilityConcept,
        >("Probability", "formal.math.probability", "Kolmogorov (1933); Shannon (1948)"),
        descriptor::<
            crate::formal::math::quantity::ontology::DimensionCategory,
            crate::formal::math::quantity::ontology::BaseDimension,
        >("Quantity", "formal.math.quantity", "BIPM SI Brochure (2019)"),
        descriptor::<
            crate::formal::math::rotation::ontology::RotationCategory,
            crate::formal::math::rotation::ontology::RotationRepr,
        >("Rotation", "formal.math.rotation", "Hamilton (1844); Shoemake (1985)"),
        descriptor::<
            crate::formal::math::signal_processing::ontology::SignalCategory,
            crate::formal::math::signal_processing::ontology::SignalDomainConcept,
        >("Signal Processing", "formal.math.signal_processing", "Shannon (1949); Nyquist (1928)"),
        descriptor::<
            crate::formal::math::statistics::ontology::StatisticalCategory,
            crate::formal::math::statistics::ontology::StatisticalConcept,
        >("Statistics", "formal.math.statistics", "Fisher (1925); Neyman & Pearson (1933)"),
        descriptor::<
            crate::formal::math::temporal::ontology::TimeSystemCategory,
            crate::formal::math::temporal::time_system::TimeSystem,
        >("Temporal", "formal.math.temporal", "Allen (1983); BIPM (UTC/TAI)"),
        descriptor::<
            crate::formal::math::control_theory::ontology::ControlCategory,
            crate::formal::math::control_theory::ontology::ControlConcept,
        >("Control Theory", "formal.math.control_theory", "Astrom & Murray (2008); Lyapunov (1892)"),
        // =====================================================================
        // Formal — Meta
        // =====================================================================
        descriptor::<
            crate::formal::meta::artifact_identity::ontology::ArtifactIdentityCategory,
            crate::formal::meta::artifact_identity::ontology::IdentityConcept,
        >("Artifact Identity", "formal.meta.artifact_identity", "Dolstra (2006)"),
        descriptor::<
            crate::formal::meta::staging::ontology::StagingCategory,
            crate::formal::meta::staging::ontology::StageConcept,
        >("Staging", "formal.meta.staging", "Futamura (1971)"),
        descriptor::<
            crate::formal::meta::ontology_diagnostics::ontology::MetaCategory,
            crate::formal::meta::ontology_diagnostics::ontology::MetaEntity,
        >("Ontology Diagnostics", "formal.meta.ontology_diagnostics", "citings pending; see domain citings.md"),
        // =====================================================================
        // Formal — Information
        // =====================================================================
        descriptor::<
            crate::formal::information::ontology::InfoCategory,
            crate::formal::information::ontology::InfoUnit,
        >("Information Theory", "formal.information", "Shannon (1948); Turing (1936)"),
        descriptor::<
            crate::formal::information::communication::ontology::CommunicationCategory,
            crate::formal::information::communication::ontology::CommunicationConcept,
        >("Communication", "formal.information.communication", "Shannon (1948); Jakobson (1960)"),
        descriptor::<
            crate::formal::information::dialogue::ontology::DialogueCategory,
            crate::formal::information::dialogue::ontology::DialogueConcept,
        >("Dialogue", "formal.information.dialogue", "Austin (1962); Traum (1994); Clark (1996)"),
        descriptor::<
            crate::formal::information::events::ontology::EventCategory,
            crate::formal::information::events::ontology::EventConcept,
        >("Events", "formal.information.events", "Fowler (2005); Guizzardi et al. UFO-B (2013)"),
        descriptor::<
            crate::formal::information::concurrency::ontology::ConcurrencyCategory,
            crate::formal::information::concurrency::ontology::ConcurrencyConcept,
        >("Concurrency", "formal.information.concurrency", "Hoare CSP (1978); Hewitt (1973)"),
        descriptor::<
            crate::formal::information::provenance::ontology::ProvenanceCategory,
            crate::formal::information::provenance::ontology::ProvenanceConcept,
        >("Provenance", "formal.information.provenance", "W3C PROV-O (2013)"),
        descriptor::<
            crate::formal::information::knowledge::ontology::KnowledgeBaseCategory,
            crate::formal::information::knowledge::ontology::KnowledgeConcept,
        >("Knowledge Base", "formal.information.knowledge", "W3C VoID (2011); Herre & Loebe (2005)"),
        descriptor::<
            crate::formal::information::diagnostics::ontology::DiagnosticCategory,
            crate::formal::information::diagnostics::ontology::DiagnosticConcept,
        >("Diagnostics", "formal.information.diagnostics", "Reiter (1987); Gertler (1998)"),
        descriptor::<
            crate::formal::information::measurement::ontology::MeasurementCategory,
            crate::formal::information::measurement::ontology::MeasurementConcept,
        >("Measurement", "formal.information.measurement", "JCGM 200:2012 (VIM); Stevens (1946)"),
        descriptor::<
            crate::formal::information::schema::ontology::SchemaCategory,
            crate::formal::information::schema::ontology::SchemaConcept,
        >("Schema", "formal.information.schema", "Spivak (2012)"),
        descriptor::<
            crate::formal::information::storage::ontology::RepositoryCategory,
            crate::formal::information::storage::ontology::RepositoryConcept,
        >("Storage", "formal.information.storage", "RDF4J; Spivak (2012)"),
        // =====================================================================
        // Formal — Systems
        // =====================================================================
        descriptor::<
            crate::formal::systems::ontology::SystemsCategory,
            crate::formal::systems::ontology::SystemConcept,
        >("Systems", "formal.systems", "von Bertalanffy (1968); Ashby (1956)"),
        descriptor::<
            crate::formal::systems::control::ControlCategory,
            crate::formal::systems::control::ControlConcept,
        >("Control Systems", "formal.systems", "Wiener (1948); Conant & Ashby (1970)"),
        // =====================================================================
        // Formal — Other (analytical, derivation, optimization, recommendation)
        // =====================================================================
        descriptor::<
            crate::formal::analytical_methods::ontology::AnalyticalCategory,
            crate::formal::analytical_methods::ontology::AnalyticalEntity,
        >("Analytical Methods", "formal.analytical_methods", "Wille (1982); Ganter & Wille (1999)"),
        descriptor::<
            crate::formal::derivation::ontology::DerivationCategory,
            crate::formal::derivation::ontology::DerivationEntity,
        >("Derivation", "formal.derivation", "Gentzen (1935); Prawitz (1965)"),
        descriptor::<
            crate::formal::optimization::ontology::OptimizationCategory,
            crate::formal::optimization::ontology::OptimizationEntity,
        >("Optimization", "formal.optimization", "Boyd & Vandenberghe (2004); Pareto (1906)"),
        descriptor::<
            crate::formal::recommendation::ontology::RecommendationCategory,
            crate::formal::recommendation::ontology::RecommendationEntity,
        >("Recommendation", "formal.recommendation", "Von Neumann & Morgenstern (1944); Keeney & Raiffa (1976)"),
        // =====================================================================
        // Applied — Data Provisioning
        // =====================================================================
        descriptor::<
            crate::applied::data_provisioning::ontology::DataProvisioningCategory,
            crate::applied::data_provisioning::ontology::ProvisioningConcept,
        >("Data Provisioning", "applied.data_provisioning", "Dolstra (2006)"),
        // =====================================================================
        // Applied — HMI
        // =====================================================================
        descriptor::<
            crate::applied::hmi::theming::ontology::ThemingCategory,
            crate::applied::hmi::theming::base16::ColorSlot,
        >("Theming", "applied.hmi.theming", "Base16 styling spec; WCAG 2.1"),
        // =====================================================================
        // Applied — Navigation
        // =====================================================================
        descriptor::<
            crate::applied::navigation::ahrs::ontology::AhrsCategory,
            crate::applied::navigation::ahrs::ontology::AhrsFilterType,
        >("AHRS", "applied.navigation.ahrs", "Madgwick (2010); Mahony et al. (2008)"),
        descriptor::<
            crate::applied::navigation::celestial::ontology::CelestialCategory,
            crate::applied::navigation::celestial::ontology::CelestialSensor,
        >("Celestial Navigation", "applied.navigation.celestial", "Bowditch (2002); Wertz (2001)"),
        descriptor::<
            crate::applied::navigation::gnss::ontology::GnssCategory,
            crate::applied::navigation::gnss::ontology::GnssObservable,
        >("GNSS", "applied.navigation.gnss", "IS-GPS-200 (2022); Groves (2013)"),
        descriptor::<
            crate::applied::navigation::imu::ontology::ImuCategory,
            crate::applied::navigation::imu::ontology::ImuMeasurement,
        >("IMU", "applied.navigation.imu", "Titterton & Weston (2004); Groves (2013)"),
        descriptor::<
            crate::applied::navigation::ins_gnss::ontology::InsGnssCategory,
            crate::applied::navigation::ins_gnss::ontology::CouplingLevel,
        >("INS/GNSS", "applied.navigation.ins_gnss", "Groves (2013); Titterton & Weston (2004)"),
        descriptor::<
            crate::applied::navigation::odometry::ontology::OdometryCategory,
            crate::applied::navigation::odometry::ontology::OdometrySource,
        >("Odometry", "applied.navigation.odometry", "Borenstein et al. (1996); Thrun et al. (2005)"),
        // =====================================================================
        // Applied — Sensor Fusion
        // =====================================================================
        descriptor::<
            crate::applied::sensor_fusion::fusion::ontology::FusionCategory,
            crate::applied::sensor_fusion::fusion::ontology::FusionPhase,
        >("Sensor Fusion", "applied.sensor_fusion.fusion", "Kalman (1960); Bar-Shalom et al. (2001)"),
        descriptor::<
            crate::applied::sensor_fusion::observation::ontology::ObservationCategory,
            crate::applied::sensor_fusion::observation::ontology::ObservationStage,
        >("Observation", "applied.sensor_fusion.observation", "JDL (1999); Bar-Shalom et al. (2001)"),
        descriptor::<
            crate::applied::sensor_fusion::sensor::ontology::SensorCategory,
            crate::applied::sensor_fusion::sensor::modality::SensorType,
        >("Sensor", "applied.sensor_fusion.sensor", "Groves (2013); Bar-Shalom et al. (2001)"),
        descriptor::<
            crate::applied::sensor_fusion::state::ontology::EstimationCategory,
            crate::applied::sensor_fusion::state::ontology::EstimationConcept,
        >("State Estimation", "applied.sensor_fusion.state", "Kalman (1960); Maybeck (1979)"),
        descriptor::<
            crate::applied::sensor_fusion::time::ontology::SensorTimeCategory,
            crate::applied::sensor_fusion::time::synchronization::SyncStrategy,
        >("Sensor Time", "applied.sensor_fusion.time", "Bar-Shalom et al. (2001); Groves (2013)"),
        // =====================================================================
        // Applied — Space
        // =====================================================================
        descriptor::<
            crate::applied::space::attitude::ontology::AttitudeCategory,
            crate::applied::space::attitude::ontology::AttitudeSensor,
        >("Attitude Determination", "applied.space.attitude", "Wertz (1978); Markley & Crassidis (2014)"),
        descriptor::<
            crate::applied::space::orbit::ontology::OrbitCategory,
            crate::applied::space::orbit::ontology::OrbitalElement,
        >("Orbit Determination", "applied.space.orbit", "Vallado (2013); Battin (1999)"),
        // =====================================================================
        // Applied — Tracking
        // =====================================================================
        descriptor::<
            crate::applied::tracking::single_target::ontology::TargetStateCategory,
            crate::applied::tracking::single_target::ontology::TargetStateComponent,
        >("Single-Target Tracking", "applied.tracking.single_target", "Bar-Shalom et al. (2001); Li & Jilkov (2003)"),
        // =====================================================================
        // Applied — Underwater
        // =====================================================================
        descriptor::<
            crate::applied::underwater::acoustic::ontology::AcousticCategory,
            crate::applied::underwater::acoustic::ontology::AcousticSystem,
        >("Underwater Acoustic", "applied.underwater.acoustic", "Milne (1983); Kinsey et al. (2006)"),
        descriptor::<
            crate::applied::underwater::auv::ontology::AuvCategory,
            crate::applied::underwater::auv::ontology::AuvSensor,
        >("AUV", "applied.underwater.auv", "Kinsey et al. (2006); Paull et al. (2014)"),
        // =====================================================================
        // Applied — Industrial
        // =====================================================================
        descriptor::<
            crate::applied::industrial::process::ontology::ProcessCategory,
            crate::applied::industrial::process::ontology::ProcessVariable,
        >("Process Control", "applied.industrial.process", "Ogunnaike & Ray (1994); Seborg et al. (2011)"),
        descriptor::<
            crate::applied::industrial::structural::ontology::StructuralCategory,
            crate::applied::industrial::structural::ontology::StructuralSensor,
        >("Structural Health", "applied.industrial.structural", "Farrar & Worden (2007); Paris & Erdogan (1963)"),
        // =====================================================================
        // Applied — Localization
        // =====================================================================
        descriptor::<
            crate::applied::localization::terrain::ontology::TerrainCategory,
            crate::applied::localization::terrain::ontology::TerrainFeature,
        >("Terrain Navigation", "applied.localization.terrain", "Goldstein (1987)"),
        // =====================================================================
        // Applied — Perception
        // =====================================================================
        descriptor::<
            crate::applied::perception::occupancy::ontology::OccupancyCategory,
            crate::applied::perception::occupancy::ontology::CellState,
        >("Occupancy Grid", "applied.perception.occupancy", "Elfes (1989); Thrun et al. (2005)"),
        // =====================================================================
        // Applied — Hardware
        // =====================================================================
        descriptor::<
            crate::applied::hardware::traffic::ontology::TrafficCategory,
            crate::applied::hardware::traffic::ontology::TrafficDirection,
        >("Traffic", "applied.hardware.traffic", "Highway Capacity Manual (TRB); Webster (1958)"),
        descriptor::<
            crate::applied::hardware::elevator::ontology::ElevatorCategory,
            crate::applied::hardware::elevator::ontology::Floor,
        >("Elevator", "applied.hardware.elevator", "Mandel (1989); Barney & Dos Santos (1985)"),
        // =====================================================================
        // Natural — Physics
        // =====================================================================
        descriptor::<
            crate::natural::physics::ontology::PhysicsCategory,
            crate::natural::physics::ontology::PhysicsLaw,
        >("Physics", "natural.physics", "Newton (1687); Maxwell (1865)"),
        descriptor::<
            crate::natural::physics::kinematics::ontology::KinematicsCategory,
            crate::natural::physics::kinematics::ontology::KinematicQuantity,
        >("Kinematics", "natural.physics.kinematics", "Newton (1687); Bar-Shalom et al. (2001)"),
        // =====================================================================
        // Natural — Geodesy
        // =====================================================================
        descriptor::<
            crate::natural::geodesy::ontology::GeodesyCategory,
            crate::natural::geodesy::ontology::CoordinateSystem,
        >("Geodesy", "natural.geodesy", "NIMA TR8350.2 (2000); Torge & Muller (2012)"),
        // =====================================================================
        // Natural — Colors
        // =====================================================================
        descriptor::<
            crate::natural::colors::ontology::ColorCategory,
            crate::natural::colors::ontology::PrimaryColor,
        >("Color", "natural.colors", "IEC 61966-2-1 (sRGB); W3C WCAG 2.1"),
        // =====================================================================
        // Natural — Biomedical
        // =====================================================================
        descriptor::<
            crate::natural::biomedical::acoustics::ontology::AcousticsCategory,
            crate::natural::biomedical::acoustics::ontology::AcousticsEntity,
        >("Biomedical Acoustics", "natural.biomedical.acoustics", "citings pending; see domain citings.md"),
        descriptor::<
            crate::natural::biomedical::biochemistry::ontology::BiochemistryCategory,
            crate::natural::biomedical::biochemistry::ontology::BiochemistryEntity,
        >("Biochemistry", "natural.biomedical.biochemistry", "Bhatt (2000); Sheng & Greenberg (1990)"),
        descriptor::<
            crate::natural::biomedical::bioelectricity::ontology::BioelectricCategory,
            crate::natural::biomedical::bioelectricity::ontology::BioelectricEntity,
        >("Bioelectricity", "natural.biomedical.bioelectricity", "Levin (2019); Fields & Levin (2022)"),
        descriptor::<
            crate::natural::biomedical::bioelectricity::morphospace::MorphospaceCategory,
            crate::natural::biomedical::bioelectricity::morphospace::MorphospaceEntity,
        >("Morphospace", "natural.biomedical.bioelectricity", "Fields & Levin (2022); Chernet & Levin (2013)"),
        descriptor::<
            crate::natural::biomedical::biology::ontology::BiologyCategory,
            crate::natural::biomedical::biology::ontology::BiologicalEntity,
        >("Biology", "natural.biomedical.biology", "Hooper (1956)"),
        descriptor::<
            crate::natural::biomedical::biophysics::ontology::BiophysicsCategory,
            crate::natural::biomedical::biophysics::ontology::BiophysicsEntity,
        >("Biophysics", "natural.biomedical.biophysics", "Fukada & Yasuda (1957); Duck (1990)"),
        descriptor::<
            crate::natural::biomedical::chemistry::ontology::ChemistryCategory,
            crate::natural::biomedical::chemistry::ontology::ChemistryEntity,
        >("Chemistry", "natural.biomedical.chemistry", "citings pending; see domain citings.md"),
        descriptor::<
            crate::natural::biomedical::electrophysiology::ontology::ElectrophysiologyCategory,
            crate::natural::biomedical::electrophysiology::ontology::ElectrophysiologyEntity,
        >("Electrophysiology", "natural.biomedical.electrophysiology", "Neher & Sakmann (1976); Levin (2024)"),
        descriptor::<
            crate::natural::biomedical::hematology::ontology::HematologyCategory,
            crate::natural::biomedical::hematology::ontology::HematologyEntity,
        >("Hematology", "natural.biomedical.hematology", "citings pending; see domain citings.md"),
        descriptor::<
            crate::natural::biomedical::immunology::ontology::ImmunologyCategory,
            crate::natural::biomedical::immunology::ontology::ImmunologyEntity,
        >("Immunology", "natural.biomedical.immunology", "Weinheimer-Haus (2014); Yu (2019)"),
        descriptor::<
            crate::natural::biomedical::mechanobiology::ontology::MechanobiologyCategory,
            crate::natural::biomedical::mechanobiology::ontology::MechanobiologyEntity,
        >("Mechanobiology", "natural.biomedical.mechanobiology", "citings pending; see domain citings.md"),
        descriptor::<
            crate::natural::biomedical::molecular::ontology::MolecularCategory,
            crate::natural::biomedical::molecular::ontology::MolecularEntity,
        >("Molecular Biology", "natural.biomedical.molecular", "Coste (2010); Fukada & Yasuda (1957)"),
        descriptor::<
            crate::natural::biomedical::pathology::ontology::PathologyCategory,
            crate::natural::biomedical::pathology::ontology::PathologyEntity,
        >("Pathology", "natural.biomedical.pathology", "Levin (2014); Chernet & Levin (2013)"),
        descriptor::<
            crate::natural::biomedical::pharmacology::ontology::PharmacologyCategory,
            crate::natural::biomedical::pharmacology::ontology::PharmacologyEntity,
        >("Pharmacology", "natural.biomedical.pharmacology", "Kofman & Levin (2024); Adams & Levin (2013)"),
        descriptor::<
            crate::natural::biomedical::regeneration::ontology::RegenerationCategory,
            crate::natural::biomedical::regeneration::ontology::RegenerationEntity,
        >("Regeneration", "natural.biomedical.regeneration", "Levin (2012); Kumar & Brockes (2012)"),
        // =====================================================================
        // Natural — Hearing
        // =====================================================================
        descriptor::<
            crate::natural::hearing::acoustics::ontology::AcousticsCategory,
            crate::natural::hearing::acoustics::ontology::AcousticEntity,
        >("Hearing Acoustics", "natural.hearing.acoustics", "Kinsler et al. (2000); Pierce (2019)"),
        descriptor::<
            crate::natural::hearing::anatomy::ontology::AnatomyCategory,
            crate::natural::hearing::anatomy::ontology::AuditoryEntity,
        >("Auditory Anatomy", "natural.hearing.anatomy", "Pickles (2012); von Bekesy (1960)"),
        descriptor::<
            crate::natural::hearing::audiology::ontology::AudiologyCategory,
            crate::natural::hearing::audiology::ontology::AudiologyEntity,
        >("Audiology", "natural.hearing.audiology", "Katz et al. (2015); Jerger (1970)"),
        descriptor::<
            crate::natural::hearing::auditory_neuroscience::ontology::NeuroscienceCategory,
            crate::natural::hearing::auditory_neuroscience::ontology::NeuralEntity,
        >("Auditory Neuroscience", "natural.hearing.auditory_neuroscience", "Kandel et al. (2021); Schnupp et al. (2011)"),
        descriptor::<
            crate::natural::hearing::bone_conduction::ontology::BoneConductionCategory,
            crate::natural::hearing::bone_conduction::ontology::BoneCondEntity,
        >("Bone Conduction", "natural.hearing.bone_conduction", "Stenfelt & Goode (2005); Tonndorf (1966)"),
        descriptor::<
            crate::natural::hearing::devices::ontology::DeviceCategory,
            crate::natural::hearing::devices::ontology::DeviceEntity,
        >("Hearing Devices", "natural.hearing.devices", "Dillon (2012); Zeng et al. (2008)"),
        descriptor::<
            crate::natural::hearing::environmental_acoustics::ontology::EnvironmentalAcousticsCategory,
            crate::natural::hearing::environmental_acoustics::ontology::EnvironmentEntity,
        >("Environmental Acoustics", "natural.hearing.environmental_acoustics", "Kuttruff (2009); Sabine (1922)"),
        descriptor::<
            crate::natural::hearing::music_perception::ontology::MusicPerceptionCategory,
            crate::natural::hearing::music_perception::ontology::MusicEntity,
        >("Music Perception", "natural.hearing.music_perception", "Helmholtz (1863); Krumhansl (1990)"),
        descriptor::<
            crate::natural::hearing::pathology::ontology::PathologyCategory,
            crate::natural::hearing::pathology::ontology::PathologyEntity,
        >("Hearing Pathology", "natural.hearing.pathology", "Moller (2006); Gates & Mills (2005)"),
        descriptor::<
            crate::natural::hearing::psychoacoustics::ontology::PsychoacousticsCategory,
            crate::natural::hearing::psychoacoustics::ontology::PsychoacousticEntity,
        >("Psychoacoustics", "natural.hearing.psychoacoustics", "Fletcher & Munson (1933); Zwicker & Fastl (2007)"),
        descriptor::<
            crate::natural::hearing::signal_processing::ontology::SignalProcessingCategory,
            crate::natural::hearing::signal_processing::ontology::SignalEntity,
        >("Hearing Signal Processing", "natural.hearing.signal_processing", "Oppenheim & Schafer (2010); Cooley & Tukey (1965)"),
        descriptor::<
            crate::natural::hearing::speech::ontology::SpeechCategory,
            crate::natural::hearing::speech::ontology::SpeechEntity,
        >("Speech", "natural.hearing.speech", "Fant (1960); Peterson & Barney (1952)"),
        descriptor::<
            crate::natural::hearing::transduction::ontology::TransductionCategory,
            crate::natural::hearing::transduction::ontology::TransductionEntity,
        >("Mechanotransduction", "natural.hearing.transduction", "Hudspeth (1989); Fettiplace & Kim (2014)"),
        descriptor::<
            crate::natural::hearing::vestibular::ontology::VestibularCategory,
            crate::natural::hearing::vestibular::ontology::VestibularEntity,
        >("Vestibular System", "natural.hearing.vestibular", "Goldberg et al. (2012); Angelaki & Cullen (2008)"),
        // =====================================================================
        // Social — Games
        // =====================================================================
        descriptor::<
            crate::social::games::chess::ontology::ChessCategory,
            crate::social::games::chess::square::Square,
        >("Chess", "social.games.chess", "FIDE Laws of Chess; Shannon (1950)"),
        descriptor::<
            crate::social::games::rubik::ontology::RubikCategory,
            crate::social::games::rubik::Face,
        >("Rubik's Cube", "social.games.rubik", "Joyner (2008); Singmaster (1981)"),
        // =====================================================================
        // Social — Judicial
        // =====================================================================
        descriptor::<
            crate::social::judicial::ontology::CaseLifecycleCategory,
            crate::social::judicial::PhaseTag,
        >("Judicial", "social.judicial", "Hart (1961); Sartor (2005)"),
        // =====================================================================
        // Social — Military
        // =====================================================================
        descriptor::<
            crate::social::military::electronic_warfare::ontology::EwCategory,
            crate::social::military::electronic_warfare::ontology::EwObservable,
        >("Electronic Warfare", "social.military.electronic_warfare", "Poisel (2012); JP 3-13.1"),
        // =====================================================================
        // Social — Software
        // =====================================================================
        descriptor::<
            crate::social::software::protocols::http::ontology::HttpMethodCategory,
            crate::social::software::protocols::http::Method,
        >("HTTP", "social.software.protocols.http", "RFC 9110 (2022); Fielding (2000)"),
        descriptor::<
            crate::social::software::markup::ontology::MarkupCategory,
            crate::social::software::markup::ontology::NodeKind,
        >("Markup", "social.software.markup", "Coombs et al. (1987); Goldfarb (1990)"),
        descriptor::<
            crate::social::software::markup::xml::ontology::XmlCategory,
            crate::social::software::markup::xml::ontology::XmlNodeKind,
        >("XML", "social.software.markup.xml", "W3C XML 1.0 (2008)"),
        descriptor::<
            crate::social::software::markup::xml::rdf::ontology::RdfCategory,
            crate::social::software::markup::xml::rdf::ontology::RdfNodeKind,
        >("RDF", "social.software.markup.xml.rdf", "W3C RDF 1.1 (2014)"),
        descriptor::<
            crate::social::software::markup::xml::owl::ontology::OwlCategory,
            crate::social::software::markup::xml::owl::ontology::OwlConcept,
        >("OWL", "social.software.markup.xml.owl", "W3C OWL 2 (2012); Baader et al. (2003)"),
    ]
}

fn descriptor<C, E>(
    name: &'static str,
    domain: &'static str,
    source: &'static str,
) -> VocabularyDescriptor
where
    C: Category + pr4xis::ontology::upper::classify::Classified,
    E: Entity,
{
    VocabularyDescriptor {
        name,
        domain,
        being: C::being(),
        reason: C::classification_reason(),
        source,
        concepts: E::variants().len(),
        morphisms: C::morphisms().len(),
    }
}
