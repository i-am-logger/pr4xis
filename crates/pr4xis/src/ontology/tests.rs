use super::property::Quality;
use crate::category::{Category, Entity, Relationship};
use crate::logic::Axiom;
use crate::ontology::Ontology;
use proptest::prelude::*;

// =============================================================================
// Example: Traffic Light Ontology
//
// Individuals: Red, Yellow, Green
// Relations: transitions between lights (including composites for closure)
// Qualities: duration (how long each light stays on)
// Axioms: no dead states, green is the longest phase
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Light {
    Red,
    Yellow,
    Green,
}

impl Entity for Light {
    fn variants() -> Vec<Self> {
        vec![Light::Red, Light::Yellow, Light::Green]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LightTransition {
    Identity(Light),
    RedToGreen,
    GreenToYellow,
    YellowToRed,
    RedToYellow,
    GreenToRed,
    YellowToGreen,
}

impl Relationship for LightTransition {
    type Object = Light;

    fn source(&self) -> Light {
        match self {
            LightTransition::Identity(l) => *l,
            LightTransition::RedToGreen | LightTransition::RedToYellow => Light::Red,
            LightTransition::GreenToYellow | LightTransition::GreenToRed => Light::Green,
            LightTransition::YellowToRed | LightTransition::YellowToGreen => Light::Yellow,
        }
    }

    fn target(&self) -> Light {
        match self {
            LightTransition::Identity(l) => *l,
            LightTransition::RedToGreen | LightTransition::YellowToGreen => Light::Green,
            LightTransition::GreenToYellow | LightTransition::RedToYellow => Light::Yellow,
            LightTransition::YellowToRed | LightTransition::GreenToRed => Light::Red,
        }
    }
}

struct TrafficLightCat;

impl Category for TrafficLightCat {
    type Object = Light;
    type Morphism = LightTransition;

    fn identity(obj: &Light) -> LightTransition {
        LightTransition::Identity(*obj)
    }

    fn compose(f: &LightTransition, g: &LightTransition) -> Option<LightTransition> {
        if f.target() != g.source() {
            return None;
        }
        if let LightTransition::Identity(_) = f {
            return Some(g.clone());
        }
        if let LightTransition::Identity(_) = g {
            return Some(f.clone());
        }
        Some(match (f.source(), g.target()) {
            (s, t) if s == t => LightTransition::Identity(s),
            (Light::Red, Light::Yellow) => LightTransition::RedToYellow,
            (Light::Red, Light::Green) => LightTransition::RedToGreen,
            (Light::Green, Light::Red) => LightTransition::GreenToRed,
            (Light::Green, Light::Yellow) => LightTransition::GreenToYellow,
            (Light::Yellow, Light::Green) => LightTransition::YellowToGreen,
            (Light::Yellow, Light::Red) => LightTransition::YellowToRed,
            _ => return None,
        })
    }

    fn morphisms() -> Vec<LightTransition> {
        vec![
            LightTransition::Identity(Light::Red),
            LightTransition::Identity(Light::Yellow),
            LightTransition::Identity(Light::Green),
            LightTransition::RedToGreen,
            LightTransition::GreenToYellow,
            LightTransition::YellowToRed,
            LightTransition::RedToYellow,
            LightTransition::GreenToRed,
            LightTransition::YellowToGreen,
        ]
    }
}

// --- Quality: duration of each light phase ---

#[derive(Debug, Clone)]
struct Duration;

impl Quality for Duration {
    type Individual = Light;
    type Value = u32; // seconds

    fn get(&self, individual: &Light) -> Option<u32> {
        match individual {
            Light::Red => Some(30),
            Light::Yellow => Some(5),
            Light::Green => Some(45),
        }
    }
}

// --- Axiom: green must be the longest phase ---

struct GreenIsLongest;

impl Axiom for GreenIsLongest {
    fn description(&self) -> &str {
        "green phase must be the longest"
    }

    fn holds(&self) -> bool {
        let dur = Duration;
        let green_dur = dur.get(&Light::Green).unwrap_or(0);
        Light::variants()
            .iter()
            .all(|l| dur.get(l).unwrap_or(0) <= green_dur)
    }
}

// --- Axiom: no dead states ---

struct NoDeadStates;

impl Axiom for NoDeadStates {
    fn description(&self) -> &str {
        "every light has at least one outgoing transition"
    }

    fn holds(&self) -> bool {
        Light::variants()
            .iter()
            .all(|obj| !TrafficLightCat::morphisms_from(obj).is_empty())
    }
}

// --- Ontology: tie it all together ---

struct TrafficLightOntology;

impl Ontology for TrafficLightOntology {
    type Cat = TrafficLightCat;
    type Qual = Duration;

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(GreenIsLongest), Box::new(NoDeadStates)]
    }
}

// =============================================================================
// Proptest strategies
// =============================================================================

fn arb_light() -> impl Strategy<Value = Light> {
    prop_oneof![Just(Light::Red), Just(Light::Yellow), Just(Light::Green),]
}

fn arb_transition() -> impl Strategy<Value = LightTransition> {
    prop_oneof![
        arb_light().prop_map(LightTransition::Identity),
        Just(LightTransition::RedToGreen),
        Just(LightTransition::GreenToYellow),
        Just(LightTransition::YellowToRed),
        Just(LightTransition::RedToYellow),
        Just(LightTransition::GreenToRed),
        Just(LightTransition::YellowToGreen),
    ]
}

// =============================================================================
// Property-based tests — Ontology invariants
// =============================================================================

proptest! {
    /// Ontology validation succeeds structurally
    #[test]
    fn prop_ontology_validates(_obj in arb_light()) {
        prop_assert!(TrafficLightOntology::validate().is_ok());
    }

    /// Every individual has a quality value (total quality)
    #[test]
    fn prop_duration_is_total(individual in arb_light()) {
        let dur = Duration;
        prop_assert!(dur.get(&individual).is_some());
    }

    /// Duration is always positive
    #[test]
    fn prop_duration_is_positive(individual in arb_light()) {
        let dur = Duration;
        let val = dur.get(&individual).unwrap();
        prop_assert!(val > 0);
    }

    /// Green is always >= any other light's duration
    #[test]
    fn prop_green_is_longest(individual in arb_light()) {
        let dur = Duration;
        let green = dur.get(&Light::Green).unwrap();
        let other = dur.get(&individual).unwrap();
        prop_assert!(green >= other);
    }

    /// Every individual has outgoing morphisms
    #[test]
    fn prop_no_dead_states(individual in arb_light()) {
        let outgoing = TrafficLightCat::morphisms_from(&individual);
        prop_assert!(!outgoing.is_empty());
    }

    /// Every individual has incoming morphisms
    #[test]
    fn prop_no_orphan_states(individual in arb_light()) {
        let incoming = TrafficLightCat::morphisms_to(&individual);
        prop_assert!(!incoming.is_empty());
    }

    /// Qualities are deterministic
    #[test]
    fn prop_quality_deterministic(individual in arb_light()) {
        let dur = Duration;
        prop_assert_eq!(dur.get(&individual), dur.get(&individual));
    }

    /// individuals_with returns all when quality is total
    #[test]
    fn prop_individuals_with_complete(_individual in arb_light()) {
        let dur = Duration;
        prop_assert_eq!(dur.individuals_with().len(), Light::variants().len());
    }

    /// All axioms hold for any context
    #[test]
    fn prop_all_axioms_hold(_individual in arb_light()) {
        for axiom in TrafficLightOntology::axioms() {
            prop_assert!(axiom.holds(), "Axiom failed: {}", axiom.description());
        }
    }
}

// =============================================================================
// Property-based tests — Category laws via Ontology
// =============================================================================

proptest! {
    /// Left identity
    #[test]
    fn prop_left_identity(m in arb_transition()) {
        let id = TrafficLightCat::identity(&m.source());
        prop_assert_eq!(TrafficLightCat::compose(&id, &m), Some(m));
    }

    /// Right identity
    #[test]
    fn prop_right_identity(m in arb_transition()) {
        let id = TrafficLightCat::identity(&m.target());
        prop_assert_eq!(TrafficLightCat::compose(&m, &id), Some(m));
    }

    /// Associativity
    #[test]
    fn prop_associativity(f in arb_transition(), g in arb_transition(), h in arb_transition()) {
        let fg = TrafficLightCat::compose(&f, &g);
        let gh = TrafficLightCat::compose(&g, &h);
        let left = fg.as_ref().and_then(|fg| TrafficLightCat::compose(fg, &h));
        let right = gh.as_ref().and_then(|gh| TrafficLightCat::compose(&f, gh));
        prop_assert_eq!(left, right);
    }

    /// Closure: composable pairs always produce Some
    #[test]
    fn prop_closure(f in arb_transition(), g in arb_transition()) {
        if f.target() == g.source() {
            prop_assert!(TrafficLightCat::compose(&f, &g).is_some());
        }
    }

    /// Morphism endpoints are valid
    #[test]
    fn prop_morphism_endpoints_valid(m in arb_transition()) {
        let variants = Light::variants();
        prop_assert!(variants.contains(&m.source()));
        prop_assert!(variants.contains(&m.target()));
    }

    /// Incompatible composition returns None
    #[test]
    fn prop_type_safety(f in arb_transition(), g in arb_transition()) {
        if f.target() != g.source() {
            prop_assert_eq!(TrafficLightCat::compose(&f, &g), None);
        }
    }
}

// =============================================================================
// Exhaustive tests
// =============================================================================

#[test]
fn test_ontology_validates() {
    TrafficLightOntology::validate().unwrap();
}

#[test]
fn test_ontology_check() {
    super::validate::check_ontology::<TrafficLightOntology>().unwrap();
}

#[test]
fn test_quality_get() {
    let dur = Duration;
    assert_eq!(dur.get(&Light::Red), Some(30));
    assert_eq!(dur.get(&Light::Yellow), Some(5));
    assert_eq!(dur.get(&Light::Green), Some(45));
}

#[test]
fn test_quality_individuals_with() {
    let dur = Duration;
    assert_eq!(dur.individuals_with().len(), 3);
}

#[test]
fn test_axiom_green_is_longest() {
    assert!(GreenIsLongest.holds());
}

#[test]
fn test_axiom_no_dead_states() {
    assert!(NoDeadStates.holds());
}

// =============================================================================
// define_ontology! macro test
// =============================================================================

mod ontology_macro_test {
    use crate::category::Entity;
    use crate::category::validate::check_category_laws;
    use crate::logic::Axiom;
    use crate::ontology::reasoning::mereology::{self, MereologyDef};
    use crate::ontology::reasoning::opposition::OppositionDef;
    use crate::ontology::reasoning::taxonomy::{self, TaxonomyDef};

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
    pub enum Animal {
        Dog,
        Cat,
        Mammal,
        Pet,
        Tail,
        Fur,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
    pub enum AnimalEvent {
        Birth,
        Growth,
        Death,
    }

    // New ontological style: concepts, is_a, has_a, causes, opposes
    define_ontology! {
        /// Test animal ontology.
        pub AnimalOntology for AnimalCategory {
            concepts: Animal,
            relation: AnimalRelation,

            is_a: AnimalTaxonomy [
                (Dog, Mammal),
                (Cat, Mammal),
                (Dog, Pet),
                (Cat, Pet),
            ],

            has_a: AnimalMereology [
                (Dog, Tail),
                (Cat, Tail),
                (Dog, Fur),
                (Cat, Fur),
            ],

            causes: AnimalCausal for AnimalEvent [
                (Birth, Growth),
                (Growth, Death),
            ],

            opposes: AnimalOpposition [
                (Dog, Cat),
            ],
        }
    }

    #[test]
    fn macro_generates_category() {
        check_category_laws::<AnimalCategory>().unwrap();
    }

    #[test]
    fn macro_generates_taxonomy() {
        let rels = AnimalTaxonomy::relations();
        assert!(rels.contains(&(Animal::Dog, Animal::Mammal)));
        assert!(taxonomy::is_a::<AnimalTaxonomy>(
            &Animal::Dog,
            &Animal::Mammal
        ));
    }

    #[test]
    fn macro_generates_mereology() {
        let rels = AnimalMereology::relations();
        assert!(rels.contains(&(Animal::Dog, Animal::Tail)));
        let parts = mereology::parts_of::<AnimalMereology>(&Animal::Dog);
        assert!(parts.contains(&Animal::Tail));
        assert!(parts.contains(&Animal::Fur));
    }

    #[test]
    fn macro_generates_opposition() {
        let pairs = AnimalOpposition::pairs();
        assert!(pairs.contains(&(Animal::Dog, Animal::Cat)));
    }

    #[test]
    fn macro_generates_meta() {
        let meta = AnimalOntology::meta();
        assert_eq!(meta.name, "AnimalOntology");
        assert!(meta.module_path.contains("ontology_macro_test"));
    }

    #[test]
    fn structural_axioms_auto_generated() {
        let axioms = AnimalOntology::generated_structural_axioms();
        // 2 taxonomy + 1 mereology + 2 causation + 2 opposition = 7
        assert_eq!(axioms.len(), 7);
        for axiom in &axioms {
            assert!(axiom.holds(), "failed: {}", axiom.description());
        }
    }

    // Full Ontology trait: user provides domain_axioms(), framework merges
    use crate::ontology::{Ontology, Quality};

    pub struct AnimalIsAlive;
    impl Axiom for AnimalIsAlive {
        fn description(&self) -> &str {
            "all animals are alive"
        }
        fn holds(&self) -> bool {
            true
        }
    }

    #[derive(Debug, Clone)]
    pub struct NoQuality;
    impl Quality for NoQuality {
        type Individual = Animal;
        type Value = ();
        fn get(&self, _: &Animal) -> Option<()> {
            None
        }
    }

    impl Ontology for AnimalOntology {
        type Cat = AnimalCategory;
        type Qual = NoQuality;

        fn structural_axioms() -> Vec<Box<dyn Axiom>> {
            Self::generated_structural_axioms()
        }

        fn domain_axioms() -> Vec<Box<dyn Axiom>> {
            vec![Box::new(AnimalIsAlive)]
        }
    }

    #[test]
    fn ontology_merges_structural_and_domain() {
        let all = AnimalOntology::axioms();
        assert_eq!(all.len(), 8); // 7 structural + 1 domain
        for a in &all {
            assert!(a.holds());
        }
    }

    #[test]
    fn ontology_validates() {
        AnimalOntology::validate().unwrap();
    }
}

// =============================================================================
// proc macro ontology! — Communication as proof of concept
// =============================================================================

mod proc_macro_test {
    use crate as pr4xis;
    use crate::category::Entity;
    use crate::category::validate::check_category_laws;

    pr4xis_derive::ontology! {
        name: "Communication",
        source: "Shannon (1948); Jakobson (1960)",
        being: AbstractObject,

        concepts: [Sender, Receiver, Message, Channel, Code, Noise, Feedback, Context],

        labels: {
            Sender: ("en", "Sender", "The agent producing the message"),
            Receiver: ("en", "Receiver", "The agent interpreting the message"),
            Message: ("en", "Message", "The information being communicated"),
            Channel: ("en", "Channel", "The medium of transmission"),
            Code: ("en", "Code", "The shared encoding/decoding system"),
            Noise: ("en", "Noise", "Interference corrupting the message"),
            Feedback: ("en", "Feedback", "Receiver's response to sender"),
            Context: ("en", "Context", "Shared referential frame"),
        },

        edges: [
            (Sender, Message, Produces),
            (Message, Channel, TransmittedThrough),
            (Receiver, Message, Interprets),
            (Code, Message, EncodesDecodes),
            (Noise, Channel, Corrupts),
            (Feedback, Sender, FlowsBack),
            (Receiver, Feedback, Produces),
            (Context, Message, Grounds),
            (Sender, Code, Shares),
            (Receiver, Code, Shares),
        ],

        composed: [
            (Sender, Channel),
            (Sender, Receiver),
            (Noise, Message),
            (Receiver, Sender),
        ],

        opposes: [(Noise, Code)],
    }

    #[test]
    fn proc_macro_generates_entity() {
        let concepts = CommunicationConcept::variants();
        assert_eq!(concepts.len(), 8);
    }

    #[test]
    fn proc_macro_generates_category() {
        check_category_laws::<CommunicationCategory>().unwrap();
    }

    #[test]
    fn proc_macro_generates_vocabulary() {
        let vocab = CommunicationOntology::vocabulary();
        assert_eq!(vocab.concept_count, 8);
        assert!(vocab.morphism_count > 0);
        assert_eq!(vocab.source, "Shannon (1948); Jakobson (1960)");
    }

    #[test]
    fn proc_macro_validates_concept_names() {
        let sender = CommunicationConcept::Sender;
        assert_eq!(sender.name(), "Sender");
    }

    #[test]
    fn proc_macro_labels() {
        let labels = CommunicationOntology::labels();
        assert_eq!(labels.len(), 8);
        let sender_label = labels
            .iter()
            .find(|(c, _, _, _)| *c == CommunicationConcept::Sender);
        assert!(sender_label.is_some());
        let (_, lang, label, def) = sender_label.unwrap();
        assert_eq!(*lang, "en");
        assert_eq!(*label, "Sender");
        assert!(def.contains("agent"));
    }

    #[test]
    fn proc_macro_opposition() {
        use crate::ontology::reasoning::opposition::OppositionDef;
        let pairs = CommunicationOpposition::pairs();
        assert!(pairs.contains(&(CommunicationConcept::Noise, CommunicationConcept::Code)));
    }
}
