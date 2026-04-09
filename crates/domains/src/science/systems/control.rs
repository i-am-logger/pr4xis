use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for ControlConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Plant,
            Self::Controller,
            Self::Sensor,
            Self::Actuator,
            Self::Setpoint,
            Self::Error,
            Self::Signal,
            Self::Disturbance,
            Self::Model,
            Self::FeedbackLoop,
        ]
    }
}

/// Types of control systems — the taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for ControlSystemKind {
    fn variants() -> Vec<Self> {
        vec![
            Self::OpenLoop,
            Self::ClosedLoop,
            Self::Cybernetic,
            Self::FirstOrderCybernetic,
            Self::SecondOrderCybernetic,
            Self::Adaptive,
        ]
    }
}

/// Relationships between control concepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ControlRelation {
    pub from: ControlConcept,
    pub to: ControlConcept,
    pub kind: ControlRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlRelationKind {
    Identity,
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
    Composed,
}

impl Relationship for ControlRelation {
    type Object = ControlConcept;
    fn source(&self) -> ControlConcept {
        self.from
    }
    fn target(&self) -> ControlConcept {
        self.to
    }
}

pub struct ControlCategory;

impl Category for ControlCategory {
    type Object = ControlConcept;
    type Morphism = ControlRelation;

    fn identity(obj: &ControlConcept) -> ControlRelation {
        ControlRelation {
            from: *obj,
            to: *obj,
            kind: ControlRelationKind::Identity,
        }
    }

    fn compose(f: &ControlRelation, g: &ControlRelation) -> Option<ControlRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == ControlRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == ControlRelationKind::Identity {
            return Some(f.clone());
        }
        Some(ControlRelation {
            from: f.from,
            to: g.to,
            kind: ControlRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<ControlRelation> {
        use ControlConcept::*;
        use ControlRelationKind::*;

        let mut m = Vec::new();

        for c in ControlConcept::variants() {
            m.push(ControlRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // The control loop: Controller → Actuator → Plant → Sensor → Error → Controller
        m.push(ControlRelation {
            from: Sensor,
            to: Plant,
            kind: Measures,
        });
        m.push(ControlRelation {
            from: Controller,
            to: Error,
            kind: ComputesFrom,
        });
        m.push(ControlRelation {
            from: Actuator,
            to: Plant,
            kind: ActsOn,
        });
        m.push(ControlRelation {
            from: Setpoint,
            to: Error,
            kind: ComparedWith,
        });
        m.push(ControlRelation {
            from: Controller,
            to: Actuator,
            kind: Carries,
        });
        m.push(ControlRelation {
            from: Sensor,
            to: Error,
            kind: Carries,
        });

        // Disturbance perturbs plant
        m.push(ControlRelation {
            from: Disturbance,
            to: Plant,
            kind: Perturbs,
        });

        // Model represents plant inside controller (Conant-Ashby theorem)
        m.push(ControlRelation {
            from: Model,
            to: Plant,
            kind: Represents,
        });
        m.push(ControlRelation {
            from: Controller,
            to: Model,
            kind: Carries,
        });

        // Feedback loop closes the causal chain
        m.push(ControlRelation {
            from: FeedbackLoop,
            to: Sensor,
            kind: Closes,
        });
        m.push(ControlRelation {
            from: FeedbackLoop,
            to: Controller,
            kind: Closes,
        });

        // Transitive: the full loop
        m.push(ControlRelation {
            from: Controller,
            to: Plant,
            kind: Composed,
        });
        m.push(ControlRelation {
            from: Sensor,
            to: Controller,
            kind: Composed,
        });
        m.push(ControlRelation {
            from: Setpoint,
            to: Controller,
            kind: Composed,
        });
        m.push(ControlRelation {
            from: Disturbance,
            to: Error,
            kind: Composed,
        });

        // Self-composed closure
        for c in ControlConcept::variants() {
            m.push(ControlRelation {
                from: c,
                to: c,
                kind: Composed,
            });
        }

        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::Category;
    use praxis::category::validate::check_category_laws;

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
