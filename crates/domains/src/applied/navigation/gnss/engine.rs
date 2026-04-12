#![allow(clippy::needless_range_loop)]
use pr4xis::engine::{Action, Situation};

/// A GNSS pseudorange measurement from a single satellite.
#[derive(Debug, Clone, PartialEq)]
pub struct GnssMeasurement {
    /// Satellite ID (PRN number).
    pub satellite_id: u32,
    /// Pseudorange in meters.
    pub pseudorange: f64,
    /// Satellite position in ECEF (x, y, z) meters.
    pub satellite_position: [f64; 3],
    /// Signal strength (C/N0 in dB-Hz).
    pub cn0: f64,
}

/// GNSS solution: the result of a position fix.
#[derive(Debug, Clone, PartialEq)]
pub struct GnssSolution {
    /// Position (x, y, z) in ECEF meters.
    pub position: [f64; 3],
    /// Receiver clock bias in meters (c * dt).
    pub clock_bias: f64,
    /// Number of satellites used.
    pub num_satellites: usize,
    /// Geometric DOP.
    pub gdop: f64,
}

/// GNSS situation: visible satellites and current measurements.
#[derive(Debug, Clone, PartialEq)]
pub struct GnssSituation {
    /// Accumulated measurements from visible satellites.
    pub measurements: Vec<GnssMeasurement>,
    /// Current solution (if computed).
    pub solution: Option<GnssSolution>,
    /// Step counter.
    pub step: usize,
}

impl Situation for GnssSituation {
    fn describe(&self) -> String {
        let pos = self
            .solution
            .as_ref()
            .map(|s| {
                format!(
                    "({:.1},{:.1},{:.1})",
                    s.position[0], s.position[1], s.position[2]
                )
            })
            .unwrap_or_else(|| "no fix".to_string());
        format!(
            "GNSS step={}, sats={}, pos={}",
            self.step,
            self.measurements.len(),
            pos
        )
    }

    fn is_terminal(&self) -> bool {
        false
    }
}

/// GNSS action: add a measurement or compute a fix.
#[derive(Debug, Clone)]
pub enum GnssAction {
    /// Add a pseudorange measurement from a satellite.
    AddMeasurement(GnssMeasurement),
    /// Compute a position fix from accumulated measurements.
    ComputeFix,
}

impl Action for GnssAction {
    type Sit = GnssSituation;

    fn describe(&self) -> String {
        match self {
            GnssAction::AddMeasurement(m) => {
                format!(
                    "add satellite PRN{} (pr={:.1}m)",
                    m.satellite_id, m.pseudorange
                )
            }
            GnssAction::ComputeFix => "compute position fix".to_string(),
        }
    }
}

/// Apply a GNSS action to the current situation.
pub fn apply_gnss(situation: &GnssSituation, action: &GnssAction) -> Result<GnssSituation, String> {
    match action {
        GnssAction::AddMeasurement(m) => {
            if m.pseudorange < 0.0 {
                return Err("pseudorange must be non-negative".into());
            }
            let mut new_measurements = situation.measurements.clone();
            new_measurements.push(m.clone());
            Ok(GnssSituation {
                measurements: new_measurements,
                solution: situation.solution.clone(),
                step: situation.step + 1,
            })
        }
        GnssAction::ComputeFix => {
            if situation.measurements.len() < 4 {
                return Err(format!(
                    "need >= 4 satellites for fix, have {}",
                    situation.measurements.len()
                ));
            }
            let solution = least_squares_fix(&situation.measurements)?;
            Ok(GnssSituation {
                measurements: situation.measurements.clone(),
                solution: Some(solution),
                step: situation.step + 1,
            })
        }
    }
}

/// Simple single-point positioning via least-squares iteration.
///
/// Linearize the pseudorange equation around an initial guess and iterate.
///
/// Source: Groves (2013) Section 8.5.
fn least_squares_fix(measurements: &[GnssMeasurement]) -> Result<GnssSolution, String> {
    let n = measurements.len();
    if n < 4 {
        return Err("need >= 4 measurements".into());
    }

    // Initial guess: average of satellite positions (crude but converges)
    let mut x = [0.0_f64; 4]; // [x, y, z, clock_bias]
    for m in measurements {
        x[0] += m.satellite_position[0];
        x[1] += m.satellite_position[1];
        x[2] += m.satellite_position[2];
    }
    x[0] /= n as f64;
    x[1] /= n as f64;
    x[2] /= n as f64;

    // Iterate (Gauss-Newton)
    for _ in 0..10 {
        let mut hth = [[0.0_f64; 4]; 4];
        let mut hty = [0.0_f64; 4];

        for m in measurements {
            let dx = x[0] - m.satellite_position[0];
            let dy = x[1] - m.satellite_position[1];
            let dz = x[2] - m.satellite_position[2];
            let range = (dx * dx + dy * dy + dz * dz).sqrt();

            if range < 1e-6 {
                continue;
            }

            // H row: [dx/range, dy/range, dz/range, 1]
            let h = [dx / range, dy / range, dz / range, 1.0];

            // Residual: measured pseudorange - predicted range - clock bias
            let residual = m.pseudorange - range - x[3];

            for i in 0..4 {
                for j in 0..4 {
                    hth[i][j] += h[i] * h[j];
                }
                hty[i] += h[i] * residual;
            }
        }

        // Solve H^T H dx = H^T y using simple Gauss elimination
        if let Some(dx) = solve_4x4(&hth, &hty) {
            x[0] += dx[0];
            x[1] += dx[1];
            x[2] += dx[2];
            x[3] += dx[3];

            let norm = (dx[0] * dx[0] + dx[1] * dx[1] + dx[2] * dx[2] + dx[3] * dx[3]).sqrt();
            if norm < 1e-6 {
                break;
            }
        } else {
            return Err("singular geometry matrix".into());
        }
    }

    Ok(GnssSolution {
        position: [x[0], x[1], x[2]],
        clock_bias: x[3],
        num_satellites: n,
        gdop: 0.0, // simplified — real impl would compute from H^T H inverse
    })
}

/// Solve a 4x4 linear system Ax = b using Gauss elimination with partial pivoting.
fn solve_4x4(a: &[[f64; 4]; 4], b: &[f64; 4]) -> Option<[f64; 4]> {
    let mut aug = [[0.0_f64; 5]; 4];
    for i in 0..4 {
        for j in 0..4 {
            aug[i][j] = a[i][j];
        }
        aug[i][4] = b[i];
    }

    for col in 0..4 {
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..4 {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                max_row = row;
            }
        }
        if max_val < 1e-12 {
            return None;
        }
        aug.swap(col, max_row);

        let pivot = aug[col][col];
        for j in 0..5 {
            aug[col][j] /= pivot;
        }
        for row in 0..4 {
            if row != col {
                let factor = aug[row][col];
                for j in 0..5 {
                    aug[row][j] -= factor * aug[col][j];
                }
            }
        }
    }

    Some([aug[0][4], aug[1][4], aug[2][4], aug[3][4]])
}
