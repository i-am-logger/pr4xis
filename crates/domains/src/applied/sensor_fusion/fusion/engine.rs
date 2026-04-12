use pr4xis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::positive_definite;
use crate::formal::math::linear_algebra::vector_space::Vector;

use crate::applied::sensor_fusion::state::estimate::StateEstimate;

// ---------------------------------------------------------------------------
// Situation: the current fusion state
// ---------------------------------------------------------------------------

/// FusionState: the Situation in the sensor fusion Engine.
///
/// This is the predict/update state machine. At any point in time,
/// the fusion system has a state estimate and knows how many sensors
/// have contributed.
#[derive(Debug, Clone, PartialEq)]
pub struct FusionState {
    pub estimate: StateEstimate,
    pub sensors_active: usize,
}

impl Situation for FusionState {
    fn describe(&self) -> String {
        format!(
            "step={}, dim={}, uncertainty={:.4}, sensors={}",
            self.estimate.step,
            self.estimate.dim(),
            self.estimate.uncertainty(),
            self.sensors_active,
        )
    }

    fn is_terminal(&self) -> bool {
        false // fusion runs indefinitely
    }
}

// ---------------------------------------------------------------------------
// Action: predict and update steps
// ---------------------------------------------------------------------------

/// FusionAction: the actions in the sensor fusion Engine.
///
/// The fundamental cycle: Predict → Update → Predict → Update → ...
///
/// Source: Kalman (1960), Bar-Shalom et al. (2001).
#[derive(Debug, Clone)]
pub enum FusionAction {
    /// Time update (prediction): propagate state forward by dt.
    /// F = state transition matrix, Q = process noise covariance.
    Predict {
        dt: f64,
        transition: Matrix,
        process_noise: Matrix,
    },
    /// Measurement update: incorporate a new observation.
    /// H = observation matrix, z = measurement, R = measurement noise.
    Update {
        observation_matrix: Matrix,
        measurement: Vector,
        measurement_noise: Matrix,
    },
    /// Reset the filter to a new initial state.
    Reset { initial: StateEstimate },
}

impl Action for FusionAction {
    type Sit = FusionState;

    fn describe(&self) -> String {
        match self {
            Self::Predict { dt, .. } => format!("Predict(dt={dt:.4})"),
            Self::Update { measurement, .. } => {
                format!("Update(dim={})", measurement.dim())
            }
            Self::Reset { .. } => "Reset".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Preconditions
// ---------------------------------------------------------------------------

/// Precondition: prediction dt must be non-negative (time moves forward).
pub struct PositiveTimeStep;

impl Precondition<FusionAction> for PositiveTimeStep {
    fn check(&self, _situation: &FusionState, action: &FusionAction) -> PreconditionResult {
        if let FusionAction::Predict { dt, .. } = action {
            if *dt >= 0.0 {
                return PreconditionResult::Satisfied {
                    rule: "PositiveTimeStep".into(),
                    reason: format!("dt={dt} >= 0"),
                };
            }
            return PreconditionResult::Violated {
                rule: "PositiveTimeStep".into(),
                reason: format!("dt={dt} is negative"),
                situation: _situation.describe(),
                attempted_action: action.describe(),
            };
        }
        PreconditionResult::Satisfied {
            rule: "PositiveTimeStep".into(),
            reason: "not a predict action".into(),
        }
    }

    fn describe(&self) -> &str {
        "prediction time step must be non-negative"
    }
}

/// Precondition: state dimension must match transition matrix.
pub struct DimensionConsistency;

impl Precondition<FusionAction> for DimensionConsistency {
    fn check(&self, situation: &FusionState, action: &FusionAction) -> PreconditionResult {
        let n = situation.estimate.dim();
        let ok = match action {
            FusionAction::Predict {
                transition,
                process_noise,
                ..
            } => {
                transition.rows == n
                    && transition.cols == n
                    && process_noise.rows == n
                    && process_noise.cols == n
            }
            FusionAction::Update {
                observation_matrix,
                measurement,
                measurement_noise,
                ..
            } => {
                observation_matrix.cols == n
                    && observation_matrix.rows == measurement.dim()
                    && measurement_noise.rows == measurement.dim()
                    && measurement_noise.cols == measurement.dim()
            }
            FusionAction::Reset { .. } => true,
        };
        if ok {
            PreconditionResult::Satisfied {
                rule: "DimensionConsistency".into(),
                reason: "dimensions match".into(),
            }
        } else {
            PreconditionResult::Violated {
                rule: "DimensionConsistency".into(),
                reason: "matrix/vector dimensions incompatible".into(),
                situation: situation.describe(),
                attempted_action: action.describe(),
            }
        }
    }

    fn describe(&self) -> &str {
        "matrix dimensions must be consistent with state dimension"
    }
}

/// Precondition: covariance must remain positive semi-definite after update.
/// This is the fundamental physical constraint — uncertainty cannot be negative.
pub struct CovariancePSD;

impl Precondition<FusionAction> for CovariancePSD {
    fn check(&self, situation: &FusionState, _action: &FusionAction) -> PreconditionResult {
        if positive_definite::is_positive_semidefinite(&situation.estimate.covariance) {
            PreconditionResult::Satisfied {
                rule: "CovariancePSD".into(),
                reason: "P is positive semi-definite".into(),
            }
        } else {
            PreconditionResult::Violated {
                rule: "CovariancePSD".into(),
                reason: "P is not positive semi-definite — filter diverged".into(),
                situation: situation.describe(),
                attempted_action: _action.describe(),
            }
        }
    }

    fn describe(&self) -> &str {
        "covariance must be positive semi-definite (uncertainty cannot be negative)"
    }
}

// ---------------------------------------------------------------------------
// Apply function: Kalman filter predict/update
// ---------------------------------------------------------------------------

/// Apply a fusion action to a state: the Kalman filter equations.
///
/// Predict: x̂⁻ = F x̂⁺,  P⁻ = F P⁺ F^T + Q
/// Update:  K = P⁻ H^T (H P⁻ H^T + R)^{-1}
///          x̂⁺ = x̂⁻ + K (z - H x̂⁻)
///          P⁺ = (I - KH) P⁻ (I - KH)^T + K R K^T  (Joseph form)
///
/// The fusion engine IS a feedback control system
/// (see `crate::formal::math::control_theory`):
/// - Plant: the physical system (state evolves via process model F)
/// - Sensor: the observation model H
/// - Controller: the Kalman gain K (computed from covariance, analogous to
///   optimal feedback gain in LQG/LQR)
/// - Error signal: the innovation nu = z - H x̂ (measurement minus prediction)
///
/// The predict step is the open-loop propagation; the update step closes the
/// loop by injecting the innovation through the gain. This duality between
/// estimation and control is formalized by the separation principle.
///
/// Source: Kalman (1960), Maybeck (1979).
///         Astrom & Murray (2008), Chapter 8 (estimation as control).
pub(crate) fn apply_fusion(
    situation: &FusionState,
    action: &FusionAction,
) -> Result<FusionState, String> {
    match action {
        FusionAction::Predict {
            dt,
            transition,
            process_noise,
        } => {
            let f = transition;
            let q = process_noise;
            let x_pred = f.multiply_vector(&situation.estimate.state);
            let p_pred = f
                .multiply(&situation.estimate.covariance)
                .multiply(&f.transpose())
                .add(q);
            Ok(FusionState {
                estimate: StateEstimate {
                    state: x_pred,
                    covariance: p_pred,
                    epoch: situation.estimate.epoch + dt,
                    step: situation.estimate.step + 1,
                },
                sensors_active: situation.sensors_active,
            })
        }
        FusionAction::Update {
            observation_matrix,
            measurement,
            measurement_noise,
        } => {
            let h = observation_matrix;
            let z = measurement;
            let r = measurement_noise;
            let x = &situation.estimate.state;
            let p = &situation.estimate.covariance;

            // Innovation: ν = z - Hx
            let z_pred = h.multiply_vector(x);
            let innovation = z.sub(&z_pred);

            // Innovation covariance: S = HPH^T + R
            let s = h.multiply(p).multiply(&h.transpose()).add(r);

            // Kalman gain: K = PH^T S^{-1} (via solve)
            let pht = p.multiply(&h.transpose());
            let n = s.rows;
            let mut k_data = Vec::with_capacity(pht.rows * n);
            for i in 0..pht.rows {
                let col: Vec<f64> = (0..n).map(|j| pht.get(i, j)).collect();
                let solved =
                    crate::formal::math::linear_algebra::decomposition::solve_spd(&s, &col)
                        .ok_or_else(|| {
                            "innovation covariance S is singular — cannot compute Kalman gain"
                                .to_string()
                        })?;
                k_data.extend(solved);
            }
            let k = Matrix::new(pht.rows, n, k_data);

            // State update: x⁺ = x + K ν
            let x_new = x.add(&k.multiply_vector(&innovation));

            // Joseph form covariance update (numerically stable, preserves PSD)
            let p_new =
                crate::formal::math::linear_algebra::positive_definite::joseph_update(p, &k, h, r);

            Ok(FusionState {
                estimate: StateEstimate {
                    state: x_new,
                    covariance: p_new,
                    epoch: situation.estimate.epoch,
                    step: situation.estimate.step + 1,
                },
                sensors_active: situation.sensors_active,
            })
        }
        FusionAction::Reset { initial } => Ok(FusionState {
            estimate: initial.clone(),
            sensors_active: situation.sensors_active,
        }),
    }
}

/// Create a new fusion engine with standard preconditions.
pub fn new_fusion_engine(initial: StateEstimate) -> Engine<FusionAction> {
    let state = FusionState {
        estimate: initial,
        sensors_active: 0,
    };
    Engine::new(
        state,
        vec![
            Box::new(PositiveTimeStep),
            Box::new(DimensionConsistency),
            Box::new(CovariancePSD),
        ],
        apply_fusion,
    )
}
