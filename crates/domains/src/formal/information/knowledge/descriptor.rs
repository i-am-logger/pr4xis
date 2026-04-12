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

/// Enumerate all ontologies loaded in the praxis knowledge base.
pub fn describe_knowledge_base() -> Vec<VocabularyDescriptor> {
    vec![
        // Science — Cognition
        descriptor::<
            crate::cognitive::cognition::distinction::DistinctionCategory,
            crate::cognitive::cognition::distinction::DistinctionElement,
        >(
            "Distinction",
            "science.cognition",
            "Spencer-Brown, Laws of Form (1969)",
        ),
        descriptor::<
            crate::cognitive::cognition::epistemics::EpistemicCategory,
            crate::cognitive::cognition::epistemics::EpistemicState,
        >(
            "Epistemics",
            "science.cognition",
            "Rumsfeld matrix; second-order epistemology",
        ),
        descriptor::<
            crate::cognitive::cognition::metacognition::MetaCognitionCategory,
            crate::cognitive::cognition::metacognition::MetaConcept,
        >(
            "Metacognition",
            "science.cognition",
            "von Foerster (1981); Olivares-Alarcos MOI (2023)",
        ),
        descriptor::<
            crate::cognitive::cognition::self_model::SelfModelCategory,
            crate::cognitive::cognition::self_model::SelfModelConcept,
        >(
            "Self-Model",
            "science.cognition",
            "von Foerster Eigenform (1981); IEEE AuR (2021); MAPE-K (2003)",
        ),
        // Science — Information
        descriptor::<
            crate::formal::information::ontology::InfoCategory,
            crate::formal::information::ontology::InfoUnit,
        >(
            "Information Theory",
            "science.information",
            "Shannon (1948)",
        ),
        descriptor::<
            crate::formal::information::communication::ontology::CommunicationCategory,
            crate::formal::information::communication::ontology::CommunicationConcept,
        >(
            "Communication",
            "science.information",
            "Shannon (1948); Jakobson (1960)",
        ),
        descriptor::<
            crate::formal::information::dialogue::ontology::DialogueCategory,
            crate::formal::information::dialogue::ontology::DialogueConcept,
        >(
            "Dialogue",
            "science.information",
            "Kamp DRT (1981); Grosz Centering (1995)",
        ),
        descriptor::<
            crate::formal::information::events::ontology::EventCategory,
            crate::formal::information::events::ontology::EventConcept,
        >("Events", "science.information", "Event-driven architecture"),
        descriptor::<
            crate::formal::information::concurrency::ontology::ConcurrencyCategory,
            crate::formal::information::concurrency::ontology::ConcurrencyConcept,
        >(
            "Concurrency",
            "science.information",
            "Hoare CSP (1978); Hewitt Actor (1973)",
        ),
        descriptor::<
            crate::formal::information::provenance::ontology::ProvenanceCategory,
            crate::formal::information::provenance::ontology::ProvenanceConcept,
        >("Provenance", "science.information", "W3C PROV-O (2013)"),
        descriptor::<
            crate::formal::information::knowledge::ontology::KnowledgeBaseCategory,
            crate::formal::information::knowledge::ontology::KnowledgeConcept,
        >(
            "Knowledge Base",
            "science.information",
            "W3C VoID (2011); DCAT (2024); Herre & Loebe FOIS (2005)",
        ),
        // Science — Systems
        descriptor::<
            crate::formal::systems::ontology::SystemsCategory,
            crate::formal::systems::ontology::SystemConcept,
        >(
            "Systems",
            "science.systems",
            "Bertalanffy (1968); Ashby (1956)",
        ),
        descriptor::<
            crate::formal::systems::control::ControlCategory,
            crate::formal::systems::control::ControlConcept,
        >(
            "Control Systems",
            "science.systems",
            "Wiener (1948); Conant-Ashby (1970)",
        ),
        // Science — Physics, Math, Colors
        descriptor::<
            crate::natural::physics::ontology::PhysicsCategory,
            crate::natural::physics::ontology::PhysicsLaw,
        >(
            "Physics",
            "science.physics",
            "Classical mechanics; electromagnetism",
        ),
        descriptor::<
            crate::formal::math::ontology::NumberHierarchy,
            crate::formal::math::ontology::MathDomain,
        >("Mathematics", "science.math", "Number hierarchy"),
        descriptor::<
            crate::natural::colors::ontology::ColorCategory,
            crate::natural::colors::ontology::PrimaryColor,
        >(
            "Color",
            "science.colors",
            "CIE colorimetry; perceptual color theory",
        ),
        // Science — Linguistics
        descriptor::<
            crate::cognitive::linguistics::lexicon::ontology::LexicalCategory,
            crate::cognitive::linguistics::lexicon::pos::PosTag,
        >(
            "Lexicon",
            "science.linguistics",
            "OLiA (2015); OntoLex-Lemon (2019)",
        ),
        descriptor::<
            crate::cognitive::linguistics::morphology::tense::TenseCategory,
            crate::cognitive::linguistics::morphology::tense::TenseAspect,
        >(
            "Tense & Aspect",
            "science.linguistics",
            "Reichenbach (1947); Comrie (1976)",
        ),
        descriptor::<
            crate::cognitive::linguistics::orthography::distance::SpellingErrorCategory,
            crate::cognitive::linguistics::orthography::distance::SpellingErrorConcept,
        >(
            "Spelling Errors",
            "science.linguistics",
            "Damerau (1964); Brill & Moore (2000)",
        ),
        descriptor::<
            crate::cognitive::linguistics::orthography::channel::ChannelCategory,
            crate::cognitive::linguistics::orthography::channel::ChannelConcept,
        >(
            "Noisy Channel",
            "science.linguistics",
            "Shannon (1948); Kernighan et al. (1990)",
        ),
        descriptor::<
            crate::cognitive::linguistics::pragmatics::reference::ReferenceCategory,
            crate::cognitive::linguistics::pragmatics::reference::ReferenceConcept,
        >(
            "Discourse Reference",
            "science.linguistics",
            "Kamp DRT (1981); Grosz Centering (1995)",
        ),
        // Technology — Games
        descriptor::<
            crate::social::games::chess::ontology::ChessCategory,
            crate::social::games::chess::square::Square,
        >("Chess", "technology.games", "FIDE Laws of Chess"),
        descriptor::<
            crate::social::games::rubik::ontology::RubikCategory,
            crate::social::games::rubik::Face,
        >(
            "Rubik's Cube",
            "technology.games",
            "Group theory; Kociemba algorithm",
        ),
        // Technology — Software
        descriptor::<
            crate::social::software::protocols::http::ontology::HttpMethodCategory,
            crate::social::software::protocols::http::Method,
        >("HTTP", "technology.software", "IETF RFC 9110 (2022)"),
        descriptor::<
            crate::social::software::markup::ontology::MarkupCategory,
            crate::social::software::markup::ontology::NodeKind,
        >("Markup", "technology.software", "Document markup theory"),
        descriptor::<
            crate::social::software::markup::xml::ontology::XmlCategory,
            crate::social::software::markup::xml::ontology::XmlNodeKind,
        >("XML", "technology.software", "W3C XML 1.0 (2008)"),
        descriptor::<
            crate::social::software::markup::xml::rdf::ontology::RdfCategory,
            crate::social::software::markup::xml::rdf::ontology::RdfNodeKind,
        >("RDF", "technology.software", "W3C RDF 1.1 (2014)"),
        descriptor::<
            crate::social::software::markup::xml::owl::ontology::OwlCategory,
            crate::social::software::markup::xml::owl::ontology::OwlConcept,
        >("OWL", "technology.software", "W3C OWL 2 (2012)"),
        // Technology — Hardware
        descriptor::<
            crate::applied::hardware::traffic::ontology::TrafficCategory,
            crate::applied::hardware::traffic::ontology::TrafficDirection,
        >(
            "Traffic",
            "technology.hardware",
            "Traffic engineering standards",
        ),
        descriptor::<
            crate::applied::hardware::elevator::ontology::ElevatorCategory,
            crate::applied::hardware::elevator::ontology::Floor,
        >(
            "Elevator",
            "technology.hardware",
            "Elevator engineering specifications",
        ),
        // Governance
        descriptor::<
            crate::social::judicial::ontology::CaseLifecycleCategory,
            crate::social::judicial::PhaseTag,
        >("Judicial", "governance.judicial", "Legal process ontology"),
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
