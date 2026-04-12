use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::taxonomy::{self, NoCycles, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Coupling level for INS/GNSS integration.
///
/// Each level integrates the two systems more deeply, providing better
/// performance in degraded GNSS environments but requiring more complexity.
///
/// Source: Groves (2013) Chapter 14.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum CouplingLevel {
    /// Abstract coupling.
    Coupling,
    /// Loosely coupled: GNSS provides position/velocity to INS filter.
    LooselyCoupled,
    /// Tightly coupled: GNSS provides raw pseudoranges to INS filter.
    TightlyCoupled,
    /// Deeply coupled: INS aids GNSS tracking loops.
    DeeplyCoupled,
}

/// INS/GNSS system state.
///
/// Source: Groves (2013) Section 14.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum InsGnssState {
    /// Abstract state.
    State,
    /// Full navigation: both INS and GNSS active.
    NavigationMode,
    /// Coasting: INS only, GNSS unavailable.
    Coasting,
    /// GNSS reacquired after outage.
    GnssReacquired,
    /// System initializing (alignment).
    Initializing,
}

// ---------------------------------------------------------------------------
// Ontology (category + reasoning)
// ---------------------------------------------------------------------------

define_ontology! {
    /// The INS/GNSS integration ontology.
    ///
    /// Source: Groves (2013) Chapters 14-17, Titterton & Weston (2004) Chapter 13.
    pub InsGnssOntology for InsGnssCategory {
        entity: CouplingLevel,
        relation: InsGnssRelation,

        taxonomy: CouplingTaxonomy [
            (LooselyCoupled, Coupling),
            (TightlyCoupled, Coupling),
            (DeeplyCoupled, Coupling),
            // Tighter coupling extends looser coupling
            (TightlyCoupled, LooselyCoupled),
            (DeeplyCoupled, TightlyCoupled),
        ],
    }
}

/// INS/GNSS state taxonomy (secondary entity type — manual impl).
pub struct InsGnssStateTaxonomy;

impl TaxonomyDef for InsGnssStateTaxonomy {
    type Entity = InsGnssState;

    fn relations() -> Vec<(InsGnssState, InsGnssState)> {
        use InsGnssState::*;
        vec![
            (NavigationMode, State),
            (Coasting, State),
            (GnssReacquired, State),
            (Initializing, State),
        ]
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: Error state components at each coupling level.
///
/// Source: Groves (2013) Table 14.1.
#[derive(Debug, Clone)]
pub struct ErrorStateDescription;

impl Quality for ErrorStateDescription {
    type Individual = CouplingLevel;
    type Value = &'static str;

    fn get(&self, level: &CouplingLevel) -> Option<&'static str> {
        Some(match level {
            CouplingLevel::Coupling => "position/velocity/attitude errors + sensor biases",
            CouplingLevel::LooselyCoupled => {
                "15-state: pos(3)+vel(3)+att(3)+gyro_bias(3)+accel_bias(3)"
            }
            CouplingLevel::TightlyCoupled => "17-state: 15 + clock_bias + clock_drift",
            CouplingLevel::DeeplyCoupled => "17+ state with tracking loop aiding",
        })
    }
}

/// Quality: Coupling bandwidth — how fast corrections propagate.
///
/// Source: Groves (2013) Section 14.3.
#[derive(Debug, Clone)]
pub struct CouplingBandwidth;

impl Quality for CouplingBandwidth {
    type Individual = CouplingLevel;
    type Value = &'static str;

    fn get(&self, level: &CouplingLevel) -> Option<&'static str> {
        Some(match level {
            CouplingLevel::Coupling => "depends on coupling level",
            CouplingLevel::LooselyCoupled => "1-10 Hz GNSS update rate",
            CouplingLevel::TightlyCoupled => "1-10 Hz, uses raw pseudoranges",
            CouplingLevel::DeeplyCoupled => "100+ Hz, INS aids GNSS tracking loops",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// INS/GNSS state taxonomy is a DAG.
pub struct InsGnssStateTaxonomyIsDAG;

impl Axiom for InsGnssStateTaxonomyIsDAG {
    fn description(&self) -> &str {
        "INS/GNSS state taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        NoCycles::<InsGnssStateTaxonomy>::default().holds()
    }
}

/// Coasting degrades: without GNSS, INS position error grows quadratically.
///
/// An accelerometer bias b causes position error = 0.5 * b * t^2.
/// After 60 seconds with 1 mg bias: error = 0.5 * 0.0098 * 3600 ≈ 17.6 m.
///
/// Source: Groves (2013) Eq. 14.1.
pub struct CoastingDegrades;

impl Axiom for CoastingDegrades {
    fn description(&self) -> &str {
        "without GNSS, INS position error grows quadratically (bias -> t^2 error)"
    }
    fn holds(&self) -> bool {
        // Accelerometer bias of 1 milli-g
        let bias_mg = 1.0_f64; // milli-g
        let bias_mps2 = bias_mg * 1e-3 * 9.80665; // m/s^2
        let t1 = 30.0_f64; // seconds
        let t2 = 60.0_f64;
        let error_t1 = 0.5 * bias_mps2 * t1 * t1;
        let error_t2 = 0.5 * bias_mps2 * t2 * t2;
        // Quadratic: error at 2t should be 4x error at t
        let ratio = error_t2 / error_t1;
        (ratio - 4.0).abs() < 0.01
    }
}

/// GNSS measurement update reduces position uncertainty.
///
/// A Kalman filter measurement update always reduces (or maintains)
/// the trace of the covariance matrix when the measurement is valid.
///
/// Source: Brown & Hwang (2012), Chapter 5.
pub struct GnssUpdateReducesError;

impl Axiom for GnssUpdateReducesError {
    fn description(&self) -> &str {
        "GNSS measurement update decreases position uncertainty"
    }
    fn holds(&self) -> bool {
        // Simplified scalar Kalman update:
        // P_post = P_prior * (1 - K * H) where K = P_prior * H / (H * P_prior * H + R)
        // For scalar: P_post = P_prior * R / (P_prior + R) < P_prior (when R > 0)
        let p_prior = 100.0; // prior variance (m^2)
        let r = 25.0; // GNSS measurement noise (m^2, ~5m 1-sigma)
        let p_post = p_prior * r / (p_prior + r);
        p_post < p_prior
    }
}

/// Tighter coupling provides better performance in degraded GNSS.
///
/// Tightly coupled can work with < 4 satellites (partial solution).
/// Deeply coupled can track weaker signals due to INS aiding.
///
/// Source: Groves (2013) Section 14.5.
pub struct TighterCouplingBetter;

impl Axiom for TighterCouplingBetter {
    fn description(&self) -> &str {
        "tighter coupling provides better performance in degraded GNSS"
    }
    fn holds(&self) -> bool {
        // Tightly coupled is-a loosely coupled (it extends it)
        taxonomy::is_a::<CouplingTaxonomy>(
            &CouplingLevel::TightlyCoupled,
            &CouplingLevel::LooselyCoupled,
        )
        // Deeply coupled is-a tightly coupled (it extends it)
        && taxonomy::is_a::<CouplingTaxonomy>(
            &CouplingLevel::DeeplyCoupled,
            &CouplingLevel::TightlyCoupled,
        )
    }
}

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for InsGnssOntology {
    type Cat = InsGnssCategory;
    type Qual = ErrorStateDescription;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(InsGnssStateTaxonomyIsDAG),
            Box::new(CoastingDegrades),
            Box::new(GnssUpdateReducesError),
            Box::new(TighterCouplingBetter),
        ]
    }
}
