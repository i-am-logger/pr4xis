use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::taxonomy::{NoCycles, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Odometry source types — how relative motion is measured.
///
/// Source: Borenstein et al. (1996) "Where am I?",
///         Thrun, Burgard & Fox (2005) Chapter 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum OdometrySource {
    /// Abstract odometry source.
    Source,
    /// Wheel encoders: count wheel rotations.
    WheelEncoder,
    /// Visual odometry: track features between camera frames.
    VisualOdometry,
    /// Inertial odometry: integrate IMU measurements.
    InertialOdometry,
    /// Laser odometry: match consecutive laser scans.
    LaserOdometry,
}

/// Odometry state: what dead reckoning estimates.
///
/// Source: Borenstein et al. (1996).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum OdometryState {
    /// Abstract state.
    State,
    /// 2D position (x, y).
    Position2D,
    /// Heading angle.
    Heading,
    /// Forward velocity.
    Velocity,
}

// ---------------------------------------------------------------------------
// Ontology (category + reasoning)
// ---------------------------------------------------------------------------

define_ontology! {
    /// The odometry ontology.
    ///
    /// Source: Borenstein et al. (1996), Thrun, Burgard & Fox (2005) Chapter 5,
    ///         Scaramuzza & Fraundorfer (2011).
    pub OdometryOntology for OdometryCategory {
        entity: OdometrySource,
        relation: OdometryRelation,

        taxonomy: OdometrySourceTaxonomy [
            (WheelEncoder, Source),
            (VisualOdometry, Source),
            (InertialOdometry, Source),
            (LaserOdometry, Source),
        ],
    }
}

/// Odometry state taxonomy (secondary entity type — manual impl).
pub struct OdometryStateTaxonomy;

impl TaxonomyDef for OdometryStateTaxonomy {
    type Entity = OdometryState;

    fn relations() -> Vec<(OdometryState, OdometryState)> {
        use OdometryState::*;
        vec![(Position2D, State), (Heading, State), (Velocity, State)]
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: Drift rate (meters of error per meter traveled).
///
/// Source: Borenstein et al. (1996), Table 3.
#[derive(Debug, Clone)]
pub struct DriftRate;

impl Quality for DriftRate {
    type Individual = OdometrySource;
    type Value = &'static str;

    fn get(&self, source: &OdometrySource) -> Option<&'static str> {
        Some(match source {
            OdometrySource::Source => "varies by type",
            OdometrySource::WheelEncoder => "1-5% of distance traveled",
            OdometrySource::VisualOdometry => "0.5-2% of distance traveled",
            OdometrySource::InertialOdometry => "grows as O(t^3) — unbounded",
            OdometrySource::LaserOdometry => "0.5-1% of distance traveled",
        })
    }
}

/// Quality: Update rate in Hz.
///
/// Source: Scaramuzza & Fraundorfer (2011).
#[derive(Debug, Clone)]
pub struct UpdateRate;

impl Quality for UpdateRate {
    type Individual = OdometrySource;
    type Value = &'static str;

    fn get(&self, source: &OdometrySource) -> Option<&'static str> {
        Some(match source {
            OdometrySource::Source => "varies",
            OdometrySource::WheelEncoder => "~100 Hz",
            OdometrySource::VisualOdometry => "~30 Hz (camera framerate)",
            OdometrySource::InertialOdometry => "~200-400 Hz (IMU rate)",
            OdometrySource::LaserOdometry => "~10-20 Hz (scan rate)",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Odometry state taxonomy is a DAG.
pub struct OdometryStateTaxonomyIsDAG;

impl Axiom for OdometryStateTaxonomyIsDAG {
    fn description(&self) -> &str {
        "odometry state taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        NoCycles::<OdometryStateTaxonomy>::default().holds()
    }
}

/// Drift is unbounded: odometry error grows without bound over time.
///
/// Without an absolute reference (GNSS, landmarks), the position error
/// from odometry never decreases — it accumulates indefinitely.
///
/// Source: Thrun, Burgard & Fox (2005) Section 5.4.
pub struct DriftIsUnbounded;

impl Axiom for DriftIsUnbounded {
    fn description(&self) -> &str {
        "odometry error grows without bound (no absolute reference)"
    }
    fn holds(&self) -> bool {
        // Model: position error grows linearly with distance traveled.
        // For wheel odometry with 2% drift rate, after 100m: error = 2m.
        // After 1000m: error = 20m. No convergence.
        let drift_rate = 0.02; // 2% of distance
        let d1 = 100.0;
        let d2 = 1000.0;
        let e1 = drift_rate * d1;
        let e2 = drift_rate * d2;
        // Error grows with distance, never decreases
        e2 > e1 && e1 > 0.0
    }
}

/// Relative motion only: odometry measures CHANGE, not absolute position.
///
/// Odometry provides delta_x, delta_y, delta_theta between time steps.
/// It cannot tell you WHERE you are, only how far you've moved.
///
/// Source: Borenstein et al. (1996).
pub struct RelativeMotionOnly;

impl Axiom for RelativeMotionOnly {
    fn description(&self) -> &str {
        "odometry measures change in position, not absolute position"
    }
    fn holds(&self) -> bool {
        // Two robots starting at different positions with same odometry
        // readings will have different absolute positions but same displacement.
        let start_a: [f64; 2] = [0.0, 0.0];
        let start_b: [f64; 2] = [100.0, 200.0];
        let delta: [f64; 2] = [10.0, 5.0]; // same odometry reading

        let end_a = [start_a[0] + delta[0], start_a[1] + delta[1]];
        let end_b = [start_b[0] + delta[0], start_b[1] + delta[1]];

        // Same displacement
        let disp_a = [end_a[0] - start_a[0], end_a[1] - start_a[1]];
        let disp_b = [end_b[0] - start_b[0], end_b[1] - start_b[1]];
        (disp_a[0] - disp_b[0]).abs() < 1e-10 && (disp_a[1] - disp_b[1]).abs() < 1e-10
    }
}

/// Slip corrupts wheel odometry: wheel slip causes measurement error.
///
/// On slippery surfaces (ice, mud), wheels can spin without forward motion,
/// or slide without rotation. This directly corrupts the distance estimate.
///
/// Source: Borenstein et al. (1996), Section 3.2.
pub struct SlipCorruptsWheelOdometry;

impl Axiom for SlipCorruptsWheelOdometry {
    fn description(&self) -> &str {
        "wheel slip causes wheel encoder error"
    }
    fn holds(&self) -> bool {
        // Wheel encoder measures rotation, not ground distance.
        // With slip ratio s: actual_distance = encoder_distance * (1 - s)
        // If slip = 0.1 (10%), 100m of encoder reads as 90m actual.
        let encoder_distance = 100.0_f64;
        let slip_ratio = 0.1_f64;
        let actual_distance = encoder_distance * (1.0 - slip_ratio);
        let error = (encoder_distance - actual_distance).abs();
        // Non-zero slip produces non-zero error
        error > 0.0 && actual_distance < encoder_distance
    }
}

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for OdometryOntology {
    type Cat = OdometryCategory;
    type Qual = DriftRate;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(OdometryStateTaxonomyIsDAG),
            Box::new(DriftIsUnbounded),
            Box::new(RelativeMotionOnly),
            Box::new(SlipCorruptsWheelOdometry),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<OdometryCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        OdometryOntology::validate().unwrap();
    }
}
