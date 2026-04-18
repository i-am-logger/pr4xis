#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use core::fmt;

/// Angle mode for trigonometric functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngleMode {
    Radians,
    Degrees,
}

/// Calculator errors — every domain violation is explicit.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcError {
    DivisionByZero,
    NegativeSquareRoot,
    LogOfNonPositive,
    TanUndefined,
    Overflow,
    Underflow,
    InvalidDomain { op: String, value: f64 },
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::DivisionByZero => write!(f, "division by zero"),
            CalcError::NegativeSquareRoot => write!(f, "square root of negative number"),
            CalcError::LogOfNonPositive => write!(f, "logarithm of non-positive number"),
            CalcError::TanUndefined => write!(f, "tangent undefined at this angle"),
            CalcError::Overflow => write!(f, "result too large"),
            CalcError::Underflow => write!(f, "result too small"),
            CalcError::InvalidDomain { op, value } => write!(f, "{} undefined for {}", op, value),
        }
    }
}

/// A value in the calculator — rational or float.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Exact rational number (numerator, denominator). Always simplified.
    Rational(i64, i64),
    /// Floating point (when exact isn't possible).
    Float(f64),
}

impl Value {
    /// Create a rational value, auto-simplified.
    pub fn rational(num: i64, den: i64) -> Result<Self, CalcError> {
        if den == 0 {
            return Err(CalcError::DivisionByZero);
        }
        let g = gcd(num.unsigned_abs(), den.unsigned_abs()) as i64;
        let sign = if den < 0 { -1 } else { 1 };
        Ok(Value::Rational((num * sign) / g, (den * sign) / g))
    }

    /// Create an integer value.
    pub fn int(n: i64) -> Self {
        Value::Rational(n, 1)
    }

    /// Create a float value with overflow/underflow checking.
    pub fn float(f: f64) -> Result<Self, CalcError> {
        if f.is_infinite() {
            return Err(CalcError::Overflow);
        }
        if f.is_nan() {
            return Err(CalcError::InvalidDomain {
                op: "result".into(),
                value: f,
            });
        }
        Ok(Value::Float(f))
    }

    /// Convert to f64 for computation.
    pub fn to_f64(&self) -> f64 {
        match self {
            Value::Rational(n, d) => *n as f64 / *d as f64,
            Value::Float(f) => *f,
        }
    }

    /// Is this value zero?
    pub fn is_zero(&self) -> bool {
        match self {
            Value::Rational(n, _) => *n == 0,
            Value::Float(f) => *f == 0.0,
        }
    }

    /// Is this value one?
    pub fn is_one(&self) -> bool {
        match self {
            Value::Rational(n, d) => *n == *d,
            Value::Float(f) => (*f - 1.0).abs() < f64::EPSILON,
        }
    }

    /// Is this value negative?
    pub fn is_negative(&self) -> bool {
        match self {
            Value::Rational(n, _) => *n < 0,
            Value::Float(f) => *f < 0.0,
        }
    }

    /// Negate.
    pub fn negate(&self) -> Self {
        match self {
            Value::Rational(n, d) => Value::Rational(-n, *d),
            Value::Float(f) => Value::Float(-f),
        }
    }

    /// Reciprocal.
    pub fn reciprocal(&self) -> Result<Self, CalcError> {
        match self {
            Value::Rational(n, d) => {
                if *n == 0 {
                    Err(CalcError::DivisionByZero)
                } else {
                    Value::rational(*d, *n)
                }
            }
            Value::Float(f) => {
                if *f == 0.0 {
                    Err(CalcError::DivisionByZero)
                } else {
                    Value::float(1.0 / f)
                }
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Rational(n, 1) => write!(f, "{}", n),
            Value::Rational(n, d) => write!(f, "{}/{}", n, d),
            Value::Float(v) => write!(f, "{}", v),
        }
    }
}

/// Greatest common divisor (Euclidean algorithm).
pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.max(1)
}

/// Least common multiple.
pub fn lcm(a: u64, b: u64) -> u64 {
    a / gcd(a, b) * b
}
