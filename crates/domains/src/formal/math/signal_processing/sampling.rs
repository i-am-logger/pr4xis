/// Nyquist-Shannon sampling theory.
///
/// Shannon, C.E. (1949). "Communication in the Presence of Noise." Proc. IRE.
/// Nyquist, H. (1928). "Certain Topics in Telegraph Transmission Theory." Trans. AIEE.
///
/// A bandlimited signal with maximum frequency f_max is completely
/// determined by its samples taken at rate f_s >= 2 * f_max.
/// Compute the Nyquist rate for a signal with the given bandwidth.
///
/// f_nyquist = 2 * bandwidth
pub fn nyquist_rate(bandwidth: f64) -> f64 {
    2.0 * bandwidth
}

/// Check whether a sample rate adequately captures a signal with the given bandwidth.
///
/// Adequate sampling requires f_s >= 2 * bandwidth (the Nyquist rate).
pub fn is_adequately_sampled(sample_rate: f64, bandwidth: f64) -> bool {
    sample_rate >= nyquist_rate(bandwidth)
}

/// Compute the alias frequency when sampling a signal of frequency f at sample rate f_s.
///
/// When f_s < 2*f, aliasing occurs. The perceived frequency folds back into [0, f_s/2].
/// The alias frequency is computed by folding f into the range [0, f_s/2].
pub fn alias_frequency(f: f64, f_s: f64) -> f64 {
    if f_s <= 0.0 {
        return f.abs();
    }
    // Fold f into [0, f_s] by taking modulo
    let f_abs = f.abs();
    let f_mod = f_abs % f_s;
    // If in the upper half [f_s/2, f_s], mirror it back
    if f_mod > f_s / 2.0 {
        f_s - f_mod
    } else {
        f_mod
    }
}
