/// M-of-N confirmation logic.
///
/// A tentative track is confirmed when it receives M detections
/// in N consecutive scans. Otherwise it's deleted.
///
/// Source: Bar-Shalom et al. (2001), Section 7.4.
#[derive(Debug, Clone)]
pub struct MofNLogic {
    /// Required hits for confirmation.
    pub m: usize,
    /// Window size.
    pub n: usize,
    /// History of hits (true) and misses (false).
    pub history: Vec<bool>,
}

impl MofNLogic {
    pub fn new(m: usize, n: usize) -> Self {
        Self {
            m,
            n,
            history: Vec::new(),
        }
    }

    /// Record a detection (hit).
    pub fn record_hit(&mut self) {
        self.history.push(true);
        if self.history.len() > self.n {
            self.history.remove(0);
        }
    }

    /// Record a miss.
    pub fn record_miss(&mut self) {
        self.history.push(false);
        if self.history.len() > self.n {
            self.history.remove(0);
        }
    }

    /// Count hits in the window.
    pub fn hits(&self) -> usize {
        self.history.iter().filter(|&&h| h).count()
    }

    /// Is the track confirmed?
    pub fn is_confirmed(&self) -> bool {
        self.history.len() >= self.n && self.hits() >= self.m
    }

    /// Should the track be deleted? (window full but not enough hits).
    pub fn should_delete(&self) -> bool {
        self.history.len() >= self.n && self.hits() < self.m
    }
}

/// Track deletion logic for coasting tracks.
///
/// A coasting track is deleted after max_misses consecutive misses.
#[derive(Debug, Clone)]
pub struct CoastingLogic {
    pub max_misses: usize,
    pub consecutive_misses: usize,
}

impl CoastingLogic {
    pub fn new(max_misses: usize) -> Self {
        Self {
            max_misses,
            consecutive_misses: 0,
        }
    }

    pub fn record_hit(&mut self) {
        self.consecutive_misses = 0;
    }

    pub fn record_miss(&mut self) {
        self.consecutive_misses += 1;
    }

    pub fn should_delete(&self) -> bool {
        self.consecutive_misses >= self.max_misses
    }
}
