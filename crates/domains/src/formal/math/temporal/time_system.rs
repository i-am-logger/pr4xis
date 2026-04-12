use pr4xis::category::Entity;

/// Time systems used in sensor fusion.
///
/// Each system has its own epoch and tick rate.
/// Conversions between systems are known offsets (a category of time systems
/// where morphisms are the offset functions).
///
/// Sources:
///   - IAU 2000/2006 resolutions on time scales
///   - GPS Interface Control Document (IS-GPS-200)
///   - ITU-R TF.460 (UTC definition)
///   - NIST Special Publication on time and frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum TimeSystem {
    /// International Atomic Time. Continuous, no leap seconds.
    /// Epoch: 1958-01-01T00:00:00.
    /// Source: Bureau International des Poids et Mesures (BIPM).
    TAI,

    /// Coordinated Universal Time. TAI with leap seconds.
    /// UTC = TAI - ΔAT (where ΔAT is the accumulated leap seconds).
    /// Source: ITU-R TF.460.
    UTC,

    /// GPS Time. Continuous atomic time, no leap seconds.
    /// Epoch: 1980-01-06T00:00:00 UTC.
    /// GPS = TAI - 19 seconds (fixed offset).
    /// Source: IS-GPS-200.
    GPS,

    /// Terrestrial Time. Idealized time on the geoid.
    /// TT = TAI + 32.184 seconds (fixed offset).
    /// Source: IAU 2000 Resolution B1.9.
    TT,

    /// Barycentric Coordinate Time. Solar system barycenter.
    /// Differs from TT by relativistic corrections.
    /// Source: IAU 2006 Resolution B3.
    TCB,

    /// Mission Elapsed Time. Seconds since mission start.
    /// Common in spacecraft operations.
    MET,

    /// Unix time. Seconds since 1970-01-01T00:00:00 UTC.
    /// Ignores leap seconds (not monotonic across leap second events).
    Unix,
}

/// Known fixed offsets between time systems (seconds).
/// TAI is the reference.
pub fn offset_from_tai(system: TimeSystem) -> Option<f64> {
    match system {
        TimeSystem::TAI => Some(0.0),
        TimeSystem::GPS => Some(-19.0), // GPS = TAI - 19
        TimeSystem::TT => Some(32.184), // TT = TAI + 32.184
        // UTC offset depends on current leap second count (not fixed)
        // TCB depends on relativistic model
        // MET and Unix are mission/epoch-specific
        _ => None,
    }
}

/// Convert between time systems with fixed offsets.
/// Returns None if conversion requires dynamic data (e.g., UTC leap seconds).
pub fn convert(seconds_in_source: f64, from: TimeSystem, to: TimeSystem) -> Option<f64> {
    let from_tai = offset_from_tai(from)?;
    let to_tai = offset_from_tai(to)?;
    // source_tai = seconds_in_source + from_tai (to get TAI)
    // but actually: if GPS = TAI - 19, then TAI = GPS + 19
    // So: seconds_in_TAI = seconds_in_source - offset_from_tai(source)
    //     seconds_in_target = seconds_in_TAI + offset_from_tai(target)
    // Wait, let me think about this more carefully.
    // offset_from_tai(GPS) = -19 means GPS = TAI - 19, so TAI = GPS + 19
    // In general: system_time = TAI + offset, so TAI = system_time - offset
    let tai = seconds_in_source - from_tai;
    Some(tai + to_tai)
}
