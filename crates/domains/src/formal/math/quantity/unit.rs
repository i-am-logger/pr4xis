#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::quantity::dimension::Dimension;

/// A unit of measurement: a dimension + a scale factor.
///
/// A unit is a specific quantity chosen as a reference for measuring
/// other quantities of the same dimension.
///
/// Source: BIPM SI Brochure (2019), Section 2.
///         QUDT Schema (qudt.org).
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub name: &'static str,
    pub symbol: &'static str,
    pub dimension: Dimension,
    /// Scale factor relative to SI base unit (e.g., km = 1000.0 * m).
    pub scale: f64,
    /// Offset for affine units (e.g., °C = K - 273.15).
    pub offset: f64,
}

impl Unit {
    /// Convert a value from this unit to SI base unit.
    pub fn to_si(&self, value: f64) -> f64 {
        value * self.scale + self.offset
    }

    /// Convert a value from SI base unit to this unit.
    pub fn from_si(&self, si_value: f64) -> f64 {
        (si_value - self.offset) / self.scale
    }

    /// Are two units compatible (same dimension)?
    pub fn is_compatible(&self, other: &Unit) -> bool {
        self.dimension.is_compatible(&other.dimension)
    }

    /// Convert a value from this unit to another unit of the same dimension.
    pub fn convert(&self, value: f64, to: &Unit) -> Option<f64> {
        if !self.is_compatible(to) {
            return None;
        }
        let si = self.to_si(value);
        Some(to.from_si(si))
    }
}

// --- SI base units ---

pub const METER: Unit = Unit {
    name: "meter",
    symbol: "m",
    dimension: Dimension::LENGTH,
    scale: 1.0,
    offset: 0.0,
};

pub const KILOGRAM: Unit = Unit {
    name: "kilogram",
    symbol: "kg",
    dimension: Dimension::MASS,
    scale: 1.0,
    offset: 0.0,
};

pub const SECOND: Unit = Unit {
    name: "second",
    symbol: "s",
    dimension: Dimension::TIME,
    scale: 1.0,
    offset: 0.0,
};

pub const KELVIN: Unit = Unit {
    name: "kelvin",
    symbol: "K",
    dimension: Dimension::TEMPERATURE,
    scale: 1.0,
    offset: 0.0,
};

// --- Derived units ---

pub const METER_PER_SECOND: Unit = Unit {
    name: "meter per second",
    symbol: "m/s",
    dimension: Dimension::VELOCITY,
    scale: 1.0,
    offset: 0.0,
};

pub const METER_PER_SECOND_SQUARED: Unit = Unit {
    name: "meter per second squared",
    symbol: "m/s²",
    dimension: Dimension::ACCELERATION,
    scale: 1.0,
    offset: 0.0,
};

pub const RADIAN: Unit = Unit {
    name: "radian",
    symbol: "rad",
    dimension: Dimension::ANGLE,
    scale: 1.0,
    offset: 0.0,
};

pub const DEGREE: Unit = Unit {
    name: "degree",
    symbol: "°",
    dimension: Dimension::ANGLE,
    scale: core::f64::consts::PI / 180.0,
    offset: 0.0,
};

pub const RADIAN_PER_SECOND: Unit = Unit {
    name: "radian per second",
    symbol: "rad/s",
    dimension: Dimension::ANGULAR_VELOCITY,
    scale: 1.0,
    offset: 0.0,
};

// --- Common non-SI ---

pub const KILOMETER: Unit = Unit {
    name: "kilometer",
    symbol: "km",
    dimension: Dimension::LENGTH,
    scale: 1000.0,
    offset: 0.0,
};

pub const CELSIUS: Unit = Unit {
    name: "degree Celsius",
    symbol: "°C",
    dimension: Dimension::TEMPERATURE,
    scale: 1.0,
    offset: -273.15,
};

pub const KNOT: Unit = Unit {
    name: "knot",
    symbol: "kn",
    dimension: Dimension::VELOCITY,
    scale: 0.514444,
    offset: 0.0,
};
