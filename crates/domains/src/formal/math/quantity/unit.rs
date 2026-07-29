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

/// Jerk — the time-derivative of acceleration (L·T⁻³).
pub const METER_PER_SECOND_CUBED: Unit = Unit {
    name: "meter per second cubed",
    symbol: "m/s³",
    dimension: Dimension {
        length: 1,
        time: -3,
        ..Dimension::DIMENSIONLESS
    },
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

/// Arcsecond: 1/3600 of a degree — the standard unit for pointing /
/// attitude-sensor accuracy (Wertz 1978; Markley & Crassidis 2014).
pub const ARCSECOND: Unit = Unit {
    name: "arcsecond",
    symbol: "″",
    dimension: Dimension::ANGLE,
    // 1″ = π / (180 · 3600) rad.
    scale: core::f64::consts::PI / 648_000.0,
    offset: 0.0,
};

/// Nat — the natural unit of information: the information content of an event
/// with probability 1/e, i.e. a negative NATURAL log of a probability.
/// ISO/IEC 80000-13:2008 unit entry 13-24.c (nat, under item 13-24
/// "information content"); Shannon (1948), Introduction, BSTJ 27(3) p. 379
/// ("The resulting units of information will be called natural units").
/// The information analogue of [`RADIAN`]: a named unit over a dimensionless
/// dimension ([`Dimension::INFORMATION`]).
pub const NAT: Unit = Unit {
    name: "nat",
    symbol: "nat",
    dimension: Dimension::INFORMATION,
    scale: 1.0,
    offset: 0.0,
};

/// Shannon — the binary unit of information (one binary digit's capacity):
/// 1 Sh = ln 2 nat. ISO/IEC 80000-13:2008 unit entry 13-24.a (shannon, under
/// item 13-24 "information content"); Shannon (1948), Introduction, BSTJ
/// 27(3) p. 379 (log base 2: "binary digits, or more briefly bits").
/// Scaled to the nat the way [`DEGREE`] scales to [`RADIAN`].
pub const SHANNON: Unit = Unit {
    name: "shannon",
    symbol: "Sh",
    dimension: Dimension::INFORMATION,
    scale: core::f64::consts::LN_2,
    offset: 0.0,
};

/// Bit (storage) — the traditional unit of storage capacity/size: the
/// count of binary digits a representation occupies. ISO/IEC 80000-13:2008
/// item 13-9.a ("bit" as the recommended unit of storage capacity/storage
/// size, item 13-9), a DIFFERENT quantity from item 13-24's information
/// content (nat/shannon/hartley) even though both are conventionally
/// spelled "bit" — storage width does not depend on a probability
/// distribution the way Shannon information content does.
pub const BIT_STORAGE: Unit = Unit {
    name: "bit (storage)",
    symbol: "bit",
    dimension: Dimension::DATA_SIZE,
    scale: 1.0,
    offset: 0.0,
};

/// Hertz — cycles per second (T⁻¹), for update rates and bandwidths.
pub const HERTZ: Unit = Unit {
    name: "hertz",
    symbol: "Hz",
    dimension: Dimension::FREQUENCY,
    scale: 1.0,
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
    // K = °C + 273.15, applied by `to_si(v) = v*scale + offset`.
    offset: 273.15,
};

pub const KNOT: Unit = Unit {
    name: "knot",
    symbol: "kn",
    dimension: Dimension::VELOCITY,
    scale: 0.514444,
    offset: 0.0,
};

/// Pascal — SI pressure (M·L⁻¹·T⁻²).
pub const PASCAL: Unit = Unit {
    name: "pascal",
    symbol: "Pa",
    dimension: Dimension {
        length: -1,
        mass: 1,
        time: -2,
        ..Dimension::DIMENSIONLESS
    },
    scale: 1.0,
    offset: 0.0,
};

/// Volt — SI electric potential.
pub const VOLT: Unit = Unit {
    name: "volt",
    symbol: "V",
    dimension: Dimension::ELECTRIC_POTENTIAL,
    scale: 1.0,
    offset: 0.0,
};

/// Millivolt — 10⁻³ V (membrane potentials).
pub const MILLIVOLT: Unit = Unit {
    name: "millivolt",
    symbol: "mV",
    dimension: Dimension::ELECTRIC_POTENTIAL,
    scale: 1e-3,
    offset: 0.0,
};

/// Siemens — SI electrical conductance.
pub const SIEMENS: Unit = Unit {
    name: "siemens",
    symbol: "S",
    dimension: Dimension::ELECTRICAL_CONDUCTANCE,
    scale: 1.0,
    offset: 0.0,
};

/// Picosiemens — 10⁻¹² S (single-channel conductance).
pub const PICOSIEMENS: Unit = Unit {
    name: "picosiemens",
    symbol: "pS",
    dimension: Dimension::ELECTRICAL_CONDUCTANCE,
    scale: 1e-12,
    offset: 0.0,
};

/// MKS rayl — specific acoustic impedance (Pa·s/m).
pub const RAYL: Unit = Unit {
    name: "rayl",
    symbol: "Pa·s/m",
    dimension: Dimension::ACOUSTIC_IMPEDANCE,
    scale: 1.0,
    offset: 0.0,
};

/// Nanometer — 10⁻⁹ m (stereocilia tip-links, molecular scales).
pub const NANOMETER: Unit = Unit {
    name: "nanometer",
    symbol: "nm",
    dimension: Dimension::LENGTH,
    scale: 1e-9,
    offset: 0.0,
};

/// Millisecond — 10⁻³ s (neural latencies).
pub const MILLISECOND: Unit = Unit {
    name: "millisecond",
    symbol: "ms",
    dimension: Dimension::TIME,
    scale: 1e-3,
    offset: 0.0,
};

/// Microsecond — 10⁻⁶ s (interaural time differences).
pub const MICROSECOND: Unit = Unit {
    name: "microsecond",
    symbol: "µs",
    dimension: Dimension::TIME,
    scale: 1e-6,
    offset: 0.0,
};

/// Minute — 60 s.
pub const MINUTE: Unit = Unit {
    name: "minute",
    symbol: "min",
    dimension: Dimension::TIME,
    scale: 60.0,
    offset: 0.0,
};

/// Day — 86 400 s.
pub const DAY: Unit = Unit {
    name: "day",
    symbol: "d",
    dimension: Dimension::TIME,
    scale: 86_400.0,
    offset: 0.0,
};

/// Candela per square meter (nit) — SI luminance.
pub const CANDELA_PER_SQUARE_METER: Unit = Unit {
    name: "candela per square meter",
    symbol: "cd/m²",
    dimension: Dimension::LUMINANCE,
    scale: 1.0,
    offset: 0.0,
};

/// Millimolar — 10⁻³ mol/L. Since 1 mol/L = 1000 mol/m³, mmol/L = 1 mol/m³ (SI).
pub const MILLIMOLAR: Unit = Unit {
    name: "millimolar",
    symbol: "mmol/L",
    dimension: Dimension::MOLAR_CONCENTRATION,
    scale: 1.0,
    offset: 0.0,
};

/// Millinewton per meter — 10⁻³ N/m (membrane / surface tension).
pub const MILLINEWTON_PER_METER: Unit = Unit {
    name: "millinewton per meter",
    symbol: "mN/m",
    dimension: Dimension::SURFACE_TENSION,
    scale: 1e-3,
    offset: 0.0,
};

/// Beat per minute — musical tempo, a frequency (T⁻¹) of 1/60 Hz.
pub const BEAT_PER_MINUTE: Unit = Unit {
    name: "beat per minute",
    symbol: "BPM",
    dimension: Dimension::FREQUENCY,
    scale: 1.0 / 60.0,
    offset: 0.0,
};

/// Cubic meter per second — volumetric flow rate (L³·T⁻¹).
pub const CUBIC_METER_PER_SECOND: Unit = Unit {
    name: "cubic meter per second",
    symbol: "m³/s",
    dimension: Dimension {
        length: 3,
        time: -1,
        ..Dimension::DIMENSIONLESS
    },
    scale: 1.0,
    offset: 0.0,
};

/// The dimensionless unit — a pure number (ratios, counts, eccentricity).
pub const UNITLESS: Unit = Unit {
    name: "unitless",
    symbol: "1",
    dimension: Dimension::DIMENSIONLESS,
    scale: 1.0,
    offset: 0.0,
};

/// Parts per million — a dimensionless ratio scaled by 1e-6 (e.g. sensor
/// scale-factor error).
pub const PART_PER_MILLION: Unit = Unit {
    name: "part per million",
    symbol: "ppm",
    dimension: Dimension::DIMENSIONLESS,
    scale: 1e-6,
    offset: 0.0,
};

/// Decibel-hertz — carrier-to-noise-density ratio (C/N0), a logarithmic
/// level referenced to 1 Hz. The stored value IS the level in dB directly
/// (10·log₁₀ of the linear C/N0 ratio in Hz) — the same already-log idiom
/// [`NAT`]/[`SHANNON`] use for information content, over [`Dimension::LEVEL`].
/// Source: ITU-R Recommendation V.574-5 (08/2015) "Use of the decibel and
///         the neper in telecommunications"; Misra & Enge (2011) —
///         GNSS Signal Processing, the C/N0 metric.
pub const DECIBEL_HERTZ: Unit = Unit {
    name: "decibel-hertz",
    symbol: "dB-Hz",
    dimension: Dimension::LEVEL,
    scale: 1.0,
    offset: 0.0,
};

/// Floating-point operation (flop) — a dimensionless algorithm-cost COUNT,
/// NOT a rate (see [`Dimension::OPERATION_COUNT`]).
/// Source: Golub & Van Loan (2013), *Matrix Computations* 4th ed., Section 1.2.
pub const FLOP: Unit = Unit {
    name: "floating-point operation",
    symbol: "FLOP",
    dimension: Dimension::OPERATION_COUNT,
    scale: 1.0,
    offset: 0.0,
};

/// Reciprocal meter — the SI coherent derived unit of curvature.
/// Source: do Carmo (1976), *Differential Geometry of Curves and Surfaces*,
/// §1-5/§3-2; BIPM SI Brochure (2019) Table 4 (coherent derived units).
pub const RECIPROCAL_METER: Unit = Unit {
    name: "reciprocal meter",
    symbol: "m⁻¹",
    dimension: Dimension::CURVATURE,
    scale: 1.0,
    offset: 0.0,
};

/// Practical Salinity Unit (PSU) — the dimensionless unit of practical
/// salinity on the Practical Salinity Scale 1978 (PSS-78). UNESCO (1981),
/// "Background Papers and Supporting Data on the Practical Salinity Scale
/// 1978," UNESCO Technical Papers in Marine Science No. 37. The
/// oceanographic analogue of [`RADIAN`]/[`NAT`]: a named unit over a
/// dimensionless dimension ([`Dimension::SALINITY`]).
pub const PSU: Unit = Unit {
    name: "practical salinity unit",
    symbol: "PSU",
    dimension: Dimension::SALINITY,
    scale: 1.0,
    offset: 0.0,
};
