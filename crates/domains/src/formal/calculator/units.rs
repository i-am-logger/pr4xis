#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::value::CalcError;

/// Unit categories for conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitCategory {
    Length,
    Mass,
    Temperature,
    Speed,
    Area,
    Volume,
    Time,
    Angle,
}

/// Individual units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    // Length
    Meter,
    Kilometer,
    Centimeter,
    Millimeter,
    Mile,
    Yard,
    Foot,
    Inch,
    // Mass
    Kilogram,
    Gram,
    Milligram,
    Pound,
    Ounce,
    // Temperature
    Celsius,
    Fahrenheit,
    Kelvin,
    // Speed
    MetersPerSecond,
    KilometersPerHour,
    MilesPerHour,
    Knot,
    // Area
    SquareMeter,
    SquareFoot,
    Acre,
    Hectare,
    // Volume
    Liter,
    Milliliter,
    Gallon,
    FluidOunce,
    // Time
    Second,
    Minute,
    Hour,
    Day,
    // Angle
    Radian,
    Degree,
    Gradian,
}

impl Unit {
    pub fn category(&self) -> UnitCategory {
        match self {
            Unit::Meter
            | Unit::Kilometer
            | Unit::Centimeter
            | Unit::Millimeter
            | Unit::Mile
            | Unit::Yard
            | Unit::Foot
            | Unit::Inch => UnitCategory::Length,

            Unit::Kilogram | Unit::Gram | Unit::Milligram | Unit::Pound | Unit::Ounce => {
                UnitCategory::Mass
            }

            Unit::Celsius | Unit::Fahrenheit | Unit::Kelvin => UnitCategory::Temperature,

            Unit::MetersPerSecond | Unit::KilometersPerHour | Unit::MilesPerHour | Unit::Knot => {
                UnitCategory::Speed
            }

            Unit::SquareMeter | Unit::SquareFoot | Unit::Acre | Unit::Hectare => UnitCategory::Area,

            Unit::Liter | Unit::Milliliter | Unit::Gallon | Unit::FluidOunce => {
                UnitCategory::Volume
            }

            Unit::Second | Unit::Minute | Unit::Hour | Unit::Day => UnitCategory::Time,

            Unit::Radian | Unit::Degree | Unit::Gradian => UnitCategory::Angle,
        }
    }

    /// Factor to convert this unit to the base unit of its category.
    /// Temperature uses a different conversion path.
    fn to_base(self) -> f64 {
        match self {
            // Length → meters
            Unit::Meter => 1.0,
            Unit::Kilometer => 1000.0,
            Unit::Centimeter => 0.01,
            Unit::Millimeter => 0.001,
            Unit::Mile => 1609.344,
            Unit::Yard => 0.9144,
            Unit::Foot => 0.3048,
            Unit::Inch => 0.0254,
            // Mass → kilograms
            Unit::Kilogram => 1.0,
            Unit::Gram => 0.001,
            Unit::Milligram => 0.000001,
            Unit::Pound => 0.453592,
            Unit::Ounce => 0.0283495,
            // Speed → m/s
            Unit::MetersPerSecond => 1.0,
            Unit::KilometersPerHour => 1.0 / 3.6,
            Unit::MilesPerHour => 0.44704,
            Unit::Knot => 0.514444,
            // Area → m²
            Unit::SquareMeter => 1.0,
            Unit::SquareFoot => 0.092903,
            Unit::Acre => 4046.86,
            Unit::Hectare => 10000.0,
            // Volume → liters
            Unit::Liter => 1.0,
            Unit::Milliliter => 0.001,
            Unit::Gallon => 3.78541,
            Unit::FluidOunce => 0.0295735,
            // Time → seconds
            Unit::Second => 1.0,
            Unit::Minute => 60.0,
            Unit::Hour => 3600.0,
            Unit::Day => 86400.0,
            // Angle → radians
            Unit::Radian => 1.0,
            Unit::Degree => core::f64::consts::PI / 180.0,
            Unit::Gradian => core::f64::consts::PI / 200.0,
            // Temperature handled separately
            Unit::Celsius | Unit::Fahrenheit | Unit::Kelvin => 1.0,
        }
    }
}

/// Convert a value between units.
/// Enforces: units must be in the same category.
pub fn convert(value: f64, from: Unit, to: Unit) -> Result<f64, CalcError> {
    if from.category() != to.category() {
        return Err(CalcError::InvalidDomain {
            op: format!("convert {:?} to {:?}", from, to),
            value,
        });
    }

    // Temperature needs special handling (not just multiplication)
    if from.category() == UnitCategory::Temperature {
        return convert_temperature(value, from, to);
    }

    // Convert: value * (from_to_base / to_to_base)
    let base_value = value * from.to_base();
    Ok(base_value / to.to_base())
}

fn convert_temperature(value: f64, from: Unit, to: Unit) -> Result<f64, CalcError> {
    // Convert to Kelvin first
    let kelvin = match from {
        Unit::Celsius => value + 273.15,
        Unit::Fahrenheit => (value - 32.0) * 5.0 / 9.0 + 273.15,
        Unit::Kelvin => value,
        _ => {
            return Err(CalcError::InvalidDomain {
                op: "temperature conversion".into(),
                value,
            });
        }
    };

    if kelvin < 0.0 {
        return Err(CalcError::InvalidDomain {
            op: "temperature below absolute zero".into(),
            value: kelvin,
        });
    }

    match to {
        Unit::Celsius => Ok(kelvin - 273.15),
        Unit::Fahrenheit => Ok((kelvin - 273.15) * 9.0 / 5.0 + 32.0),
        Unit::Kelvin => Ok(kelvin),
        _ => Err(CalcError::InvalidDomain {
            op: "temperature conversion".into(),
            value: kelvin,
        }),
    }
}

/// Combinatorics: n choose r.
pub fn combinations(n: u64, r: u64) -> Result<u64, CalcError> {
    if r > n {
        return Err(CalcError::InvalidDomain {
            op: "nCr".into(),
            value: r as f64,
        });
    }
    let r = r.min(n - r); // optimization: C(n,r) = C(n, n-r)
    let mut result: u64 = 1;
    for i in 0..r {
        result = result.checked_mul(n - i).ok_or(CalcError::Overflow)? / (i + 1);
    }
    Ok(result)
}

/// Combinatorics: n permute r.
pub fn permutations(n: u64, r: u64) -> Result<u64, CalcError> {
    if r > n {
        return Err(CalcError::InvalidDomain {
            op: "nPr".into(),
            value: r as f64,
        });
    }
    let mut result: u64 = 1;
    for i in 0..r {
        result = result.checked_mul(n - i).ok_or(CalcError::Overflow)?;
    }
    Ok(result)
}
