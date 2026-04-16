use pr4xis::ontology::OntologyDescriptor;
use pr4xis::ontology::upper::being::Being;

pub fn describe_knowledge_base() -> Vec<OntologyDescriptor> {
    static CACHE: std::sync::OnceLock<Vec<OntologyDescriptor>> = std::sync::OnceLock::new();
    CACHE.get_or_init(build_descriptors).clone()
}

fn build_descriptors() -> Vec<OntologyDescriptor> {
    vec![
        // =================================================================
        // Cognitive — Cognition (define_ontology!)
        // =================================================================
        crate::cognitive::cognition::distinction::DistinctionOntology::descriptor(),
        crate::cognitive::cognition::epistemics::EpistemicOntology::descriptor(),
        crate::cognitive::cognition::metacognition::MetaCognitionOntology::descriptor(),
        crate::cognitive::cognition::self_model::SelfModelOntology::descriptor(),
        // =================================================================
        // Cognitive — Linguistics (mixed: define_ontology! + manual)
        // =================================================================
        manual::<
            crate::cognitive::linguistics::lexicon::ontology::LexicalCategory,
            crate::cognitive::linguistics::lexicon::pos::PosTag,
        >("Lexicon", "pr4xis_domains::cognitive::linguistics::lexicon", "Lambek (1958); Chiarcos & Sukhareva OLiA (2015)", Being::SocialObject),
        manual::<
            crate::cognitive::linguistics::morphology::tense::TenseCategory,
            crate::cognitive::linguistics::morphology::tense::TenseAspect,
        >("Tense & Aspect", "pr4xis_domains::cognitive::linguistics::morphology::tense", "Reichenbach (1947); Comrie (1976)", Being::AbstractObject),
        manual::<
            crate::cognitive::linguistics::orthography::distance::SpellingErrorCategory,
            crate::cognitive::linguistics::orthography::distance::SpellingErrorConcept,
        >("Spelling Errors", "pr4xis_domains::cognitive::linguistics::orthography::distance", "Damerau (1964); Brill & Moore (2000)", Being::Quality),
        crate::cognitive::linguistics::orthography::channel::ChannelOntology::descriptor(),
        crate::cognitive::linguistics::pragmatics::reference::ReferenceOntology::descriptor(),
        crate::cognitive::linguistics::pragmatics::generation::ProductionOntology::descriptor(),
        crate::cognitive::linguistics::pragmatics::nlg::NlgOntology::descriptor(),
        crate::cognitive::linguistics::pragmatics::response::ResponseOntology::descriptor(),
        // =================================================================
        // Formal — Mathematics (macro-generated)
        // =================================================================
        crate::formal::math::ontology::MathOntology::descriptor(),
        crate::formal::math::geometry::ontology::EuclideanGeometryOntology::descriptor(),
        crate::formal::math::linear_algebra::ontology::LinearAlgebraOntology::descriptor(),
        crate::formal::math::probability::ontology::ProbabilityOntology::descriptor(),
        crate::formal::math::quantity::ontology::QuantityOntology::descriptor(),
        crate::formal::math::rotation::ontology::RotationOntology::descriptor(),
        crate::formal::math::signal_processing::ontology::SignalProcessingOntology::descriptor(),
        crate::formal::math::statistics::ontology::StatisticsOntology::descriptor(),
        crate::formal::math::temporal::ontology::TimeOntology::descriptor(),
        crate::formal::math::control_theory::ontology::ControlTheoryOntology::descriptor(),
        // =================================================================
        // Formal — Meta (macro-generated)
        // =================================================================
        crate::formal::meta::artifact_identity::ontology::ArtifactIdentityOntology::descriptor(),
        crate::formal::meta::staging::ontology::StagingOntology::descriptor(),
        crate::formal::meta::ontology_diagnostics::ontology::MetaOntology::descriptor(),
        // =================================================================
        // Formal — Information (macro-generated)
        // =================================================================
        crate::formal::information::ontology::InformationOntology::descriptor(),
        crate::formal::information::communication::ontology::CommunicationOntology::descriptor(),
        crate::formal::information::dialogue::ontology::DialogueOntology::descriptor(),
        crate::formal::information::events::ontology::EventOntology::descriptor(),
        crate::formal::information::concurrency::ontology::ConcurrencyOntology::descriptor(),
        crate::formal::information::provenance::ontology::ProvenanceOntology::descriptor(),
        crate::formal::information::knowledge::ontology::KnowledgeOntology::descriptor(),
        crate::formal::information::diagnostics::ontology::DiagnosticOntology::descriptor(),
        crate::formal::information::measurement::ontology::MeasurementOntology::descriptor(),
        crate::formal::information::schema::ontology::SchemaOntology::descriptor(),
        crate::formal::information::storage::ontology::StorageOntology::descriptor(),
        // =================================================================
        // Formal — Systems (macro-generated)
        // =================================================================
        crate::formal::systems::ontology::SystemsOntology::descriptor(),
        crate::formal::systems::control::ControlOntology::descriptor(),
        // =================================================================
        // Formal — Other (macro-generated)
        // =================================================================
        crate::formal::analytical_methods::ontology::AnalyticalMethodsOntology::descriptor(),
        crate::formal::derivation::ontology::DerivationOntology::descriptor(),
        crate::formal::optimization::ontology::OptimizationOntology::descriptor(),
        crate::formal::recommendation::ontology::RecommendationOntology::descriptor(),
        // =================================================================
        // Applied — Data Provisioning (macro-generated)
        // =================================================================
        crate::applied::data_provisioning::ontology::DataProvisioningOntology::descriptor(),
        // =================================================================
        // Applied — HMI (manual)
        // =================================================================
        manual::<
            crate::applied::hmi::theming::ontology::ThemingCategory,
            crate::applied::hmi::theming::base16::ColorSlot,
        >("Theming", "pr4xis_domains::applied::hmi::theming", "Base16 styling spec; WCAG 2.1", Being::Quality),
        // =================================================================
        // Applied — Navigation (macro-generated)
        // =================================================================
        crate::applied::navigation::ahrs::ontology::AhrsOntology::descriptor(),
        crate::applied::navigation::celestial::ontology::CelestialOntology::descriptor(),
        crate::applied::navigation::gnss::ontology::GnssOntology::descriptor(),
        crate::applied::navigation::imu::ontology::ImuOntology::descriptor(),
        crate::applied::navigation::ins_gnss::ontology::InsGnssOntology::descriptor(),
        crate::applied::navigation::odometry::ontology::OdometryOntology::descriptor(),
        // =================================================================
        // Applied — Sensor Fusion (macro-generated)
        // =================================================================
        crate::applied::sensor_fusion::fusion::ontology::FusionOntology::descriptor(),
        crate::applied::sensor_fusion::observation::ontology::ObservationOntology::descriptor(),
        crate::applied::sensor_fusion::sensor::ontology::SensorOntology::descriptor(),
        crate::applied::sensor_fusion::state::ontology::StateEstimationOntology::descriptor(),
        crate::applied::sensor_fusion::time::ontology::SensorTimeOntology::descriptor(),
        // =================================================================
        // Applied — Space (macro-generated)
        // =================================================================
        crate::applied::space::attitude::ontology::AttitudeOntology::descriptor(),
        crate::applied::space::orbit::ontology::OrbitOntology::descriptor(),
        // =================================================================
        // Applied — Tracking (macro-generated)
        // =================================================================
        crate::applied::tracking::single_target::ontology::SingleTargetOntology::descriptor(),
        // =================================================================
        // Applied — Underwater (macro-generated)
        // =================================================================
        crate::applied::underwater::acoustic::ontology::AcousticOntology::descriptor(),
        crate::applied::underwater::auv::ontology::AuvOntology::descriptor(),
        // =================================================================
        // Applied — Industrial (macro-generated)
        // =================================================================
        crate::applied::industrial::process::ontology::ProcessOntology::descriptor(),
        crate::applied::industrial::structural::ontology::StructuralOntology::descriptor(),
        // =================================================================
        // Applied — Localization (macro-generated)
        // =================================================================
        crate::applied::localization::terrain::ontology::TerrainOntology::descriptor(),
        // =================================================================
        // Applied — Perception (macro-generated)
        // =================================================================
        crate::applied::perception::occupancy::ontology::OccupancyOntology::descriptor(),
        // =================================================================
        // Applied — Hardware (mixed: define_ontology! + manual)
        // =================================================================
        crate::applied::hardware::traffic::ontology::TrafficOntology::descriptor(),
        manual::<
            crate::applied::hardware::elevator::ontology::ElevatorCategory,
            crate::applied::hardware::elevator::ontology::Floor,
        >("Elevator", "pr4xis_domains::applied::hardware::elevator", "Mandel (1989); Barney & Dos Santos (1985)", Being::SocialObject),
        // =================================================================
        // Natural — Physics (macro-generated)
        // =================================================================
        crate::natural::physics::ontology::PhysicsOntology::descriptor(),
        crate::natural::physics::kinematics::ontology::KinematicsOntology::descriptor(),
        // =================================================================
        // Natural — Geodesy (macro-generated)
        // =================================================================
        crate::natural::geodesy::ontology::GeodesyOntology::descriptor(),
        // =================================================================
        // Natural — Colors (macro-generated)
        // =================================================================
        crate::natural::colors::ontology::ColorOntology::descriptor(),
        // =================================================================
        // Natural — Biomedical (macro-generated)
        // =================================================================
        crate::natural::biomedical::acoustics::ontology::AcousticsOntologyMeta::descriptor(),
        crate::natural::biomedical::biochemistry::ontology::BiochemistryOntologyMeta::descriptor(),
        crate::natural::biomedical::bioelectricity::ontology::BioelectricOntologyMeta::descriptor(),
        crate::natural::biomedical::bioelectricity::morphospace::MorphospaceOntologyMeta::descriptor(),
        crate::natural::biomedical::biology::ontology::BiologyOntologyMeta::descriptor(),
        crate::natural::biomedical::biophysics::ontology::BiophysicsOntologyMeta::descriptor(),
        crate::natural::biomedical::chemistry::ontology::ChemistryOntologyMeta::descriptor(),
        crate::natural::biomedical::electrophysiology::ontology::ElectrophysiologyOntologyMeta::descriptor(),
        crate::natural::biomedical::hematology::ontology::HematologyOntologyMeta::descriptor(),
        crate::natural::biomedical::immunology::ontology::ImmunologyOntologyMeta::descriptor(),
        crate::natural::biomedical::mechanobiology::ontology::MechanobiologyOntologyMeta::descriptor(),
        crate::natural::biomedical::molecular::ontology::MolecularOntologyMeta::descriptor(),
        crate::natural::biomedical::pathology::ontology::PathologyOntologyMeta::descriptor(),
        crate::natural::biomedical::pharmacology::ontology::PharmacologyOntologyMeta::descriptor(),
        crate::natural::biomedical::regeneration::ontology::RegenerationOntologyMeta::descriptor(),
        // =================================================================
        // Natural — Hearing (macro-generated)
        // =================================================================
        crate::natural::hearing::acoustics::ontology::AcousticsOntology::descriptor(),
        crate::natural::hearing::anatomy::ontology::AnatomyOntology::descriptor(),
        crate::natural::hearing::audiology::ontology::AudiologyOntology::descriptor(),
        crate::natural::hearing::auditory_neuroscience::ontology::NeuroscienceOntology::descriptor(),
        crate::natural::hearing::bone_conduction::ontology::BoneConductionOntology::descriptor(),
        crate::natural::hearing::devices::ontology::DeviceOntology::descriptor(),
        crate::natural::hearing::environmental_acoustics::ontology::EnvironmentalAcousticsOntology::descriptor(),
        crate::natural::hearing::music_perception::ontology::MusicPerceptionOntology::descriptor(),
        crate::natural::hearing::pathology::ontology::PathologyOntology::descriptor(),
        crate::natural::hearing::psychoacoustics::ontology::PsychoacousticsOntology::descriptor(),
        crate::natural::hearing::signal_processing::ontology::SignalProcessingOntology::descriptor(),
        crate::natural::hearing::speech::ontology::SpeechOntology::descriptor(),
        crate::natural::hearing::transduction::ontology::TransductionOntology::descriptor(),
        crate::natural::hearing::vestibular::ontology::VestibularOntology::descriptor(),
        // =================================================================
        // Social — Games (manual)
        // =================================================================
        manual::<
            crate::social::games::chess::ontology::ChessCategory,
            crate::social::games::chess::square::Square,
        >("Chess", "pr4xis_domains::social::games::chess", "FIDE Laws of Chess; Shannon (1950)", Being::SocialObject),
        manual::<
            crate::social::games::rubik::ontology::RubikCategory,
            crate::social::games::rubik::Face,
        >("Rubik's Cube", "pr4xis_domains::social::games::rubik", "Joyner (2008); Singmaster (1981)", Being::SocialObject),
        // =================================================================
        // Social — Judicial (manual)
        // =================================================================
        manual::<
            crate::social::judicial::ontology::CaseLifecycleCategory,
            crate::social::judicial::PhaseTag,
        >("Judicial", "pr4xis_domains::social::judicial", "Hart (1961); Sartor (2005)", Being::Process),
        // =================================================================
        // Social — Military (macro-generated)
        // =================================================================
        crate::social::military::electronic_warfare::ontology::EwOntology::descriptor(),
        // =================================================================
        // Social — Software (manual)
        // =================================================================
        manual::<
            crate::social::software::protocols::http::ontology::HttpMethodCategory,
            crate::social::software::protocols::http::Method,
        >("HTTP", "pr4xis_domains::social::software::protocols::http", "RFC 9110 (2022); Fielding (2000)", Being::SocialObject),
        manual::<
            crate::social::software::markup::ontology::MarkupCategory,
            crate::social::software::markup::ontology::NodeKind,
        >("Markup", "pr4xis_domains::social::software::markup", "Coombs et al. (1987); Goldfarb (1990)", Being::SocialObject),
        manual::<
            crate::social::software::markup::xml::ontology::XmlCategory,
            crate::social::software::markup::xml::ontology::XmlNodeKind,
        >("XML", "pr4xis_domains::social::software::markup::xml", "W3C XML 1.0 (2008)", Being::SocialObject),
        manual::<
            crate::social::software::markup::xml::rdf::ontology::RdfCategory,
            crate::social::software::markup::xml::rdf::ontology::RdfNodeKind,
        >("RDF", "pr4xis_domains::social::software::markup::xml::rdf", "W3C RDF 1.1 (2014)", Being::SocialObject),
        manual::<
            crate::social::software::markup::xml::owl::ontology::OwlCategory,
            crate::social::software::markup::xml::owl::ontology::OwlConcept,
        >("OWL", "pr4xis_domains::social::software::markup::xml::owl", "W3C OWL 2 (2012); Baader et al. (2003)", Being::SocialObject),
    ]
}

fn manual<C: pr4xis::category::Category, E: pr4xis::category::entity::Entity>(
    name: &'static str,
    module_path: &'static str,
    source: &'static str,
    being: Being,
) -> OntologyDescriptor {
    OntologyDescriptor::manual::<C, E>(name, module_path, source, Some(being))
}
