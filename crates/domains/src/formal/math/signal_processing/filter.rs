/// Digital filter types and first-order implementations.
///
/// Oppenheim & Willsky (1997). *Signals and Systems* (2nd ed.).
/// Classification of filter types by their frequency response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    BandStop,
    AllPass,
}

/// Characteristics of a digital filter.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterSpec {
    /// Filter type.
    pub filter_type: FilterType,
    /// Cutoff frequency (Hz). For bandpass/bandstop, this is the lower cutoff.
    pub cutoff_frequency: f64,
    /// Filter order (number of poles).
    pub order: usize,
    /// Maximum passband ripple (dB). 0 means ideal (no ripple).
    pub passband_ripple_db: f64,
}

impl FilterSpec {
    /// Create a new filter specification.
    pub fn new(filter_type: FilterType, cutoff_frequency: f64, order: usize) -> Self {
        Self {
            filter_type,
            cutoff_frequency,
            order,
            passband_ripple_db: 0.0,
        }
    }
}

/// A first-order IIR low-pass filter (exponential moving average).
///
/// Difference equation: y[n] = α * x[n] + (1 - α) * y[n-1]
/// where α = dt / (RC + dt), and RC = 1 / (2π * f_cutoff).
#[derive(Debug, Clone, PartialEq)]
pub struct FirstOrderLowPass {
    /// Smoothing factor α ∈ (0, 1].
    pub alpha: f64,
    /// Previous output value.
    pub prev_output: f64,
}

impl FirstOrderLowPass {
    /// Create a first-order low-pass filter from cutoff frequency and sample period.
    ///
    /// α = dt / (RC + dt) where RC = 1 / (2π * f_cutoff).
    pub fn new(cutoff_freq: f64, sample_period: f64) -> Self {
        let rc = 1.0 / (2.0 * std::f64::consts::PI * cutoff_freq);
        let alpha = sample_period / (rc + sample_period);
        Self {
            alpha,
            prev_output: 0.0,
        }
    }

    /// Create from a raw alpha value.
    pub fn from_alpha(alpha: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
            prev_output: 0.0,
        }
    }

    /// Process a single sample through the filter.
    pub fn update(&mut self, input: f64) -> f64 {
        let output = self.alpha * input + (1.0 - self.alpha) * self.prev_output;
        self.prev_output = output;
        output
    }

    /// Process a sequence of samples, returning filtered outputs.
    pub fn filter(&mut self, inputs: &[f64]) -> Vec<f64> {
        inputs.iter().map(|&x| self.update(x)).collect()
    }

    /// Reset the filter state.
    pub fn reset(&mut self) {
        self.prev_output = 0.0;
    }
}
