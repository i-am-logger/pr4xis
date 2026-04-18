#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::temporal::duration::Duration;
use crate::formal::math::temporal::instant::Instant;

/// A temporal interval — a bounded region of time with a beginning and end.
///
/// W3C OWL-Time: "Intervals are things with extent."
/// Allen (1983): intervals are the primitive ontological entities
/// in interval temporal logic.
#[derive(Debug, Clone)]
pub struct Interval {
    pub begin: Instant,
    pub end: Instant,
}

impl Interval {
    /// Create an interval. Begin must be before end.
    pub fn new(begin: Instant, end: Instant) -> Option<Self> {
        if begin.system != end.system || begin.seconds >= end.seconds {
            return None;
        }
        Some(Self { begin, end })
    }

    /// Duration of the interval.
    pub fn duration(&self) -> Duration {
        self.begin.duration_to(&self.end).unwrap()
    }

    /// Midpoint of the interval.
    pub fn midpoint(&self) -> Instant {
        let mid = (self.begin.seconds + self.end.seconds) / 2.0;
        Instant::new(mid, self.begin.system)
    }

    /// Does this interval contain an instant?
    pub fn contains_instant(&self, t: &Instant) -> bool {
        t.system == self.begin.system
            && t.seconds >= self.begin.seconds
            && t.seconds <= self.end.seconds
    }

    /// Does this interval overlap with another?
    pub fn overlaps(&self, other: &Self) -> bool {
        self.begin.system == other.begin.system
            && self.begin.seconds < other.end.seconds
            && other.begin.seconds < self.end.seconds
    }

    /// Does this interval contain another entirely?
    pub fn contains_interval(&self, other: &Self) -> bool {
        self.begin.system == other.begin.system
            && self.begin.seconds <= other.begin.seconds
            && self.end.seconds >= other.end.seconds
    }
}
