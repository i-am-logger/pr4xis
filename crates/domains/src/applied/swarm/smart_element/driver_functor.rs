//! Functor: SmartElement → Driver.
//!
//! The operating-system reading, and the **synthesis anchor** of this
//! whole ontology: a `SmartDriver` is a `Driver` (Corbet, Rubini &
//! Kroah-Hartman 2005) made autonomic. Three arms are genuinely faithful,
//! not collapses, and the tests pin them:
//!
//! - `SmartDriver → Driver` — the synthesis anchor;
//! - `Teds → DeviceModel` — a TEDS *is* a formal device self-description
//!   (IEEE Std 1451.0-2007 §5; Ryzhyk et al. 2009 synthesize a driver from
//!   exactly such a model);
//! - `SelfHealing → Recovery` — autonomic healing *is* driver recovery
//!   (Swift, Bershad & Levy 2003, shadow/restartable drivers).
//!
//! # Object mapping (each arm documented)
//!
//! | SmartElement | Driver | Why |
//! |---|---|---|
//! | `SmartDriver` | `Driver` | Faithful — the synthesis anchor |
//! | `Teds` | `DeviceModel` | Faithful — a TEDS is a formal device self-description (IEEE 1451.0 §5; Ryzhyk et al. 2009) |
//! | `LocalOntology` | `DeviceModel` | The element's queryable self-description is, in OS terms, a device model (documented) |
//! | `SelfHealing` | `Recovery` | Faithful — autonomic healing is driver recovery (Swift et al. 2003) |
//! | `SelfProtection` | `IsolationDomain` | Protection is fault containment — the Nooks isolation boundary (documented; Swift et al. 2003) |
//! | `SmartSensor`, `Transducer`, `ManagedElement` | `Device` | The sensed / managed hardware side |
//! | `SmartElement`, `AutonomicManager`, `Ncap` | `Driver` | The driver umbrella (documented collapse) |
//! | `SelfStarProperty`, `SelfConfiguration`, `SelfOptimization` | `Driver` | Documented forgetful collapse onto the driver umbrella |
//!
//! # Morphism-kind mapping
//!
//! The four canonical Relations-ontology kinds map to their namesakes; the
//! five custom edges collapse into two driver buckets:
//!
//! - `Operates`, `Manages`, `Exhibits` → `Drives` (a driver/manager acting
//!   on a device — `Operates` and `Manages` land on the *real*
//!   `Driver → Device Drives` edge under the object map);
//! - `Carries`, `DescribedBy` → `SynthesizedFrom` (binding a thing to its
//!   device model — `Carries` lands on the *real*
//!   `Driver → DeviceModel SynthesizedFrom` edge under the object map).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    SmartElementCategory, SmartElementConcept, SmartElementRelation, SmartElementRelationKind,
};
use crate::applied::operating_system::driver::ontology::{
    DriverCategory, DriverConcept, DriverRelation, DriverRelationKind,
};

/// Maps each smart-element concept to its operating-system driver reading.
pub struct SmartElementToDriver;

impl Functor for SmartElementToDriver {
    type Source = SmartElementCategory;
    type Target = DriverCategory;

    fn map_object(obj: &SmartElementConcept) -> DriverConcept {
        use SmartElementConcept as C;
        match obj {
            // Faithful — the synthesis anchor.
            C::SmartDriver => DriverConcept::Driver,
            // Faithful — a TEDS is a formal device self-description; the
            // element's local ontology is, in OS terms, that device model.
            C::Teds | C::LocalOntology => DriverConcept::DeviceModel,
            // Faithful — autonomic healing is driver recovery (Swift 2003).
            C::SelfHealing => DriverConcept::Recovery,
            // Protection is fault containment — the Nooks isolation domain.
            C::SelfProtection => DriverConcept::IsolationDomain,
            // The sensed / managed hardware side.
            C::SmartSensor | C::Transducer | C::ManagedElement => DriverConcept::Device,
            // The driver umbrella, and the documented forgetful collapses.
            C::SmartElement
            | C::AutonomicManager
            | C::Ncap
            | C::SelfStarProperty
            | C::SelfConfiguration
            | C::SelfOptimization => DriverConcept::Driver,
        }
    }

    fn map_morphism(m: &SmartElementRelation) -> DriverRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            SmartElementRelationKind::Identity => return DriverCategory::identity(&from),
            // The four canonical Relations-ontology kinds map to namesakes
            // (Smith 2005 OBO-RO).
            SmartElementRelationKind::Subsumption => DriverRelationKind::Subsumption,
            SmartElementRelationKind::Parthood => DriverRelationKind::Parthood,
            SmartElementRelationKind::Causation => DriverRelationKind::Causation,
            SmartElementRelationKind::Opposition => DriverRelationKind::Opposition,
            // A driver/manager acting on a device — Operates and Manages
            // land on the real Driver -> Device Drives edge.
            SmartElementRelationKind::Operates
            | SmartElementRelationKind::Manages
            | SmartElementRelationKind::Exhibits => DriverRelationKind::Drives,
            // Binding a thing to its device model — Carries lands on the
            // real Driver -> DeviceModel SynthesizedFrom edge.
            SmartElementRelationKind::Carries | SmartElementRelationKind::DescribedBy => {
                DriverRelationKind::SynthesizedFrom
            }
        };
        DriverRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    SmartElementToDriver,
    "Corbet, Rubini & Kroah-Hartman (2005) Linux Device Drivers 3rd ed. Ch. 1; Swift, Bershad & Levy (2003) SOSP; IEEE Std 1451.0-2007"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn smart_element_to_driver_functor_laws() {
        assert_functor_laws::<SmartElementToDriver>();
    }

    /// The three faithful anchors of the synthesis.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn faithful_anchors() {
        use SmartElementConcept as C;
        assert_eq!(
            SmartElementToDriver::map_object(&C::SmartDriver),
            DriverConcept::Driver
        );
        assert_eq!(
            SmartElementToDriver::map_object(&C::Teds),
            DriverConcept::DeviceModel
        );
        assert_eq!(
            SmartElementToDriver::map_object(&C::SelfHealing),
            DriverConcept::Recovery
        );
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn protection_is_isolation_and_sensor_is_device() {
        use SmartElementConcept as C;
        assert_eq!(
            SmartElementToDriver::map_object(&C::SelfProtection),
            DriverConcept::IsolationDomain
        );
        assert_eq!(
            SmartElementToDriver::map_object(&C::SmartSensor),
            DriverConcept::Device
        );
    }

    /// The `Operates` edge (`SmartDriver → Transducer`) lands on the real
    /// driver edge `Driver → Device Drives`.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn operates_lands_on_declared_drives_edge() {
        let source_edge = SmartElementCategory::morphisms()
            .into_iter()
            .find(|m| {
                m.source() == SmartElementConcept::SmartDriver
                    && m.target() == SmartElementConcept::Transducer
                    && m.kind() == SmartElementRelationKind::Operates
            })
            .expect("the SmartDriver operates Transducer edge is declared");
        let image = SmartElementToDriver::map_morphism(&source_edge);
        assert_eq!(image.from, DriverConcept::Driver);
        assert_eq!(image.to, DriverConcept::Device);
        assert_eq!(image.kind, DriverRelationKind::Drives);
        let declared = DriverCategory::morphisms().into_iter().any(|m| {
            m.source() == DriverConcept::Driver
                && m.target() == DriverConcept::Device
                && m.kind() == DriverRelationKind::Drives
        });
        assert!(declared, "Driver -> Device Drives is a real driver edge");
    }
}
