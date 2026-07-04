#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::quantity::dimension::Dimension;
use crate::formal::math::quantity::unit::Unit;

/// A physical quantity: a numerical value WITH a dimension.
///
/// This is the central type. In sensor fusion, every measurement
/// and every state component IS a quantity — not a bare float.
///
/// Adding quantities with different dimensions is a type error
/// (enforced at runtime; could be compile-time with const generics).
///
/// Source: BIPM SI Brochure (2019), Section 1.
///         Tao (2012). "A mathematical formalization of dimensional analysis."
#[derive(Debug, Clone, PartialEq)]
pub struct Quantity {
    pub value: f64,
    pub dimension: Dimension,
}

impl Quantity {
    pub fn new(value: f64, dimension: Dimension) -> Self {
        Self { value, dimension }
    }

    /// Dimensionless quantity (pure number).
    pub fn dimensionless(value: f64) -> Self {
        Self {
            value,
            dimension: Dimension::DIMENSIONLESS,
        }
    }

    /// Create from a value and unit.
    pub fn from_unit(value: f64, unit: &Unit) -> Self {
        Self {
            value: unit.to_si(value),
            dimension: unit.dimension,
        }
    }

    /// Add two quantities. Returns None if dimensions don't match.
    ///
    /// This is the fundamental constraint: you can only add
    /// quantities with the same dimension.
    pub fn add(&self, other: &Self) -> Option<Self> {
        if !self.dimension.is_compatible(&other.dimension) {
            return None;
        }
        Some(Self {
            value: self.value + other.value,
            dimension: self.dimension,
        })
    }

    /// Subtract two quantities. Returns None if dimensions don't match.
    pub fn sub(&self, other: &Self) -> Option<Self> {
        if !self.dimension.is_compatible(&other.dimension) {
            return None;
        }
        Some(Self {
            value: self.value - other.value,
            dimension: self.dimension,
        })
    }

    /// Multiply two quantities. Dimensions multiply (exponents add).
    pub fn mul(&self, other: &Self) -> Self {
        Self {
            value: self.value * other.value,
            dimension: self.dimension.multiply(&other.dimension),
        }
    }

    /// Divide two quantities. Dimensions divide (exponents subtract).
    pub fn div(&self, other: &Self) -> Self {
        Self {
            value: self.value / other.value,
            dimension: self.dimension.divide(&other.dimension),
        }
    }

    /// Scale by a dimensionless factor.
    pub fn scale(&self, factor: f64) -> Self {
        Self {
            value: self.value * factor,
            dimension: self.dimension,
        }
    }

    /// Raise to integer power.
    pub fn power(&self, n: i8) -> Self {
        Self {
            value: self.value.powi(n as i32),
            dimension: self.dimension.power(n),
        }
    }

    /// Is this quantity dimensionless?
    pub fn is_dimensionless(&self) -> bool {
        self.dimension.is_dimensionless()
    }

    /// Express in a given unit. Returns None if incompatible dimension.
    pub fn in_unit(&self, unit: &Unit) -> Option<f64> {
        if !self.dimension.is_compatible(&unit.dimension) {
            return None;
        }
        Some(unit.from_si(self.value))
    }
}

/// A closed interval of quantities of a single dimension — `[min, max]`.
///
/// The typed replacement for an ad-hoc `(f64, f64)` or a prose accuracy string
/// ("0.1–1 m"): both endpoints must share a dimension (a range from 1 m to 5 s
/// is meaningless), and `min ≤ max`. The constructor refuses a malformed pair,
/// mirroring `Quantity::add`'s dimensional guard.
///
/// Source: BIPM SI Brochure (2019) — a measured range is an interval of
/// quantities of one quantity-kind.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantityRange {
    pub min: Quantity,
    pub max: Quantity,
}

impl QuantityRange {
    /// Construct `[min, max]`. Returns `None` if the endpoints have different
    /// dimensions or if `max < min`.
    pub fn new(min: Quantity, max: Quantity) -> Option<Self> {
        if !min.dimension.is_compatible(&max.dimension) || max.value < min.value {
            return None;
        }
        Some(Self { min, max })
    }

    /// The shared dimension of the interval's endpoints.
    pub fn dimension(&self) -> Dimension {
        self.min.dimension
    }

    /// Does `q` lie within `[min, max]`? False if `q` has a different dimension.
    pub fn contains(&self, q: &Quantity) -> bool {
        q.dimension.is_compatible(&self.min.dimension)
            && q.value >= self.min.value
            && q.value <= self.max.value
    }
}
