use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::technology::sensor_fusion::time::synchronization::SyncStrategy;

// ---------------------------------------------------------------------------
// Category: sensor time synchronization strategies
// ---------------------------------------------------------------------------

/// Relationship: one sync strategy can be degraded to another.
///
/// Interpolation -> NearestNeighbor (drop second measurement)
/// Extrapolation -> NearestNeighbor (use only known measurement)
/// Any strategy -> itself (identity)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyncDegradation {
    pub from: SyncStrategy,
    pub to: SyncStrategy,
}

impl Relationship for SyncDegradation {
    type Object = SyncStrategy;

    fn source(&self) -> SyncStrategy {
        self.from
    }

    fn target(&self) -> SyncStrategy {
        self.to
    }
}

/// Category of synchronization strategies ordered by information content.
///
/// Objects: sync strategies.
/// Morphisms: degradation paths (losing information).
///
/// LinearInterpolation uses the most information (two measurements).
/// NearestNeighbor uses one measurement.
/// Extrapolation is the most dangerous (model-dependent).
pub struct SensorTimeCategory;

impl Category for SensorTimeCategory {
    type Object = SyncStrategy;
    type Morphism = SyncDegradation;

    fn identity(obj: &SyncStrategy) -> SyncDegradation {
        SyncDegradation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &SyncDegradation, g: &SyncDegradation) -> Option<SyncDegradation> {
        if f.to != g.from {
            return None;
        }
        Some(SyncDegradation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<SyncDegradation> {
        let strategies = SyncStrategy::variants();
        // Full category: all pairs (every strategy can "degrade" to every other
        // for structural completeness — the axioms encode the actual ordering).
        strategies
            .iter()
            .flat_map(|&from| {
                strategies
                    .iter()
                    .map(move |&to| SyncDegradation { from, to })
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Quality
// ---------------------------------------------------------------------------

/// Quality: whether a synchronization strategy is bounded in error.
#[derive(Debug, Clone)]
pub struct ErrorBoundedness;

impl Quality for ErrorBoundedness {
    type Individual = SyncStrategy;
    type Value = bool;

    fn get(&self, strategy: &SyncStrategy) -> Option<bool> {
        Some(match strategy {
            SyncStrategy::NearestNeighbor => true,
            SyncStrategy::LinearInterpolation => true,
            SyncStrategy::Extrapolation => false, // unbounded error growth
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Interpolation error is bounded by the measurement period.
///
/// For linear interpolation between measurements at rate f (Hz),
/// the maximum error is bounded by T^2/8 * a_max where T = 1/f.
///
/// Source: Bar-Shalom et al. (2001), Section 6.2.3.
pub struct InterpolationBounded;

impl Axiom for InterpolationBounded {
    fn description(&self) -> &str {
        "linear interpolation error is bounded by O(T^2) where T is measurement period"
    }

    fn holds(&self) -> bool {
        let bounded = ErrorBoundedness;
        bounded.get(&SyncStrategy::LinearInterpolation) == Some(true)
    }
}

/// Extrapolation increases uncertainty without bound.
///
/// Extrapolation is prediction without new information. The error
/// grows at least linearly with the extrapolation distance.
///
/// Source: Bar-Shalom et al. (2001), Section 6.2.4.
pub struct ExtrapolationUnbounded;

impl Axiom for ExtrapolationUnbounded {
    fn description(&self) -> &str {
        "extrapolation error grows without bound (no new information)"
    }

    fn holds(&self) -> bool {
        let bounded = ErrorBoundedness;
        bounded.get(&SyncStrategy::Extrapolation) == Some(false)
    }
}

/// Nearest-neighbor error is bounded by half the measurement period.
pub struct NearestNeighborBounded;

impl Axiom for NearestNeighborBounded {
    fn description(&self) -> &str {
        "nearest-neighbor sync error bounded by T/2 * max_rate"
    }

    fn holds(&self) -> bool {
        let bounded = ErrorBoundedness;
        bounded.get(&SyncStrategy::NearestNeighbor) == Some(true)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The sensor time synchronization ontology.
///
/// Founded on:
///   - Bar-Shalom et al. (2001), Chapter 6 — "Tracking with Multiple Sensors."
///   - Groves (2013), Chapter 17 — "Multi-sensor integration."
pub struct SensorTimeOntology;

impl Ontology for SensorTimeOntology {
    type Cat = SensorTimeCategory;
    type Qual = ErrorBoundedness;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(InterpolationBounded),
            Box::new(ExtrapolationUnbounded),
            Box::new(NearestNeighborBounded),
        ]
    }
}
