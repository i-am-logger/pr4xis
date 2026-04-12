use crate::natural::physics::kinematics::trajectory::KinematicState;

/// Motion models define how an object moves over time.
///
/// In sensor fusion, the motion model is the process model f(x, dt):
/// it predicts the next state given the current state and elapsed time.
///
/// Each model has different assumptions about the dynamics.
///
/// Source: Bar-Shalom, Li & Kirubarajan, *Estimation with Applications
///         to Tracking and Navigation* (2001), Chapter 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MotionModelType {
    /// No motion. x(t+dt) = x(t).
    Static,
    /// Constant velocity. a = 0.
    /// x(t+dt) = x(t) + v*dt
    ConstantVelocity,
    /// Constant acceleration.
    /// x(t+dt) = x(t) + v*dt + 0.5*a*dt²
    ConstantAcceleration,
    /// Coordinated turn (constant speed + turn rate).
    /// Used for tracking maneuvering aircraft.
    CoordinatedTurn,
}

/// Propagate a kinematic state forward using the specified motion model.
pub fn propagate(state: &KinematicState, dt: f64, model: MotionModelType) -> KinematicState {
    match model {
        MotionModelType::Static => state.clone(),
        MotionModelType::ConstantVelocity => {
            let displacement = state.velocity.displace(dt);
            KinematicState {
                position: state.position.translate(&displacement),
                velocity: state.velocity.clone(),
                acceleration: crate::natural::physics::kinematics::acceleration::Acceleration::zero(
                ),
            }
        }
        MotionModelType::ConstantAcceleration => state.propagate(dt),
        MotionModelType::CoordinatedTurn => {
            // Simplified: treat as constant velocity for now
            // Full implementation requires turn rate ω in the state
            let displacement = state.velocity.displace(dt);
            KinematicState {
                position: state.position.translate(&displacement),
                velocity: state.velocity.clone(),
                acceleration: state.acceleration.clone(),
            }
        }
    }
}
