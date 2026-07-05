//! Functor: Microkernel → Bus.
//!
//! Kernel IPC *is* a message bus: Liedtke (1995) SOSP §2.2 makes IPC
//! the kernel's communication medium, and Eugster, Felber, Guerraoui &
//! Kermarrec (2003) ACM CSUR 35(2) §2 give the bus reading — a medium
//! decoupling senders from receivers, operated by an intermediary. The
//! faithful heart of this functor is that pairing: `Ipc → MessageBus`,
//! `Message → Message`, `Endpoint → Topic` (the named rendezvous is the
//! named channel), `Kernel → Broker` (Brinch Hansen 1970: every message
//! passes through the nucleus — the nucleus is the routing
//! intermediary), and the parties — a thread emits messages
//! (`Thread → Publisher`), servers and pagers serve requests arriving
//! over IPC (`UserServer`/`Pager → Subscriber`).
//!
//! Everything else is a *forgetful* reading, documented per arm: the
//! bus vocabulary has no concept of privilege, protection hardware, or
//! kernel-design contrasts, so those concepts collapse onto the
//! `Broker` (the trusted intermediary and everything that constitutes
//! or decides for it) and `MessageBus` (the medium and the domains it
//! decouples) umbrellas.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    MicrokernelCategory, MicrokernelConcept, MicrokernelRelation, MicrokernelRelationKind,
};
use crate::applied::operating_system::bus::ontology::{
    BusCategory, BusConcept, BusRelation, BusRelationKind,
};

/// Maps each microkernel concept to its message-bus image
/// (Liedtke 1995; Eugster et al. 2003).
pub struct MicrokernelToBus;

impl Functor for MicrokernelToBus {
    type Source = MicrokernelCategory;
    type Target = BusCategory;

    fn map_object(obj: &MicrokernelConcept) -> BusConcept {
        use BusConcept as B;
        use MicrokernelConcept as M;
        match obj {
            // === The faithful pairings (Liedtke 1995; Eugster et al. 2003) ===
            // Kernel IPC is the communication medium the parties share —
            // the message bus itself (Liedtke 1995 sec 2.2).
            M::Ipc => B::MessageBus,
            // The payload is the payload on both readings (Brinch
            // Hansen 1970; Hewitt et al. 1973).
            M::Message => B::Message,
            // The endpoint is the named rendezvous messages are
            // addressed to — the bus's named channel, i.e. the topic
            // (Eugster et al. 2003 sec 4.1).
            M::Endpoint => B::Topic,
            // Every message passes through the nucleus (Brinch Hansen
            // 1970): the kernel is the routing intermediary — the broker.
            M::Kernel => B::Broker,

            // === The parties ===
            // A thread is the active entity that emits messages into
            // the medium — the producer side of the bus (Eugster et al.
            // 2003): a thread publishes messages.
            M::Thread => B::Publisher,
            // A user-level server waits on an endpoint and serves what
            // arrives (Liedtke 1995 sec 3) — the consumer registered on
            // a topic: the subscriber.
            M::UserServer => B::Subscriber,
            // A pager is one such server (Liedtke 1995 sec 3) —
            // subscriber, forgetting what it pages.
            M::Pager => B::Subscriber,

            // === Documented collapse onto Broker ===
            // The scheduler is kernel machinery deciding which party
            // runs — bus vocabulary sees only the intermediary it is
            // part of; the broker absorbs it (Liedtke 1996: scheduling
            // is a kernel mechanism).
            M::Scheduler => B::Broker,

            // === Forgetful collapses onto the Broker umbrella ===
            // The kernel kinds are kinds of intermediary — the design
            // contrast between them (Liedtke 1996) is invisible to the
            // bus reading, which sees only "the broker".
            M::Microkernel | M::MonolithicKernel | M::Nucleus => B::Broker,
            // The TCB is the set of components a party must trust; in
            // pub/sub the trusted component is the intermediary
            // (Klein et al. 2009, read at the bus scale).
            M::TrustedComputingBase => B::Broker,
            // A capability is a kernel-protected token checked at every
            // mediation (Klein et al. 2009) — the authority record the
            // intermediary holds, like the broker's stored interests
            // (Eugster et al. 2003 sec 2) — forgetting its token nature.
            M::Capability => B::Broker,
            // A resource-use decision is visible to the bus only where
            // decisions live: at the intermediary (Carzaniga et al.
            // 2001: routing decisions are the service's logic) —
            // forgetting who was supposed to decide (Levin et al. 1975).
            M::Policy => B::Broker,
            // The privileged mode is the domain the intermediary
            // executes in — the bus reading keeps only "the broker's
            // side" of the split (Dijkstra 1968, forgetting the
            // hardware mode).
            M::PrivilegedMode => B::Broker,

            // === Forgetful collapses onto the MessageBus umbrella ===
            // The address space is the isolation medium that keeps the
            // parties spatially apart; the bus reading retains exactly
            // its decoupling-medium aspect (Eugster et al. 2003 sec 2:
            // space decoupling) — forgetting that it is memory.
            M::AddressSpace => B::MessageBus,
            // The policy-free primitive family: the bus IS the
            // policy-free communication mechanism (Levin et al. 1975
            // read through Eugster et al. 2003) — the abstract parent
            // collapses onto the medium.
            M::Mechanism => B::MessageBus,
            // User mode is the domain of the decoupled parties, who
            // reach each other only through the medium — the bus
            // reading keeps only "the far side of the bus" (Dijkstra
            // 1968, forgetting the hardware mode).
            M::UserMode => B::MessageBus,
        }
    }

    fn map_morphism(m: &MicrokernelRelation) -> BusRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            MicrokernelRelationKind::Identity => return BusCategory::identity(&from),
            // Privileges: the kernel implementing its primitives
            // privileged reads as the intermediary's operational
            // authority over the apparatus — whose bus-visible form is
            // routing, the broker's one privileged act on the medium
            // (Carzaniga et al. 2001).
            MicrokernelRelationKind::Privileges => BusRelationKind::Routes,
            // Isolates: the address space keeping threads apart IS
            // space decoupling — the medium decouples the parties
            // (Liedtke 1995 sec 2.1; Eugster et al. 2003 sec 2).
            MicrokernelRelationKind::Isolates => BusRelationKind::Decouples,
            // Mediates: every message passes through the nucleus
            // (Brinch Hansen 1970) — kernel-mediated transfer is the
            // broker's delivery of the message stream.
            MicrokernelRelationKind::Mediates => BusRelationKind::Delivers,
            // Grants: the capability check at each mediation reads as
            // the intermediary matching a stored authority against the
            // named channel — the registered-interest predicate
            // (Klein et al. 2009; Carzaniga et al. 2001).
            MicrokernelRelationKind::Grants => BusRelationKind::Matches,
            // RunsInUserSpace: an unprivileged server is an ordinary
            // party attached to the medium rather than operating it —
            // the subscriber's registration (Liedtke 1995 sec 3;
            // Eugster et al. 2003).
            MicrokernelRelationKind::RunsInUserSpace => BusRelationKind::Subscribes,
            // Separates: the mechanism/policy separation reads as the
            // bus's own separation discipline — decoupling (Levin et
            // al. 1975; Eugster et al. 2003 sec 2).
            MicrokernelRelationKind::Separates => BusRelationKind::Decouples,
            // The four canonical kinds map to their namesakes.
            MicrokernelRelationKind::Subsumption => BusRelationKind::Subsumption,
            MicrokernelRelationKind::Parthood => BusRelationKind::Parthood,
            MicrokernelRelationKind::Causation => BusRelationKind::Causation,
            MicrokernelRelationKind::Opposition => BusRelationKind::Opposition,
        };
        BusRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    MicrokernelToBus,
    "Liedtke (1995) SOSP; Eugster, Felber, Guerraoui & Kermarrec (2003) ACM CSUR 35(2)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<MicrokernelToBus>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn faithful_pairings() {
        // Liedtke (1995) sec 2.2 + Eugster et al. (2003): kernel IPC
        // IS the message bus, the endpoint IS the named channel.
        assert_eq!(
            MicrokernelToBus::map_object(&MicrokernelConcept::Ipc),
            BusConcept::MessageBus
        );
        assert_eq!(
            MicrokernelToBus::map_object(&MicrokernelConcept::Message),
            BusConcept::Message
        );
        assert_eq!(
            MicrokernelToBus::map_object(&MicrokernelConcept::Endpoint),
            BusConcept::Topic
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn kernel_is_the_broker() {
        // Brinch Hansen (1970): every message passes through the
        // nucleus — the kernel is the routing intermediary.
        assert_eq!(
            MicrokernelToBus::map_object(&MicrokernelConcept::Kernel),
            BusConcept::Broker
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn parties_map_to_pub_sub_roles() {
        // A thread publishes messages; servers consume what arrives on
        // their endpoints (Liedtke 1995 sec 3; Eugster et al. 2003).
        assert_eq!(
            MicrokernelToBus::map_object(&MicrokernelConcept::Thread),
            BusConcept::Publisher
        );
        for c in [MicrokernelConcept::UserServer, MicrokernelConcept::Pager] {
            assert_eq!(
                MicrokernelToBus::map_object(&c),
                BusConcept::Subscriber,
                "{c:?} serves requests arriving over IPC"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn isolation_reads_as_decoupling() {
        // Liedtke (1995) sec 2.1 read through Eugster et al. (2003)
        // sec 2: address-space isolation is space decoupling.
        let isolates = MicrokernelCategory::morphisms()
            .into_iter()
            .find(|m| m.kind() == MicrokernelRelationKind::Isolates)
            .expect("the Isolates edge exists (Liedtke 1995 sec 2.1)");
        let image = MicrokernelToBus::map_morphism(&isolates);
        assert_eq!(image.kind(), BusRelationKind::Decouples);
        assert_eq!(image.source(), BusConcept::MessageBus);
        assert_eq!(image.target(), BusConcept::Publisher);
    }
}
