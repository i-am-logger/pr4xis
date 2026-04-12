use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// AUV navigation sensor types.
///
/// Source: Kinsey et al. (2006), "A Survey of Underwater Vehicle Navigation"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum AuvSensor {
    /// Doppler Velocity Log: measures velocity relative to seabed.
    DVL,
    /// Depth/pressure sensor.
    DepthSensor,
    /// Magnetic compass / heading sensor.
    Compass,
    /// Acoustic Doppler Current Profiler: measures water current profile.
    ADCP,
}

define_dense_category! {
    /// Category for AUV sensor fusion.
    ///
    /// All sensors can be fused in the navigation filter; the category
    /// is fully connected since measurements can be correlated.
    pub AuvCategory {
        entity: AuvSensor,
        relation: AuvSensorRelation,
    }
}

/// Quality: what physical quantity each sensor measures.
#[derive(Debug, Clone)]
pub struct MeasuredQuantity;

impl Quality for MeasuredQuantity {
    type Individual = AuvSensor;
    type Value = &'static str;

    fn get(&self, sensor: &AuvSensor) -> Option<&'static str> {
        Some(match sensor {
            AuvSensor::DVL => "velocity relative to seabed (m/s)",
            AuvSensor::DepthSensor => "depth/pressure (meters)",
            AuvSensor::Compass => "magnetic heading (rad)",
            AuvSensor::ADCP => "water current velocity profile (m/s)",
        })
    }
}

/// Axiom: depth measurements are non-negative (below surface).
pub struct DepthNonNegative;

impl Axiom for DepthNonNegative {
    fn description(&self) -> &str {
        "depth measurements are non-negative (at or below the surface)"
    }
    fn holds(&self) -> bool {
        // Depth is measured as pressure relative to surface.
        // Depth = (P - P_atm) / (rho * g) >= 0 when submerged.
        true
    }
}

/// Axiom: DVL requires bottom lock (limited altitude).
pub struct DvlRequiresBottomLock;

impl Axiom for DvlRequiresBottomLock {
    fn description(&self) -> &str {
        "DVL velocity measurement requires bottom lock (finite altitude above seabed)"
    }
    fn holds(&self) -> bool {
        // DVL works by reflecting acoustic pulses off the seabed.
        // Maximum altitude depends on frequency (typically 200-500m for 300 kHz).
        true
    }
}

pub struct AuvOntology;

impl Ontology for AuvOntology {
    type Cat = AuvCategory;
    type Qual = MeasuredQuantity;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(DepthNonNegative), Box::new(DvlRequiresBottomLock)]
    }
}
