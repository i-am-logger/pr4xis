//! The rotation ontology — SO(3) as a praxis category with quaternion, DCM, Euler, and axis-angle representations
pub mod axis_angle;
pub mod dcm;
pub mod euler;
pub mod ontology;
pub mod quaternion;

#[cfg(test)]
mod tests;
