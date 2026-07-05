//! Functor: SmartElement → Sensor.
//!
//! The smart-transducer reading (IEEE Std 1451.0-2007; Lee 2000): a smart
//! element, seen only as a sensing device, is a `Sensor`. The sensor
//! ontology (Groves 2013) classifies sensors by modality — proprioceptive
//! / exteroceptive, active / passive — a taxonomy the smart-element
//! concepts do not refine, so this is a **forgetful** functor: every
//! smart-element concept collapses onto the `Sensor` umbrella.
//!
//! # Object mapping (each arm documented)
//!
//! | SmartElement | Sensor | Why |
//! |---|---|---|
//! | `SmartSensor` | `Sensor` | The sensing element IS a sensor (IEEE 1451 "smart transducer") |
//! | `Transducer` | `Sensor` | Groves (2013) §1: the physical element that converts a property to a reading is a sensor |
//! | everything else | `Sensor` | Documented forgetful collapse: the autonomic-loop and self-* concepts have no place in the sensor modality taxonomy, so they collapse onto the umbrella |
//!
//! # Morphism-kind mapping
//!
//! The sensor ontology's vocabulary is purely structural (Subsumption /
//! Parthood / Opposition — no custom edges). The four canonical kinds map
//! to their namesakes; the five smart-element custom edges (`Carries`,
//! `Exhibits`, `Operates`, `DescribedBy`, `Manages`) all collapse onto
//! `Parthood` — under the forgetful sensor reading, every constitutive
//! relation of the element is internal makeup of the one sensing thing
//! (documented collapse; the source has no transitive custom-kind chains,
//! so the collapse is lawful — only identity morphisms compose).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Functor};

use super::ontology::{
    SmartElementCategory, SmartElementConcept, SmartElementRelation, SmartElementRelationKind,
};
use crate::applied::sensor_fusion::sensor::ontology::{
    SensorCategory, SensorConcept, SensorRelation, SensorRelationKind,
};

/// Maps every smart-element concept onto the `Sensor` umbrella — the
/// forgetful smart-transducer reading.
pub struct SmartElementToSensor;

impl Functor for SmartElementToSensor {
    type Source = SmartElementCategory;
    type Target = SensorCategory;

    fn map_object(_obj: &SmartElementConcept) -> SensorConcept {
        // Forgetful collapse: seen purely as a sensing device, every
        // smart-element concept is the abstract Sensor (Groves 2013 §1;
        // IEEE 1451.0-2007). The modality taxonomy is not refined here.
        SensorConcept::Sensor
    }

    fn map_morphism(m: &SmartElementRelation) -> SensorRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            SmartElementRelationKind::Identity => return SensorCategory::identity(&from),
            // The four canonical Relations-ontology kinds map to namesakes
            // (Smith 2005 OBO-RO).
            SmartElementRelationKind::Subsumption => SensorRelationKind::Subsumption,
            SmartElementRelationKind::Parthood => SensorRelationKind::Parthood,
            SmartElementRelationKind::Causation => SensorRelationKind::Causation,
            SmartElementRelationKind::Opposition => SensorRelationKind::Opposition,
            // Under the forgetful collapse onto the single Sensor umbrella,
            // every constitutive edge of the element reads as internal
            // makeup of the one sensing thing (documented collapse).
            SmartElementRelationKind::Carries
            | SmartElementRelationKind::Exhibits
            | SmartElementRelationKind::Operates
            | SmartElementRelationKind::DescribedBy
            | SmartElementRelationKind::Manages => SensorRelationKind::Parthood,
        };
        SensorRelation { from, to, kind }
    }
}
pr4xis::register_functor!(
    SmartElementToSensor,
    "IEEE Std 1451.0-2007; Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed."
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn smart_element_to_sensor_functor_laws() {
        assert_functor_laws::<SmartElementToSensor>();
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn smart_sensor_and_transducer_are_sensors() {
        use SmartElementConcept as C;
        assert_eq!(
            SmartElementToSensor::map_object(&C::SmartSensor),
            SensorConcept::Sensor
        );
        assert_eq!(
            SmartElementToSensor::map_object(&C::Transducer),
            SensorConcept::Sensor
        );
    }

    /// The functor is total and forgetful: every concept lands on the
    /// Sensor umbrella.
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn forgetful_collapse_is_total() {
        use pr4xis::category::FinitelyGenerated;
        for c in SmartElementConcept::variants() {
            assert_eq!(SmartElementToSensor::map_object(&c), SensorConcept::Sensor);
        }
    }
}
