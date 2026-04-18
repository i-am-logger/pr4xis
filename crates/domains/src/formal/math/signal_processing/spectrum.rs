#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Frequency domain concepts for spectral analysis.
///
/// Oppenheim & Willsky (1997). *Signals and Systems* (2nd ed.).
///
/// The DFT of N samples taken at rate f_s gives frequency resolution Δf = f_s / N.
/// Bandwidth, center frequency, and spectral density characterize a signal in the
/// frequency domain.
/// A spectral band defined by lower and upper frequency bounds.
#[derive(Debug, Clone, PartialEq)]
pub struct SpectralBand {
    /// Lower frequency bound (Hz).
    pub f_low: f64,
    /// Upper frequency bound (Hz).
    pub f_high: f64,
}

impl SpectralBand {
    /// Create a new spectral band. Requires f_low >= 0 and f_high > f_low.
    pub fn new(f_low: f64, f_high: f64) -> Option<Self> {
        if f_low < 0.0 || f_high <= f_low {
            return None;
        }
        Some(Self { f_low, f_high })
    }

    /// Bandwidth of the spectral band: f_high - f_low.
    pub fn bandwidth(&self) -> f64 {
        self.f_high - self.f_low
    }

    /// Center frequency: (f_low + f_high) / 2.
    pub fn center_frequency(&self) -> f64 {
        (self.f_low + self.f_high) / 2.0
    }

    /// Whether a frequency falls within this band.
    pub fn contains(&self, f: f64) -> bool {
        f >= self.f_low && f <= self.f_high
    }
}

/// Frequency resolution of a DFT: Δf = f_s / N.
///
/// N samples taken at sample rate f_s yield N frequency bins
/// with spacing Δf = f_s / N.
pub fn frequency_resolution(sample_rate: f64, num_samples: usize) -> f64 {
    if num_samples == 0 {
        return 0.0;
    }
    sample_rate / num_samples as f64
}

/// Maximum resolvable frequency (Nyquist frequency) = f_s / 2.
pub fn max_resolvable_frequency(sample_rate: f64) -> f64 {
    sample_rate / 2.0
}

/// Number of unique frequency bins in a DFT of N samples (N/2 + 1 for real signals).
pub fn num_frequency_bins(num_samples: usize) -> usize {
    num_samples / 2 + 1
}
