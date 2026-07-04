use crate::applied::perception::occupancy::ontology::OccupancyConcept;

/// Log-odds saturation bounds for occupancy mapping.
///
/// A Bayesian occupancy grid clamps its accumulated log-odds to prevent
/// *overconfidence*: an unbounded update would drive a cell to `p → 1` or
/// `p → 0`, after which no finite number of contrary observations could move
/// it, and the map could never react to a changed world (Thrun, Burgard & Fox
/// 2005 §9.2; Elfes 1989). The bound is a cited, typed parameter here, not an
/// inline `±5.0` literal in the constructor.
#[derive(Debug, Clone, Copy)]
pub struct LogOddsSaturation {
    pub min: f64,
    pub max: f64,
}

impl LogOddsSaturation {
    /// Standard symmetric ±5 log-odds clamp (Thrun, Burgard & Fox 2005 §9.2),
    /// keeping the posterior in `p ∈ (0.007, 0.993)` so the map stays
    /// responsive.
    pub fn standard() -> Self {
        Self {
            min: -5.0,
            max: 5.0,
        }
    }
}

impl Default for LogOddsSaturation {
    fn default() -> Self {
        Self::standard()
    }
}

/// A Bayesian occupancy grid using log-odds representation.
///
/// Source: Thrun, Burgard & Fox (2005), *Probabilistic Robotics*, Chapter 9.
#[derive(Debug, Clone)]
pub struct OccupancyGrid {
    /// Grid dimensions.
    pub width: usize,
    pub height: usize,
    /// Log-odds values for each cell. 0.0 = unknown (p=0.5).
    pub log_odds: Vec<f64>,
    /// Clamping thresholds for log-odds to prevent overconfidence.
    pub log_odds_min: f64,
    pub log_odds_max: f64,
}

impl OccupancyGrid {
    /// Create a new occupancy grid initialized to unknown (log-odds = 0), with
    /// the standard cited log-odds saturation ([`LogOddsSaturation::standard`]).
    pub fn new(width: usize, height: usize) -> Self {
        let saturation = LogOddsSaturation::standard();
        Self {
            width,
            height,
            log_odds: vec![0.0; width * height],
            log_odds_min: saturation.min,
            log_odds_max: saturation.max,
        }
    }

    /// Convert log-odds to probability.
    pub fn log_odds_to_probability(l: f64) -> f64 {
        1.0 / (1.0 + (-l).exp())
    }

    /// Convert probability to log-odds.
    pub fn probability_to_log_odds(p: f64) -> f64 {
        (p / (1.0 - p)).ln()
    }

    /// Get the cell index.
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Update a cell with a sensor observation (log-odds increment).
    pub fn update(&mut self, x: usize, y: usize, sensor_log_odds: f64) {
        let idx = self.index(x, y);
        self.log_odds[idx] =
            (self.log_odds[idx] + sensor_log_odds).clamp(self.log_odds_min, self.log_odds_max);
    }

    /// Get the occupancy probability for a cell.
    pub fn probability(&self, x: usize, y: usize) -> f64 {
        Self::log_odds_to_probability(self.log_odds[self.index(x, y)])
    }

    /// Get the cell state based on threshold.
    pub fn cell_state(&self, x: usize, y: usize, threshold: f64) -> OccupancyConcept {
        let p = self.probability(x, y);
        if (p - 0.5).abs() < threshold {
            OccupancyConcept::Unknown
        } else if p > 0.5 {
            OccupancyConcept::Occupied
        } else {
            OccupancyConcept::Free
        }
    }
}
