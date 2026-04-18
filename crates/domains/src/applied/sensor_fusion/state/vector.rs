#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// State vector: the quantities being estimated.
///
/// This is a semantic wrapper — the Vector from linear_algebra
/// carries the math, this module provides the sensor fusion meaning.
///
/// A state vector contains components like position, velocity,
/// attitude, sensor biases, clock offset — each with a physical dimension.
///
/// Source: Maybeck (1979), Vol. 1, Chapter 1.
/// Named components of a state vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateComponent {
    PositionX,
    PositionY,
    PositionZ,
    VelocityX,
    VelocityY,
    VelocityZ,
    AccelerationX,
    AccelerationY,
    AccelerationZ,
    AttitudeQ0,
    AttitudeQ1,
    AttitudeQ2,
    AttitudeQ3,
    GyroBiasX,
    GyroBiasY,
    GyroBiasZ,
    AccelBiasX,
    AccelBiasY,
    AccelBiasZ,
    ClockBias,
    ClockDrift,
}

/// State vector layout: maps component names to indices.
#[derive(Debug, Clone)]
pub struct StateLayout {
    pub components: Vec<StateComponent>,
}

impl StateLayout {
    /// 6-state position+velocity layout.
    pub fn position_velocity_3d() -> Self {
        use StateComponent::*;
        Self {
            components: vec![
                PositionX, PositionY, PositionZ, VelocityX, VelocityY, VelocityZ,
            ],
        }
    }

    /// 15-state INS error state layout (Groves 2013, Table 14.1).
    pub fn ins_error_15() -> Self {
        use StateComponent::*;
        Self {
            components: vec![
                PositionX, PositionY, PositionZ, VelocityX, VelocityY, VelocityZ, AttitudeQ0,
                AttitudeQ1, AttitudeQ2, AccelBiasX, AccelBiasY, AccelBiasZ, GyroBiasX, GyroBiasY,
                GyroBiasZ,
            ],
        }
    }

    /// Dimension of the state vector.
    pub fn dim(&self) -> usize {
        self.components.len()
    }

    /// Index of a component (None if not in layout).
    pub fn index_of(&self, component: StateComponent) -> Option<usize> {
        self.components.iter().position(|c| *c == component)
    }
}
