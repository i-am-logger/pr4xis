//! Driver engine — a typed driver/device state machine with fault
//! injection, after the Nooks fault-containment experiments (Swift,
//! Bershad & Levy 2003, SOSP, "Improving the Reliability of Commodity
//! Operating Systems"):
//!
//! - the **situation** is device state × driver state × kernel
//!   integrity × isolation-domain membership;
//! - the **actions** are the driver lifecycle steps of Corbet, Rubini &
//!   Kroah-Hartman (2005) *Linux Device Drivers* 3e — probe, read,
//!   write, raise/handle interrupt — plus the Nooks experiment's fault
//!   injection and recovery;
//! - the fixture runs the same fault-injection script twice: once with
//!   the driver in the kernel address space and once inside an
//!   isolation domain. Only the first corrupts kernel state (Swift et
//!   al. 2003 sec. 3), which is exactly what the
//!   `IsolatedDriverContainsFault` axiom in [`super::ontology`]
//!   demonstrates.
//!
//! Every state component is a closed typed enum with a documented `ALL`
//! set — no free magic numbers anywhere in the fixture.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

// ---------------------------------------------------------------------------
// State components
// ---------------------------------------------------------------------------

/// The device's operational state as the operating system sees it —
/// Corbet, Rubini & Kroah-Hartman (2005): a device is reachable only
/// once its driver has bound to it, and it signals asynchronously by
/// asserting its interrupt line (Ch. 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    /// Present on the bus but not yet bound to a driver.
    Unprobed,
    /// Bound to its driver and able to accept read/write requests.
    Ready,
    /// The device has asserted its interrupt line and awaits service
    /// (Corbet et al. 2005, Ch. 10).
    InterruptAsserted,
}

impl DeviceState {
    /// The closed set of device states — the typed state space of the
    /// fixture (no numeric encoding).
    pub const ALL: [DeviceState; 3] = [
        DeviceState::Unprobed,
        DeviceState::Ready,
        DeviceState::InterruptAsserted,
    ];
}

/// The driver's lifecycle state — Corbet, Rubini & Kroah-Hartman
/// (2005) Ch. 1 (the bound driver translating OS requests) and Swift,
/// Bershad & Levy (2003) sec. 1 (the faulted driver as the dominant
/// crash cause).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverState {
    /// Not yet bound to any device.
    Unloaded,
    /// Bound: translating operating-system requests into device
    /// operations (Corbet et al. 2005, Ch. 1).
    Bound,
    /// A fault has fired in driver code (Swift et al. 2003 sec. 1).
    Faulted,
}

impl DriverState {
    /// The closed set of driver states.
    pub const ALL: [DriverState; 3] = [
        DriverState::Unloaded,
        DriverState::Bound,
        DriverState::Faulted,
    ];
}

/// Whether kernel state is still trustworthy — Swift, Bershad & Levy
/// (2003) sec. 1: a fault in an in-kernel driver corrupts the state of
/// the kernel it shares an address space with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelIntegrity {
    /// Kernel data structures are uncorrupted.
    Intact,
    /// Driver-fault damage has propagated into kernel state; only a
    /// reboot restores it (Swift et al. 2003 sec. 1).
    Corrupted,
}

impl KernelIntegrity {
    /// The closed set of integrity values.
    pub const ALL: [KernelIntegrity; 2] = [KernelIntegrity::Intact, KernelIntegrity::Corrupted];
}

/// Where the driver executes — the isolation design space: in the
/// kernel address space (the commodity default, Swift et al. 2003
/// sec. 1) or inside a fault-containment boundary (Nooks' lightweight
/// protection domain, Swift et al. 2003 sec. 3; the user-space server
/// of Liedtke 1995 sec. 4 is the strong end of the same axis).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationMembership {
    /// The driver shares the kernel's address space — its faults write
    /// directly into kernel state.
    KernelAddressSpace,
    /// The driver runs inside a fault-containment boundary — its
    /// faults are stopped at the domain edge.
    IsolationDomain,
}

impl IsolationMembership {
    /// The closed set of membership values.
    pub const ALL: [IsolationMembership; 2] = [
        IsolationMembership::KernelAddressSpace,
        IsolationMembership::IsolationDomain,
    ];
}

// ---------------------------------------------------------------------------
// Situation + actions
// ---------------------------------------------------------------------------

/// The joint state of one device, its driver, the kernel it serves,
/// and the driver's isolation-domain membership — the engine
/// `Situation` of the Nooks containment experiment (Swift et al. 2003
/// sec. 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverSituation {
    /// The device's operational state.
    pub device: DeviceState,
    /// The driver's lifecycle state.
    pub driver: DriverState,
    /// Whether kernel state is still trustworthy.
    pub kernel: KernelIntegrity,
    /// Where the driver executes.
    pub domain: IsolationMembership,
}

impl Situation for DriverSituation {}

/// One step of the driver/device machine — the engine `Action`.
///
/// Probe/Read/Write are the driver translating operating-system
/// requests into device operations (Corbet et al. 2005, Ch. 1);
/// RaiseInterrupt/HandleInterrupt are the asynchronous service path
/// (Ch. 10); InjectFault/Recover are the Nooks experiment (Swift et
/// al. 2003 sec. 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverAction {
    /// Bind the driver to the device (Corbet et al. 2005).
    Probe,
    /// Service an OS read request through the bound driver.
    Read,
    /// Service an OS write request through the bound driver.
    Write,
    /// The device asserts its interrupt line (hardware-initiated —
    /// Corbet et al. 2005, Ch. 10).
    RaiseInterrupt,
    /// The driver's interrupt handler services and acknowledges the
    /// asserted interrupt (Corbet et al. 2005, Ch. 10).
    HandleInterrupt,
    /// A fault fires in driver code (the injection step of Swift et
    /// al. 2003 sec. 3's containment experiment).
    InjectFault,
    /// Restart the failed driver without a kernel crash (Swift et al.
    /// 2003) — possible only while kernel state is intact.
    Recover,
}

impl DriverAction {
    /// The closed set of actions.
    pub const ALL: [DriverAction; 7] = [
        DriverAction::Probe,
        DriverAction::Read,
        DriverAction::Write,
        DriverAction::RaiseInterrupt,
        DriverAction::HandleInterrupt,
        DriverAction::InjectFault,
        DriverAction::Recover,
    ];
}

impl Action for DriverAction {
    type Sit = DriverSituation;
}

// ---------------------------------------------------------------------------
// Transition function
// ---------------------------------------------------------------------------

/// Apply one action. `Err` when the action is not enabled: OS-side
/// steps (probe, read, write, interrupt handling, recovery) require
/// intact kernel state; requests require a bound driver; recovery
/// requires a contained fault. The device-side `RaiseInterrupt` is
/// hardware-initiated and needs only a probed device (Corbet et al.
/// 2005, Ch. 10).
pub fn apply(
    situation: &DriverSituation,
    action: &DriverAction,
) -> Result<DriverSituation, String> {
    let mut next = situation.clone();
    match action {
        DriverAction::Probe => {
            require_intact_kernel(situation, "probe")?;
            if situation.driver != DriverState::Unloaded {
                return Err("probe requires an unloaded driver".to_string());
            }
            if situation.device != DeviceState::Unprobed {
                return Err("probe requires an unprobed device".to_string());
            }
            next.driver = DriverState::Bound;
            next.device = DeviceState::Ready;
        }
        DriverAction::Read | DriverAction::Write => {
            require_intact_kernel(situation, "request")?;
            require_bound_driver(situation)?;
            if situation.device != DeviceState::Ready {
                return Err("request requires a ready device".to_string());
            }
            // The bound driver translates the request into device
            // operations and completes it; the machine's state is
            // unchanged (Corbet et al. 2005, Ch. 1).
        }
        DriverAction::RaiseInterrupt => {
            if situation.device != DeviceState::Ready {
                return Err("only a ready device raises its interrupt line".to_string());
            }
            next.device = DeviceState::InterruptAsserted;
        }
        DriverAction::HandleInterrupt => {
            require_intact_kernel(situation, "interrupt handling")?;
            require_bound_driver(situation)?;
            if situation.device != DeviceState::InterruptAsserted {
                return Err("no interrupt is asserted".to_string());
            }
            next.device = DeviceState::Ready;
        }
        DriverAction::InjectFault => {
            require_bound_driver(situation)?;
            next.driver = DriverState::Faulted;
            // The Nooks claim (Swift et al. 2003 sec. 1 and sec. 3): a
            // fault in the kernel address space corrupts kernel state;
            // the same fault inside an isolation domain is stopped at
            // the domain boundary.
            if situation.domain == IsolationMembership::KernelAddressSpace {
                next.kernel = KernelIntegrity::Corrupted;
            }
        }
        DriverAction::Recover => {
            require_intact_kernel(situation, "recovery")?;
            if situation.driver != DriverState::Faulted {
                return Err("recovery requires a faulted driver".to_string());
            }
            // Restart the failed driver without a kernel crash (Swift
            // et al. 2003): the driver rebinds and the device is
            // re-initialised to its ready state.
            next.driver = DriverState::Bound;
            next.device = DeviceState::Ready;
        }
    }
    Ok(next)
}

fn require_intact_kernel(situation: &DriverSituation, step: &str) -> Result<(), String> {
    if situation.kernel == KernelIntegrity::Corrupted {
        return Err(format!(
            "{step} impossible: kernel state is corrupted (Swift et al. 2003 sec. 1)"
        ));
    }
    Ok(())
}

fn require_bound_driver(situation: &DriverSituation) -> Result<(), String> {
    match situation.driver {
        DriverState::Bound => Ok(()),
        DriverState::Unloaded => Err("no driver is bound to the device".to_string()),
        DriverState::Faulted => Err("the driver has faulted".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Initial situation of the commodity configuration: the driver will
/// run in the kernel address space (Swift et al. 2003 sec. 1 — the
/// default that makes driver faults the dominant crash cause).
pub fn in_kernel_initial() -> DriverSituation {
    DriverSituation {
        device: DeviceState::Unprobed,
        driver: DriverState::Unloaded,
        kernel: KernelIntegrity::Intact,
        domain: IsolationMembership::KernelAddressSpace,
    }
}

/// Initial situation of the isolated configuration: the driver will
/// run inside a fault-containment boundary (Swift et al. 2003 sec. 3;
/// Liedtke 1995 sec. 4).
pub fn isolated_initial() -> DriverSituation {
    DriverSituation {
        device: DeviceState::Unprobed,
        driver: DriverState::Unloaded,
        kernel: KernelIntegrity::Intact,
        domain: IsolationMembership::IsolationDomain,
    }
}

/// The fault-injection script of the containment experiment — bind the
/// driver, then fire a fault in it (Swift et al. 2003 sec. 3: identical
/// faults are injected into isolated and non-isolated drivers and the
/// kernel's fate compared).
pub const FAULT_CONTAINMENT_SCRIPT: [DriverAction; 2] =
    [DriverAction::Probe, DriverAction::InjectFault];

/// Run a script of actions from an initial situation, failing on the
/// first disabled action.
pub fn run_script(
    initial: &DriverSituation,
    script: &[DriverAction],
) -> Result<DriverSituation, String> {
    let mut current = initial.clone();
    for action in script {
        current = apply(&current, action)?;
    }
    Ok(current)
}
