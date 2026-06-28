#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::value::{AngleMode, CalcError, Value};

/// Unary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Abs,
    Sqrt,
    Cbrt,
    Square,
    Reciprocal,
    Factorial,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Ln,
    Log10,
    Log2,
    Exp,
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    Floor,
    Ceil,
    Round,
    ToRadians,
    ToDegrees,
}

impl UnaryOp {
    /// Apply this operation with domain enforcement.
    pub fn apply(&self, val: &Value, angle_mode: AngleMode) -> Result<Value, CalcError> {
        match self {
            UnaryOp::Negate => Ok(val.negate()),
            UnaryOp::Abs => {
                if val.is_negative() {
                    Ok(val.negate())
                } else {
                    Ok(val.clone())
                }
            }
            UnaryOp::Square => match val {
                // Checked square with float fallback — an unchecked `n * n` would
                // overflow-panic (debug) / wrap to a wrong answer (release).
                Value::Rational(n, d) => match (n.checked_mul(*n), d.checked_mul(*d)) {
                    (Some(num), Some(den)) => Value::rational(num, den),
                    _ => Value::float(val.to_f64() * val.to_f64()),
                },
                Value::Float(f) => Value::float(f * f),
            },
            UnaryOp::Reciprocal => val.reciprocal(),
            UnaryOp::Sqrt => {
                let f = val.to_f64();
                if f < 0.0 {
                    return Err(CalcError::NegativeSquareRoot);
                }
                // Check if result is exact rational
                if let Value::Rational(n, d) = val {
                    let sn = isqrt(*n);
                    let sd = isqrt(*d);
                    if sn * sn == *n && sd * sd == *d {
                        return Value::rational(sn, sd);
                    }
                }
                Value::float(f.sqrt())
            }
            UnaryOp::Cbrt => Value::float(val.to_f64().cbrt()),
            UnaryOp::Factorial => {
                let n = val.to_f64();
                if n < 0.0 || n != n.floor() {
                    return Err(CalcError::InvalidDomain {
                        op: "factorial".into(),
                        value: n,
                    });
                }
                let n = n as u64;
                if n > 20 {
                    return Err(CalcError::Overflow);
                }
                let result: u64 = (1..=n).product();
                Ok(Value::int(result as i64))
            }
            UnaryOp::Sin => {
                let angle = to_radians(val.to_f64(), angle_mode);
                Value::float(angle.sin())
            }
            UnaryOp::Cos => {
                let angle = to_radians(val.to_f64(), angle_mode);
                Value::float(angle.cos())
            }
            UnaryOp::Tan => {
                let angle = to_radians(val.to_f64(), angle_mode);
                let cos = angle.cos();
                if cos.abs() < 1e-10 {
                    return Err(CalcError::TanUndefined);
                }
                Value::float(angle.tan())
            }
            UnaryOp::Asin => {
                let f = val.to_f64();
                if !(-1.0..=1.0).contains(&f) {
                    return Err(CalcError::InvalidDomain {
                        op: "asin".into(),
                        value: f,
                    });
                }
                let result = f.asin();
                Value::float(from_radians(result, angle_mode))
            }
            UnaryOp::Acos => {
                let f = val.to_f64();
                if !(-1.0..=1.0).contains(&f) {
                    return Err(CalcError::InvalidDomain {
                        op: "acos".into(),
                        value: f,
                    });
                }
                let result = f.acos();
                Value::float(from_radians(result, angle_mode))
            }
            UnaryOp::Atan => {
                let result = val.to_f64().atan();
                Value::float(from_radians(result, angle_mode))
            }
            UnaryOp::Ln => {
                let f = val.to_f64();
                if f <= 0.0 {
                    return Err(CalcError::LogOfNonPositive);
                }
                Value::float(f.ln())
            }
            UnaryOp::Log10 => {
                let f = val.to_f64();
                if f <= 0.0 {
                    return Err(CalcError::LogOfNonPositive);
                }
                Value::float(f.log10())
            }
            UnaryOp::Log2 => {
                let f = val.to_f64();
                if f <= 0.0 {
                    return Err(CalcError::LogOfNonPositive);
                }
                Value::float(f.log2())
            }
            UnaryOp::Exp => Value::float(val.to_f64().exp()),
            UnaryOp::Sinh => Value::float(val.to_f64().sinh()),
            UnaryOp::Cosh => Value::float(val.to_f64().cosh()),
            UnaryOp::Tanh => Value::float(val.to_f64().tanh()),
            UnaryOp::Asinh => Value::float(val.to_f64().asinh()),
            UnaryOp::Acosh => {
                let f = val.to_f64();
                if f < 1.0 {
                    return Err(CalcError::InvalidDomain {
                        op: "acosh".into(),
                        value: f,
                    });
                }
                Value::float(f.acosh())
            }
            UnaryOp::Atanh => {
                let f = val.to_f64();
                if f <= -1.0 || f >= 1.0 {
                    return Err(CalcError::InvalidDomain {
                        op: "atanh".into(),
                        value: f,
                    });
                }
                Value::float(f.atanh())
            }
            UnaryOp::Floor => Ok(Value::int(val.to_f64().floor() as i64)),
            UnaryOp::Ceil => Ok(Value::int(val.to_f64().ceil() as i64)),
            UnaryOp::Round => Ok(Value::int(val.to_f64().round() as i64)),
            UnaryOp::ToRadians => Value::float(val.to_f64() * core::f64::consts::PI / 180.0),
            UnaryOp::ToDegrees => Value::float(val.to_f64() * 180.0 / core::f64::consts::PI),
        }
    }
}

/// Binary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,
}

impl BinaryOp {
    /// Precedence for order of operations (higher = binds tighter).
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOp::Add | BinaryOp::Subtract => 1,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 2,
            BinaryOp::Power => 3,
        }
    }

    /// Apply this operation with domain enforcement.
    pub fn apply(&self, lhs: &Value, rhs: &Value) -> Result<Value, CalcError> {
        match self {
            BinaryOp::Add => add(lhs, rhs),
            BinaryOp::Subtract => {
                let neg = rhs.negate();
                add(lhs, &neg)
            }
            BinaryOp::Multiply => multiply(lhs, rhs),
            BinaryOp::Divide => {
                if rhs.is_zero() {
                    return Err(CalcError::DivisionByZero);
                }
                let recip = rhs.reciprocal()?;
                multiply(lhs, &recip)
            }
            BinaryOp::Power => Value::float(lhs.to_f64().powf(rhs.to_f64())),
            BinaryOp::Modulo => {
                if rhs.is_zero() {
                    return Err(CalcError::DivisionByZero);
                }
                Value::float(lhs.to_f64() % rhs.to_f64())
            }
        }
    }
}

fn add(a: &Value, b: &Value) -> Result<Value, CalcError> {
    match (a, b) {
        (Value::Rational(an, ad), Value::Rational(bn, bd)) => {
            // Exact rational add when the i64 num/den fit; otherwise fall back to
            // float. Unchecked i64 arithmetic here would panic on overflow (debug)
            // or silently wrap to a wrong answer (release) — both dishonest. Honest:
            // lose precision gracefully, never crash or confabulate.
            match (
                ad.checked_mul(*bd),
                an.checked_mul(*bd),
                bn.checked_mul(*ad),
            ) {
                (Some(den), Some(t1), Some(t2)) => match t1.checked_add(t2) {
                    Some(num) => Value::rational(num, den),
                    None => Value::float(a.to_f64() + b.to_f64()),
                },
                _ => Value::float(a.to_f64() + b.to_f64()),
            }
        }
        _ => Value::float(a.to_f64() + b.to_f64()),
    }
}

fn multiply(a: &Value, b: &Value) -> Result<Value, CalcError> {
    match (a, b) {
        (Value::Rational(an, ad), Value::Rational(bn, bd)) => {
            // Checked rational multiply with float fallback — see `add`.
            match (an.checked_mul(*bn), ad.checked_mul(*bd)) {
                (Some(num), Some(den)) => Value::rational(num, den),
                _ => Value::float(a.to_f64() * b.to_f64()),
            }
        }
        _ => Value::float(a.to_f64() * b.to_f64()),
    }
}

fn to_radians(angle: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Radians => angle,
        AngleMode::Degrees => angle * core::f64::consts::PI / 180.0,
    }
}

fn from_radians(angle: f64, mode: AngleMode) -> f64 {
    match mode {
        AngleMode::Radians => angle,
        AngleMode::Degrees => angle * 180.0 / core::f64::consts::PI,
    }
}

/// Integer square root (returns floor(sqrt(n))).
fn isqrt(n: i64) -> i64 {
    if n < 0 {
        return 0;
    }
    let s = (n as f64).sqrt() as i64;
    // Correct for floating point errors
    if (s + 1) * (s + 1) == n { s + 1 } else { s }
}
