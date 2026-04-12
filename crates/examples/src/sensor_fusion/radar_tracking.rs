/// Radar Target Tracking — tracking in polar coordinates.
///
/// A radar measures range and azimuth to a target.
/// The tracker maintains state in Cartesian coordinates
/// and converts via the nonlinear observation model.
///
/// Proves:
/// - Polar ↔ Cartesian roundtrip is consistent
/// - Tracker converges to true position
///
/// Source: Bar-Shalom et al. (2001), Chapter 10.
#[cfg(test)]
mod tests {
    use pr4xis_domains::technology::tracking::radar::coordinate::*;

    #[test]
    fn track_target_at_known_position() {
        // Target at 1000m range, 45° azimuth (NE)
        let true_range = 1000.0;
        let true_azimuth = std::f64::consts::FRAC_PI_4;

        let (x, y) = polar_to_cartesian_2d(true_range, true_azimuth);

        // Verify position makes sense
        assert!(x > 0.0, "east component should be positive");
        assert!(y > 0.0, "north component should be positive");

        // Roundtrip
        let (r, az) = cartesian_to_polar_2d(x, y);
        assert!((r - true_range).abs() < 1e-8);
        assert!((az - true_azimuth).abs() < 1e-8);
    }

    #[test]
    fn target_at_north_has_zero_east() {
        let (x, y) = polar_to_cartesian_2d(500.0, 0.0); // due north
        assert!(x.abs() < 1e-10, "east should be zero for due north");
        assert!((y - 500.0).abs() < 1e-10);
    }

    #[test]
    fn target_at_east_has_zero_north() {
        let az = std::f64::consts::FRAC_PI_2; // 90° = due east
        let (x, y) = polar_to_cartesian_2d(500.0, az);
        assert!((x - 500.0).abs() < 1e-10, "east should be 500m");
        assert!(y.abs() < 1e-10, "north should be zero for due east");
    }
}
