use super::value::CalcError;

/// Bitwise operations (programmer mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitwiseOp {
    And,
    Or,
    Xor,
    Not,
    ShiftLeft,
    ShiftRight,
}

impl BitwiseOp {
    /// Apply a binary bitwise operation.
    pub fn apply(&self, lhs: i64, rhs: i64) -> Result<i64, CalcError> {
        match self {
            BitwiseOp::And => Ok(lhs & rhs),
            BitwiseOp::Or => Ok(lhs | rhs),
            BitwiseOp::Xor => Ok(lhs ^ rhs),
            BitwiseOp::Not => Ok(!lhs), // rhs ignored
            BitwiseOp::ShiftLeft => {
                if !(0..=63).contains(&rhs) {
                    return Err(CalcError::InvalidDomain {
                        op: "shift".into(),
                        value: rhs as f64,
                    });
                }
                Ok(lhs << rhs)
            }
            BitwiseOp::ShiftRight => {
                if !(0..=63).contains(&rhs) {
                    return Err(CalcError::InvalidDomain {
                        op: "shift".into(),
                        value: rhs as f64,
                    });
                }
                Ok(lhs >> rhs)
            }
        }
    }
}

/// Base representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl Base {
    pub fn radix(&self) -> u32 {
        match self {
            Base::Binary => 2,
            Base::Octal => 8,
            Base::Decimal => 10,
            Base::Hexadecimal => 16,
        }
    }

    /// Format a number in this base.
    pub fn format(&self, value: i64) -> String {
        match self {
            Base::Binary => format!("0b{:b}", value),
            Base::Octal => format!("0o{:o}", value),
            Base::Decimal => format!("{}", value),
            Base::Hexadecimal => format!("0x{:X}", value),
        }
    }

    /// Parse a string in this base.
    pub fn parse(&self, s: &str) -> Result<i64, CalcError> {
        let s = s
            .trim_start_matches("0b")
            .trim_start_matches("0o")
            .trim_start_matches("0x")
            .trim_start_matches("0X");
        i64::from_str_radix(s, self.radix()).map_err(|_| CalcError::InvalidDomain {
            op: format!("parse base {}", self.radix()),
            value: 0.0,
        })
    }
}
