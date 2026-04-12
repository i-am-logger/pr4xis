use pr4xis::category::Entity;
use pr4xis::define_category;

// Control Systems Ontology — the science of feedback and regulation.
//
// Control theory is the GENERAL science. Cybernetics is a SPECIFIC TYPE:
// control systems that involve communication (Wiener 1948).
//
//   Control System (general)
//     ├── Classical Control (plant, controller, PID)
//     ├── Cybernetic System (control + communication — Wiener 1948)
//     │   ├── First-order cybernetics (observing systems)
//     │   └── Second-order cybernetics (observing the observer — von Foerster)
//     └── Adaptive Control (changes own parameters — Ashby's ultrastability)
//
// Three key theorems:
// 1. Requisite Variety (Ashby 1956): controller variety >= disturbance variety
// 2. Good Regulator (Conant & Ashby 1970): every good regulator must be a model
//    of its system — THIS IS WHY THE ENGINE NEEDS AN ONTOLOGY
// 3. Perceptual Control (Powers 1973): systems control inputs, not outputs
//
// References:
// - Wiener, Cybernetics (1948) — control + communication
// - Ashby, An Introduction to Cybernetics (1956) — requisite variety
// - Conant & Ashby, Every Good Regulator (1970) — the regulator theorem
// - Powers, Behavior: The Control of Perception (1973)
// - Beer, Brain of the Firm (1972) — Viable System Model
// - von Foerster, Observing Systems (1981) — second-order cybernetics
// - Åström & Murray, Feedback Systems (2008) — modern treatment

/// Core concepts of a control system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ControlConcept {
    /// The system being controlled — the "thing in the world."
    Plant,
    /// The decision-maker — computes control action from error.
    Controller,
    /// Measures the plant's actual output state.
    Sensor,
    /// Applies the control action to the plant.
    Actuator,
    /// The desired state — what the system "wants."
    Setpoint,
    /// The difference between setpoint and measured output: e(t) = r(t) - y(t).
    Error,
    /// Information flowing between components.
    Signal,
    /// External perturbation acting on the plant.
    Disturbance,
    /// The controller's representation of the plant.
    /// Conant & Ashby (1970): every good regulator must be a model.
    Model,
    /// The return path from output to input — closes the causal loop.
    FeedbackLoop,
}

/// Types of control systems — the taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ControlSystemKind {
    /// No feedback — controller acts "blind."
    OpenLoop,
    /// Output measured and fed back — the standard control loop.
    ClosedLoop,
    /// Closed-loop + communication between controller and plant.
    /// Wiener (1948): cybernetics = control + communication.
    Cybernetic,
    /// First-order cybernetics: observing systems (von Foerster).
    FirstOrderCybernetic,
    /// Second-order cybernetics: the observer observing itself.
    SecondOrderCybernetic,
    /// Changes its own parameters when the inner loop fails.
    /// Ashby's ultrastability: fast inner loop + slow outer restructuring loop.
    Adaptive,
}

define_category! {
    pub ControlCategory {
        entity: ControlConcept,
        relation: ControlRelation,
        kind: ControlRelationKind,
        kinds: [
            /// Sensor measures Plant output.
            Measures,
            /// Controller computes from Error.
            ComputesFrom,
            /// Actuator acts on Plant.
            ActsOn,
            /// Setpoint compared with Measured to produce Error.
            ComparedWith,
            /// Disturbance perturbs Plant.
            Perturbs,
            /// Model represents Plant inside Controller.
            Represents,
            /// FeedbackLoop closes the causal chain.
            Closes,
            /// Signal carries information between components.
            Carries,
        ],
        edges: [
            // The control loop: Controller → Actuator → Plant → Sensor → Error → Controller
            (Sensor, Plant, Measures),
            (Controller, Error, ComputesFrom),
            (Actuator, Plant, ActsOn),
            (Setpoint, Error, ComparedWith),
            (Controller, Actuator, Carries),
            (Sensor, Error, Carries),
            // Disturbance perturbs plant
            (Disturbance, Plant, Perturbs),
            // Model represents plant inside controller (Conant-Ashby theorem)
            (Model, Plant, Represents),
            (Controller, Model, Carries),
            // Feedback loop closes the causal chain
            (FeedbackLoop, Sensor, Closes),
            (FeedbackLoop, Controller, Closes),
        ],
        composed: [
            // Transitive: the full loop
            (Controller, Plant),
            (Sensor, Controller),
            (Setpoint, Controller),
            (Disturbance, Error),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<ControlCategory>().unwrap();
    }

    #[test]
    fn ten_concepts() {
        assert_eq!(ControlConcept::variants().len(), 10);
    }

    #[test]
    fn six_control_system_kinds() {
        assert_eq!(ControlSystemKind::variants().len(), 6);
    }

    #[test]
    fn sensor_measures_plant() {
        let morphisms = ControlCategory::morphisms();
        assert!(morphisms.iter().any(|m| m.from == ControlConcept::Sensor
            && m.to == ControlConcept::Plant
            && m.kind == ControlRelationKind::Measures));
    }

    #[test]
    fn actuator_acts_on_plant() {
        let morphisms = ControlCategory::morphisms();
        assert!(morphisms.iter().any(|m| m.from == ControlConcept::Actuator
            && m.to == ControlConcept::Plant
            && m.kind == ControlRelationKind::ActsOn));
    }

    #[test]
    fn model_represents_plant() {
        // Conant & Ashby (1970): the controller's model represents the plant
        let morphisms = ControlCategory::morphisms();
        assert!(morphisms.iter().any(|m| m.from == ControlConcept::Model
            && m.to == ControlConcept::Plant
            && m.kind == ControlRelationKind::Represents));
    }

    #[test]
    fn controller_reaches_plant_through_composition() {
        // Controller → Actuator → Plant composes
        let morphisms = ControlCategory::morphisms();
        assert!(
            morphisms
                .iter()
                .any(|m| m.from == ControlConcept::Controller
                    && m.to == ControlConcept::Plant
                    && m.kind == ControlRelationKind::Composed)
        );
    }

    #[test]
    fn feedback_closes_loop() {
        let morphisms = ControlCategory::morphisms();
        assert!(morphisms.iter().any(
            |m| m.from == ControlConcept::FeedbackLoop && m.kind == ControlRelationKind::Closes
        ));
    }
}
