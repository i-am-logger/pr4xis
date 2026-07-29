use pr4xis::category::Functor;

use super::ontology::*;
use crate::formal::systems::control::*;

/// Functor: ControlTheory → Control.
///
/// `control_theory` (this module) is the classical, frequency-domain
/// instance of feedback control (Åström & Murray 2008; Ogata 2010) —
/// `PidController`, `Adwin`, transfer functions. `formal::systems::control`
/// (Wiener 1948; Ashby 1956; Conant & Ashby 1970) is the general
/// systems-theoretic vocabulary that spans beyond linear control. This
/// functor embeds the classical instance into the general framework:
/// every classical-control concept IS a case of the general cybernetic
/// loop, proving the textbook PID/frequency-domain treatment is a
/// specialization of Ashby's cybernetics rather than an unrelated theory.
///
/// The object mapping:
/// - Plant → Plant, Controller → Controller, Sensor → Sensor,
///   Actuator → Actuator, Error → Error (direct correspondence — the
///   same loop-component roles under different vocabularies).
/// - Reference → Setpoint (Åström & Murray call it "reference"; Ashby's
///   general framework and `control.rs` call the same role "setpoint").
/// - Feedback → FeedbackLoop (the return path that closes the loop).
/// - DriftDetector → Sensor: Bifet & Gavaldà's ADWIN is a specialized
///   *measuring* instrument (it monitors a signal's distribution rather
///   than the plant's raw output), the same loop role Ashby's Sensor
///   occupies — a sensor for a statistical property instead of a
///   physical one, not a distinct role in the loop.
///
/// The kind mapping is the identity on kind *names*: `Parthood`,
/// `Opposition`, `Subsumption`, `Causation`, and `Identity` are the
/// same canonical Relations-ontology vocabulary (Smith 2005 OBO-RO) in
/// both categories — `control_theory`'s edges are built entirely from
/// `has_a`/`opposes` sugar, so no domain-specific kind ever needs a
/// forced re-projection.
pub struct ControlTheoryToControl;

impl Functor for ControlTheoryToControl {
    type Source = ControlTheoryCategory;
    type Target = ControlCategory;

    fn map_object(obj: &ControlTheoryConcept) -> ControlConcept {
        match obj {
            ControlTheoryConcept::Plant => ControlConcept::Plant,
            ControlTheoryConcept::Controller => ControlConcept::Controller,
            ControlTheoryConcept::Sensor => ControlConcept::Sensor,
            ControlTheoryConcept::Actuator => ControlConcept::Actuator,
            ControlTheoryConcept::Reference => ControlConcept::Setpoint,
            ControlTheoryConcept::Error => ControlConcept::Error,
            ControlTheoryConcept::Feedback => ControlConcept::FeedbackLoop,
            ControlTheoryConcept::DriftDetector => ControlConcept::Sensor,
        }
    }

    fn map_morphism(m: &ControlTheoryRelation) -> ControlRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            ControlTheoryRelationKind::Identity => ControlRelationKind::Identity,
            ControlTheoryRelationKind::Parthood => ControlRelationKind::Parthood,
            ControlTheoryRelationKind::Opposition => ControlRelationKind::Opposition,
            ControlTheoryRelationKind::Subsumption => ControlRelationKind::Subsumption,
            ControlTheoryRelationKind::Causation => ControlRelationKind::Causation,
        };
        ControlRelation { from, to, kind }
    }
}
pr4xis::register_functor!(ControlTheoryToControl);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn control_theory_to_control_functor_laws() {
        assert_functor_laws::<ControlTheoryToControl>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn plant_maps_to_plant() {
        assert_eq!(
            ControlTheoryToControl::map_object(&ControlTheoryConcept::Plant),
            ControlConcept::Plant
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn reference_maps_to_setpoint() {
        assert_eq!(
            ControlTheoryToControl::map_object(&ControlTheoryConcept::Reference),
            ControlConcept::Setpoint
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn feedback_maps_to_feedback_loop() {
        assert_eq!(
            ControlTheoryToControl::map_object(&ControlTheoryConcept::Feedback),
            ControlConcept::FeedbackLoop
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn drift_detector_maps_to_sensor() {
        // ADWIN (Bifet & Gavalda 2007) is a measuring instrument, the
        // same loop role Ashby's Sensor occupies.
        assert_eq!(
            ControlTheoryToControl::map_object(&ControlTheoryConcept::DriftDetector),
            ControlConcept::Sensor
        );
    }
}
