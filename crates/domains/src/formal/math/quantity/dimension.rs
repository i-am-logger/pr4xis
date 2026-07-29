#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Physical dimension — an element of the dimension group.
///
/// The 7 SI base dimensions form a basis for an abelian group under
/// multiplication. Every physical dimension is a product of powers
/// of these base dimensions:
///
///   `[Q] = L^a · M^b · T^c · I^d · Θ^e · N^f · J^g`
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
    /// Information: dimensionless (ISO/IEC 80000-13:2008 item 13-24
    /// "information content", unit entries 13-24.a/.b/.c — the shannon,
    /// hartley and nat are dimensionless units of information content,
    /// exactly as the radian is a dimensionless unit of angle; Shannon 1948).
    pub const INFORMATION: Self = Self::DIMENSIONLESS;
    /// Data size: dimensionless (ISO/IEC 80000-13:2008 item 13-9 "storage
    /// capacity"/"storage size", unit entry 13-9.a — the bit, octet and
    /// byte are dimensionless traditional units of storage width, a
    /// DIFFERENT quantity from [`Dimension::INFORMATION`]'s probabilistic
    /// information content despite sharing the "bit" name).
    pub const DATA_SIZE: Self = Self::DIMENSIONLESS;
    /// Angular velocity: T⁻¹ (rad/s)
    pub const ANGULAR_VELOCITY: Self = Self {
        time: -1,
        ..Self::DIMENSIONLESS
    };
    /// Pressure: M·L⁻¹·T⁻² (Pa)
    pub const PRESSURE: Self = Self {
        length: -1,
        mass: 1,
        time: -2,
        ..Self::DIMENSIONLESS
    };
    /// Electric potential: M·L²·T⁻³·I⁻¹ (V)
    pub const ELECTRIC_POTENTIAL: Self = Self {
        length: 2,
        mass: 1,
        time: -3,
        current: -1,
        ..Self::DIMENSIONLESS
    };
    /// Electrical conductance: M⁻¹·L⁻²·T³·I² (S = siemens)
    pub const ELECTRICAL_CONDUCTANCE: Self = Self {
        length: -2,
        mass: -1,
        time: 3,
        current: 2,
        ..Self::DIMENSIONLESS
    };
    /// Specific acoustic impedance: M·L⁻²·T⁻¹ (Pa·s/m = MKS rayl)
    pub const ACOUSTIC_IMPEDANCE: Self = Self {
        length: -2,
        mass: 1,
        time: -1,
        ..Self::DIMENSIONLESS
    };
    /// Luminance: J·L⁻² (cd/m² = nit)
    pub const LUMINANCE: Self = Self {
        length: -2,
        luminous: 1,
        ..Self::DIMENSIONLESS
    };
    /// Amount-of-substance concentration: N·L⁻³ (mol/m³)
    pub const MOLAR_CONCENTRATION: Self = Self {
        length: -3,
        amount: 1,
        ..Self::DIMENSIONLESS
    };
    /// Surface tension: M·T⁻² (N/m = kg/s²)
    pub const SURFACE_TENSION: Self = Self {
        mass: 1,
        time: -2,
        ..Self::DIMENSIONLESS
    };
    /// Momentum: L·M·T⁻¹ (kg·m/s). BIPM SI Brochure (2019), Table 3
    /// (momentum = mass × velocity).
    pub const MOMENTUM: Self = Self {
        length: 1,
        mass: 1,
        time: -1,
        ..Self::DIMENSIONLESS
    };
    /// Power: L²·M·T⁻³ (W = J/s). BIPM SI Brochure (2019), Table 4
    /// (power = energy / time).
    pub const POWER: Self = Self {
        length: 2,
        mass: 1,
        time: -3,
        ..Self::DIMENSIONLESS
    };
    /// Angular momentum: L²·M·T⁻¹ (kg·m²/s = J·s). BIPM SI Brochure (2019),
    /// Table 3 (angular momentum = position × momentum).
    pub const ANGULAR_MOMENTUM: Self = Self {
        length: 2,
        mass: 1,
        time: -1,
        ..Self::DIMENSIONLESS
    };
    /// Standard gravitational parameter μ = G·M: L³·T⁻² (m³/s²). BIPM SI
    /// Brochure (2019) gives G the dimension L³·M⁻¹·T⁻²; μ = G·M cancels the
    /// mass factor. Vallado (2013), *Fundamentals of Astrodynamics and
    /// Applications* 4th ed., §1.4 (two-body gravitational parameter).
    pub const GRAVITATIONAL_PARAMETER: Self = Self {
        length: 3,
        time: -2,
        ..Self::DIMENSIONLESS
    };
    /// Specific orbital energy: L²·T⁻² (J/kg = energy per unit mass).
    /// BIPM SI Brochure (2019) Table 4 gives energy the dimension
    /// M·L²·T⁻²; dividing by mass cancels M. Vallado (2013) §2.3
    /// (vis-viva equation, specific mechanical energy ε = v²/2 − μ/r).
    pub const SPECIFIC_ENERGY: Self = Self {
        length: 2,
        time: -2,
        ..Self::DIMENSIONLESS
    };
    /// Level of a quantity: dimensionless (ITU-R Recommendation V.574-5
    /// (08/2015) "Use of the decibel and the neper in telecommunications" —
    /// the decibel is a dimensionless logarithmic ratio, the same
    /// identity-dimension idiom as [`Dimension::ANGLE`]/
    /// [`Dimension::INFORMATION`]).
    pub const LEVEL: Self = Self::DIMENSIONLESS;
    /// Operation count: dimensionless (Golub & Van Loan (2013), *Matrix
    /// Computations* 4th ed., Section 1.2 "flop counting" — algorithm cost
    /// is conventionally reported as a COUNT of floating-point operations,
    /// a DIFFERENT quantity from a computational *rate* despite sharing the
    /// "flop(s)" name — the same naming trap [`Dimension::DATA_SIZE`]
    /// documents against [`Dimension::INFORMATION`]).
    pub const OPERATION_COUNT: Self = Self::DIMENSIONLESS;
    /// Curvature: L⁻¹ (m⁻¹) — the reciprocal of the local radius of
    /// curvature of a curve or surface. do Carmo, M.P. (1976), *Differential
    /// Geometry of Curves and Surfaces*, Prentice-Hall, §1-5/§3-2 (curvature
    /// κ = 1/R; principal curvatures of a surface). The dimension of the
    /// principal-curvature signs Goldstein (1987) §3 classifies terrain
    /// features by.
    pub const CURVATURE: Self = Self {
        length: -1,
        ..Self::DIMENSIONLESS
    };
    /// Practical salinity: dimensionless (UNESCO (1981), "Background Papers
    /// and Supporting Data on the Practical Salinity Scale 1978," UNESCO
    /// Technical Papers in Marine Science No. 37 — practical salinity S is
    /// defined on the Practical Salinity Scale 1978 (PSS-78) as a function
    /// of the conductivity RATIO of the sample to a standard KCl solution,
    /// so it carries no physical dimension despite the traditional "‰"/PSU
    /// notation — the same identity-dimension idiom as [`Dimension::ANGLE`]/
    /// [`Dimension::INFORMATION`]).
    pub const SALINITY: Self = Self::DIMENSIONLESS;

    /// Group operation: multiply dimensions (add exponents).
    ///
    /// This is the abelian group operation.
    /// `[A] · [B] = L^(a1+a2) · M^(b1+b2) · ...`
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
    /// `[A]⁻¹ = L^(-a) · M^(-b) · ...`
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

    /// Divide dimensions: `[A] / [B] = [A] · [B]⁻¹`.
    pub fn divide(&self, other: &Self) -> Self {
        self.multiply(&other.inverse())
    }

    /// Raise to an integer power: `[A]^n`.
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
