//! Driver — device drivers and the hardware abstraction layer: the
//! operating-system half of the driver/device seam.
//!
//! Source traditions:
//!
//! - **Corbet, Rubini & Kroah-Hartman (2005)** *Linux Device Drivers*,
//!   3rd ed., O'Reilly — the driver as software translating OS
//!   requests into device operations and the three device classes
//!   (Ch. 1); hardware registers (Ch. 9); interrupts and their
//!   handlers (Ch. 10); DMA (Ch. 15).
//! - **Liedtke (1995)** *On µ-Kernel Construction*, SOSP, sec. 3
//!   (Flexibility, Device Driver) — the driver as an isolated
//!   user-space server.
//! - **Swift, Bershad & Levy (2003)** *Improving the Reliability of
//!   Commodity Operating Systems*, SOSP — Nooks: driver faults
//!   dominate OS crashes (sec. 1); lightweight protection domains
//!   contain them (sec. 3); a failed driver restarts without a kernel
//!   crash.
//! - **Ganapathy, Renzelmann, Balakrishnan, Swift & Jha (2008)** *The
//!   Design and Implementation of Microdrivers*, ASPLOS — the split
//!   driver: critical path in the kernel, bulk in user space.
//! - **Ryzhyk, Chubb, Kuz, Le Sueur & Heiser (2009)** *Automatic
//!   Device Driver Synthesis with Termite*, SOSP — the driver as a
//!   derivable artifact: synthesized from a formal device model plus
//!   an OS interface specification.
//!
//! The fault-containment axiom is discharged against the driver/device
//! state machine in [`super::engine`] — the same fault is injected
//! into an isolated and a non-isolated driver, and only the second
//! corrupts kernel state.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use super::engine::{
    DriverAction, DriverState, FAULT_CONTAINMENT_SCRIPT, KernelIntegrity, apply, in_kernel_initial,
    isolated_initial, run_script,
};

pr4xis::ontology! {
    name: "Driver",
    source: "Corbet, Rubini & Kroah-Hartman (2005) Linux Device Drivers, 3rd ed.; Swift, Bershad & Levy (2003) SOSP; Ganapathy, Renzelmann, Balakrishnan, Swift & Jha (2008) ASPLOS; Ryzhyk, Chubb, Kuz, Le Sueur & Heiser (2009) SOSP; Liedtke (1995) SOSP",

    concepts: [
        // === The software/hardware seam (Corbet et al. 2005, Ch. 1) ===
        Driver,
        Device,
        CharacterDevice,
        BlockDevice,
        NetworkDevice,

        // === The hardware side (Corbet et al. 2005, Ch. 9, 10, 15) ===
        HardwareRegister,
        Interrupt,
        InterruptHandler,
        Dma,
        Hal,

        // === Isolation architectures (Liedtke 1995; Swift et al. 2003;
        //     Ganapathy et al. 2008) ===
        DriverAsServer,
        IsolationDomain,
        Microdriver,

        // === Synthesis (Ryzhyk et al. 2009) ===
        DeviceModel,

        // === Faults and recovery (Swift et al. 2003) ===
        DriverFault,
        Recovery,
    ],

    labels: {
        Driver: ("en", "Driver", "Corbet, Rubini & Kroah-Hartman (2005) Linux Device Drivers 3e, Ch. 1: software translating operating-system requests into operations on a specific device."),
        Device: ("en", "Device", "Corbet et al. (2005) Ch. 1: a hardware peripheral the operating system reaches only through its driver."),
        CharacterDevice: ("en", "Character device", "Corbet et al. (2005) Ch. 1: a device accessed as a stream of bytes (console, serial port) - one of the three Linux device classes."),
        BlockDevice: ("en", "Block device", "Corbet et al. (2005) Ch. 1: a device that hosts filesystems and is accessed in blocks - one of the three Linux device classes."),
        NetworkDevice: ("en", "Network device", "Corbet et al. (2005) Ch. 1: an interface that exchanges packets with other hosts - one of the three Linux device classes."),
        HardwareRegister: ("en", "Hardware register", "Corbet et al. (2005) Ch. 9: a memory-mapped or I/O-port control/status word through which the driver operates the device."),
        Interrupt: ("en", "Interrupt", "Corbet et al. (2005) Ch. 10: an asynchronous hardware signal by which the device requests service."),
        InterruptHandler: ("en", "Interrupt handler", "Corbet et al. (2005) Ch. 10: the driver routine that services an interrupt."),
        Dma: ("en", "DMA", "Corbet et al. (2005) Ch. 15: direct memory access - device-driven memory transfer that bypasses the CPU."),
        Hal: ("en", "HAL", "Corbet et al. (2005): the hardware abstraction layer decoupling drivers from hardware specifics."),
        DriverAsServer: ("en", "Driver as server", "Liedtke (1995) SOSP sec. 3 (Flexibility, Device Driver): the driver as an isolated user-space process; Swift, Bershad & Levy (2003) SOSP: the strong-isolation end of the driver design space."),
        IsolationDomain: ("en", "Isolation domain", "Swift, Bershad & Levy (2003) SOSP: a fault-containment boundary around a driver - Nooks' lightweight kernel protection domain."),
        Microdriver: ("en", "Microdriver", "Ganapathy, Renzelmann, Balakrishnan, Swift & Jha (2008) ASPLOS: a split driver - the critical path stays in the kernel, the bulk of the code moves to user space."),
        DeviceModel: ("en", "Device model", "Ryzhyk, Chubb, Kuz, Le Sueur & Heiser (2009) SOSP: a formal specification of device behaviour from which the driver is synthesized."),
        DriverFault: ("en", "Driver fault", "Swift, Bershad & Levy (2003) SOSP sec. 1: a fault originating in driver code - the dominant cause of operating-system crashes."),
        Recovery: ("en", "Recovery", "Swift, Bershad & Levy (2003) SOSP: restart/reload of a failed driver without a kernel crash."),
    },

    is_a: [
        // Corbet et al. (2005) Ch. 1: the three Linux device classes.
        (CharacterDevice, Device),
        (BlockDevice, Device),
        (NetworkDevice, Device),
        // Liedtke (1995); Ganapathy et al. (2008): the isolation
        // architectures are drivers.
        (DriverAsServer, Driver),
        (Microdriver, Driver),
    ],

    has_a: [
        // Corbet et al. (2005) Ch. 10: the handler is the driver's own
        // interrupt-servicing routine.
        (Driver, InterruptHandler),
    ],

    edges: [
        // Corbet et al. (2005) Ch. 1: the driver drives the device.
        (Driver, Device, Drives),
        // Corbet et al. (2005) Ch. 10: the handler services the signal.
        (InterruptHandler, Interrupt, Handles),
        // Corbet et al. (2005) Ch. 9 / Ch. 15: the driver operates the
        // device through registers and DMA.
        (Driver, HardwareRegister, Accesses),
        (Driver, Dma, Accesses),
        // Swift et al. (2003): the containment boundary wraps the
        // driver — Nooks isolates unmodified in-kernel drivers, and
        // the user-space server is the strong end of the same axis.
        (IsolationDomain, Driver, Isolates),
        (IsolationDomain, DriverAsServer, Isolates),
        // Ganapathy et al. (2008): the microdriver's user-mode portion
        // runs in its own protection domain.
        (IsolationDomain, Microdriver, Isolates),
        // Ryzhyk et al. (2009): the driver is derivable from the
        // device model (plus the OS interface specification).
        (Driver, DeviceModel, SynthesizedFrom),
        // Swift et al. (2003): recovery restarts the faulted driver.
        (Recovery, DriverFault, Recovers),
        // Corbet et al. (2005): the HAL presents the device without
        // its hardware specifics.
        (Hal, Device, Abstracts),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// The three Linux device classes — Corbet, Rubini & Kroah-Hartman
/// (2005) Ch. 1: char, block, and network are the fundamental device
/// types the kernel distinguishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevClass {
    /// Accessed as a stream of bytes.
    Character,
    /// Hosts filesystems; accessed in blocks.
    Block,
    /// Exchanges packets with other hosts.
    Network,
}

impl DevClass {
    /// The closed set of the three classes, in the order Corbet et al.
    /// (2005) Ch. 1 introduces them.
    pub const ALL: [DevClass; 3] = [DevClass::Character, DevClass::Block, DevClass::Network];
}

/// Which Linux device class a concept realises — Corbet et al. (2005)
/// Ch. 1. `Some` for exactly the three device-class concepts; `None`
/// for every other concept, including the abstract `Device` parent.
#[derive(Debug, Clone)]
pub struct DeviceClass;

impl Quality for DeviceClass {
    type Individual = DriverConcept;
    type Value = DevClass;

    fn get(&self, c: &DriverConcept) -> Option<DevClass> {
        use DriverConcept as C;
        match c {
            C::CharacterDevice => Some(DevClass::Character),
            C::BlockDevice => Some(DevClass::Block),
            C::NetworkDevice => Some(DevClass::Network),
            _ => None,
        }
    }
}

/// Whether a driver architecture runs behind a fault-containment
/// boundary — Swift, Bershad & Levy (2003); Liedtke (1995). The plain
/// in-kernel `Driver` is *not* isolated (the commodity default whose
/// faults corrupt the kernel — Swift et al. 2003 sec. 1); the
/// user-space server and the microdriver are. `None` for concepts that
/// are not driver architectures.
#[derive(Debug, Clone)]
pub struct IsIsolated;

impl Quality for IsIsolated {
    type Individual = DriverConcept;
    type Value = bool;

    fn get(&self, c: &DriverConcept) -> Option<bool> {
        use DriverConcept as C;
        match c {
            C::Driver => Some(false),
            C::DriverAsServer | C::Microdriver => Some(true),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn direct_children_of(parent: DriverConcept) -> Vec<DriverConcept> {
    use pr4xis::category::{Arrow, Category};
    DriverCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == DriverRelationKind::Subsumption && m.target() == parent)
        .map(|m| m.source())
        .collect()
}

fn kinded_edge_exists(from: DriverConcept, to: DriverConcept, kind: DriverRelationKind) -> bool {
    use pr4xis::category::{Arrow, Category};
    DriverCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Corbet, Rubini & Kroah-Hartman (2005) Ch. 1: the Subsumption
/// children of `Device` are exactly the three Linux device classes —
/// char, block, and network — and the `DeviceClass` quality puts them
/// in bijection with the closed `DevClass` set (each child carries a
/// distinct class; the abstract parent carries none).
pub struct ThreeDeviceClasses;

impl Axiom for ThreeDeviceClasses {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let children = direct_children_of(DriverConcept::Device);
        let expected = [
            DriverConcept::CharacterDevice,
            DriverConcept::BlockDevice,
            DriverConcept::NetworkDevice,
        ];
        let set_equal =
            children.len() == expected.len() && expected.iter().all(|c| children.contains(c));
        // Bijection with the closed class set: every child carries
        // Some(class), no two children share one, and every class is
        // realised.
        let classes: Option<Vec<DevClass>> = children.iter().map(|c| DeviceClass.get(c)).collect();
        let bijective = match classes {
            Some(classes) => {
                let injective = classes
                    .iter()
                    .enumerate()
                    .all(|(i, a)| classes.iter().skip(i + 1).all(|b| a != b));
                let surjective = DevClass::ALL.iter().all(|k| classes.contains(k));
                injective && surjective && classes.len() == DevClass::ALL.len()
            }
            None => false,
        };
        let parent_unclassified = DeviceClass.get(&DriverConcept::Device).is_none();
        if set_equal && bijective && parent_unclassified {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ThreeDeviceClasses",
        "the Subsumption children of Device are exactly {CharacterDevice, BlockDevice, NetworkDevice}, in bijection with the closed DevClass set",
        "Corbet, Rubini & Kroah-Hartman (2005) Linux Device Drivers 3e, Ch. 1"
    );
}
pr4xis::register_axiom!(
    ThreeDeviceClasses,
    "Corbet, Rubini & Kroah-Hartman (2005) Linux Device Drivers 3e, Ch. 1"
);

/// Swift, Bershad & Levy (2003) sec. 3 — the Nooks claim, demonstrated:
/// every isolated driver architecture sits inside a containment
/// boundary (an incoming `Isolates` edge from `IsolationDomain`), and
/// on the engine fixture the same injected fault leaves kernel state
/// intact — and the driver recoverable — inside the isolation domain,
/// while it corrupts kernel state — and defeats recovery — in the
/// kernel address space.
pub struct IsolatedDriverContainsFault;

impl Axiom for IsolatedDriverContainsFault {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::FinitelyGenerated;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        // Structural half: every IsIsolated = Some(true) concept has an
        // incoming Isolates edge from the containment boundary.
        let isolated: Vec<DriverConcept> = DriverConcept::variants()
            .into_iter()
            .filter(|c| IsIsolated.get(c) == Some(true))
            .collect();
        let all_bounded = !isolated.is_empty()
            && isolated.iter().all(|c| {
                kinded_edge_exists(
                    DriverConcept::IsolationDomain,
                    *c,
                    DriverRelationKind::Isolates,
                )
            });

        // Engine half: the containment experiment. Isolated run — the
        // fault fires but kernel state stays intact and recovery
        // rebinds the driver.
        let contained_and_recoverable =
            match run_script(&isolated_initial(), &FAULT_CONTAINMENT_SCRIPT) {
                Ok(end) => {
                    let contained =
                        end.driver == DriverState::Faulted && end.kernel == KernelIntegrity::Intact;
                    let recovered = match apply(&end, &DriverAction::Recover) {
                        Ok(after) => {
                            after.driver == DriverState::Bound
                                && after.kernel == KernelIntegrity::Intact
                        }
                        Err(_) => false,
                    };
                    contained && recovered
                }
                Err(_) => false,
            };
        // In-kernel run — the identical fault corrupts kernel state
        // and recovery is impossible.
        let corrupted_and_unrecoverable =
            match run_script(&in_kernel_initial(), &FAULT_CONTAINMENT_SCRIPT) {
                Ok(end) => {
                    end.driver == DriverState::Faulted
                        && end.kernel == KernelIntegrity::Corrupted
                        && apply(&end, &DriverAction::Recover).is_err()
                }
                Err(_) => false,
            };

        if all_bounded && contained_and_recoverable && corrupted_and_unrecoverable {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "IsolatedDriverContainsFault",
        "every isolated driver architecture has an incoming Isolates edge; on the fixture the same injected fault leaves kernel state intact (and the driver recoverable) inside an isolation domain, and corrupts it (defeating recovery) in the kernel address space",
        "Swift, Bershad & Levy (2003) SOSP sec. 3"
    );
}
pr4xis::register_axiom!(
    IsolatedDriverContainsFault,
    "Swift, Bershad & Levy (2003) SOSP sec. 3"
);

/// Corbet, Rubini & Kroah-Hartman (2005) Ch. 1: the driver is exactly
/// the software/hardware bridge — it carries both the `Drives` edge to
/// the device and the `Accesses` edge to the hardware register. On the
/// engine, a bound driver services read and write requests while an
/// unbound one cannot (no driver, no device operation).
pub struct DriverBridgesSoftwareHardware;

impl Axiom for DriverBridgesSoftwareHardware {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let drives = kinded_edge_exists(
            DriverConcept::Driver,
            DriverConcept::Device,
            DriverRelationKind::Drives,
        );
        let accesses = kinded_edge_exists(
            DriverConcept::Driver,
            DriverConcept::HardwareRegister,
            DriverRelationKind::Accesses,
        );
        // Engine grounding: the bridge in operation.
        let bound_services = match apply(&in_kernel_initial(), &DriverAction::Probe) {
            Ok(bound) => {
                apply(&bound, &DriverAction::Read).is_ok()
                    && apply(&bound, &DriverAction::Write).is_ok()
            }
            Err(_) => false,
        };
        let unbound_rejected = apply(&in_kernel_initial(), &DriverAction::Read).is_err();
        if drives && accesses && bound_services && unbound_rejected {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DriverBridgesSoftwareHardware",
        "Driver carries both a Drives edge to Device and an Accesses edge to HardwareRegister; on the fixture a bound driver services read/write requests and an unbound one cannot",
        "Corbet, Rubini & Kroah-Hartman (2005) Linux Device Drivers 3e, Ch. 1"
    );
}
pr4xis::register_axiom!(
    DriverBridgesSoftwareHardware,
    "Corbet, Rubini & Kroah-Hartman (2005) Linux Device Drivers 3e, Ch. 1"
);

/// Ryzhyk, Chubb, Kuz, Le Sueur & Heiser (2009): a driver is derivable
/// from the formal specification of the device's behaviour plus the OS
/// interface specification — the category carries the `SynthesizedFrom`
/// edge from `Driver` to `DeviceModel`.
pub struct SynthesizableFromModel;

impl Axiom for SynthesizableFromModel {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if kinded_edge_exists(
            DriverConcept::Driver,
            DriverConcept::DeviceModel,
            DriverRelationKind::SynthesizedFrom,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SynthesizableFromModel",
        "the SynthesizedFrom edge Driver -> DeviceModel exists: a driver is derivable from device spec + OS interface spec",
        "Ryzhyk, Chubb, Kuz, Le Sueur & Heiser (2009) SOSP"
    );
}
pr4xis::register_axiom!(
    SynthesizableFromModel,
    "Ryzhyk, Chubb, Kuz, Le Sueur & Heiser (2009) SOSP"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for DriverOntology {
    type Cat = DriverCategory;
    type Qual = DeviceClass;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ThreeDeviceClasses));
        axioms.push(Box::new(IsolatedDriverContainsFault));
        axioms.push(Box::new(DriverBridgesSoftwareHardware));
        axioms.push(Box::new(SynthesizableFromModel));
        axioms
    }
}

/// The three Linux device classes — direct Subsumption children of
/// `Device` (Corbet et al. 2005, Ch. 1). Grounded in the category's
/// edges; used by tests.
pub fn device_classes() -> Vec<DriverConcept> {
    direct_children_of(DriverConcept::Device)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<DriverCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        DriverOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn three_device_classes_holds() {
        assert!(ThreeDeviceClasses.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn isolated_driver_contains_fault_holds() {
        assert!(IsolatedDriverContainsFault.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn driver_bridges_software_hardware_holds() {
        assert!(DriverBridgesSoftwareHardware.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn synthesizable_from_model_holds() {
        assert!(SynthesizableFromModel.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn device_class_taxonomy() {
        let classes = device_classes();
        let expected = [
            DriverConcept::CharacterDevice,
            DriverConcept::BlockDevice,
            DriverConcept::NetworkDevice,
        ];
        assert_eq!(classes.len(), expected.len());
        for c in expected {
            assert!(classes.contains(&c), "{c:?} should be a device class");
            assert!(
                DeviceClass.get(&c).is_some(),
                "{c:?} must carry a device class"
            );
        }
        assert_eq!(
            DeviceClass.get(&DriverConcept::Device),
            None,
            "the abstract parent carries no device class of its own"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn isolation_classification() {
        assert_eq!(
            IsIsolated.get(&DriverConcept::Driver),
            Some(false),
            "the in-kernel driver is the non-isolated commodity default (Swift et al. 2003 sec. 1)"
        );
        for c in [DriverConcept::DriverAsServer, DriverConcept::Microdriver] {
            assert_eq!(IsIsolated.get(&c), Some(true), "{c:?} is isolated");
        }
        assert_eq!(IsIsolated.get(&DriverConcept::Device), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn interrupt_handler_is_part_of_driver() {
        use pr4xis::category::{Arrow, Category};
        // has_a desugars part -> whole (BFO part_of orientation).
        assert!(DriverCategory::morphisms().iter().any(|m| {
            m.kind() == DriverRelationKind::Parthood
                && m.source() == DriverConcept::InterruptHandler
                && m.target() == DriverConcept::Driver
        }));
    }
}
