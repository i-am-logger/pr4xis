#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// A temporal duration — the measure of elapsed time.
///
/// Duration forms a metric on the space of instants:
/// d(t1, t2) = |t2 - t1|.
///
/// Durations also form a vector space over R:
/// - Addition: d1 + d2 is a duration
/// - Scalar multiplication: α · d is a duration
/// - Zero element: 0 seconds
/// - Additive inverse: -d
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Duration {
    secs: f64,
}

impl Duration {
    pub fn from_seconds(secs: f64) -> Self {
        Self { secs }
    }

    pub fn from_milliseconds(ms: f64) -> Self {
        Self { secs: ms / 1000.0 }
    }

    pub fn from_microseconds(us: f64) -> Self {
        Self { secs: us / 1e6 }
    }

    pub fn from_nanoseconds(ns: f64) -> Self {
        Self { secs: ns / 1e9 }
    }

    pub fn zero() -> Self {
        Self { secs: 0.0 }
    }

    pub fn seconds(&self) -> f64 {
        self.secs
    }

    pub fn milliseconds(&self) -> f64 {
        self.secs * 1000.0
    }

    pub fn microseconds(&self) -> f64 {
        self.secs * 1e6
    }

    pub fn nanoseconds(&self) -> f64 {
        self.secs * 1e9
    }

    /// Absolute value (non-negative duration).
    pub fn abs(&self) -> Self {
        Self {
            secs: self.secs.abs(),
        }
    }

    /// Is this a positive (forward) duration?
    pub fn is_positive(&self) -> bool {
        self.secs > 0.0
    }

    /// Is this a negative (backward) duration?
    pub fn is_negative(&self) -> bool {
        self.secs < 0.0
    }

    /// Vector space: addition.
    pub fn add(&self, other: &Self) -> Self {
        Self {
            secs: self.secs + other.secs,
        }
    }

    /// Vector space: subtraction.
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            secs: self.secs - other.secs,
        }
    }

    /// Vector space: scalar multiplication.
    pub fn scale(&self, factor: f64) -> Self {
        Self {
            secs: self.secs * factor,
        }
    }

    /// Vector space: additive inverse.
    pub fn negate(&self) -> Self {
        Self { secs: -self.secs }
    }
}
