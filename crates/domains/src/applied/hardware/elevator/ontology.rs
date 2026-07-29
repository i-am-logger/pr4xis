//! Elevator — floor / travel / dispatch ontology.
//!
//! Models the abstract concepts of an elevator system: floors, travel
//! between floors, the elevator car as a sliding actuator, and dispatch
//! requests. The rich runtime types (`Elevator`, `Building`, `Request`,
//! `Dispatch`) in sibling modules carry the simulation state; this
//! ontology is the categorical view used by Praxis-level reasoning.
//!
//! # Literature
//!
//! - **Mandel (1989)** "Elevator Scheduling" — the canonical reference
//!   for elevator group control and dispatch strategy. Defines the
//!   floor / car / call vocabulary used in modern dispatch.
//! - **Barney & Dos Santos (1985)** *Elevator Traffic Analysis, Design
//!   and Control* (Peter Peregrinus / IEE) — the standard text on
//!   elevator dynamics, arrival processes, and the floor topology.

use pr4xis::category::{Concept, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Elevator",
    source: "Mandel (1989) Elevator Scheduling; Barney & Dos Santos (1985) Elevator Traffic Analysis, Design and Control",

    concepts: [
        // === Static topology (Barney & Dos Santos 1985 §2) ===
        Floor,
        GroundFloor,
        UpperFloor,
        Shaft,

        // === Mobile assets (Mandel 1989 §1) ===
        Car,
        DoorAssembly,

        // === Service events (Mandel 1989 §3 dispatch) ===
        HallCall,
        CarCall,
        Travel,
        Stop,
    ],

    labels: {
        Floor: ("en", "Floor",
            "Barney & Dos Santos (1985) §2: a horizontal landing served by the elevator system."),
        GroundFloor: ("en", "Ground floor",
            "The lowest served Floor — typically the lobby; reference for height computations."),
        UpperFloor: ("en", "Upper floor",
            "Any Floor above the ground floor."),
        Shaft: ("en", "Shaft",
            "Barney & Dos Santos (1985) §2: the vertical channel in which a Car travels; connects all Floors of a bank."),
        Car: ("en", "Car",
            "Mandel (1989): the mobile cab that transports passengers between Floors."),
        DoorAssembly: ("en", "Door assembly",
            "The interlocking landing-door + car-door pair that admits passengers at a Stop."),
        HallCall: ("en", "Hall call",
            "Mandel (1989) §3: a service request from a passenger on a Floor (hall) waiting to board."),
        CarCall: ("en", "Car call",
            "Mandel (1989) §3: a service request from a passenger inside the Car for a destination Floor."),
        Travel: ("en", "Travel",
            "A trip segment — the Car moving from one Floor to another."),
        Stop: ("en", "Stop",
            "The Car halting at a Floor to load / unload passengers (door cycle)."),
    },

    is_a: [
        // Floor taxonomy: every served level is-a Floor.
        (GroundFloor, Floor),
        (UpperFloor, Floor),

        // Service events specialise dispatch concepts.
        // (HallCall and CarCall both ARE service-request flavours.)
    ],

    has_a: [
        // A Shaft connects multiple Floors.
        (Shaft, Floor),

        // A Car runs in a Shaft and has a Door assembly.
        (Car, Shaft),
        (Car, DoorAssembly),

        // A Travel involves two Floors (source, destination) and a Car.
        (Travel, Floor),
        (Travel, Car),

        // A Stop happens at a Floor and uses the DoorAssembly.
        (Stop, Floor),
        (Stop, DoorAssembly),

        // Calls are addressed to Floors.
        (HallCall, Floor),
        (CarCall, Floor),
    ],

    opposes: [
        // GroundFloor vs UpperFloor — disjoint partitioning of Floor.
        (GroundFloor, UpperFloor),
        (UpperFloor, GroundFloor),

        // HallCall (external, on a landing) vs CarCall (internal, in the car).
        (HallCall, CarCall),
        (CarCall, HallCall),
    ],
}

/// A concrete floor index in a building. The ontology layer above models
/// `Floor` as an abstract concept; this is the rich-runtime indexed
/// representation used by `building.rs` / `dispatch.rs` / the simulation.
///
/// The runtime variants enumerated by `Concept::variants()` are
/// `Floor(0) .. Floor(MAX_FLOORS - 1)` for the default 10-storey building.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FloorIndex(pub usize);

impl FloorIndex {
    /// Construct a `FloorIndex`, asserting `n < max`.
    pub fn new(n: usize, max: usize) -> Self {
        assert!(n < max);
        FloorIndex(n)
    }
}

/// Default building size for `Concept::variants()`. The rich `Building`
/// type in `building.rs` carries its own configurable size; this constant
/// is only the enumeration of indexed `FloorIndex` values for the ontology.
const MAX_FLOORS: usize = 10;

impl Concept for FloorIndex {}
impl FinitelyGenerated for FloorIndex {
    fn variants() -> Vec<Self> {
        (0..MAX_FLOORS).map(FloorIndex).collect()
    }
}

/// Quality: physical height (in floor units) of a `FloorIndex` above the
/// ground. The mapping is the identity — `FloorIndex(n) → FloorIndex(n)` —
/// since per Barney & Dos Santos (1985) every Floor sits one storey above
/// the previous in a uniform-storey building.
///
/// The value type is `FloorIndex` itself, not a bare `usize`: `FloorIndex`
/// is already the typed index for "storeys above ground", so wrapping it a
/// second time (e.g. in a `HeightAboveGround(usize)` newtype) would add a
/// distinct Rust type with no distinct ontological content — the identity
/// map from one typed index to the same typed index.
#[derive(Debug, Clone)]
pub struct HeightFromGround;

impl Quality for HeightFromGround {
    type Individual = FloorIndex;
    type Value = FloorIndex;

    fn get(&self, floor: &FloorIndex) -> Option<FloorIndex> {
        Some(*floor)
    }
}

/// The scholarly tradition a concept descends from — a closed two-element
/// set drawn from the ontology's `source:` literature.
///
/// - **Barney & Dos Santos (1985)** *Elevator Traffic Analysis, Design and
///   Control* (Peter Peregrinus / IEE) — the floor / shaft topology and
///   traffic-dynamics lineage.
/// - **Mandel (1989)** "Elevator Scheduling" — the car / call / dispatch
///   lineage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElevatorLiterature {
    /// Barney & Dos Santos (1985) — static topology (floors, shaft) and
    /// traffic dynamics.
    BarneyDosSantos,
    /// Mandel (1989) — the mobile car, hall/car calls, and dispatch events.
    Mandel,
}

/// Quality: which tradition each abstract concept comes from.
#[derive(Debug, Clone)]
pub struct ElevatorTradition;

impl Quality for ElevatorTradition {
    type Individual = ElevatorConcept;
    type Value = ElevatorLiterature;

    fn get(&self, c: &ElevatorConcept) -> Option<ElevatorLiterature> {
        use ElevatorConcept as E;
        Some(match c {
            E::Floor | E::GroundFloor | E::UpperFloor | E::Shaft => {
                ElevatorLiterature::BarneyDosSantos
            }
            E::Car | E::DoorAssembly | E::HallCall | E::CarCall | E::Travel | E::Stop => {
                ElevatorLiterature::Mandel
            }
        })
    }
}

impl Ontology for ElevatorOntology {
    type Cat = ElevatorCategory;
    type Qual = ElevatorTradition;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(GroundFloorIsLowest));
        axioms
    }
}

/// Axiom: the GroundFloor is the lowest served Floor — its height above
/// ground is zero.
///
/// Barney & Dos Santos (1985) §2 — by convention, the ground floor is the
/// reference level (height = 0) for an elevator bank.
pub struct GroundFloorIsLowest;

impl Axiom for GroundFloorIsLowest {
    fn verify(&self) -> Verdict {
        if HeightFromGround.get(&FloorIndex(0)) == Some(FloorIndex(0)) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GroundFloorIsLowest",
        "the ground floor sits at height 0 above ground",
        "Barney & Dos Santos (1985) Elevator Traffic Analysis, Design and Control §2"
    );
}

pr4xis::register_axiom!(
    GroundFloorIsLowest,
    "Barney & Dos Santos (1985) Elevator Traffic Analysis, Design and Control §2"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ElevatorCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ElevatorOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ten_concepts() {
        // Floor, GroundFloor, UpperFloor, Shaft, Car, DoorAssembly,
        // HallCall, CarCall, Travel, Stop.
        assert_eq!(ElevatorConcept::variants().len(), 10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ground_floor_index_height_zero() {
        assert_eq!(HeightFromGround.get(&FloorIndex(0)), Some(FloorIndex(0)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn floor_index_height_matches_index() {
        assert_eq!(HeightFromGround.get(&FloorIndex(5)), Some(FloorIndex(5)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn floor_index_default_building_ten_floors() {
        assert_eq!(FloorIndex::variants().len(), MAX_FLOORS);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn floor_taxonomy_ground_and_upper_subsume_floor() {
        let sub: Vec<_> = ElevatorCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ElevatorRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(sub.contains(&(ElevatorConcept::GroundFloor, ElevatorConcept::Floor)));
        assert!(sub.contains(&(ElevatorConcept::UpperFloor, ElevatorConcept::Floor)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ground_upper_floors_oppose() {
        let opp: Vec<_> = ElevatorCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ElevatorRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(ElevatorConcept::GroundFloor, ElevatorConcept::UpperFloor)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hall_and_car_calls_oppose() {
        let opp: Vec<_> = ElevatorCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ElevatorRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(ElevatorConcept::HallCall, ElevatorConcept::CarCall)));
        assert!(opp.contains(&(ElevatorConcept::CarCall, ElevatorConcept::HallCall)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ground_floor_is_lowest_axiom_holds() {
        assert!(GroundFloorIsLowest.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_concept_has_tradition() {
        let q = ElevatorTradition;
        for c in ElevatorConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing tradition", c);
        }
    }

    fn arb_concept() -> impl Strategy<Value = ElevatorConcept> {
        proptest::sample::select(ElevatorConcept::variants())
    }

    fn arb_floor_index() -> impl Strategy<Value = FloorIndex> {
        proptest::sample::select(FloorIndex::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in ElevatorCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ElevatorOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_tradition_total(c in arb_concept()) {
            prop_assert!(ElevatorTradition.get(&c).is_some());
        }

        #[test]
        fn prop_height_matches_index(f in arb_floor_index()) {
            // HeightFromGround is the identity on the floor index.
            prop_assert_eq!(HeightFromGround.get(&f), Some(f));
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = ElevatorCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == ElevatorRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_tradition_total, Verifiable);
    pr4xis::register_praxis_value!(prop_height_matches_index, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
}
