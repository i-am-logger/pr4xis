//! Microkernel — the minimality principle of kernel construction.
//!
//! The source traditions this ontology draws on:
//!
//! - **Dijkstra (1968)** *The Structure of the "THE"-Multiprogramming
//!   System*, CACM 11(5) — the layered privileged structure beneath all
//!   other software.
//! - **Brinch Hansen (1970)** *The Nucleus of a Multiprogramming
//!   System*, CACM 13(4) — the minimal core: processes plus message
//!   primitives; everything else outside.
//! - **Levin, Cohen, Corwin, Pollack & Wulf (1975)** *Policy/Mechanism
//!   Separation in HYDRA*, SOSP — mechanisms are policy-free primitives;
//!   resource-use decisions are kept out of the kernel.
//! - **Liedtke (1995)** *On µ-Kernel Construction*, SOSP — the
//!   minimality principle (§2): a concept is tolerated inside the
//!   kernel only if moving it out would prevent required functionality;
//!   the kernel provides only address spaces (§2.1), threads and IPC
//!   (§2.2); servers, pagers, and drivers run in user space (§3,
//!   Flexibility).
//! - **Liedtke (1996)** *Toward Real Microkernels*, CACM 39(9) — the
//!   monolithic contrast and the scheduling mechanism.
//! - **Klein et al. (2009)** *seL4: Formal Verification of an OS
//!   Kernel*, SOSP — capabilities, endpoints, and the minimised trusted
//!   computing base.
//!
//! The four domain axioms are graph-structural claims about this
//! category, each falsifiable by adding or removing an edge; the
//! runtime guarantees they describe (isolation, kernel mediation) are
//! exercised by the fixtures in [`super::engine`].

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Microkernel",
    source: "Liedtke (1995) SOSP; Liedtke (1996) CACM 39(9); Klein et al. (2009) SOSP; Brinch Hansen (1970) CACM 13(4); Dijkstra (1968) CACM 11(5); Levin et al. (1975) SOSP",

    concepts: [
        // === Kernels (Dijkstra 1968; Brinch Hansen 1970; Liedtke 1995, 1996) ===
        Kernel,
        Microkernel,
        MonolithicKernel,
        Nucleus,

        // === The three privileged abstractions (Liedtke 1995 §2.1-2.3) ===
        AddressSpace,
        Thread,
        Ipc,

        // === Kernel objects and payloads (Klein et al. 2009; Brinch Hansen 1970) ===
        Capability,
        Message,
        Endpoint,

        // === User space (Liedtke 1995 §3; Haertig et al. 1997) ===
        UserServer,
        Pager,

        // === Mechanism vs. policy (Levin et al. 1975; Liedtke 1996) ===
        Scheduler,
        Mechanism,
        Policy,

        // === Protection (Dijkstra 1968; Klein et al. 2009) ===
        PrivilegedMode,
        UserMode,
        TrustedComputingBase,
    ],

    labels: {
        Kernel: ("en", "Kernel", "Dijkstra (1968) CACM 11(5), The Structure of the 'THE'-Multiprogramming System; Brinch Hansen (1970) CACM 13(4): the privileged layer beneath all other software - the core of the trusted computing base."),
        Microkernel: ("en", "Microkernel", "Liedtke (1995) SOSP, On u-Kernel Construction, sec 2 (minimality principle): a kernel providing ONLY address spaces, threads, and IPC - a concept is tolerated inside the kernel only if moving it outside would prevent required functionality."),
        MonolithicKernel: ("en", "Monolithic kernel", "Liedtke (1996) CACM 39(9), Toward Real Microkernels: the contrast concept - drivers, file systems, and policy all run privileged inside the kernel."),
        Nucleus: ("en", "Nucleus", "Brinch Hansen (1970) CACM 13(4), The Nucleus of a Multiprogramming System: the minimal core implementing processes and the message primitives; all other operating-system functions live outside it."),
        AddressSpace: ("en", "Address space", "Liedtke (1995) sec 2.1: a virtual-to-physical memory mapping; the unit of isolation - page faults are exported to user-level pagers."),
        Thread: ("en", "Thread", "Liedtke (1995) sec 2.2 (Threads and IPC): an activity with a register set - instruction pointer and stack pointer - executing inside an address space."),
        Capability: ("en", "Capability", "Klein et al. (2009) SOSP, seL4: Formal Verification of an OS Kernel; Levin et al. (1975) SOSP: an unforgeable token conferring a specific access right on its holder."),
        Ipc: ("en", "IPC", "Liedtke (1995) sec 2.2 (Threads and IPC): the kernel's message-passing primitive - the only communication mechanism the microkernel provides."),
        Message: ("en", "Message", "Brinch Hansen (1970) CACM 13(4): the payload an IPC transfers between processes, buffered by the nucleus."),
        Endpoint: ("en", "Endpoint", "Klein et al. (2009) SOSP: the kernel object messages are sent to and received from - the rendezvous point of IPC."),
        UserServer: ("en", "User server", "Liedtke (1995) sec 3 (Flexibility); Haertig et al. (1997) SOSP, The Performance of u-Kernel-Based Systems: a user-space process implementing a system service - a device driver, a file system, a pager."),
        Pager: ("en", "Pager", "Liedtke (1995) sec 3 (Flexibility): a user-space server resolving page faults - memory-management policy exported out of the kernel."),
        Scheduler: ("en", "Scheduler", "Liedtke (1996) CACM 39(9): the privileged mechanism choosing which thread runs next."),
        Mechanism: ("en", "Mechanism", "Levin et al. (1975) SOSP, Policy/Mechanism Separation in HYDRA: a policy-free primitive the kernel provides."),
        Policy: ("en", "Policy", "Levin et al. (1975) SOSP: a resource-use decision deliberately kept OUT of the kernel and delegated to unprivileged software."),
        PrivilegedMode: ("en", "Privileged mode", "Dijkstra (1968) CACM 11(5): the CPU protection mode of the lowest system layer - the THE system's level structure grounds the privileged/unprivileged split."),
        UserMode: ("en", "User mode", "Dijkstra (1968) CACM 11(5): the unprivileged CPU protection mode in which the upper levels - and, in a microkernel, all servers - execute."),
        TrustedComputingBase: ("en", "Trusted computing base", "Klein et al. (2009) SOSP: the set of components whose correctness is critical to the security of the whole system - what the microkernel design minimises."),
    },

    is_a: [
        // Kernel taxonomy: Liedtke (1995, 1996); Brinch Hansen (1970).
        (Microkernel, Kernel),
        (MonolithicKernel, Kernel),
        (Nucleus, Kernel),
        // A pager is one kind of user-level server (Liedtke 1995 sec 3).
        (Pager, UserServer),
        // IPC and scheduling are policy-free kernel mechanisms
        // (Levin et al. 1975; Liedtke 1995 sec 2.2; Liedtke 1996).
        (Ipc, Mechanism),
        (Scheduler, Mechanism),
    ],

    opposes: [
        // The design contrast Liedtke (1996) draws: everything
        // privileged vs. only the minimal three abstractions.
        (MonolithicKernel, Microkernel),
        // The hardware protection split (Dijkstra 1968).
        (PrivilegedMode, UserMode),
    ],

    edges: [
        // === Privileges: Liedtke (1995) sec 2 - the ONLY concepts the
        // kernel implements privileged are address spaces, threads,
        // IPC, and (Liedtke 1996) the scheduling mechanism. ===
        (Kernel, AddressSpace, Privileges),
        (Kernel, Thread, Privileges),
        (Kernel, Ipc, Privileges),
        (Kernel, Scheduler, Privileges),

        // Liedtke (1995) sec 2.1: the address space is the unit of
        // isolation for the threads executing inside it.
        (AddressSpace, Thread, Isolates),

        // Brinch Hansen (1970): every message passes through the
        // nucleus - communication is kernel-mediated.
        (Kernel, Message, Mediates),

        // Klein et al. (2009): a capability confers a specific right
        // on a kernel object - a space or an endpoint.
        (Capability, AddressSpace, Grants),
        (Capability, Endpoint, Grants),

        // Liedtke (1995) sec 3: servers - including pagers - execute
        // unprivileged.
        (UserServer, UserMode, RunsInUserSpace),
        (Pager, UserMode, RunsInUserSpace),

        // Levin et al. (1975): mechanism and policy are SEPARATED -
        // a design discipline, deliberately not an Opposition edge.
        (Mechanism, Policy, Separates),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// The CPU protection domain a concept lives in — the two-mode split of
/// Dijkstra (1968) CACM 11(5) as inherited by every kernel design
/// (Liedtke 1995 §2: what is in the kernel runs privileged; §3: servers
/// run unprivileged).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Privilege {
    /// Runs in (or is) the privileged CPU mode.
    Privileged,
    /// Runs in (or is) the unprivileged CPU mode.
    UserSpace,
}

/// Which side of the privilege boundary a concept lives on.
///
/// `Some(Privileged)` for the kernels and everything they implement
/// privileged (Liedtke 1995 §2; Liedtke 1996 for the scheduler) plus
/// the privileged mode itself; `Some(UserSpace)` for servers, pagers,
/// policy (Liedtke 1995 §3; Levin et al. 1975), and user mode itself.
///
/// `None` for the five mode-neutral concepts, each justified:
///
/// - `Capability` — a token is held and exercised, not executed; it has
///   no CPU mode of its own (Levin et al. 1975; Klein et al. 2009:
///   capabilities are kernel-protected objects invoked from user space).
/// - `Message` — the payload crosses the boundary: composed in user
///   space, transferred by the kernel (Brinch Hansen 1970).
/// - `Endpoint` — the rendezvous object sits exactly ON the boundary:
///   a kernel object named from user space (Klein et al. 2009).
/// - `Mechanism` — the abstract parent: its concrete children divide
///   over the boundary (kernel IPC vs. user-level mechanisms are both
///   possible in HYDRA — Levin et al. 1975), so the parent carries no
///   privilege of its own.
/// - `TrustedComputingBase` — a *set* of components spanning hardware,
///   kernel, and possibly servers (Klein et al. 2009); not a single
///   subject with one mode.
#[derive(Debug, Clone)]
pub struct KernelPrivilege;

impl Quality for KernelPrivilege {
    type Individual = MicrokernelConcept;
    type Value = Privilege;

    fn get(&self, c: &MicrokernelConcept) -> Option<Privilege> {
        use MicrokernelConcept as C;
        match c {
            C::AddressSpace
            | C::Thread
            | C::Ipc
            | C::Scheduler
            | C::Kernel
            | C::Microkernel
            | C::MonolithicKernel
            | C::Nucleus
            | C::PrivilegedMode => Some(Privilege::Privileged),
            C::UserServer | C::Pager | C::Policy | C::UserMode => Some(Privilege::UserSpace),
            C::Capability | C::Message | C::Endpoint | C::Mechanism | C::TrustedComputingBase => {
                None
            }
        }
    }
}

/// Whether a concept is a policy-free mechanism — Levin et al. (1975):
/// `true` for `Mechanism` and its concrete kernel instances (`Ipc`,
/// `Scheduler`); `false` for `Policy` (the explicit non-mechanism the
/// separation keeps out of the kernel); `None` for every concept the
/// mechanism/policy distinction does not classify.
#[derive(Debug, Clone)]
pub struct IsMechanism;

impl Quality for IsMechanism {
    type Individual = MicrokernelConcept;
    type Value = bool;

    fn get(&self, c: &MicrokernelConcept) -> Option<bool> {
        use MicrokernelConcept as C;
        match c {
            C::Mechanism | C::Ipc | C::Scheduler => Some(true),
            C::Policy => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn kinded_edge_exists(
    from: MicrokernelConcept,
    to: MicrokernelConcept,
    kind: MicrokernelRelationKind,
) -> bool {
    use pr4xis::category::{Arrow, Category};
    MicrokernelCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

fn direct_children_of(parent: MicrokernelConcept) -> Vec<MicrokernelConcept> {
    use pr4xis::category::{Arrow, Category};
    MicrokernelCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == MicrokernelRelationKind::Subsumption && m.target() == parent)
        .map(|m| m.source())
        .collect()
}

/// The `Privileges`-edge targets of `Kernel` — what the kernel
/// implements privileged, read off the category (Liedtke 1995 §2).
pub fn privileged_primitives() -> Vec<MicrokernelConcept> {
    use pr4xis::category::{Arrow, Category};
    MicrokernelCategory::morphisms()
        .iter()
        .filter(|m| {
            m.kind() == MicrokernelRelationKind::Privileges
                && m.source() == MicrokernelConcept::Kernel
        })
        .map(|m| m.target())
        .collect()
}

/// The sources of `RunsInUserSpace` edges — everything the category
/// asserts to execute unprivileged (Liedtke 1995 §3).
pub fn user_space_runners() -> Vec<MicrokernelConcept> {
    use pr4xis::category::{Arrow, Category};
    MicrokernelCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == MicrokernelRelationKind::RunsInUserSpace)
        .map(|m| m.source())
        .collect()
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Liedtke (1995) §2, the minimality principle as set equality: the
/// `Privileges`-edge targets of `Kernel` are EXACTLY the set
/// {`AddressSpace`, `Thread`, `Ipc`, `Scheduler`} — the three
/// abstractions of §3 plus the scheduling mechanism (Liedtke 1996).
/// Adding any further `Privileges` edge (a driver, a file system, a
/// policy) falsifies the axiom; so does dropping one of the four.
pub struct MinimalPrivilegedSet;

impl Axiom for MinimalPrivilegedSet {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let targets = privileged_primitives();
        let expected = [
            MicrokernelConcept::AddressSpace,
            MicrokernelConcept::Thread,
            MicrokernelConcept::Ipc,
            MicrokernelConcept::Scheduler,
        ];
        // Set equality on the collected edge targets — not a count.
        let no_duplicates = targets
            .iter()
            .enumerate()
            .all(|(i, t)| targets.iter().skip(i + 1).all(|u| t != u));
        let covers = expected.iter().all(|e| targets.contains(e));
        let covered_by = targets.iter().all(|t| expected.contains(t));
        if no_duplicates && covers && covered_by {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MinimalPrivilegedSet",
        "the Privileges-edge targets of Kernel are exactly the set {AddressSpace, Thread, Ipc, Scheduler} - no more, no fewer",
        "Liedtke (1995) SOSP, On u-Kernel Construction, sec 2"
    );
}
pr4xis::register_axiom!(
    MinimalPrivilegedSet,
    "Liedtke (1995) SOSP, On u-Kernel Construction, sec 2"
);

/// Levin et al. (1975): the kernel provides mechanisms, never policy.
/// The category carries the `Separates` edge `Mechanism → Policy`, and
/// the `IsMechanism` classification is disjoint and non-vacuous: the
/// concepts classified `true` (mechanisms) and those classified `false`
/// (policy) share no member, and neither side is empty.
pub struct MechanismPolicySeparation;

impl Axiom for MechanismPolicySeparation {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::FinitelyGenerated;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let edge = kinded_edge_exists(
            MicrokernelConcept::Mechanism,
            MicrokernelConcept::Policy,
            MicrokernelRelationKind::Separates,
        );
        let q = IsMechanism;
        let mechanisms: Vec<MicrokernelConcept> = MicrokernelConcept::variants()
            .into_iter()
            .filter(|c| q.get(c) == Some(true))
            .collect();
        let policies: Vec<MicrokernelConcept> = MicrokernelConcept::variants()
            .into_iter()
            .filter(|c| q.get(c) == Some(false))
            .collect();
        let disjoint = mechanisms.iter().all(|m| !policies.contains(m));
        let non_vacuous = !mechanisms.is_empty() && !policies.is_empty();
        if edge && disjoint && non_vacuous {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MechanismPolicySeparation",
        "the Separates edge Mechanism->Policy exists, and the IsMechanism classification is disjoint (no concept both mechanism and policy) and non-vacuous",
        "Levin, Cohen, Corwin, Pollack & Wulf (1975) SOSP, Policy/Mechanism Separation in HYDRA"
    );
}
pr4xis::register_axiom!(
    MechanismPolicySeparation,
    "Levin, Cohen, Corwin, Pollack & Wulf (1975) SOSP, Policy/Mechanism Separation in HYDRA"
);

/// Liedtke (1995) §3 / Haertig et al. (1997): services run
/// unprivileged. Every concept with a `RunsInUserSpace` edge has
/// `KernelPrivilege = Some(UserSpace)`, and both `UserServer` and
/// `Pager` carry such an edge — so the claim is non-vacuous and the
/// edge set and the quality agree.
pub struct ServersRunUnprivileged;

impl Axiom for ServersRunUnprivileged {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let runners = user_space_runners();
        let q = KernelPrivilege;
        let all_unprivileged = runners
            .iter()
            .all(|c| q.get(c) == Some(Privilege::UserSpace));
        let servers_covered = runners.contains(&MicrokernelConcept::UserServer)
            && runners.contains(&MicrokernelConcept::Pager);
        if all_unprivileged && servers_covered {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ServersRunUnprivileged",
        "every concept with a RunsInUserSpace edge has KernelPrivilege = UserSpace, and UserServer and Pager both carry such an edge",
        "Liedtke (1995) SOSP sec 3 (Flexibility); Haertig et al. (1997) SOSP, The Performance of u-Kernel-Based Systems"
    );
}
pr4xis::register_axiom!(
    ServersRunUnprivileged,
    "Liedtke (1995) SOSP sec 3 (Flexibility); Haertig et al. (1997) SOSP, The Performance of u-Kernel-Based Systems"
);

/// Klein et al. (2009) / Liedtke (1996): the microkernel and the
/// monolithic kernel are rival *designs of the same thing* — both are
/// Subsumption-children of `Kernel` — and the category records their
/// design contrast as an `Opposition` edge (in both directions, per the
/// symmetric-opposition structural axiom). What the contrast buys is
/// the minimised trusted computing base seL4's verification rests on.
pub struct MicrokernelMinimizesTcb;

impl Axiom for MicrokernelMinimizesTcb {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let children = direct_children_of(MicrokernelConcept::Kernel);
        let both_kernels = children.contains(&MicrokernelConcept::Microkernel)
            && children.contains(&MicrokernelConcept::MonolithicKernel);
        let opposed = kinded_edge_exists(
            MicrokernelConcept::Microkernel,
            MicrokernelConcept::MonolithicKernel,
            MicrokernelRelationKind::Opposition,
        ) && kinded_edge_exists(
            MicrokernelConcept::MonolithicKernel,
            MicrokernelConcept::Microkernel,
            MicrokernelRelationKind::Opposition,
        );
        if both_kernels && opposed {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MicrokernelMinimizesTcb",
        "Microkernel and MonolithicKernel are both Subsumption-children of Kernel, and an Opposition edge connects them (both directions)",
        "Klein et al. (2009) SOSP, seL4: Formal Verification of an OS Kernel; Liedtke (1996) CACM 39(9)"
    );
}
pr4xis::register_axiom!(
    MicrokernelMinimizesTcb,
    "Klein et al. (2009) SOSP, seL4: Formal Verification of an OS Kernel; Liedtke (1996) CACM 39(9)"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for MicrokernelOntology {
    type Cat = MicrokernelCategory;
    type Qual = KernelPrivilege;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(MinimalPrivilegedSet));
        axioms.push(Box::new(MechanismPolicySeparation));
        axioms.push(Box::new(ServersRunUnprivileged));
        axioms.push(Box::new(MicrokernelMinimizesTcb));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<MicrokernelCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        MicrokernelOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn minimal_privileged_set_holds() {
        assert!(MinimalPrivilegedSet.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mechanism_policy_separation_holds() {
        assert!(MechanismPolicySeparation.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn servers_run_unprivileged_holds() {
        assert!(ServersRunUnprivileged.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn microkernel_minimizes_tcb_holds() {
        assert!(MicrokernelMinimizesTcb.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn privilege_classification() {
        let q = KernelPrivilege;
        for c in [
            MicrokernelConcept::AddressSpace,
            MicrokernelConcept::Thread,
            MicrokernelConcept::Ipc,
            MicrokernelConcept::Scheduler,
            MicrokernelConcept::Kernel,
            MicrokernelConcept::Microkernel,
            MicrokernelConcept::MonolithicKernel,
            MicrokernelConcept::Nucleus,
            MicrokernelConcept::PrivilegedMode,
        ] {
            assert_eq!(q.get(&c), Some(Privilege::Privileged), "{c:?}");
        }
        for c in [
            MicrokernelConcept::UserServer,
            MicrokernelConcept::Pager,
            MicrokernelConcept::Policy,
            MicrokernelConcept::UserMode,
        ] {
            assert_eq!(q.get(&c), Some(Privilege::UserSpace), "{c:?}");
        }
        for c in [
            MicrokernelConcept::Capability,
            MicrokernelConcept::Message,
            MicrokernelConcept::Endpoint,
            MicrokernelConcept::Mechanism,
            MicrokernelConcept::TrustedComputingBase,
        ] {
            assert_eq!(q.get(&c), None, "{c:?} is mode-neutral");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mechanism_classification() {
        for c in [
            MicrokernelConcept::Mechanism,
            MicrokernelConcept::Ipc,
            MicrokernelConcept::Scheduler,
        ] {
            assert_eq!(IsMechanism.get(&c), Some(true), "{c:?}");
        }
        assert_eq!(IsMechanism.get(&MicrokernelConcept::Policy), Some(false));
        assert_eq!(
            IsMechanism.get(&MicrokernelConcept::Kernel),
            None,
            "the kernel is classified by what it provides, not as a mechanism itself"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn kernel_taxonomy() {
        let children = direct_children_of(MicrokernelConcept::Kernel);
        let expected = [
            MicrokernelConcept::Microkernel,
            MicrokernelConcept::MonolithicKernel,
            MicrokernelConcept::Nucleus,
        ];
        assert_eq!(children.len(), expected.len());
        for c in expected {
            assert!(children.contains(&c), "{c:?} should be a kind of Kernel");
        }
    }
}
