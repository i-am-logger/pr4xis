use super::property::Quality;
use crate::category::{Arrow, Category, Concept, FinitelyGenerated};
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

impl Concept for Light {}
impl FinitelyGenerated for Light {
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

impl Arrow for LightTransition {
    type Object = Light;
    type Kind = ();

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

    fn kind(&self) {}
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
    fn verify(&self) -> crate::logic::proof::Verdict {
        let dur = Duration;
        let green_dur = dur.get(&Light::Green).unwrap_or(0);
        if Light::variants()
            .iter()
            .all(|l| dur.get(l).unwrap_or(0) <= green_dur)
        {
            Ok(Box::new(crate::logic::SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(crate::logic::proof::SimpleCounterexample::new(
                self.meta(),
            )))
        }
    }

    fn citation(&self) -> crate::ontology::meta::Citation {
        crate::ontology::meta::Citation::parse_static(
            "MUTCD (2009) §4D.26 — traffic-signal green-phase duration",
        )
    }
}

// --- Axiom: no dead states ---

struct NoDeadStates;

impl Axiom for NoDeadStates {
    fn verify(&self) -> crate::logic::proof::Verdict {
        if Light::variants()
            .iter()
            .all(|obj| !TrafficLightCat::morphisms_from(obj).is_empty())
        {
            Ok(Box::new(crate::logic::SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(crate::logic::proof::SimpleCounterexample::new(
                self.meta(),
            )))
        }
    }

    fn citation(&self) -> crate::ontology::meta::Citation {
        crate::ontology::meta::Citation::parse_static(
            "Mac Lane (1971) Categories for the Working Mathematician Ch. I — every object has outgoing morphisms",
        )
    }
}

// --- Ontology: tie it all together ---

struct TrafficLightOntology;

impl Ontology for TrafficLightOntology {
    type Cat = TrafficLightCat;
    type Qual = Duration;

    fn axioms() -> Vec<Box<dyn Axiom>> {
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
        match TrafficLightOntology::validate() {
            Ok(_) => {}
            Err(c) => prop_assert!(
                false,
                "validation failed: {}",
                c.meta().description.as_str()
            ),
        }
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
            match axiom.verify() {
                Ok(_) => {}
                Err(c) => prop_assert!(
                    false,
                    "Axiom failed: {}",
                    c.meta().description.as_str()
                ),
            }
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
    match TrafficLightOntology::validate() {
        Ok(_) => {}
        Err(c) => panic!(
            "ontology validation failed: {}",
            c.meta().description.as_str()
        ),
    }
}

#[test]
fn test_ontology_check() {
    match super::validate::check_ontology::<TrafficLightOntology>() {
        Ok(_) => {}
        Err(c) => panic!("check_ontology failed: {}", c.meta().description.as_str()),
    }
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

/// Ontological test helper — pattern-match Verdict, no bool shortcuts.
fn expect_proves<A: Axiom>(axiom: A) {
    match axiom.verify() {
        Ok(_) => {}
        Err(c) => panic!("expected proof for {}, got counterexample", c.meta().name),
    }
}

#[test]
fn test_axiom_green_is_longest() {
    expect_proves(GreenIsLongest);
}

#[test]
fn test_axiom_no_dead_states() {
    expect_proves(NoDeadStates);
}

// =============================================================================
// proc macro ontology! — Communication as proof of concept
// =============================================================================

mod proc_macro_test {
    use crate as pr4xis;
    use crate::category::laws::assert_category_laws;
    use crate::category::{Category, Concept, FinitelyGenerated};

    pr4xis::ontology! {
        name: "Communication",
        source: "Shannon (1948); Jakobson (1960)",

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
        assert_category_laws::<CommunicationCategory>();
    }

    #[test]
    fn proc_macro_generates_vocabulary() {
        let vocab = CommunicationOntology::vocabulary();
        assert_eq!(vocab.concepts().len(), 8);
        assert!(!vocab.morphisms().is_empty());
        assert_eq!(vocab.source.as_str(), "Shannon (1948); Jakobson (1960)");
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
        // Opposition is now expressed as kinded morphisms in the category.
        use crate::category::Arrow;
        let has_opposition = CommunicationCategory::morphisms().iter().any(|m| {
            m.kind() == CommunicationRelationKind::Opposition
                && m.source() == CommunicationConcept::Noise
                && m.target() == CommunicationConcept::Code
        });
        assert!(has_opposition);
    }
}

// =============================================================================
// proc macro ontology! — dense category case (no explicit edges)
// =============================================================================

mod proc_macro_dense_test {
    use crate as pr4xis;
    use crate::category::laws::assert_category_laws;
    use crate::category::{Category, FinitelyGenerated};

    pr4xis::ontology! {
        name: "Biology",
        source: "Mayr (1982)",

        concepts: [Cell, Tissue, Organ, Organism],

        labels: {
            Cell: ("en", "Cell", "The basic structural unit of all organisms"),
            Tissue: ("en", "Tissue", "Aggregation of similar cells"),
        },

        is_a: [
            (Cell, Tissue),
            (Tissue, Organ),
            (Organ, Organism),
        ],
    }

    #[test]
    fn dense_category_generated() {
        let concepts = BiologyConcept::variants();
        assert_eq!(concepts.len(), 4);
    }

    #[test]
    fn dense_category_laws() {
        assert_category_laws::<BiologyCategory>();
    }

    #[test]
    fn dense_vocabulary() {
        let vocab = BiologyOntology::vocabulary();
        assert_eq!(vocab.concepts().len(), 4);
        assert_eq!(vocab.source.as_str(), "Mayr (1982)");
    }

    #[test]
    fn dense_taxonomy() {
        // Taxonomy is now expressed as kinded morphisms — filter by Subsumption.
        use crate::category::Arrow;
        let subsumption_edges: Vec<_> = BiologyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == BiologyRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(subsumption_edges.len() >= 3);
        assert!(subsumption_edges.contains(&(BiologyConcept::Cell, BiologyConcept::Tissue)));
    }

    #[test]
    fn dense_labels() {
        let labels = BiologyOntology::labels();
        assert_eq!(labels.len(), 2);
    }
}

// =============================================================================
// proc macro transitive closure — morphisms() must be closed under compose()
// =============================================================================

mod proc_macro_closure_test {
    use crate as pr4xis;
    use crate::category::Category;

    // Custom (non-canonical) kinds — NOT in the transitive inheritance set
    // (Subsumption / Parthood / Causation). Per OBO-RO partial-compose
    // (#166), these chained edges have NO transitive closure emitted as
    // typed morphisms.
    pr4xis::ontology! {
        name: "ChainedEdges",
        concepts: [A, B, C, D],
        edges: [
            (A, B, Step1),
            (B, C, Step2),
            (C, D, Step3),
        ],
    }

    // Canonical Subsumption chain — IS in the transitive inheritance set.
    // Per OBO-RO `transitive_over`, Sub ∘ Sub = Sub, so the closure IS
    // emitted.
    pr4xis::ontology! {
        name: "SubsumptionChain",
        concepts: [Dog, Mammal, Animal],
        is_a: [
            (Dog, Mammal),
            (Mammal, Animal),
        ],
    }

    #[test]
    fn custom_kinds_have_no_transitive_closure() {
        // Step1/2/3 aren't in the canonical transitive set, so no
        // heterogeneous closure edges are emitted (partial category).
        let morphisms = ChainedEdgesCategory::morphisms();
        let ac = morphisms
            .iter()
            .find(|m| m.from == ChainedEdgesConcept::A && m.to == ChainedEdgesConcept::C);
        assert!(
            ac.is_none(),
            "custom-kind chain (A, C) must NOT be in morphisms() — partial compose (#166)"
        );
    }

    #[test]
    fn subsumption_chain_has_transitive_closure() {
        // Sub ∘ Sub = Sub per OBO-RO transitive_over — closure IS emitted.
        let morphisms = SubsumptionChainCategory::morphisms();
        let dog_animal = morphisms.iter().find(|m| {
            m.from == SubsumptionChainConcept::Dog && m.to == SubsumptionChainConcept::Animal
        });
        assert!(
            dog_animal.is_some(),
            "(Dog, Animal) should be in morphisms() via Subsumption transitive closure"
        );
    }

    #[test]
    fn compose_output_is_in_morphisms() {
        let morphisms = ChainedEdgesCategory::morphisms();
        for f in &morphisms {
            for g in &morphisms {
                if let Some(composed) = ChainedEdgesCategory::compose(f, g) {
                    let found = morphisms
                        .iter()
                        .any(|m| m.from == composed.from && m.to == composed.to);
                    assert!(
                        found,
                        "compose({:?}, {:?}) = {:?} not in morphisms()",
                        f, g, composed
                    );
                }
            }
        }
    }
}
