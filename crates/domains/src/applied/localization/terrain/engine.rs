#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::localization::terrain::ontology::TerrainFeature;

/// A Digital Elevation Model (DEM) tile.
#[derive(Debug, Clone)]
pub struct DemTile {
    /// Elevation values in row-major order (meters).
    pub elevations: Vec<f64>,
    /// Number of columns.
    pub cols: usize,
    /// Number of rows.
    pub rows: usize,
    /// Grid spacing in meters.
    pub resolution: f64,
}

impl DemTile {
    pub fn new(elevations: Vec<f64>, cols: usize, rows: usize, resolution: f64) -> Self {
        assert_eq!(elevations.len(), cols * rows);
        Self {
            elevations,
            cols,
            rows,
            resolution,
        }
    }

    /// Get elevation at grid position.
    pub fn elevation(&self, col: usize, row: usize) -> f64 {
        self.elevations[row * self.cols + col]
    }

    /// Classify terrain feature at a grid cell (simple 3x3 neighborhood analysis).
    pub fn classify_feature(&self, col: usize, row: usize) -> Option<TerrainFeature> {
        if col == 0 || col >= self.cols - 1 || row == 0 || row >= self.rows - 1 {
            return None; // border cell
        }
        let center = self.elevation(col, row);
        let mut higher_count = 0;
        let mut lower_count = 0;

        for dr in [-1_i32, 0, 1] {
            for dc in [-1_i32, 0, 1] {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let nr = (row as i32 + dr) as usize;
                let nc = (col as i32 + dc) as usize;
                let neighbor = self.elevation(nc, nr);
                if neighbor > center {
                    higher_count += 1;
                } else if neighbor < center {
                    lower_count += 1;
                }
            }
        }

        if lower_count == 8 {
            Some(TerrainFeature::Peak)
        } else if higher_count == 8 {
            Some(TerrainFeature::Valley)
        } else if lower_count >= 6 {
            Some(TerrainFeature::Ridge)
        } else if higher_count >= 3 && lower_count >= 3 {
            Some(TerrainFeature::Saddle)
        } else {
            None
        }
    }

    /// Compute terrain match score between a measured profile and the DEM.
    ///
    /// Returns the mean absolute elevation difference.
    pub fn match_profile(&self, col_start: usize, row: usize, profile: &[f64]) -> f64 {
        let n = profile.len().min(self.cols - col_start);
        if n == 0 {
            return f64::INFINITY;
        }
        let sum: f64 = (0..n)
            .map(|i| (self.elevation(col_start + i, row) - profile[i]).abs())
            .sum();
        sum / n as f64
    }
}
