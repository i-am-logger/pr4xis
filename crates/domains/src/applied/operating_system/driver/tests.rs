//! Property-based tests for the Driver ontology, engine, and the
//! Driver → Dependability functor.

#![cfg(test)]

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use proptest::prelude::*;

use super::engine::{
    DeviceState, DriverAction, DriverSituation, DriverState, FAULT_CONTAINMENT_SCRIPT,
    IsolationMembership, KernelIntegrity, apply, in_kernel_initial, isolated_initial, run_script,
};
use super::ontology::{DeviceClass, DriverCategory, DriverConcept, DriverOntology, IsIsolated};
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Ontology, Quality};

fn arb_concept() -> impl Strategy<Value = DriverConcept> {
    proptest::sample::select(DriverConcept::variants())
}

fn arb_situation() -> impl Strategy<Value = DriverSituation> {
    (
        proptest::sample::select(DeviceState::ALL.to_vec()),
        proptest::sample::select(DriverState::ALL.to_vec()),
        proptest::sample::select(KernelIntegrity::ALL.to_vec()),
        proptest::sample::select(IsolationMembership::ALL.to_vec()),
    )
        .prop_map(|(device, driver, kernel, domain)| DriverSituation {
            device,
            driver,
            kernel,
            domain,
        })
}

fn arb_action() -> impl Strategy<Value = DriverAction> {
    proptest::sample::select(DriverAction::ALL.to_vec())
}

proptest! {
    /// `DeviceClass` is defined exactly on the three Linux device
    /// classes (Corbet et al. 2005, Ch. 1) — not on the abstract
    /// `Device` parent, not on anything else.
    #[test]
    fn prop_device_class_exactly_on_the_three_classes(c in arb_concept()) {
        use DriverConcept as C;
        let is_class = matches!(c, C::CharacterDevice | C::BlockDevice | C::NetworkDevice);
        prop_assert_eq!(DeviceClass.get(&c).is_some(), is_class);
    }

    /// `IsIsolated` is defined exactly on the driver architectures
    /// (Swift et al. 2003; Liedtke 1995), and every isolated one is a
    /// Subsumption child of `Driver`.
    #[test]
    fn prop_is_isolated_exactly_on_driver_architectures(c in arb_concept()) {
        use DriverConcept as C;
        let is_architecture = matches!(c, C::Driver | C::DriverAsServer | C::Microdriver);
        prop_assert_eq!(IsIsolated.get(&c).is_some(), is_architecture);
        if IsIsolated.get(&c) == Some(true) {
            let is_a_driver = DriverCategory::morphisms().iter().any(|m| {
                m.kind() == super::ontology::DriverRelationKind::Subsumption
                    && m.source() == c
                    && m.target() == C::Driver
            });
            prop_assert!(is_a_driver, "{:?} should be a Driver", c);
        }
    }

    /// Every arrow of the category carries a non-empty name.
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in DriverCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    /// Structural + domain axioms all discharge, regardless of the
    /// sampling that drives the test.
    #[test]
    fn prop_all_axioms_hold(_seed in 0..16u32) {
        for axiom in DriverOntology::axioms() {
            if let Err(c) = axiom.verify() {
                prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
            }
        }
    }

    /// Determinism of the engine: applying the same action to the same
    /// situation always yields the same outcome — over the whole typed
    /// state/action product, not just reachable states.
    #[test]
    fn prop_step_is_deterministic(s in arb_situation(), a in arb_action()) {
        prop_assert_eq!(apply(&s, &a), apply(&s, &a));
    }

    /// The Nooks containment claim over the whole domain axis (Swift
    /// et al. 2003 sec. 3): after the fault-injection script, kernel
    /// state is corrupted exactly when the driver ran in the kernel
    /// address space.
    #[test]
    fn prop_containment_is_domain_determined(
        domain in proptest::sample::select(IsolationMembership::ALL.to_vec())
    ) {
        let initial = match domain {
            IsolationMembership::KernelAddressSpace => in_kernel_initial(),
            IsolationMembership::IsolationDomain => isolated_initial(),
        };
        let end = run_script(&initial, &FAULT_CONTAINMENT_SCRIPT)
            .unwrap_or_else(|e| panic!("containment script must be enabled: {e}"));
        prop_assert_eq!(end.driver, DriverState::Faulted, "the fault fired");
        let corrupted = end.kernel == KernelIntegrity::Corrupted;
        prop_assert_eq!(
            corrupted,
            domain == IsolationMembership::KernelAddressSpace,
            "kernel corruption iff the driver shared the kernel address space"
        );
    }

    /// Recovery is enabled only for a contained fault: whenever
    /// `Recover` succeeds, the pre-state had a faulted driver and an
    /// intact kernel, and the post-state has the driver rebound
    /// (Swift et al. 2003).
    #[test]
    fn prop_recovery_only_for_contained_faults(s in arb_situation()) {
        if let Ok(after) = apply(&s, &DriverAction::Recover) {
            prop_assert_eq!(s.driver, DriverState::Faulted);
            prop_assert_eq!(s.kernel, KernelIntegrity::Intact);
            prop_assert_eq!(after.driver, DriverState::Bound);
            prop_assert_eq!(after.kernel, KernelIntegrity::Intact);
        }
    }

    /// Kernel integrity is monotone under OS-side actions: no action
    /// ever restores a corrupted kernel — only `InjectFault` in the
    /// kernel address space degrades it (Swift et al. 2003 sec. 1:
    /// only a reboot restores corrupted kernel state).
    #[test]
    fn prop_no_action_restores_kernel_integrity(s in arb_situation(), a in arb_action()) {
        if s.kernel == KernelIntegrity::Corrupted
            && let Ok(after) = apply(&s, &a) {
                prop_assert_eq!(after.kernel, KernelIntegrity::Corrupted);
            }
    }
}

pr4xis::register_praxis_value!(prop_device_class_exactly_on_the_three_classes, Verifiable);
pr4xis::register_praxis_value!(prop_is_isolated_exactly_on_driver_architectures, Verifiable);
pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
pr4xis::register_praxis_value!(prop_all_axioms_hold, Verifiable);
pr4xis::register_praxis_value!(prop_step_is_deterministic, Deterministic);
pr4xis::register_praxis_value!(prop_containment_is_domain_determined, Verifiable);
pr4xis::register_praxis_value!(prop_recovery_only_for_contained_faults, Verifiable);
pr4xis::register_praxis_value!(prop_no_action_restores_kernel_integrity, Verifiable);

/// A corrupted kernel accepts no OS-side action: probe, requests,
/// interrupt handling, and recovery are all rejected (Swift et al.
/// 2003 sec. 1); only the hardware-initiated interrupt line can still
/// change state.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn corrupted_kernel_blocks_os_side_actions() {
    let end = run_script(&in_kernel_initial(), &FAULT_CONTAINMENT_SCRIPT)
        .unwrap_or_else(|e| panic!("containment script must be enabled: {e}"));
    assert_eq!(end.kernel, KernelIntegrity::Corrupted);
    for action in [
        DriverAction::Probe,
        DriverAction::Read,
        DriverAction::Write,
        DriverAction::HandleInterrupt,
        DriverAction::Recover,
    ] {
        assert!(
            apply(&end, &action).is_err(),
            "{action:?} must be rejected on a corrupted kernel"
        );
    }
}

/// The interrupt round trip: a ready device raises its line, the bound
/// driver's handler services it back to ready (Corbet et al. 2005,
/// Ch. 10) — and an unloaded driver cannot handle the interrupt.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn interrupt_round_trip() {
    let bound = apply(&isolated_initial(), &DriverAction::Probe)
        .unwrap_or_else(|e| panic!("probe must be enabled initially: {e}"));
    let asserted = apply(&bound, &DriverAction::RaiseInterrupt)
        .unwrap_or_else(|e| panic!("a ready device raises its line: {e}"));
    assert_eq!(asserted.device, DeviceState::InterruptAsserted);
    let serviced = apply(&asserted, &DriverAction::HandleInterrupt)
        .unwrap_or_else(|e| panic!("the bound driver services the interrupt: {e}"));
    assert_eq!(serviced.device, DeviceState::Ready);
    // No handler without a driver: the same interrupt in the unprobed
    // configuration cannot even be raised, let alone handled.
    assert!(apply(&isolated_initial(), &DriverAction::RaiseInterrupt).is_err());
    assert!(apply(&isolated_initial(), &DriverAction::HandleInterrupt).is_err());
}
