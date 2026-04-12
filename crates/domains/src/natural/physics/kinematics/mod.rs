//! The kinematics ontology — position, velocity, acceleration, jerk, trajectory as geometry + time on SE(3)
pub mod acceleration;
pub mod motion_model;
pub mod ontology;
pub mod position;
pub mod trajectory;
pub mod velocity;

#[cfg(test)]
mod tests;
