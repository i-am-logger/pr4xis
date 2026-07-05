//! Functor: Driver → Dependability.
//!
//! Reads the driver ontology through the Avizienis, Laprie, Randell &
//! Landwehr (2004) taxonomy of dependable computing — the reading Swift,
//! Bershad & Levy (2003) made canonical when they measured driver code
//! as the dominant cause of OS failures:
//!
//! - `DriverFault` is a `Fault` — the adjudged cause of kernel errors
//!   (Swift et al. 2003 sec. 1; Avizienis sec. 2.2).
//! - `IsolationDomain` is `FaultHandling` — Avizienis sec. 5.2's
//!   "diagnosis, isolation, reconfiguration": preventing located
//!   faults from being activated again is exactly what the Nooks
//!   containment boundary does.
//! - `Recovery` is `ErrorRecovery` — replacing erroneous state with
//!   error-free state (the driver restart without a kernel crash).
//! - `DriverAsServer` and `Microdriver` are `FaultTolerance` means —
//!   architectures that deliver correct service in the presence of
//!   driver faults (Liedtke 1995; Ganapathy et al. 2008 read through
//!   Avizienis sec. 5.2).
//! - `Interrupt` maps to `Activation` — the activation-related event
//!   concept of the taxonomy (an honest analogy, not an equation: the
//!   interrupt is an asynchronous event that activates a response, not
//!   a fault becoming active — see `citings.md`).
//! - `DeviceModel` maps to `FaultPrevention` — Ryzhyk et al. (2009)
//!   synthesis-by-construction is a development methodology that
//!   prevents driver development faults (Avizienis sec. 5.1).
//! - **Everything else collapses to `Service`** — the delivered
//!   behaviour (Avizienis sec. 2.1). Documented collapses: `Driver`,
//!   `Device`, `CharacterDevice`, `BlockDevice`, `NetworkDevice`,
//!   `HardwareRegister`, `InterruptHandler`, `Dma`, and `Hal` are all
//!   below the dependability taxonomy's resolution — it sees only the
//!   service they jointly deliver, not the mechanism delivering it.
//!
//! The map is TOTAL: every driver concept has a dependability image.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{DriverCategory, DriverConcept, DriverRelation, DriverRelationKind};
use crate::applied::dependability::ontology::{
    DependabilityCategory, DependabilityConcept, DependabilityRelation, DependabilityRelationKind,
};

/// Maps each driver concept to its place in the Avizienis taxonomy
/// (Avizienis et al. 2004; Swift et al. 2003).
pub struct DriverToDependability;

impl Functor for DriverToDependability {
    type Source = DriverCategory;
    type Target = DependabilityCategory;

    fn map_object(obj: &DriverConcept) -> DependabilityConcept {
        use DriverConcept as D;
        match obj {
            // The threat: driver code is the dominant fault source
            // (Swift et al. 2003 sec. 1).
            D::DriverFault => DependabilityConcept::Fault,
            // The containment boundary is fault handling — diagnosis,
            // isolation, reconfiguration (Avizienis sec. 5.2).
            D::IsolationDomain => DependabilityConcept::FaultHandling,
            // Driver restart without kernel crash is error recovery.
            D::Recovery => DependabilityConcept::ErrorRecovery,
            // The isolated architectures are fault-tolerance means.
            D::DriverAsServer | D::Microdriver => DependabilityConcept::FaultTolerance,
            // The asynchronous service-requesting event lands on the
            // taxonomy's activation event (documented analogy).
            D::Interrupt => DependabilityConcept::Activation,
            // Synthesis-by-construction prevents development faults
            // (Ryzhyk et al. 2009; Avizienis sec. 5.1).
            D::DeviceModel => DependabilityConcept::FaultPrevention,
            // Everything else is the delivered service — the collapse
            // documented in the module header.
            D::Driver
            | D::Device
            | D::CharacterDevice
            | D::BlockDevice
            | D::NetworkDevice
            | D::HardwareRegister
            | D::InterruptHandler
            | D::Dma
            | D::Hal => DependabilityConcept::Service,
        }
    }

    fn map_morphism(m: &DriverRelation) -> DependabilityRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            DriverRelationKind::Identity => return DependabilityCategory::identity(&from),
            // Below the taxonomy's resolution: both endpoints of every
            // Drives / Accesses / Abstracts / Parthood edge collapse
            // into the one delivered Service, so the arrow's content is
            // invisible to dependability — the image is the identity on
            // the collapsed object (constant-functor reading).
            DriverRelationKind::Drives
            | DriverRelationKind::Accesses
            | DriverRelationKind::Abstracts
            | DriverRelationKind::Parthood => return DependabilityCategory::identity(&from),
            // The handler counters the raised event — the response/
            // event pair reads as opposition (cf. the concurrency
            // functor's Violates -> Opposition precedent).
            DriverRelationKind::Handles => DependabilityRelationKind::Opposition,
            // Containment causally protects what it wraps: isolation
            // is what brings about continued correct service (Swift et
            // al. 2003 sec. 3).
            DriverRelationKind::Isolates => DependabilityRelationKind::Causation,
            // Recovery counters the fault — error recovery vs Fault is
            // the taxonomy's counter-relation.
            DriverRelationKind::Recovers => DependabilityRelationKind::Opposition,
            // Synthesis-by-construction classifies the driver's
            // development under the fault-prevention means (documented
            // reads-as collapse — dependability has no derivation
            // relation).
            DriverRelationKind::SynthesizedFrom => DependabilityRelationKind::Subsumption,
            // The canonical Relations-ontology kinds map to their
            // namesakes (Smith 2005 OBO-RO). Causation/Opposition are
            // declared by the kind enum even though this ontology emits
            // no such edges.
            DriverRelationKind::Subsumption => DependabilityRelationKind::Subsumption,
            DriverRelationKind::Causation => DependabilityRelationKind::Causation,
            DriverRelationKind::Opposition => DependabilityRelationKind::Opposition,
        };
        DependabilityRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    DriverToDependability,
    "Avizienis, Laprie, Randell & Landwehr (2004) IEEE TDSC 1(1); Swift, Bershad & Levy (2003) SOSP"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws_hold() {
        assert_functor_laws::<DriverToDependability>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn driver_fault_is_a_dependability_fault() {
        assert_eq!(
            DriverToDependability::map_object(&DriverConcept::DriverFault),
            DependabilityConcept::Fault
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn containment_is_fault_handling_and_recovery_is_error_recovery() {
        assert_eq!(
            DriverToDependability::map_object(&DriverConcept::IsolationDomain),
            DependabilityConcept::FaultHandling
        );
        assert_eq!(
            DriverToDependability::map_object(&DriverConcept::Recovery),
            DependabilityConcept::ErrorRecovery
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn isolated_architectures_are_fault_tolerance_means() {
        for c in [DriverConcept::DriverAsServer, DriverConcept::Microdriver] {
            assert_eq!(
                DriverToDependability::map_object(&c),
                DependabilityConcept::FaultTolerance,
                "{c:?} should be a fault-tolerance means"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn interrupt_activates_and_model_prevents() {
        assert_eq!(
            DriverToDependability::map_object(&DriverConcept::Interrupt),
            DependabilityConcept::Activation
        );
        assert_eq!(
            DriverToDependability::map_object(&DriverConcept::DeviceModel),
            DependabilityConcept::FaultPrevention
        );
    }

    /// The map is total, and every concept without a specific
    /// dependability role collapses onto the delivered Service — the
    /// documented collapse set.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn remaining_concepts_collapse_to_service() {
        let collapsed = [
            DriverConcept::Driver,
            DriverConcept::Device,
            DriverConcept::CharacterDevice,
            DriverConcept::BlockDevice,
            DriverConcept::NetworkDevice,
            DriverConcept::HardwareRegister,
            DriverConcept::InterruptHandler,
            DriverConcept::Dma,
            DriverConcept::Hal,
        ];
        for c in DriverConcept::variants() {
            let image = DriverToDependability::map_object(&c);
            if collapsed.contains(&c) {
                assert_eq!(image, DependabilityConcept::Service, "{c:?}");
            } else {
                assert_ne!(image, DependabilityConcept::Service, "{c:?}");
            }
        }
    }
}
