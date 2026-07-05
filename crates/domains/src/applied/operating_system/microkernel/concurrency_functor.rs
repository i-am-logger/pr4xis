//! Functor: Microkernel → Concurrency.
//!
//! Hoare (1978) CACM 21(8): communicating sequential processes IS the
//! microkernel execution model — a thread is a sequential activity, and
//! kernel IPC is the channel it communicates over (Liedtke 1995 §3.2,
//! §3.3). Those two pairings (`Thread → Process`, `Ipc → Channel`) are
//! the faithful heart of this functor.
//!
//! Everything else is a *forgetful* reading, documented per arm:
//! concurrency theory has no vocabulary for privilege, protection
//! hardware, or kernel-design contrasts, so those concepts collapse
//! onto the `Process` and `Synchronization` umbrellas, and the IPC
//! apparatus (`Message`, `Endpoint`) collapses onto `Channel`. The
//! address space maps to `MutualExclusion` — isolation read as an
//! exclusion property.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    MicrokernelCategory, MicrokernelConcept, MicrokernelRelation, MicrokernelRelationKind,
};
use crate::formal::systems::concurrency::ontology::{
    ConcurrencyCategory, ConcurrencyConcept, ConcurrencyRelation, ConcurrencyRelationKind,
};

/// Maps each microkernel concept to its concurrency-theoretic image
/// (Hoare 1978; Liedtke 1995).
pub struct MicrokernelToConcurrency;

impl Functor for MicrokernelToConcurrency {
    type Source = MicrokernelCategory;
    type Target = ConcurrencyCategory;

    fn map_object(obj: &MicrokernelConcept) -> ConcurrencyConcept {
        use ConcurrencyConcept as C;
        use MicrokernelConcept as M;
        match obj {
            // === The faithful pairings (Hoare 1978) ===
            // A thread is a sequential activity — the CSP process.
            M::Thread => C::Process,
            // Kernel IPC is the medium processes communicate over —
            // the CSP channel (Liedtke 1995 sec 3.3).
            M::Ipc => C::Channel,

            // === Documented collapses onto Channel ===
            // The payload is visible to concurrency theory only as the
            // communication itself — CSP has no payload concept
            // separate from the channel event.
            M::Message => C::Channel,
            // The rendezvous object collapses too: CSP names channels,
            // not the kernel objects that implement their ends.
            M::Endpoint => C::Channel,

            // === Isolation as exclusion (forgetful reading) ===
            // The address space excludes foreign threads from its
            // memory; read at the concurrency scale that is a mutual-
            // exclusion property — forgetting that the exclusion is
            // spatial (a mapping) rather than temporal (a lock).
            M::AddressSpace => C::MutualExclusion,

            // === Forgetful collapses onto the Process umbrella ===
            // A user server is a communicating sequential process that
            // serves requests over IPC (Liedtke 1995 sec 4) — process,
            // forgetting its service role.
            M::UserServer => C::Process,
            // A pager is one such server process (Liedtke 1995
            // sec 3.1) — forgetting what it pages.
            M::Pager => C::Process,

            // === Forgetful collapses onto the Synchronization umbrella ===
            // Seen from pure concurrency theory, a kernel's behavioural
            // content is the coordination substrate it implements
            // (Brinch Hansen 1970: the nucleus exists to provide the
            // message primitives) — everything else about it
            // (privilege, size, design) is invisible here.
            M::Kernel | M::Microkernel | M::MonolithicKernel | M::Nucleus => C::Synchronization,
            // The scheduler chooses the interleaving; concurrency
            // models that choice as nondeterminism, so the chooser
            // itself collapses onto the coordination umbrella.
            M::Scheduler => C::Synchronization,
            // The policy-free primitive family — its concrete members
            // map into channel/synchronization above; the abstract
            // parent collapses onto the umbrella.
            M::Mechanism => C::Synchronization,
            // A resource-use decision is visible here only as part of
            // the coordination discipline — forgetting who decides.
            M::Policy => C::Synchronization,
            // An access token conditions who may coordinate with whom;
            // concurrency has no authority vocabulary — forgetful.
            M::Capability => C::Synchronization,
            // CPU protection modes exist to protect the coordination
            // primitives; the mode distinction itself has no
            // concurrency reading — forgetful.
            M::PrivilegedMode | M::UserMode => C::Synchronization,
            // The TCB is the set of components implementing the
            // coordination substrate — collapses onto it.
            M::TrustedComputingBase => C::Synchronization,
        }
    }

    fn map_morphism(m: &MicrokernelRelation) -> ConcurrencyRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            MicrokernelRelationKind::Identity => return ConcurrencyCategory::identity(&from),
            // Privileges: the kernel upholding the integrity of its
            // primitives reads as enforcement (Dijkstra 1968: the
            // lower layer enforces the discipline for the upper).
            MicrokernelRelationKind::Privileges => ConcurrencyRelationKind::Enforces,
            // Isolates: the address space enforcing exclusion on its
            // threads — the exclusion guarantee acting.
            MicrokernelRelationKind::Isolates => ConcurrencyRelationKind::Enforces,
            // Mediates: kernel-mediated messaging reads as the
            // communication path itself (Hoare 1978).
            MicrokernelRelationKind::Mediates => ConcurrencyRelationKind::CommunicatesVia,
            // Grants: possession of the capability is a necessary
            // condition for the access (Levin et al. 1975; Klein et
            // al. 2009) — necessity is the closest concurrency kind.
            MicrokernelRelationKind::Grants => ConcurrencyRelationKind::NecessaryFor,
            // RunsInUserSpace: an unprivileged server respects the mode
            // discipline — it keeps to its side of the split.
            MicrokernelRelationKind::RunsInUserSpace => ConcurrencyRelationKind::Respects,
            // Separates: the mechanism keeps to its side of the
            // policy/mechanism separation — a design discipline it
            // respects, deliberately NOT Opposition (Levin et al.
            // 1975: separation, not contrariety).
            MicrokernelRelationKind::Separates => ConcurrencyRelationKind::Respects,
            // The four canonical kinds map to their namesakes.
            MicrokernelRelationKind::Subsumption => ConcurrencyRelationKind::Subsumption,
            MicrokernelRelationKind::Parthood => ConcurrencyRelationKind::Parthood,
            MicrokernelRelationKind::Causation => ConcurrencyRelationKind::Causation,
            MicrokernelRelationKind::Opposition => ConcurrencyRelationKind::Opposition,
        };
        ConcurrencyRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    MicrokernelToConcurrency,
    "Hoare (1978) CACM 21(8); Liedtke (1995) SOSP"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<MicrokernelToConcurrency>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn faithful_pairings() {
        // Hoare (1978): thread + IPC IS the CSP process + channel model.
        assert_eq!(
            MicrokernelToConcurrency::map_object(&MicrokernelConcept::Thread),
            ConcurrencyConcept::Process
        );
        assert_eq!(
            MicrokernelToConcurrency::map_object(&MicrokernelConcept::Ipc),
            ConcurrencyConcept::Channel
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn isolation_reads_as_exclusion() {
        assert_eq!(
            MicrokernelToConcurrency::map_object(&MicrokernelConcept::AddressSpace),
            ConcurrencyConcept::MutualExclusion
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn ipc_apparatus_collapses_onto_channel() {
        for c in [MicrokernelConcept::Message, MicrokernelConcept::Endpoint] {
            assert_eq!(
                MicrokernelToConcurrency::map_object(&c),
                ConcurrencyConcept::Channel,
                "{c:?} collapses onto Channel"
            );
        }
    }
}
