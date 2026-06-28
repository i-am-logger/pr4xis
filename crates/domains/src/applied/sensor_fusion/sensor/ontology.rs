//! Sensor — modality taxonomy and composite-sensor parthood.
//!
//! Classifies sensors along the canonical sensor-fusion modality axes:
//! proprioceptive vs exteroceptive (what the sensor measures — internal
//! vs external state), and active vs passive (whether the sensor emits
//! its own energy). Composite sensors (IMU, AHRS, INS) are modelled as
//! `has-a` aggregations of their component sensors.
//!
//! # Literature
//!
//! - **Groves (2013)** *Principles of GNSS, Inertial, and Multisensor
//!   Integrated Navigation Systems*, 2nd ed., Ch. 1 — the canonical
//!   proprioceptive / exteroceptive / active / passive sensor-modality
//!   taxonomy used throughout integrated navigation literature.
//! - **Bar-Shalom, Li & Kirubarajan (2001)** *Estimation with
//!   Applications to Tracking and Navigation*, Ch. 1 — sensor modality
//!   classification for multi-sensor fusion.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Sensor",
    source: "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. Ch. 1; Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation Ch. 1",

    concepts: [
        // === Modality abstractions (Groves 2013 §1) ===
        Sensor,
        ProprioceptiveSensor,
        ExteroceptiveSensor,
        ActiveSensor,
        PassiveSensor,

        // === Inertial (proprioceptive) ===
        Accelerometer,
        Gyroscope,
        Magnetometer,

        // === Position (exteroceptive, passive) ===
        GnssReceiver,
        StarTracker,

        // === Range (exteroceptive, active) ===
        Radar,
        LiDAR,
        Sonar,

        // === Vision (exteroceptive, passive/active) ===
        Camera,
        InfraredCamera,
        DepthCamera,

        // === Pressure (proprioceptive) ===
        Barometer,
        DepthSensor,

        // === Velocity (exteroceptive, active) ===
        DopplerVelocityLog,

        // === Composite (Groves 2013 §7, §13) ===
        IMU,
        AHRS,
        INS,
    ],

    labels: {
        Sensor: ("en", "Sensor", "Groves (2013) §1: any device that converts a physical property to a quantitative reading."),
        ProprioceptiveSensor: ("en", "Proprioceptive sensor", "Groves (2013) §1: a sensor that measures the platform's own internal state (e.g. IMU, odometry)."),
        ExteroceptiveSensor: ("en", "Exteroceptive sensor", "Groves (2013) §1: a sensor that measures the external environment (e.g. GNSS, LiDAR, camera, radar)."),
        ActiveSensor: ("en", "Active sensor", "Groves (2013) §1: a sensor that emits its own energy and measures the return (radar, LiDAR, sonar)."),
        PassiveSensor: ("en", "Passive sensor", "Groves (2013) §1: a sensor that measures ambient energy (camera, GNSS, magnetometer)."),

        Accelerometer: ("en", "Accelerometer", "An inertial sensor measuring specific force (proper acceleration)."),
        Gyroscope: ("en", "Gyroscope", "An inertial sensor measuring angular rate."),
        Magnetometer: ("en", "Magnetometer", "A passive sensor measuring local magnetic-field vector."),
        GnssReceiver: ("en", "GNSS receiver", "A passive exteroceptive sensor receiving satellite navigation signals."),
        StarTracker: ("en", "Star tracker", "A passive optical sensor measuring attitude against a star catalogue."),
        Radar: ("en", "Radar", "An active exteroceptive sensor using radio waves for range and velocity."),
        LiDAR: ("en", "LiDAR", "An active exteroceptive sensor using laser pulses for range."),
        Sonar: ("en", "Sonar", "An active exteroceptive sensor using acoustic waves for range."),
        Camera: ("en", "Camera", "A passive optical sensor recording visible-band imagery."),
        InfraredCamera: ("en", "Infrared camera", "A passive optical sensor recording thermal-infrared imagery."),
        DepthCamera: ("en", "Depth camera", "An active range-imaging camera (e.g. structured light, time-of-flight)."),
        Barometer: ("en", "Barometer", "A proprioceptive sensor measuring atmospheric pressure for altitude."),
        DepthSensor: ("en", "Depth sensor", "A proprioceptive sensor measuring hydrostatic pressure for underwater depth."),
        DopplerVelocityLog: ("en", "Doppler velocity log", "An active exteroceptive sensor measuring platform velocity via Doppler shift off the seabed or surface."),
        IMU: ("en", "IMU", "Groves (2013) §7: Inertial Measurement Unit — accelerometer + gyroscope triad."),
        AHRS: ("en", "AHRS", "Groves (2013) §6: Attitude and Heading Reference System — IMU + magnetometer."),
        INS: ("en", "INS", "Groves (2013) §13: Inertial Navigation System — IMU plus integration to position."),
    },

    is_a: [
        // Modality taxonomy (Groves 2013 §1).
        (ProprioceptiveSensor, Sensor),
        (ExteroceptiveSensor, Sensor),
        (ActiveSensor, Sensor),
        (PassiveSensor, Sensor),

        // Proprioceptive sensors.
        (Accelerometer, ProprioceptiveSensor),
        (Gyroscope, ProprioceptiveSensor),
        (Magnetometer, ProprioceptiveSensor),
        (Barometer, ProprioceptiveSensor),
        (DepthSensor, ProprioceptiveSensor),
        (IMU, ProprioceptiveSensor),
        (AHRS, ProprioceptiveSensor),
        (INS, ProprioceptiveSensor),

        // Exteroceptive sensors split by active/passive.
        (GnssReceiver, ExteroceptiveSensor),
        (GnssReceiver, PassiveSensor),
        (StarTracker, ExteroceptiveSensor),
        (StarTracker, PassiveSensor),
        (Radar, ExteroceptiveSensor),
        (Radar, ActiveSensor),
        (LiDAR, ExteroceptiveSensor),
        (LiDAR, ActiveSensor),
        (Sonar, ExteroceptiveSensor),
        (Sonar, ActiveSensor),
        (DopplerVelocityLog, ExteroceptiveSensor),
        (DopplerVelocityLog, ActiveSensor),
        (Camera, ExteroceptiveSensor),
        (Camera, PassiveSensor),
        (InfraredCamera, ExteroceptiveSensor),
        (InfraredCamera, PassiveSensor),
        (DepthCamera, ExteroceptiveSensor),
        (DepthCamera, ActiveSensor),
    ],

    has_a: [
        // Composite sensors (Groves 2013 §7, §13).
        (IMU, Accelerometer),
        (IMU, Gyroscope),
        (AHRS, Accelerometer),
        (AHRS, Gyroscope),
        (AHRS, Magnetometer),
        (INS, Accelerometer),
        (INS, Gyroscope),
    ],

    opposes: [
        // The modality axes are disjoint partitions.
        (ProprioceptiveSensor, ExteroceptiveSensor),
        (ExteroceptiveSensor, ProprioceptiveSensor),
        (ActiveSensor, PassiveSensor),
        (PassiveSensor, ActiveSensor),
    ],
}

/// Look up the part-of relation: which components make up a composite
/// sensor (Groves 2013 §7, §13).
pub fn parts_of(whole: SensorConcept) -> Vec<SensorConcept> {
    SensorCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == SensorRelationKind::Parthood && m.target() == whole)
        .map(|m| m.source())
        .collect()
}

/// Check whether `child` is-a `parent` via the subsumption edges.
pub fn is_a(child: SensorConcept, parent: SensorConcept) -> bool {
    SensorCategory::morphisms().iter().any(|m| {
        m.kind() == SensorRelationKind::Subsumption && m.source() == child && m.target() == parent
    })
}

/// Quality: whether a sensor is proprioceptive (measures internal platform
/// state) per Groves (2013) §1.
#[derive(Debug, Clone)]
pub struct IsProprioceptive;

impl Quality for IsProprioceptive {
    type Individual = SensorConcept;
    type Value = bool;

    fn get(&self, s: &SensorConcept) -> Option<bool> {
        Some(is_a(*s, SensorConcept::ProprioceptiveSensor))
    }
}

impl Ontology for SensorOntology {
    type Cat = SensorCategory;
    type Qual = IsProprioceptive;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(AccelerometerIsSensor));
        axioms.push(Box::new(ImuComposition));
        axioms.push(Box::new(RadarDualClassification));
        axioms.push(Box::new(CameraIsPassive));
        axioms
    }
}

/// Axiom: Accelerometer is-a Sensor (transitive via ProprioceptiveSensor).
///
/// Groves (2013) §1 — every modality subdivision is-a Sensor; transitivity
/// of the is-a relation gives Accelerometer ↪ ProprioceptiveSensor ↪ Sensor.
pub struct AccelerometerIsSensor;

impl Axiom for AccelerometerIsSensor {
    fn verify(&self) -> Verdict {
        if is_a(
            SensorConcept::Accelerometer,
            SensorConcept::ProprioceptiveSensor,
        ) && is_a(SensorConcept::ProprioceptiveSensor, SensorConcept::Sensor)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AccelerometerIsSensor",
        "Accelerometer is-a Sensor (transitive via ProprioceptiveSensor)",
        "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. §1"
    );
}

pr4xis::register_axiom!(
    AccelerometerIsSensor,
    "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. §1"
);

/// Axiom: IMU has-a Accelerometer and has-a Gyroscope.
///
/// Groves (2013) §7 — an Inertial Measurement Unit is by definition the
/// triad pairing of accelerometers and rate gyroscopes.
pub struct ImuComposition;

impl Axiom for ImuComposition {
    fn verify(&self) -> Verdict {
        let parts = parts_of(SensorConcept::IMU);
        if parts.contains(&SensorConcept::Accelerometer)
            && parts.contains(&SensorConcept::Gyroscope)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ImuComposition",
        "IMU has-a Accelerometer and has-a Gyroscope",
        "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. §7"
    );
}

pr4xis::register_axiom!(
    ImuComposition,
    "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. §7"
);

/// Axiom: Radar is-a ExteroceptiveSensor AND is-a ActiveSensor.
///
/// Groves (2013) §1 — modalities are not mutually exclusive across axes;
/// radar is both exteroceptive (measures the environment) and active
/// (emits its own waveform).
pub struct RadarDualClassification;

impl Axiom for RadarDualClassification {
    fn verify(&self) -> Verdict {
        if is_a(SensorConcept::Radar, SensorConcept::ExteroceptiveSensor)
            && is_a(SensorConcept::Radar, SensorConcept::ActiveSensor)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RadarDualClassification",
        "Radar is-a ExteroceptiveSensor AND is-a ActiveSensor",
        "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. §1"
    );
}

pr4xis::register_axiom!(
    RadarDualClassification,
    "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. §1"
);

/// Axiom: Camera is-a PassiveSensor and is NOT ActiveSensor.
///
/// Groves (2013) §1 — visible-band cameras are passive (they record
/// ambient illumination); compare with the depth-camera variant which
/// is active. Models the modality-axis disjointness for the camera kind.
pub struct CameraIsPassive;

impl Axiom for CameraIsPassive {
    fn verify(&self) -> Verdict {
        if is_a(SensorConcept::Camera, SensorConcept::PassiveSensor)
            && !is_a(SensorConcept::Camera, SensorConcept::ActiveSensor)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CameraIsPassive",
        "Camera is-a PassiveSensor and is NOT ActiveSensor",
        "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. §1"
    );
}

pr4xis::register_axiom!(
    CameraIsPassive,
    "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. §1"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<SensorCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        SensorOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn twenty_two_sensor_concepts() {
        // 5 modality abstractions + 17 concrete sensors = 22.
        assert_eq!(SensorConcept::variants().len(), 22);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn modality_axes_oppose() {
        let opp: Vec<_> = SensorCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == SensorRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(
            SensorConcept::ProprioceptiveSensor,
            SensorConcept::ExteroceptiveSensor
        )));
        assert!(opp.contains(&(SensorConcept::ActiveSensor, SensorConcept::PassiveSensor)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn imu_parts_include_accel_and_gyro() {
        let parts = parts_of(SensorConcept::IMU);
        assert!(parts.contains(&SensorConcept::Accelerometer));
        assert!(parts.contains(&SensorConcept::Gyroscope));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ahrs_includes_magnetometer() {
        let parts = parts_of(SensorConcept::AHRS);
        assert!(parts.contains(&SensorConcept::Magnetometer));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn accelerometer_is_proprioceptive() {
        assert_eq!(
            IsProprioceptive.get(&SensorConcept::Accelerometer),
            Some(true)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn camera_is_not_proprioceptive() {
        assert_eq!(IsProprioceptive.get(&SensorConcept::Camera), Some(false));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn accelerometer_is_sensor_axiom() {
        assert!(AccelerometerIsSensor.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn imu_composition_axiom() {
        assert!(ImuComposition.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn radar_dual_classification_axiom() {
        assert!(RadarDualClassification.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn camera_is_passive_axiom() {
        assert!(CameraIsPassive.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = SensorConcept> {
        proptest::sample::select(SensorConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in SensorCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in SensorOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_is_proprioceptive_total(c in arb_concept()) {
            // IsProprioceptive is total over every sensor concept (returns
            // true or false, never None).
            prop_assert!(IsProprioceptive.get(&c).is_some());
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = SensorCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == SensorRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = SensorConcept::variants();
            for m in SensorCategory::morphisms() {
                if m.kind() == SensorRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_is_proprioceptive_total, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
    pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
}
