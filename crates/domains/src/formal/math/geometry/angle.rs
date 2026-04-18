#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use core::f64::consts::PI;

/// An angle in Euclidean geometry.
///
/// Hilbert's congruence axioms (III.4, III.5) define angle congruence.
/// Stored in radians.
#[derive(Debug, Clone, PartialEq)]
pub struct Angle {
    radians: f64,
}

impl Angle {
    pub fn from_radians(rad: f64) -> Self {
        Self { radians: rad }
    }

    pub fn from_degrees(deg: f64) -> Self {
        Self {
            radians: deg * PI / 180.0,
        }
    }

    pub fn zero() -> Self {
        Self { radians: 0.0 }
    }

    /// Right angle (π/2 = 90°).
    pub fn right() -> Self {
        Self { radians: PI / 2.0 }
    }

    /// Straight angle (π = 180°).
    pub fn straight() -> Self {
        Self { radians: PI }
    }

    /// Full turn (2π = 360°).
    pub fn full_turn() -> Self {
        Self { radians: 2.0 * PI }
    }

    pub fn radians(&self) -> f64 {
        self.radians
    }

    pub fn degrees(&self) -> f64 {
        self.radians * 180.0 / PI
    }

    /// Supplementary: α + β = π.
    pub fn supplementary(&self) -> Self {
        Self {
            radians: PI - self.radians,
        }
    }

    /// Complementary: α + β = π/2.
    pub fn complementary(&self) -> Self {
        Self {
            radians: PI / 2.0 - self.radians,
        }
    }

    pub fn is_acute(&self) -> bool {
        self.radians > 0.0 && self.radians < PI / 2.0
    }

    pub fn is_right(&self) -> bool {
        (self.radians - PI / 2.0).abs() < 1e-10
    }

    pub fn is_obtuse(&self) -> bool {
        self.radians > PI / 2.0 && self.radians < PI
    }

    pub fn sin(&self) -> f64 {
        self.radians.sin()
    }

    pub fn cos(&self) -> f64 {
        self.radians.cos()
    }

    pub fn tan(&self) -> f64 {
        self.radians.tan()
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            radians: self.radians + other.radians,
        }
    }
}
