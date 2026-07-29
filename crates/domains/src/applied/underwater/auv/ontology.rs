//! AUV navigation sensor types.
//!
//! Source: Kinsey et al. (2006), "A Survey of Underwater Vehicle Navigation"

use crate::formal::math::quantity::dimension::Dimension;
use pr4xis::category::FinitelyGenerated;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Auv",
    source: "Kinsey et al. (2006); Paull et al. (2014)",

    concepts: [DVL, DepthSensor, Compass, ADCP],

    labels: {
        DVL: ("en", "Doppler Velocity Log", "Doppler Velocity Log: measures velocity relative to seabed."),
        DepthSensor: ("en", "Depth sensor", "Depth/pressure sensor."),
        Compass: ("en", "Compass", "Magnetic compass / heading sensor."),
        ADCP: ("en", "Acoustic Doppler Current Profiler", "Acoustic Doppler Current Profiler: measures water current profile."),
    },
}

/// Quality: what physical quantity each sensor measures, as its SI
/// [`Dimension`] — DVL/ADCP measure velocity (`L·T⁻¹`), the depth sensor
/// measures a length (depth below the surface), and the compass measures an
/// angle (heading) (Kinsey et al. 2006 §II).
#[derive(Debug, Clone)]
pub struct MeasuredQuantity;

impl Quality for MeasuredQuantity {
    type Individual = AuvConcept;
    type Value = Dimension;

    fn get(&self, sensor: &AuvConcept) -> Option<Dimension> {
        Some(match sensor {
            AuvConcept::DVL => Dimension::VELOCITY,
            AuvConcept::DepthSensor => Dimension::LENGTH,
            AuvConcept::Compass => Dimension::ANGLE,
            AuvConcept::ADCP => Dimension::VELOCITY,
        })
    }
}

/// The reference frame against which a Doppler sensor's velocity is measured.
///
/// A Doppler velocity is only meaningful relative to a scatterer: the DVL's
/// beams reflect off the *stationary seabed* (bottom lock), so its velocity is
/// referenced to the seabed; the ADCP's beams reflect off *moving suspended
/// particles in the water column*, so its velocity is referenced to the water.
/// This distinction — seabed vs. water column — IS the physical content of
/// "bottom lock" (Kinsey et al. 2006 §II; Paull et al. 2014 §3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VelocityReference {
    /// Velocity relative to the stationary seabed — requires bottom lock.
    Seabed,
    /// Velocity relative to the moving water column — no bottom lock.
    WaterColumn,
}

/// Quality: the velocity reference frame of a Doppler-type AUV sensor.
///
/// Partial: only the two Doppler velocity sensors (DVL, ADCP) have a velocity
/// reference frame at all. The depth (pressure) and compass (magnetic heading)
/// sensors measure no velocity, so the quality does not inhere in them (`None`).
#[derive(Debug, Clone)]
pub struct VelocityFrame;

impl Quality for VelocityFrame {
    type Individual = AuvConcept;
    type Value = VelocityReference;

    fn get(&self, sensor: &AuvConcept) -> Option<VelocityReference> {
        match sensor {
            // Bottom lock: DVL velocity is referenced to the seabed.
            AuvConcept::DVL => Some(VelocityReference::Seabed),
            // ADCP velocity is referenced to the moving water column.
            AuvConcept::ADCP => Some(VelocityReference::WaterColumn),
            // Depth (pressure) and compass (heading) measure no velocity.
            AuvConcept::DepthSensor | AuvConcept::Compass => None,
        }
    }
}

/// Axiom: depth measurements are non-negative (below surface).
pub struct DepthNonNegative;

impl Axiom for DepthNonNegative {
    fn verify(&self) -> Verdict {
        use crate::applied::underwater::auv::engine::{AuvState, DvlMeasurement, dead_reckon};
        use crate::formal::math::angle::Angle;
        use crate::formal::math::geometry::point::Point3;
        use crate::formal::math::temporal::duration::Duration;
        use crate::natural::physics::kinematics::velocity::Velocity;

        // Hydrostatic depth is surface-referenced and positive-downward
        // (Kinsey et al. 2006 §II): P = ρ·g·h with ρ, g > 0 and h ≥ 0 below
        // the free surface. Exercise this on the real `dead_reckon` engine —
        // a vehicle at or below the surface, propagated with a non-negative
        // (descending or level) downward velocity, must dead-reckon to a
        // non-negative depth. Each fixture is `(start_depth, downward, dt)`.
        let fixtures = [
            (0.0_f64, 0.0_f64, 10.0_f64), // holding level at the free surface
            (10.0, 0.5, 20.0),            // descending from 10 m over 20 s
            (100.0, 0.0, 5.0),            // holding station at 100 m
            (5.0, 1.0, 2.0),              // descending from 5 m over 2 s
        ];
        let all_non_negative = fixtures.iter().all(|&(depth, downward, dt)| {
            let state = AuvState {
                position: Point3::new(0.0, 0.0, depth),
                heading: Angle::from_radians(0.0),
            };
            let dvl = DvlMeasurement {
                velocity: Velocity::new(0.0, 0.0, downward),
                bottom_lock: true,
            };
            // Real depth propagation: depth' = depth + downward·dt.
            dead_reckon(
                &state,
                &dvl,
                Angle::from_radians(0.0),
                Duration::from_seconds(dt),
            )
            .position
            .z >= 0.0
        });
        if all_non_negative {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DepthNonNegative",
        "depth measurements are non-negative (at or below the surface)",
        "Kinsey et al. (2006) A Survey of Underwater Vehicle Navigation"
    );
}
pr4xis::register_axiom!(
    DepthNonNegative,
    "Kinsey et al. (2006) A Survey of Underwater Vehicle Navigation"
);

/// Axiom: DVL requires bottom lock (limited altitude).
pub struct DvlRequiresBottomLock;

impl Axiom for DvlRequiresBottomLock {
    fn verify(&self) -> Verdict {
        // Bottom lock IS the seabed reference frame: a DVL derives velocity from
        // the Doppler shift of beams reflected off the stationary seabed, so its
        // velocity is referenced to the SEABED. The ADCP is the foil — it is also
        // a Doppler sensor, but its beams reflect off the moving water column, so
        // it has no bottom lock. Compute the distinguishing property over the
        // ontology: among all AUV sensors, the DVL is the UNIQUE one whose
        // velocity reference frame is the seabed. This reads the real
        // `VelocityFrame` quality over every declared concept, so it fails if the
        // model reassigned the DVL to the water column, gave another sensor a
        // seabed reference, or dropped the DVL's velocity quality entirely.
        let frame = VelocityFrame;
        let seabed_referenced: Vec<AuvConcept> = AuvConcept::variants()
            .into_iter()
            .filter(|sensor| frame.get(sensor) == Some(VelocityReference::Seabed))
            .collect();
        let dvl_is_unique_bottom_lock =
            seabed_referenced.len() == 1 && seabed_referenced.first() == Some(&AuvConcept::DVL);
        if dvl_is_unique_bottom_lock {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DvlRequiresBottomLock",
        "the DVL is the unique AUV sensor whose measured velocity is referenced to the stationary seabed (bottom lock); the ADCP references the moving water column and the depth/compass sensors measure no velocity",
        "Paull et al. (2014) AUV Navigation and Localization §3.2"
    );
}
pr4xis::register_axiom!(
    DvlRequiresBottomLock,
    "Paull et al. (2014) AUV Navigation and Localization §3.2"
);

impl Ontology for AuvOntology {
    type Cat = AuvCategory;
    type Qual = MeasuredQuantity;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(DepthNonNegative));
        axioms.push(Box::new(DvlRequiresBottomLock));
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
        assert_category_laws::<AuvCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        AuvOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn physical_axioms_hold() {
        assert!(DepthNonNegative.verify().is_ok());
        assert!(DvlRequiresBottomLock.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn bottom_lock_axiom_discriminates_over_the_model() {
        // The DvlRequiresBottomLock verify() is not a rubber stamp: it reads the
        // real VelocityFrame quality over every declared sensor. Bottom lock IS
        // the seabed reference frame, and only the DVL has it — the ADCP (also
        // Doppler) references the water column, and depth/compass measure no
        // velocity. If any of these assignments changed, the axiom would flip to
        // a counterexample, so the check genuinely depends on the model.
        let frame = VelocityFrame;
        assert_eq!(frame.get(&AuvConcept::DVL), Some(VelocityReference::Seabed));
        assert_eq!(
            frame.get(&AuvConcept::ADCP),
            Some(VelocityReference::WaterColumn)
        );
        assert_eq!(frame.get(&AuvConcept::DepthSensor), None);
        assert_eq!(frame.get(&AuvConcept::Compass), None);

        // The predicate the axiom evaluates: exactly the DVL is seabed-referenced.
        let seabed_referenced: Vec<AuvConcept> = AuvConcept::variants()
            .into_iter()
            .filter(|s| frame.get(s) == Some(VelocityReference::Seabed))
            .collect();
        assert_eq!(seabed_referenced.len(), 1);
        assert_eq!(seabed_referenced[0], AuvConcept::DVL);

        // Falsification witness: a model where the DVL were reassigned to the
        // water column (no bottom lock) has NO seabed-referenced sensor, so the
        // axiom's uniqueness predicate would fail. We evaluate that same
        // predicate against the counterfactual assignment to show it can fail.
        let counterfactual = [
            (AuvConcept::DVL, Some(VelocityReference::WaterColumn)),
            (AuvConcept::ADCP, Some(VelocityReference::WaterColumn)),
            (AuvConcept::DepthSensor, None),
            (AuvConcept::Compass, None),
        ];
        let counterfactual_seabed = counterfactual
            .iter()
            .filter(|(_, r)| *r == Some(VelocityReference::Seabed))
            .count();
        assert_eq!(counterfactual_seabed, 0);
    }
}
