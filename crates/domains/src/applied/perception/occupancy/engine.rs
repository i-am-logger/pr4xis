#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::perception::occupancy::ontology::CellState;

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
    /// Create a new occupancy grid initialized to unknown (log-odds = 0).
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            log_odds: vec![0.0; width * height],
            log_odds_min: -5.0,
            log_odds_max: 5.0,
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
    pub fn cell_state(&self, x: usize, y: usize, threshold: f64) -> CellState {
        let p = self.probability(x, y);
        if (p - 0.5).abs() < threshold {
            CellState::Unknown
        } else if p > 0.5 {
            CellState::Occupied
        } else {
            CellState::Free
        }
    }
}
