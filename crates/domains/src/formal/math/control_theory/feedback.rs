/// Feedback systems: open-loop vs closed-loop, error signals.
///
/// Åström & Murray (2008). *Feedback Systems*. Princeton University Press.
///
/// In a closed-loop system, the output is measured and fed back to compute
/// an error signal e = reference - measured. Negative feedback stabilizes;
/// positive feedback destabilizes.
/// Type of feedback in a control system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedbackType {
    /// Negative feedback: subtracts output from reference (stabilizing).
    Negative,
    /// Positive feedback: adds output to reference (destabilizing).
    Positive,
}

/// Type of control loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoopType {
    /// No feedback — output does not influence input.
    OpenLoop,
    /// Output is measured and fed back to influence input.
    ClosedLoop,
}

/// Compute the error signal: e = reference - measured.
///
/// In a negative feedback system, the controller acts on this error
/// to drive the output toward the reference.
pub fn error_signal(reference: f64, measured: f64) -> f64 {
    reference - measured
}

/// Compute the closed-loop response with negative feedback.
///
/// For a plant with gain G and feedback gain H:
/// Closed-loop gain = G / (1 + G*H)
///
/// This is the fundamental result for negative feedback systems.
pub fn closed_loop_gain(plant_gain: f64, feedback_gain: f64) -> f64 {
    plant_gain / (1.0 + plant_gain * feedback_gain)
}

/// Compute the sensitivity function S = 1 / (1 + G*H).
///
/// Sensitivity measures how much disturbances are attenuated.
/// S < 1 means disturbances are reduced (good).
pub fn sensitivity(plant_gain: f64, feedback_gain: f64) -> f64 {
    1.0 / (1.0 + plant_gain * feedback_gain)
}
