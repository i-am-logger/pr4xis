/// What a sensor measures — the physical quantity dimension.
///
/// Connects the sensor ontology to the quantity ontology.
/// Each sensor produces measurements of a specific physical dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeasurementDimension {
    /// Specific force / acceleration (m/s²).
    Acceleration,
    /// Angular rate (rad/s).
    AngularRate,
    /// Magnetic field (µT).
    MagneticField,
    /// Position (m).
    Position,
    /// Velocity (m/s).
    Velocity,
    /// Range / distance (m).
    Range,
    /// Bearing / angle (rad).
    Bearing,
    /// Pressure (Pa).
    Pressure,
    /// Temperature (K).
    Temperature,
    /// Intensity / power (W/m²).
    Intensity,
    /// Image (pixels).
    Image,
    /// Depth (m).
    Depth,
    /// Doppler velocity (m/s).
    DopplerVelocity,
}

/// Rich sensor characteristics — NOT just numbers, but bounded physical quantities.
///
/// Praxis principle: carry the structure in the type.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorCharacteristics {
    /// What physical quantity this sensor measures.
    pub measures: MeasurementDimension,
    /// Number of measurement axes (1, 2, or 3).
    pub axes: u8,
    /// Typical sample rate range (min_hz, max_hz).
    pub sample_rate_range: (f64, f64),
    /// Measurement range (min_value, max_value) in SI units.
    pub measurement_range: (f64, f64),
    /// Typical noise spectral density (SI units / √Hz).
    pub typical_noise_density: f64,
}
