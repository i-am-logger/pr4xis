//! Functor: Microkernel → System.
//!
//! A kernel is a system in the sense of von Bertalanffy (1968): threads
//! and servers are its components, IPC objects its interactions, the
//! kernels and their mechanisms its controllers, capabilities and
//! policy its constraints, and the protection structure — address
//! spaces, CPU modes, the trusted computing base — its boundaries.
//! The functor makes Liedtke (1995)'s architectural reading a verified
//! structure-preserving map instead of an analogy.
//!
//! Every object-map choice is documented on its arm.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    MicrokernelCategory, MicrokernelConcept, MicrokernelRelation, MicrokernelRelationKind,
};
use crate::formal::systems::ontology::{
    SystemCategory, SystemConcept, SystemRelation, SystemRelationKind,
};

/// Maps each microkernel concept to the systems-thinking role it plays
/// (von Bertalanffy 1968; Liedtke 1995).
pub struct MicrokernelToSystem;

impl Functor for MicrokernelToSystem {
    type Source = MicrokernelCategory;
    type Target = SystemCategory;

    fn map_object(obj: &MicrokernelConcept) -> SystemConcept {
        use MicrokernelConcept as M;
        match obj {
            // The executing activities and the user-level services are
            // the system's elements (Liedtke 1995 sec 3.2, sec 4).
            M::Thread | M::UserServer | M::Pager => SystemConcept::Component,
            // IPC, its payload, and its rendezvous object are the
            // relational glue between components (Liedtke 1995 sec 3.3;
            // Brinch Hansen 1970; Klein et al. 2009).
            M::Ipc | M::Message | M::Endpoint => SystemConcept::Interaction,
            // The kernels are the regulator: they decide what runs,
            // what communicates, and what is reachable — the System
            // ontology's Controller role. The scheduler and the
            // abstract Mechanism parent are the regulator's policy-free
            // instruments (Liedtke 1996; Levin et al. 1975).
            M::Kernel
            | M::Microkernel
            | M::MonolithicKernel
            | M::Nucleus
            | M::Scheduler
            | M::Mechanism => SystemConcept::Controller,
            // Capabilities restrict which accesses are admissible;
            // policy restricts how resources are used — both are the
            // System ontology's Constraint role (Klein et al. 2009;
            // Levin et al. 1975).
            M::Capability | M::Policy => SystemConcept::Constraint,
            // The protection structure demarcates: the address space is
            // the isolation boundary (Liedtke 1995 sec 3.1), the two
            // CPU modes are the privilege boundary (Dijkstra 1968), and
            // the TCB is the trust boundary (Klein et al. 2009).
            M::AddressSpace | M::PrivilegedMode | M::UserMode | M::TrustedComputingBase => {
                SystemConcept::Boundary
            }
        }
    }

    fn map_morphism(m: &MicrokernelRelation) -> SystemRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            MicrokernelRelationKind::Identity => return SystemCategory::identity(&from),
            // Privileges: the kernel (Controller) exclusively
            // implementing a primitive is the regulator acting on what
            // it controls (Liedtke 1995 sec 2).
            MicrokernelRelationKind::Privileges => SystemRelationKind::Regulates,
            // Isolates: the address space (Boundary) demarcating its
            // threads (Component) is exactly the System ontology's
            // Boundary-Separates-Component edge.
            MicrokernelRelationKind::Isolates => SystemRelationKind::Separates,
            // Mediates: kernel-mediated messaging restricts which
            // interactions are admissible — governance of the
            // interaction by the controller (Brinch Hansen 1970).
            MicrokernelRelationKind::Mediates => SystemRelationKind::Governs,
            // Grants: a capability (Constraint) governing what a
            // subject may reach on a kernel object (Klein et al. 2009).
            MicrokernelRelationKind::Grants => SystemRelationKind::Governs,
            // RunsInUserSpace: a server composing into the unprivileged
            // side of the system's boundary (Liedtke 1995 sec 4).
            MicrokernelRelationKind::RunsInUserSpace => SystemRelationKind::ComposesInto,
            // Separates: policy/mechanism separation is a demarcation,
            // name-preserved into the target (Levin et al. 1975) —
            // deliberately NOT Opposition.
            MicrokernelRelationKind::Separates => SystemRelationKind::Separates,
            // The four canonical Relations-ontology kinds map to their
            // target namesakes.
            MicrokernelRelationKind::Subsumption => SystemRelationKind::Subsumption,
            MicrokernelRelationKind::Parthood => SystemRelationKind::Parthood,
            MicrokernelRelationKind::Causation => SystemRelationKind::Causation,
            MicrokernelRelationKind::Opposition => SystemRelationKind::Opposition,
        };
        SystemRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    MicrokernelToSystem,
    "von Bertalanffy (1968) General System Theory; Liedtke (1995) SOSP"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<MicrokernelToSystem>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn kernels_map_to_controller() {
        for c in [
            MicrokernelConcept::Kernel,
            MicrokernelConcept::Microkernel,
            MicrokernelConcept::MonolithicKernel,
            MicrokernelConcept::Nucleus,
            MicrokernelConcept::Scheduler,
            MicrokernelConcept::Mechanism,
        ] {
            assert_eq!(
                MicrokernelToSystem::map_object(&c),
                SystemConcept::Controller,
                "{c:?} should be a Controller"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn protection_structure_maps_to_boundary() {
        for c in [
            MicrokernelConcept::AddressSpace,
            MicrokernelConcept::PrivilegedMode,
            MicrokernelConcept::UserMode,
            MicrokernelConcept::TrustedComputingBase,
        ] {
            assert_eq!(
                MicrokernelToSystem::map_object(&c),
                SystemConcept::Boundary,
                "{c:?} should be a Boundary"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn ipc_objects_map_to_interaction() {
        for c in [
            MicrokernelConcept::Ipc,
            MicrokernelConcept::Message,
            MicrokernelConcept::Endpoint,
        ] {
            assert_eq!(
                MicrokernelToSystem::map_object(&c),
                SystemConcept::Interaction,
                "{c:?} should be an Interaction"
            );
        }
    }
}
