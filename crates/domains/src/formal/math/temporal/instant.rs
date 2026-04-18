#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::temporal::duration::Duration;
use crate::formal::math::temporal::time_system::TimeSystem;

/// A point in time — the primitive temporal entity.
///
/// W3C OWL-Time: "An instant is point-like in that it has no interior
/// points... it is generally safe to think of an instant as an interval
/// with zero length."
///
/// Stored as seconds since an epoch, qualified by a time system.
#[derive(Debug, Clone, PartialEq)]
pub struct Instant {
    /// Seconds since epoch in the given time system.
    pub seconds: f64,
    /// Which time system this instant is expressed in.
    pub system: TimeSystem,
}

impl Instant {
    pub fn new(seconds: f64, system: TimeSystem) -> Self {
        Self { seconds, system }
    }

    /// Duration from self to other (other - self).
    /// Both must be in the same time system.
    pub fn duration_to(&self, other: &Self) -> Option<Duration> {
        if self.system != other.system {
            return None;
        }
        Some(Duration::from_seconds(other.seconds - self.seconds))
    }

    /// Advance by a duration.
    pub fn advance(&self, dt: &Duration) -> Self {
        Self {
            seconds: self.seconds + dt.seconds(),
            system: self.system,
        }
    }

    /// Go back by a duration.
    pub fn retreat(&self, dt: &Duration) -> Self {
        Self {
            seconds: self.seconds - dt.seconds(),
            system: self.system,
        }
    }

    /// Total order: is self before other?
    /// Both must be in the same time system.
    pub fn is_before(&self, other: &Self) -> bool {
        self.system == other.system && self.seconds < other.seconds
    }

    /// Is self after other?
    pub fn is_after(&self, other: &Self) -> bool {
        self.system == other.system && self.seconds > other.seconds
    }

    /// Are two instants simultaneous (within tolerance)?
    pub fn is_simultaneous(&self, other: &Self, tolerance: f64) -> bool {
        self.system == other.system && (self.seconds - other.seconds).abs() < tolerance
    }
}
