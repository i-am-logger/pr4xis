#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Physical dimension — an element of the dimension group.
///
/// The 7 SI base dimensions form a basis for an abelian group under
/// multiplication. Every physical dimension is a product of powers
/// of these base dimensions:
///
///   [Q] = L^a · M^b · T^c · I^d · Θ^e · N^f · J^g
///
/// where the exponents (a,b,c,d,e,f,g) uniquely identify the dimension.
///
/// Examples:
///   Velocity = L¹·T⁻¹         → (1, 0, -1, 0, 0, 0, 0)
///   Force    = L¹·M¹·T⁻²     → (1, 1, -2, 0, 0, 0, 0)
///   Energy   = L²·M¹·T⁻²     → (2, 1, -2, 0, 0, 0, 0)
///
/// Source: Tao, T. (2012). "A mathematical formalization of dimensional analysis."
///         Hart, J. (2021). "Dimensioned Algebra." ArXiv 2108.08703.
///         Bureau International des Poids et Mesures (BIPM), SI Brochure (2019).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dimension {
    /// Length (L), meter. Exponent.
    pub length: i8,
    /// Mass (M), kilogram. Exponent.
    pub mass: i8,
    /// Time (T), second. Exponent.
    pub time: i8,
    /// Electric current (I), ampere. Exponent.
    pub current: i8,
    /// Thermodynamic temperature (Θ), kelvin. Exponent.
    pub temperature: i8,
    /// Amount of substance (N), mole. Exponent.
    pub amount: i8,
    /// Luminous intensity (J), candela. Exponent.
    pub luminous: i8,
}

impl Dimension {
    /// Dimensionless (all exponents zero). The identity element.
    pub const DIMENSIONLESS: Self = Self {
        length: 0,
        mass: 0,
        time: 0,
        current: 0,
        temperature: 0,
        amount: 0,
        luminous: 0,
    };

    // --- SI base dimensions ---

    pub const LENGTH: Self = Self {
        length: 1,
        ..Self::DIMENSIONLESS
    };
    pub const MASS: Self = Self {
        mass: 1,
        ..Self::DIMENSIONLESS
    };
    pub const TIME: Self = Self {
        time: 1,
        ..Self::DIMENSIONLESS
    };
    pub const CURRENT: Self = Self {
        current: 1,
        ..Self::DIMENSIONLESS
    };
    pub const TEMPERATURE: Self = Self {
        temperature: 1,
        ..Self::DIMENSIONLESS
    };
    pub const AMOUNT: Self = Self {
        amount: 1,
        ..Self::DIMENSIONLESS
    };
    pub const LUMINOUS: Self = Self {
        luminous: 1,
        ..Self::DIMENSIONLESS
    };

    // --- Common derived dimensions ---

    /// Velocity: L·T⁻¹ (m/s)
    pub const VELOCITY: Self = Self {
        length: 1,
        time: -1,
        ..Self::DIMENSIONLESS
    };
    /// Acceleration: L·T⁻² (m/s²)
    pub const ACCELERATION: Self = Self {
        length: 1,
        time: -2,
        ..Self::DIMENSIONLESS
    };
    /// Force: L·M·T⁻² (N = kg·m/s²)
    pub const FORCE: Self = Self {
        length: 1,
        mass: 1,
        time: -2,
        ..Self::DIMENSIONLESS
    };
    /// Energy: L²·M·T⁻² (J = kg·m²/s²)
    pub const ENERGY: Self = Self {
        length: 2,
        mass: 1,
        time: -2,
        ..Self::DIMENSIONLESS
    };
    /// Frequency: T⁻¹ (Hz)
    pub const FREQUENCY: Self = Self {
        time: -1,
        ..Self::DIMENSIONLESS
    };
    /// Angle: dimensionless (radian is L/L)
    pub const ANGLE: Self = Self::DIMENSIONLESS;
    /// Angular velocity: T⁻¹ (rad/s)
    pub const ANGULAR_VELOCITY: Self = Self {
        time: -1,
        ..Self::DIMENSIONLESS
    };

    /// Group operation: multiply dimensions (add exponents).
    ///
    /// This is the abelian group operation.
    /// [A] · [B] = L^(a1+a2) · M^(b1+b2) · ...
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            length: self.length + other.length,
            mass: self.mass + other.mass,
            time: self.time + other.time,
            current: self.current + other.current,
            temperature: self.temperature + other.temperature,
            amount: self.amount + other.amount,
            luminous: self.luminous + other.luminous,
        }
    }

    /// Group inverse: reciprocal dimension (negate exponents).
    ///
    /// [A]⁻¹ = L^(-a) · M^(-b) · ...
    pub fn inverse(&self) -> Self {
        Self {
            length: -self.length,
            mass: -self.mass,
            time: -self.time,
            current: -self.current,
            temperature: -self.temperature,
            amount: -self.amount,
            luminous: -self.luminous,
        }
    }

    /// Divide dimensions: [A] / [B] = [A] · [B]⁻¹.
    pub fn divide(&self, other: &Self) -> Self {
        self.multiply(&other.inverse())
    }

    /// Raise to an integer power: [A]^n.
    pub fn power(&self, n: i8) -> Self {
        Self {
            length: self.length * n,
            mass: self.mass * n,
            time: self.time * n,
            current: self.current * n,
            temperature: self.temperature * n,
            amount: self.amount * n,
            luminous: self.luminous * n,
        }
    }

    /// Is this dimensionless?
    pub fn is_dimensionless(&self) -> bool {
        *self == Self::DIMENSIONLESS
    }

    /// Are two dimensions compatible (can be added)?
    ///
    /// Quantities can only be added if they have the same dimension.
    /// This is the fundamental rule of dimensional analysis.
    pub fn is_compatible(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl core::fmt::Display for Dimension {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut parts = Vec::new();
        if self.length != 0 {
            parts.push(format!("L^{}", self.length));
        }
        if self.mass != 0 {
            parts.push(format!("M^{}", self.mass));
        }
        if self.time != 0 {
            parts.push(format!("T^{}", self.time));
        }
        if self.current != 0 {
            parts.push(format!("I^{}", self.current));
        }
        if self.temperature != 0 {
            parts.push(format!("Θ^{}", self.temperature));
        }
        if self.amount != 0 {
            parts.push(format!("N^{}", self.amount));
        }
        if self.luminous != 0 {
            parts.push(format!("J^{}", self.luminous));
        }
        if parts.is_empty() {
            write!(f, "1")
        } else {
            write!(f, "{}", parts.join("·"))
        }
    }
}
