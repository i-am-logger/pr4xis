use crate::category::entity::Entity;
use crate::category::relationship::Relationship;
use crate::category::validate::check_category_laws;
use crate::category::{Category, Functor};
use crate::logic::Axiom;
use crate::ontology::Quality;

use super::analogy::Analogy;
use super::causation::{self, Asymmetric, CausalCategory, CausalDef, NoSelfCausation};
use super::context::{self, ContextDef};
use super::equivalence::{self, EquivalenceCategory, EquivalenceDef};
use super::mereology::{self, MereologyCategory, MereologyDef, WeakSupplementation};
use super::opposition::{self, OppositionDef};
use super::taxonomy::{self, Antisymmetric, TaxonomyCategory, TaxonomyDef};

// =============================================================================
// Example domains for testing
// =============================================================================

// ---- Animal taxonomy ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::enum_variant_names)]
enum Animal {
    LivingThing,
    Animal,
    Mammal,
    Bird,
    Dog,
    Cat,
    Eagle,
}

impl Entity for Animal {
    fn variants() -> Vec<Self> {
        vec![
            Animal::LivingThing,
            Animal::Animal,
            Animal::Mammal,
            Animal::Bird,
            Animal::Dog,
            Animal::Cat,
            Animal::Eagle,
        ]
    }
}

struct AnimalTaxonomy;

impl TaxonomyDef for AnimalTaxonomy {
    type Entity = Animal;
    fn relations() -> Vec<(Animal, Animal)> {
        vec![
            (Animal::Animal, Animal::LivingThing),
            (Animal::Mammal, Animal::Animal),
            (Animal::Bird, Animal::Animal),
            (Animal::Dog, Animal::Mammal),
            (Animal::Cat, Animal::Mammal),
            (Animal::Eagle, Animal::Bird),
        ]
    }
}

#[derive(Debug, Clone)]
struct IsAlive;

impl Quality for IsAlive {
    type Individual = Animal;
    type Value = bool;

    fn get(&self, animal: &Animal) -> Option<bool> {
        match animal {
            Animal::LivingThing => Some(true),
            _ => None, // Only defined on the root — descendants inherit it
        }
    }
}

#[derive(Debug, Clone)]
struct Locomotion;

impl Quality for Locomotion {
    type Individual = Animal;
    type Value = &'static str;

    fn get(&self, animal: &Animal) -> Option<&'static str> {
        match animal {
            Animal::Mammal => Some("walk"),
            Animal::Bird => Some("fly"),
            _ => None,
        }
    }
}

// ---- Car mereology ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CarPart {
    Car,
    Body,
    Chassis,
    Engine,
    Wheel,
    Piston,
    Crankshaft,
}

impl Entity for CarPart {
    fn variants() -> Vec<Self> {
        vec![
            CarPart::Car,
            CarPart::Body,
            CarPart::Chassis,
            CarPart::Engine,
            CarPart::Wheel,
            CarPart::Piston,
            CarPart::Crankshaft,
        ]
    }
}

struct CarMereology;

impl MereologyDef for CarMereology {
    type Entity = CarPart;
    fn relations() -> Vec<(CarPart, CarPart)> {
        vec![
            (CarPart::Car, CarPart::Body),
            (CarPart::Car, CarPart::Chassis),
            (CarPart::Car, CarPart::Engine),
            (CarPart::Car, CarPart::Wheel),
            (CarPart::Engine, CarPart::Piston),
            (CarPart::Engine, CarPart::Crankshaft),
        ]
    }
}

// ---- Heat causal chain ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HeatState {
    Heating,
    Boiling,
    Steam,
    Condensation,
}

impl Entity for HeatState {
    fn variants() -> Vec<Self> {
        vec![
            HeatState::Heating,
            HeatState::Boiling,
            HeatState::Steam,
            HeatState::Condensation,
        ]
    }
}

struct HeatCausal;

impl CausalDef for HeatCausal {
    type Entity = HeatState;
    fn relations() -> Vec<(HeatState, HeatState)> {
        vec![
            (HeatState::Heating, HeatState::Boiling),
            (HeatState::Boiling, HeatState::Steam),
            (HeatState::Steam, HeatState::Condensation),
        ]
    }
}

// ---- Analogy: simple force domains ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EMConcept {
    Charge,
    ElectricField,
    CoulombLaw,
}

impl Entity for EMConcept {
    fn variants() -> Vec<Self> {
        vec![
            EMConcept::Charge,
            EMConcept::ElectricField,
            EMConcept::CoulombLaw,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EMRelation {
    from: EMConcept,
    to: EMConcept,
}

impl Relationship for EMRelation {
    type Object = EMConcept;
    fn source(&self) -> EMConcept {
        self.from
    }
    fn target(&self) -> EMConcept {
        self.to
    }
}

struct EMCategory;

impl Category for EMCategory {
    type Object = EMConcept;
    type Morphism = EMRelation;

    fn identity(obj: &EMConcept) -> EMRelation {
        EMRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &EMRelation, g: &EMRelation) -> Option<EMRelation> {
        if f.to != g.from {
            return None;
        }
        Some(EMRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<EMRelation> {
        let v = EMConcept::variants();
        let mut m = Vec::new();
        for &a in &v {
            for &b in &v {
                m.push(EMRelation { from: a, to: b });
            }
        }
        m
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GravConcept {
    Mass,
    GravField,
    NewtonLaw,
}

impl Entity for GravConcept {
    fn variants() -> Vec<Self> {
        vec![
            GravConcept::Mass,
            GravConcept::GravField,
            GravConcept::NewtonLaw,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GravRelation {
    from: GravConcept,
    to: GravConcept,
}

impl Relationship for GravRelation {
    type Object = GravConcept;
    fn source(&self) -> GravConcept {
        self.from
    }
    fn target(&self) -> GravConcept {
        self.to
    }
}

struct GravCategory;

impl Category for GravCategory {
    type Object = GravConcept;
    type Morphism = GravRelation;

    fn identity(obj: &GravConcept) -> GravRelation {
        GravRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &GravRelation, g: &GravRelation) -> Option<GravRelation> {
        if f.to != g.from {
            return None;
        }
        Some(GravRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<GravRelation> {
        let v = GravConcept::variants();
        let mut m = Vec::new();
        for &a in &v {
            for &b in &v {
                m.push(GravRelation { from: a, to: b });
            }
        }
        m
    }
}

struct EMGravAnalogy;

impl Functor for EMGravAnalogy {
    type Source = EMCategory;
    type Target = GravCategory;

    fn map_object(obj: &EMConcept) -> GravConcept {
        match obj {
            EMConcept::Charge => GravConcept::Mass,
            EMConcept::ElectricField => GravConcept::GravField,
            EMConcept::CoulombLaw => GravConcept::NewtonLaw,
        }
    }

    fn map_morphism(m: &EMRelation) -> GravRelation {
        GravRelation {
            from: Self::map_object(&m.from),
            to: Self::map_object(&m.to),
        }
    }
}

// =============================================================================
// Taxonomy tests
// =============================================================================

#[test]
fn taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<AnimalTaxonomy>>().unwrap();
}

#[test]
fn taxonomy_no_cycles() {
    let axiom = taxonomy::NoCycles::<AnimalTaxonomy>::new();
    assert!(axiom.holds());
}

#[test]
fn taxonomy_antisymmetric() {
    let axiom = Antisymmetric::<AnimalTaxonomy>::new();
    assert!(axiom.holds());
}

#[test]
fn taxonomy_direct_is_a() {
    assert!(taxonomy::is_a::<AnimalTaxonomy>(
        &Animal::Dog,
        &Animal::Mammal
    ));
}

#[test]
fn taxonomy_transitive_is_a() {
    // Dog is-a Animal (via Dog -> Mammal -> Animal)
    assert!(taxonomy::is_a::<AnimalTaxonomy>(
        &Animal::Dog,
        &Animal::Animal
    ));
    // Dog is-a LivingThing (via Dog -> Mammal -> Animal -> LivingThing)
    assert!(taxonomy::is_a::<AnimalTaxonomy>(
        &Animal::Dog,
        &Animal::LivingThing
    ));
}

#[test]
fn taxonomy_reflexive() {
    assert!(taxonomy::is_a::<AnimalTaxonomy>(&Animal::Dog, &Animal::Dog));
}

#[test]
fn taxonomy_not_is_a() {
    // Dog is NOT a Bird
    assert!(!taxonomy::is_a::<AnimalTaxonomy>(
        &Animal::Dog,
        &Animal::Bird
    ));
    // Mammal is NOT a Dog (direction matters)
    assert!(!taxonomy::is_a::<AnimalTaxonomy>(
        &Animal::Mammal,
        &Animal::Dog
    ));
}

#[test]
fn taxonomy_ancestors() {
    let ancestors = taxonomy::ancestors::<AnimalTaxonomy>(&Animal::Dog);
    assert!(ancestors.contains(&Animal::Mammal));
    assert!(ancestors.contains(&Animal::Animal));
    assert!(ancestors.contains(&Animal::LivingThing));
    assert!(!ancestors.contains(&Animal::Dog)); // not self
    assert!(!ancestors.contains(&Animal::Bird));
}

#[test]
fn taxonomy_descendants() {
    let desc = taxonomy::descendants::<AnimalTaxonomy>(&Animal::Mammal);
    assert!(desc.contains(&Animal::Dog));
    assert!(desc.contains(&Animal::Cat));
    assert!(!desc.contains(&Animal::Eagle)); // Eagle is-a Bird, not Mammal
    assert!(!desc.contains(&Animal::Mammal)); // not self
}

#[test]
fn taxonomy_root_has_no_ancestors() {
    let ancestors = taxonomy::ancestors::<AnimalTaxonomy>(&Animal::LivingThing);
    assert!(ancestors.is_empty());
}

#[test]
fn taxonomy_leaf_has_no_descendants() {
    let desc = taxonomy::descendants::<AnimalTaxonomy>(&Animal::Dog);
    assert!(desc.is_empty());
}

// ---- Quality inheritance ----

#[test]
fn taxonomy_quality_inheritance_from_root() {
    // IsAlive is only defined on LivingThing, but Dog inherits it
    let result = taxonomy::inherit_quality::<AnimalTaxonomy, _>(&Animal::Dog, &IsAlive);
    assert_eq!(result, Some(true));
}

#[test]
fn taxonomy_quality_inheritance_nearest() {
    // Locomotion is defined on Mammal ("walk") and Bird ("fly")
    // Dog inherits "walk" from Mammal
    let result = taxonomy::inherit_quality::<AnimalTaxonomy, _>(&Animal::Dog, &Locomotion);
    assert_eq!(result, Some("walk"));

    // Eagle inherits "fly" from Bird
    let result = taxonomy::inherit_quality::<AnimalTaxonomy, _>(&Animal::Eagle, &Locomotion);
    assert_eq!(result, Some("fly"));
}

#[test]
fn taxonomy_quality_no_inheritance_for_unrelated() {
    // LivingThing has no Locomotion quality (and no ancestors to inherit from)
    let result = taxonomy::inherit_quality::<AnimalTaxonomy, _>(&Animal::LivingThing, &Locomotion);
    assert_eq!(result, None);
}

// =============================================================================
// Mereology tests
// =============================================================================

#[test]
fn mereology_category_laws() {
    check_category_laws::<MereologyCategory<CarMereology>>().unwrap();
}

#[test]
fn mereology_no_cycles() {
    let axiom = mereology::NoCycles::<CarMereology>::new();
    assert!(axiom.holds());
}

#[test]
fn mereology_weak_supplementation() {
    let axiom = WeakSupplementation::<CarMereology>::new();
    assert!(axiom.holds());
}

#[test]
fn mereology_direct_parts() {
    let parts = mereology::parts_of::<CarMereology>(&CarPart::Car);
    assert!(parts.contains(&CarPart::Body));
    assert!(parts.contains(&CarPart::Engine));
    assert!(parts.contains(&CarPart::Wheel));
    assert!(parts.contains(&CarPart::Chassis));
}

#[test]
fn mereology_transitive_parts() {
    // Car has-a Piston (via Car -> Engine -> Piston)
    let parts = mereology::parts_of::<CarMereology>(&CarPart::Car);
    assert!(parts.contains(&CarPart::Piston));
    assert!(parts.contains(&CarPart::Crankshaft));
}

#[test]
fn mereology_whole_of() {
    // Piston is part of Engine and (transitively) Car
    let wholes = mereology::whole_of::<CarMereology>(&CarPart::Piston);
    assert!(wholes.contains(&CarPart::Engine));
    assert!(wholes.contains(&CarPart::Car));
}

#[test]
fn mereology_leaf_has_no_parts() {
    let parts = mereology::parts_of::<CarMereology>(&CarPart::Wheel);
    assert!(parts.is_empty());
}

#[test]
fn mereology_root_is_not_part_of_anything() {
    let wholes = mereology::whole_of::<CarMereology>(&CarPart::Car);
    assert!(wholes.is_empty());
}

// =============================================================================
// Causation tests
// =============================================================================

#[test]
fn causation_category_laws() {
    check_category_laws::<CausalCategory<HeatCausal>>().unwrap();
}

#[test]
fn causation_asymmetric() {
    let axiom = Asymmetric::<HeatCausal>::new();
    assert!(axiom.holds());
}

#[test]
fn causation_no_self_causation() {
    let axiom = NoSelfCausation::<HeatCausal>::new();
    assert!(axiom.holds());
}

#[test]
fn causation_direct_effects() {
    let effects = causation::effects_of::<HeatCausal>(&HeatState::Heating);
    assert!(effects.contains(&HeatState::Boiling));
}

#[test]
fn causation_transitive_effects() {
    // Heating causes Steam (via Heating -> Boiling -> Steam)
    let effects = causation::effects_of::<HeatCausal>(&HeatState::Heating);
    assert!(effects.contains(&HeatState::Steam));
    assert!(effects.contains(&HeatState::Condensation));
}

#[test]
fn causation_reverse_causes() {
    // What causes Condensation?
    let causes = causation::causes_of::<HeatCausal>(&HeatState::Condensation);
    assert!(causes.contains(&HeatState::Steam));
    assert!(causes.contains(&HeatState::Boiling));
    assert!(causes.contains(&HeatState::Heating));
}

#[test]
fn causation_no_reverse() {
    // Boiling does NOT cause Heating
    let effects = causation::effects_of::<HeatCausal>(&HeatState::Boiling);
    assert!(!effects.contains(&HeatState::Heating));
}

#[test]
fn causation_end_has_no_effects() {
    let effects = causation::effects_of::<HeatCausal>(&HeatState::Condensation);
    assert!(effects.is_empty());
}

// =============================================================================
// Analogy tests
// =============================================================================

#[test]
fn analogy_validates() {
    Analogy::<EMGravAnalogy>::validate().unwrap();
}

#[test]
fn analogy_translates_objects() {
    assert_eq!(
        Analogy::<EMGravAnalogy>::translate(&EMConcept::Charge),
        GravConcept::Mass
    );
    assert_eq!(
        Analogy::<EMGravAnalogy>::translate(&EMConcept::ElectricField),
        GravConcept::GravField
    );
    assert_eq!(
        Analogy::<EMGravAnalogy>::translate(&EMConcept::CoulombLaw),
        GravConcept::NewtonLaw
    );
}

#[test]
fn analogy_translates_morphisms() {
    let em_relation = EMRelation {
        from: EMConcept::Charge,
        to: EMConcept::ElectricField,
    };
    let grav_relation = Analogy::<EMGravAnalogy>::translate_morphism(&em_relation);
    assert_eq!(grav_relation.from, GravConcept::Mass);
    assert_eq!(grav_relation.to, GravConcept::GravField);
}

#[test]
fn analogy_preserves_identity() {
    for obj in EMConcept::variants() {
        let em_id = EMCategory::identity(&obj);
        let mapped = EMGravAnalogy::map_morphism(&em_id);
        let grav_id = GravCategory::identity(&EMGravAnalogy::map_object(&obj));
        assert_eq!(mapped, grav_id);
    }
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_animal() -> impl Strategy<Value = Animal> {
        prop_oneof![
            Just(Animal::LivingThing),
            Just(Animal::Animal),
            Just(Animal::Mammal),
            Just(Animal::Bird),
            Just(Animal::Dog),
            Just(Animal::Cat),
            Just(Animal::Eagle),
        ]
    }

    fn arb_car_part() -> impl Strategy<Value = CarPart> {
        prop_oneof![
            Just(CarPart::Car),
            Just(CarPart::Body),
            Just(CarPart::Chassis),
            Just(CarPart::Engine),
            Just(CarPart::Wheel),
            Just(CarPart::Piston),
            Just(CarPart::Crankshaft),
        ]
    }

    fn arb_heat_state() -> impl Strategy<Value = HeatState> {
        prop_oneof![
            Just(HeatState::Heating),
            Just(HeatState::Boiling),
            Just(HeatState::Steam),
            Just(HeatState::Condensation),
        ]
    }

    fn arb_word() -> impl Strategy<Value = Word> {
        prop_oneof![
            Just(Word::Big),
            Just(Word::Large),
            Just(Word::Huge),
            Just(Word::Small),
            Just(Word::Tiny),
            Just(Word::Hot),
            Just(Word::Cold),
            Just(Word::Warm),
            Just(Word::Cool),
        ]
    }

    proptest! {
        /// Taxonomy is reflexive: everything is-a itself.
        #[test]
        fn prop_taxonomy_reflexive(animal in arb_animal()) {
            prop_assert!(taxonomy::is_a::<AnimalTaxonomy>(&animal, &animal));
        }

        /// Taxonomy transitivity: if A is-a B and B is-a C, then A is-a C.
        #[test]
        fn prop_taxonomy_transitive(
            a in arb_animal(),
            b in arb_animal(),
            c in arb_animal()
        ) {
            if taxonomy::is_a::<AnimalTaxonomy>(&a, &b)
                && taxonomy::is_a::<AnimalTaxonomy>(&b, &c)
            {
                prop_assert!(taxonomy::is_a::<AnimalTaxonomy>(&a, &c));
            }
        }

        /// Taxonomy antisymmetry: if A is-a B and B is-a A, then A == B.
        #[test]
        fn prop_taxonomy_antisymmetric(a in arb_animal(), b in arb_animal()) {
            if taxonomy::is_a::<AnimalTaxonomy>(&a, &b)
                && taxonomy::is_a::<AnimalTaxonomy>(&b, &a)
            {
                prop_assert_eq!(a, b);
            }
        }

        /// Ancestors never contain self.
        #[test]
        fn prop_ancestors_exclude_self(animal in arb_animal()) {
            let anc = taxonomy::ancestors::<AnimalTaxonomy>(&animal);
            prop_assert!(!anc.contains(&animal));
        }

        /// Descendants never contain self.
        #[test]
        fn prop_descendants_exclude_self(animal in arb_animal()) {
            let desc = taxonomy::descendants::<AnimalTaxonomy>(&animal);
            prop_assert!(!desc.contains(&animal));
        }

        /// Parts never contain the whole itself.
        #[test]
        fn prop_parts_exclude_self(part in arb_car_part()) {
            let parts = mereology::parts_of::<CarMereology>(&part);
            prop_assert!(!parts.contains(&part));
        }

        /// Causation: effects never contain the cause itself.
        #[test]
        fn prop_effects_exclude_self(state in arb_heat_state()) {
            let effects = causation::effects_of::<HeatCausal>(&state);
            prop_assert!(!effects.contains(&state));
        }

        /// Causation asymmetry: if A causes B (A != B), then B does not cause A.
        #[test]
        fn prop_causation_asymmetric(a in arb_heat_state(), b in arb_heat_state()) {
            let a_effects = causation::effects_of::<HeatCausal>(&a);
            let b_effects = causation::effects_of::<HeatCausal>(&b);
            if a != b && a_effects.contains(&b) {
                prop_assert!(!b_effects.contains(&a));
            }
        }

        /// Inherited quality is always defined for entities with ancestors that have the quality.
        #[test]
        fn prop_is_alive_inherited(animal in arb_animal()) {
            // Everything is-a LivingThing, which has IsAlive = true
            let result = taxonomy::inherit_quality::<AnimalTaxonomy, _>(&animal, &IsAlive);
            prop_assert_eq!(result, Some(true));
        }

        /// Equivalence is reflexive.
        #[test]
        fn prop_equivalence_reflexive(word in arb_word()) {
            prop_assert!(equivalence::are_equivalent::<WordSynonyms>(&word, &word));
        }

        /// Equivalence is symmetric.
        #[test]
        fn prop_equivalence_symmetric(a in arb_word(), b in arb_word()) {
            if equivalence::are_equivalent::<WordSynonyms>(&a, &b) {
                prop_assert!(equivalence::are_equivalent::<WordSynonyms>(&b, &a));
            }
        }

        /// Equivalence is transitive.
        #[test]
        fn prop_equivalence_transitive(a in arb_word(), b in arb_word(), c in arb_word()) {
            if equivalence::are_equivalent::<WordSynonyms>(&a, &b)
                && equivalence::are_equivalent::<WordSynonyms>(&b, &c)
            {
                prop_assert!(equivalence::are_equivalent::<WordSynonyms>(&a, &c));
            }
        }

        /// Opposition is symmetric.
        #[test]
        fn prop_opposition_symmetric(a in arb_word(), b in arb_word()) {
            if opposition::are_opposed::<WordAntonyms>(&a, &b) {
                prop_assert!(opposition::are_opposed::<WordAntonyms>(&b, &a));
            }
        }

        /// Opposites are never equivalent.
        #[test]
        fn prop_opposites_not_equivalent(a in arb_word(), b in arb_word()) {
            if opposition::are_opposed::<WordAntonyms>(&a, &b) {
                prop_assert!(!equivalence::are_equivalent::<WordSynonyms>(&a, &b));
            }
        }
    }
}

// =============================================================================
// Example domains: word semantics
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Word {
    Big,
    Large,
    Huge,
    Small,
    Tiny,
    Hot,
    Cold,
    Warm,
    Cool,
}

impl Entity for Word {
    fn variants() -> Vec<Self> {
        vec![
            Word::Big,
            Word::Large,
            Word::Huge,
            Word::Small,
            Word::Tiny,
            Word::Hot,
            Word::Cold,
            Word::Warm,
            Word::Cool,
        ]
    }
}

struct WordSynonyms;

impl EquivalenceDef for WordSynonyms {
    type Entity = Word;
    fn pairs() -> Vec<(Word, Word)> {
        vec![
            (Word::Big, Word::Large),
            (Word::Large, Word::Huge), // transitive: Big ≡ Huge
            (Word::Small, Word::Tiny),
        ]
    }
}

struct WordAntonyms;

impl OppositionDef for WordAntonyms {
    type Entity = Word;
    fn pairs() -> Vec<(Word, Word)> {
        vec![
            (Word::Big, Word::Small),
            (Word::Hot, Word::Cold),
            (Word::Warm, Word::Cool),
        ]
    }
}

// ---- Disambiguation example ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AmbiguousWord {
    Bank,
    Bat,
    Spring,
}

impl Entity for AmbiguousWord {
    fn variants() -> Vec<Self> {
        vec![
            AmbiguousWord::Bank,
            AmbiguousWord::Bat,
            AmbiguousWord::Spring,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ContextSignal {
    Money,
    River,
    Sports,
    Animal,
    Season,
    Water,
}

impl Entity for ContextSignal {
    fn variants() -> Vec<Self> {
        vec![
            ContextSignal::Money,
            ContextSignal::River,
            ContextSignal::Sports,
            ContextSignal::Animal,
            ContextSignal::Season,
            ContextSignal::Water,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Meaning {
    FinancialInstitution,
    Riverbank,
    BaseballBat,
    FlyingMammal,
    SpringSeason,
    WaterSpring,
}

impl Entity for Meaning {
    fn variants() -> Vec<Self> {
        vec![
            Meaning::FinancialInstitution,
            Meaning::Riverbank,
            Meaning::BaseballBat,
            Meaning::FlyingMammal,
            Meaning::SpringSeason,
            Meaning::WaterSpring,
        ]
    }
}

struct WordContext;

impl ContextDef for WordContext {
    type Entity = AmbiguousWord;
    type Signal = ContextSignal;
    type Resolution = Meaning;

    fn resolutions() -> Vec<(AmbiguousWord, ContextSignal, Meaning)> {
        vec![
            (
                AmbiguousWord::Bank,
                ContextSignal::Money,
                Meaning::FinancialInstitution,
            ),
            (
                AmbiguousWord::Bank,
                ContextSignal::River,
                Meaning::Riverbank,
            ),
            (
                AmbiguousWord::Bat,
                ContextSignal::Sports,
                Meaning::BaseballBat,
            ),
            (
                AmbiguousWord::Bat,
                ContextSignal::Animal,
                Meaning::FlyingMammal,
            ),
            (
                AmbiguousWord::Spring,
                ContextSignal::Season,
                Meaning::SpringSeason,
            ),
            (
                AmbiguousWord::Spring,
                ContextSignal::Water,
                Meaning::WaterSpring,
            ),
        ]
    }
}

// =============================================================================
// Equivalence tests
// =============================================================================

#[test]
fn equivalence_category_laws() {
    check_category_laws::<EquivalenceCategory<WordSynonyms>>().unwrap();
}

#[test]
fn equivalence_direct() {
    assert!(equivalence::are_equivalent::<WordSynonyms>(
        &Word::Big,
        &Word::Large
    ));
}

#[test]
fn equivalence_transitive() {
    // Big ≡ Huge (via Big ≡ Large ≡ Huge)
    assert!(equivalence::are_equivalent::<WordSynonyms>(
        &Word::Big,
        &Word::Huge
    ));
}

#[test]
fn equivalence_symmetric() {
    assert!(equivalence::are_equivalent::<WordSynonyms>(
        &Word::Large,
        &Word::Big
    ));
}

#[test]
fn equivalence_not_equivalent() {
    assert!(!equivalence::are_equivalent::<WordSynonyms>(
        &Word::Big,
        &Word::Small
    ));
}

#[test]
fn equivalence_class_big() {
    let class = equivalence::equivalence_class::<WordSynonyms>(&Word::Big);
    assert!(class.contains(&Word::Big));
    assert!(class.contains(&Word::Large));
    assert!(class.contains(&Word::Huge));
    assert_eq!(class.len(), 3);
}

#[test]
fn equivalence_all_classes() {
    let classes = equivalence::all_classes::<WordSynonyms>();
    // {Big, Large, Huge}, {Small, Tiny}, {Hot}, {Cold}, {Warm}, {Cool}
    assert_eq!(classes.len(), 6);
}

#[test]
fn equivalence_symmetric_axiom() {
    let axiom = equivalence::Symmetric::<WordSynonyms>::new();
    assert!(axiom.holds());
}

#[test]
fn equivalence_no_self_equivalence_axiom() {
    let axiom = equivalence::NoSelfEquivalence::<WordSynonyms>::new();
    assert!(axiom.holds());
}

// =============================================================================
// Opposition tests
// =============================================================================

#[test]
fn opposition_direct() {
    assert!(opposition::are_opposed::<WordAntonyms>(
        &Word::Big,
        &Word::Small
    ));
    assert!(opposition::are_opposed::<WordAntonyms>(
        &Word::Hot,
        &Word::Cold
    ));
}

#[test]
fn opposition_symmetric() {
    assert!(opposition::are_opposed::<WordAntonyms>(
        &Word::Small,
        &Word::Big
    ));
}

#[test]
fn opposition_not_opposed() {
    assert!(!opposition::are_opposed::<WordAntonyms>(
        &Word::Big,
        &Word::Hot
    ));
}

#[test]
fn opposition_irreflexive() {
    let axiom = opposition::Irreflexive::<WordAntonyms>::new();
    assert!(axiom.holds());
}

#[test]
fn opposition_symmetric_axiom() {
    let axiom = opposition::Symmetric::<WordAntonyms>::new();
    assert!(axiom.holds());
}

#[test]
fn opposition_exclusive_with_equivalence() {
    let axiom = opposition::ExclusiveWithEquivalence::<WordAntonyms, _>::new(|a, b| {
        equivalence::are_equivalent::<WordSynonyms>(a, b)
    });
    assert!(axiom.holds());
}

#[test]
fn opposition_not_transitive() {
    // Big opposes Small, Small opposes Big — but Big does NOT oppose Big
    // This verifies opposition is not transitive
    assert!(opposition::are_opposed::<WordAntonyms>(
        &Word::Big,
        &Word::Small
    ));
    assert!(!opposition::are_opposed::<WordAntonyms>(
        &Word::Big,
        &Word::Big
    ));
}

// =============================================================================
// Context tests
// =============================================================================

#[test]
fn context_resolve_bank_money() {
    let result = context::resolve::<WordContext>(&AmbiguousWord::Bank, &ContextSignal::Money);
    assert_eq!(result, Some(Meaning::FinancialInstitution));
}

#[test]
fn context_resolve_bank_river() {
    let result = context::resolve::<WordContext>(&AmbiguousWord::Bank, &ContextSignal::River);
    assert_eq!(result, Some(Meaning::Riverbank));
}

#[test]
fn context_resolve_no_signal() {
    // Bank + Sports → None (no resolution defined)
    let result = context::resolve::<WordContext>(&AmbiguousWord::Bank, &ContextSignal::Sports);
    assert_eq!(result, None);
}

#[test]
fn context_interpretations() {
    let interps = context::interpretations::<WordContext>(&AmbiguousWord::Bank);
    assert_eq!(interps.len(), 2);
}

#[test]
fn context_ambiguous_entities() {
    let ambiguous = context::ambiguous_entities::<WordContext>();
    assert!(ambiguous.contains(&AmbiguousWord::Bank));
    assert!(ambiguous.contains(&AmbiguousWord::Bat));
    assert!(ambiguous.contains(&AmbiguousWord::Spring));
}

#[test]
fn context_deterministic_axiom() {
    let axiom = context::Deterministic::<WordContext>::new();
    assert!(axiom.holds());
}

#[test]
fn context_true_ambiguity_axiom() {
    let axiom = context::TrueAmbiguity::<WordContext>::new();
    assert!(axiom.holds());
}
