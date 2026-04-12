/// PID controller with anti-windup.
///
/// Åström & Murray (2008). *Feedback Systems*. Princeton University Press.
/// Ogata (2010). *Modern Control Engineering* (5th ed.).
///
/// PID control law: u(t) = Kp*e(t) + Ki*∫e(τ)dτ + Kd*de(t)/dt
///
/// Discrete-time approximation:
///   u[n] = Kp*e[n] + Ki*Σe[k]*dt + Kd*(e[n] - e[n-1])/dt
/// PID controller gains.
#[derive(Debug, Clone, PartialEq)]
pub struct PidGains {
    /// Proportional gain.
    pub kp: f64,
    /// Integral gain.
    pub ki: f64,
    /// Derivative gain.
    pub kd: f64,
}

impl PidGains {
    pub fn new(kp: f64, ki: f64, kd: f64) -> Self {
        Self { kp, ki, kd }
    }

    /// P-only controller.
    pub fn proportional(kp: f64) -> Self {
        Self::new(kp, 0.0, 0.0)
    }

    /// PI controller (no derivative).
    pub fn pi(kp: f64, ki: f64) -> Self {
        Self::new(kp, ki, 0.0)
    }

    /// PD controller (no integral).
    pub fn pd(kp: f64, kd: f64) -> Self {
        Self::new(kp, 0.0, kd)
    }
}

/// Discrete PID controller with anti-windup.
#[derive(Debug, Clone, PartialEq)]
pub struct PidController {
    /// PID gains.
    pub gains: PidGains,
    /// Sample period (seconds).
    pub dt: f64,
    /// Accumulated integral of error.
    pub integral: f64,
    /// Previous error (for derivative term).
    pub prev_error: f64,
    /// Output saturation limits for anti-windup.
    pub output_min: f64,
    pub output_max: f64,
}

impl PidController {
    /// Create a new PID controller.
    pub fn new(gains: PidGains, dt: f64) -> Self {
        Self {
            gains,
            dt,
            integral: 0.0,
            prev_error: 0.0,
            output_min: f64::NEG_INFINITY,
            output_max: f64::INFINITY,
        }
    }

    /// Set output saturation limits for anti-windup.
    pub fn with_limits(mut self, min: f64, max: f64) -> Self {
        self.output_min = min;
        self.output_max = max;
        self
    }

    /// Compute the control output for the given error.
    ///
    /// u = Kp*e + Ki*integral(e) + Kd*de/dt
    ///
    /// Anti-windup: integral is clamped when output saturates.
    pub fn update(&mut self, error: f64) -> f64 {
        // Proportional term
        let p_term = self.gains.kp * error;

        // Integral term (trapezoidal integration)
        self.integral += error * self.dt;

        let i_term = self.gains.ki * self.integral;

        // Derivative term
        let derivative = if self.dt > 0.0 {
            (error - self.prev_error) / self.dt
        } else {
            0.0
        };
        let d_term = self.gains.kd * derivative;

        self.prev_error = error;

        // Compute raw output
        let output = p_term + i_term + d_term;

        // Anti-windup: clamp output and back-calculate integral
        let clamped = output.clamp(self.output_min, self.output_max);
        if (clamped - output).abs() > 1e-15 {
            // Output is saturated — undo the integral accumulation
            self.integral -= error * self.dt;
        }

        clamped
    }

    /// Reset the controller state.
    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.prev_error = 0.0;
    }
}

/// Ziegler-Nichols tuning: given the ultimate gain Ku and ultimate period Tu,
/// compute PID gains.
///
/// Classic Ziegler-Nichols (1942) tuning rules:
/// - P:   Kp = 0.5 * Ku
/// - PI:  Kp = 0.45 * Ku,  Ki = 1.2 * Kp / Tu
/// - PID: Kp = 0.6 * Ku,   Ki = 2 * Kp / Tu,   Kd = Kp * Tu / 8
pub fn ziegler_nichols_pid(ku: f64, tu: f64) -> PidGains {
    let kp = 0.6 * ku;
    let ki = 2.0 * kp / tu;
    let kd = kp * tu / 8.0;
    PidGains::new(kp, ki, kd)
}
