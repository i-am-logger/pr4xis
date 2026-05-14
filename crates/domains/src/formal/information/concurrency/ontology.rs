//! Concurrency — agents, shared resources, synchronisation, and hazards.
//!
//! Concurrency is not just "threads" — it is the fundamental concept of
//! multiple agents acting on shared resources with coordination. Chess is
//! concurrent (two players, one board, turn-taking); traffic is concurrent
//! (many cars, shared intersections, signal control); conversation is
//! concurrent (two speakers, shared discourse, turn-taking).
//!
//! # Literature
//!
//! - **Hoare (1978)** "Communicating Sequential Processes",
//!   *Communications of the ACM* 21(8):666-677 — CSP; processes
//!   synchronise on shared channels.
//! - **Milner (1980)** *A Calculus of Communicating Systems*, LNCS 92 —
//!   CCS; agents, actions, observation equivalence.
//! - **Hewitt (1973)** "A Universal Modular ACTOR Formalism for Artificial
//!   Intelligence", IJCAI-73 — actor model: agents react to messages.
//! - **Lamport (1978)** "Time, Clocks, and the Ordering of Events in a
//!   Distributed System", *Communications of the ACM* 21(7):558-565 —
//!   happens-before; race conditions arise from unordered observations.
//! - **Coffman, Elphick & Shoshani (1971)** "System Deadlocks",
//!   *Computing Surveys* 3(2):67-78 — deadlock arises from circular
//!   wait on synchronisation.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Concurrency",
    source: "Hoare (1978) Communicating Sequential Processes, CACM 21(8); Milner (1980) A Calculus of Communicating Systems, LNCS 92; Hewitt (1973) A Universal Modular ACTOR Formalism for AI, IJCAI-73; Lamport (1978) Time, Clocks, and the Ordering of Events in a Distributed System, CACM 21(7); Coffman, Elphick & Shoshani (1971) System Deadlocks, Computing Surveys 3(2)",

    concepts: [
        Agent,
        SharedResource,
        Action,
        Synchronization,
        State,
        Protocol,
        Deadlock,
        RaceCondition,
        Future,
        Message,
    ],

    labels: {
        Agent: ("en", "Agent",
            "Hewitt (1973): an entity that can act — a player, a car, a speaker, a process."),
        SharedResource: ("en", "Shared resource",
            "Hoare (1978): something agents compete for or share — the board, the road, the channel."),
        Action: ("en", "Action",
            "Milner (1980) CCS: an action performed by an agent on a shared resource."),
        Synchronization: ("en", "Synchronization",
            "Hoare (1978) CSP: a mechanism that controls who can act when — turn-taking, locks, semaphores, traffic signals."),
        State: ("en", "State",
            "Milner (1980): the current configuration of all agents and the shared resource."),
        Protocol: ("en", "Protocol",
            "Hoare (1978) §3: a rule about what an agent is allowed to do — in chess legal moves, in traffic right of way."),
        Deadlock: ("en", "Deadlock",
            "Coffman, Elphick & Shoshani (1971): agents cannot proceed because each is waiting for another. Chess prevents it by turn-taking; traffic exhibits gridlock."),
        RaceCondition: ("en", "Race condition",
            "Lamport (1978): the outcome depends on the order of concurrent actions — chess avoids it via strict alternation; traffic exhibits it at unsignalled intersections."),
        Future: ("en", "Future",
            "Hewitt (1973): a value that will exist after an action completes — the opponent's response, the light change, the server reply."),
        Message: ("en", "Message",
            "Hewitt (1973): a message passed between agents — a move announced, a signal displayed, an utterance spoken."),
    },

    edges: [
        // Hoare (1978) CSP: agent acts on shared resource.
        (Agent, SharedResource, ActsOn),
        // Synchronization controls Agent (who can act when).
        (Synchronization, Agent, Controls),
        // Protocol governs Action.
        (Protocol, Action, Governs),
        // Action changes State.
        (Action, State, Changes),
        // Action produces Message (Hewitt 1973 actor model).
        (Action, Message, Produces),
        // Message becomes Future once dispatched.
        (Message, Future, Becomes),
        // Coffman et al. (1971): Deadlock arises from Synchronization.
        (Synchronization, Deadlock, ArisesFrom),
        // Lamport (1978): RaceCondition arises from unsynchronised SharedResource access.
        (SharedResource, RaceCondition, UnsynchronizedAccess),
    ],
}

/// Quality: whether a concurrency concept represents a hazard. Coffman et
/// al. (1971) classify Deadlock and RaceCondition as the two canonical
/// concurrency hazards.
#[derive(Debug, Clone)]
pub struct IsHazard;

impl Quality for IsHazard {
    type Individual = ConcurrencyConcept;
    type Value = bool;

    fn get(&self, c: &ConcurrencyConcept) -> Option<bool> {
        use ConcurrencyConcept as C;
        Some(matches!(c, C::Deadlock | C::RaceCondition))
    }
}

impl Ontology for ConcurrencyOntology {
    type Cat = ConcurrencyCategory;
    type Qual = IsHazard;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<ConcurrencyCategory>();
    }

    #[test]
    fn ontology_validates() {
        ConcurrencyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn ten_concepts() {
        assert_eq!(ConcurrencyConcept::variants().len(), 10);
    }

    #[test]
    fn agent_acts_on_shared_resource() {
        assert!(ConcurrencyCategory::morphisms().iter().any(|m| {
            m.source() == ConcurrencyConcept::Agent
                && m.target() == ConcurrencyConcept::SharedResource
                && m.kind() == ConcurrencyRelationKind::ActsOn
        }));
    }

    #[test]
    fn deadlock_arises_from_synchronization() {
        // Coffman, Elphick & Shoshani (1971).
        assert!(ConcurrencyCategory::morphisms().iter().any(|m| {
            m.source() == ConcurrencyConcept::Synchronization
                && m.target() == ConcurrencyConcept::Deadlock
        }));
    }

    #[test]
    fn race_condition_arises_from_shared_resource() {
        // Lamport (1978).
        assert!(ConcurrencyCategory::morphisms().iter().any(|m| {
            m.source() == ConcurrencyConcept::SharedResource
                && m.target() == ConcurrencyConcept::RaceCondition
        }));
    }

    #[test]
    fn hazards_marked() {
        assert_eq!(IsHazard.get(&ConcurrencyConcept::Deadlock), Some(true));
        assert_eq!(IsHazard.get(&ConcurrencyConcept::RaceCondition), Some(true));
        assert_eq!(IsHazard.get(&ConcurrencyConcept::Agent), Some(false));
    }

    fn arb_concept() -> impl Strategy<Value = ConcurrencyConcept> {
        proptest::sample::select(ConcurrencyConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in ConcurrencyCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ConcurrencyOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_hazard_total(c in arb_concept()) {
            prop_assert!(IsHazard.get(&c).is_some());
        }
    }
}
