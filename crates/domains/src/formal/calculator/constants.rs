#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::value::Value;
use crate::formal::math::fibonacci;

/// Mathematical constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constant {
    Pi,
    E,
    GoldenRatio,
    Sqrt2,
    Ln2,
    Ln10,
}

impl Constant {
    pub fn value(&self) -> Value {
        Value::Float(match self {
            Constant::Pi => core::f64::consts::PI,
            Constant::E => core::f64::consts::E,
            Constant::GoldenRatio => fibonacci::golden_ratio(),
            Constant::Sqrt2 => core::f64::consts::SQRT_2,
            Constant::Ln2 => core::f64::consts::LN_2,
            Constant::Ln10 => core::f64::consts::LN_10,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Constant::Pi => "π",
            Constant::E => "e",
            Constant::GoldenRatio => "φ",
            Constant::Sqrt2 => "√2",
            Constant::Ln2 => "ln(2)",
            Constant::Ln10 => "ln(10)",
        }
    }

    pub fn all() -> Vec<Constant> {
        vec![
            Constant::Pi,
            Constant::E,
            Constant::GoldenRatio,
            Constant::Sqrt2,
            Constant::Ln2,
            Constant::Ln10,
        ]
    }
}
