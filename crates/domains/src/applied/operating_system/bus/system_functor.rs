//! Functor: Bus → System.
//!
//! A message bus is a system in the sense of von Bertalanffy (1968):
//! publishers, subscribers, and actors are its components; messages,
//! events, and topics the interactions binding them; the broker (and
//! each routing discipline) the controller regulating those
//! interactions; subscriptions and delivery guarantees the constraints
//! on admissible behaviour; the bus itself — the decoupling medium —
//! the boundary separating the parties (Eugster et al. 2003 §2); and
//! virtual synchrony the homeostasis that keeps every operational
//! member's view identical under failures (Birman & Joseph 1987).
//!
//! # Literature
//!
//! - **von Bertalanffy (1968)** *General System Theory* — components,
//!   interactions, boundaries, emergence.
//! - **Eugster, Felber, Guerraoui & Kermarrec (2003)** ACM CSUR 35(2) —
//!   the decoupling analysis this mapping preserves: the bus is a
//!   *boundary*, never a direct publisher–subscriber link.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{BusCategory, BusConcept, BusRelation, BusRelationKind};
use crate::formal::systems::ontology::{
    SystemCategory, SystemConcept, SystemRelation, SystemRelationKind,
};

/// Maps each bus concept to the systems-thinking role it plays
/// (von Bertalanffy 1968; Eugster et al. 2003).
pub struct BusToSystem;

impl Functor for BusToSystem {
    type Source = BusCategory;
    type Target = SystemCategory;

    fn map_object(obj: &BusConcept) -> SystemConcept {
        use BusConcept as B;
        match obj {
            // The communicating parties are the system's elements.
            B::Publisher | B::Subscriber | B::Actor => SystemConcept::Component,
            // What travels between them makes the parts a system.
            B::Message | B::Event | B::Topic => SystemConcept::Interaction,
            // The mailbox is the actor's buffered configuration at a
            // point in time (Hewitt et al. 1973).
            B::Mailbox => SystemConcept::State,
            // The registered predicate and the delivery guarantees
            // restrict which behaviours are admissible.
            B::Subscription
            | B::DeliveryGuarantee
            | B::AtMostOnce
            | B::AtLeastOnce
            | B::ExactlyOnce => SystemConcept::Constraint,
            // The broker and each routing discipline regulate the flow
            // (Ashby 1956 §10: the regulator).
            B::Broker | B::TopicBasedRouting | B::ContentBasedRouting => SystemConcept::Controller,
            // The bus is the boundary separating the decoupled parties
            // (Eugster et al. 2003 §2), and decoupling is what the
            // boundary does.
            B::MessageBus | B::Decoupling => SystemConcept::Boundary,
            // Virtual synchrony keeps every operational member's view
            // identical under failures — the stabilising mechanism
            // (Birman & Joseph 1987).
            B::VirtualSynchrony => SystemConcept::Homeostasis,
        }
    }

    fn map_morphism(m: &BusRelation) -> SystemRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            BusRelationKind::Identity => return SystemCategory::identity(&from),
            // A publisher emitting an event brings the interaction
            // about.
            BusRelationKind::Publishes => SystemRelationKind::Causation,
            // Registering interest composes the subscriber into the
            // system's interaction structure.
            BusRelationKind::Subscribes => SystemRelationKind::ComposesInto,
            // The broker routing messages is the regulator acting on
            // the flow.
            BusRelationKind::Routes => SystemRelationKind::Regulates,
            // A delivery changes the receiving component's state
            // (Hewitt et al. 1973: message arrival drives behaviour).
            BusRelationKind::Delivers => SystemRelationKind::Changes,
            // A subscription predicate governs which interactions pass
            // (Carzaniga et al. 2001).
            BusRelationKind::Matches => SystemRelationKind::Governs,
            // The bus-as-boundary separates the decoupled components
            // (Eugster et al. 2003 §2).
            BusRelationKind::Decouples => SystemRelationKind::Separates,
            // The four canonical Relations-ontology kinds map to their
            // namesakes (Smith 2005 OBO-RO).
            BusRelationKind::Subsumption => SystemRelationKind::Subsumption,
            BusRelationKind::Parthood => SystemRelationKind::Parthood,
            BusRelationKind::Causation => SystemRelationKind::Causation,
            BusRelationKind::Opposition => SystemRelationKind::Opposition,
        };
        SystemRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    BusToSystem,
    "von Bertalanffy (1968); Eugster et al. (2003) ACM CSUR 35(2)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<BusToSystem>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn broker_is_the_controller() {
        assert_eq!(
            BusToSystem::map_object(&BusConcept::Broker),
            SystemConcept::Controller
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn bus_is_a_boundary() {
        // The decoupling medium is a boundary, never a direct link
        // (Eugster et al. 2003 sec 2).
        for c in [BusConcept::MessageBus, BusConcept::Decoupling] {
            assert_eq!(
                BusToSystem::map_object(&c),
                SystemConcept::Boundary,
                "{c:?} should be a boundary"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn delivery_guarantees_are_constraints() {
        for c in [
            BusConcept::DeliveryGuarantee,
            BusConcept::AtMostOnce,
            BusConcept::AtLeastOnce,
            BusConcept::ExactlyOnce,
            BusConcept::Subscription,
        ] {
            assert_eq!(
                BusToSystem::map_object(&c),
                SystemConcept::Constraint,
                "{c:?} should be a constraint"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn virtual_synchrony_is_homeostasis() {
        assert_eq!(
            BusToSystem::map_object(&BusConcept::VirtualSynchrony),
            SystemConcept::Homeostasis
        );
    }
}
