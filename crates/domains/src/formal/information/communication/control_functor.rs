use pr4xis::category::Functor;

use super::ontology::*;
use crate::formal::systems::control::*;

// Communication → Control functor.
//
// Proof that communication IS cybernetic control (Wiener 1948).
// "Cybernetics: Or Control and Communication in the Animal and the Machine"
// — the title IS the functor.
//
// The mapping:
//   Sender     → Controller (the agent producing control actions / messages)
//   Receiver   → Plant (the system being acted upon / informed)
//   Message    → Signal (information flowing in the loop)
//   Channel    → Actuator (the medium that delivers the action)
//   Code       → Model (the shared representation — Conant-Ashby)
//   Noise      → Disturbance (external perturbation)
//   Feedback   → FeedbackLoop (the return path)
//   Context    → Setpoint (the shared reference / desired state)

pub struct CommunicationToControl;

impl Functor for CommunicationToControl {
    type Source = CommunicationCategory;
    type Target = ControlCategory;

    fn map_object(obj: &CommunicationConcept) -> ControlConcept {
        match obj {
            CommunicationConcept::Sender => ControlConcept::Controller,
            CommunicationConcept::Receiver => ControlConcept::Plant,
            CommunicationConcept::Message => ControlConcept::Signal,
            CommunicationConcept::Channel => ControlConcept::Actuator,
            CommunicationConcept::Code => ControlConcept::Model,
            CommunicationConcept::Noise => ControlConcept::Disturbance,
            CommunicationConcept::Feedback => ControlConcept::FeedbackLoop,
            CommunicationConcept::Context => ControlConcept::Setpoint,
        }
    }

    fn map_morphism(m: &CommunicationRelation) -> ControlRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            CommunicationRelationKind::Identity => ControlRelationKind::Identity,
            CommunicationRelationKind::Produces => ControlRelationKind::Carries,
            CommunicationRelationKind::TransmittedThrough => ControlRelationKind::ActsOn,
            CommunicationRelationKind::Interprets => ControlRelationKind::Measures,
            CommunicationRelationKind::EncodesDecodes => ControlRelationKind::Represents,
            CommunicationRelationKind::Corrupts => ControlRelationKind::Perturbs,
            CommunicationRelationKind::FlowsBack => ControlRelationKind::Closes,
            CommunicationRelationKind::Grounds => ControlRelationKind::ComparedWith,
            CommunicationRelationKind::Shares => ControlRelationKind::Carries,
            CommunicationRelationKind::Composed => ControlRelationKind::Composed,
        };
        ControlRelation { from, to, kind }
    }
}
pr4xis::register_functor!(CommunicationToControl);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws() {
        check_functor_laws::<CommunicationToControl>().unwrap();
    }

    #[test]
    fn sender_is_controller() {
        assert_eq!(
            CommunicationToControl::map_object(&CommunicationConcept::Sender),
            ControlConcept::Controller
        );
    }

    #[test]
    fn feedback_is_feedback_loop() {
        assert_eq!(
            CommunicationToControl::map_object(&CommunicationConcept::Feedback),
            ControlConcept::FeedbackLoop
        );
    }

    #[test]
    fn code_is_model() {
        // Conant-Ashby: the shared code IS the model
        assert_eq!(
            CommunicationToControl::map_object(&CommunicationConcept::Code),
            ControlConcept::Model
        );
    }
}
