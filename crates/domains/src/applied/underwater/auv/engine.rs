/// AUV navigation state and dead reckoning.
///
/// Source: Kinsey et al. (2006), "A Survey of Underwater Vehicle Navigation"
/// AUV navigation state (2D + depth).
#[derive(Debug, Clone)]
pub struct AuvState {
    /// North position (meters).
    pub north: f64,
    /// East position (meters).
    pub east: f64,
    /// Depth (meters, positive downward).
    pub depth: f64,
    /// Heading (radians, from north clockwise).
    pub heading: f64,
}

/// DVL velocity measurement in body frame.
#[derive(Debug, Clone)]
pub struct DvlMeasurement {
    /// Forward velocity (m/s).
    pub forward: f64,
    /// Starboard velocity (m/s).
    pub starboard: f64,
    /// Downward velocity (m/s).
    pub downward: f64,
    /// Whether bottom lock is achieved.
    pub bottom_lock: bool,
}

/// Depth sensor measurement.
#[derive(Debug, Clone)]
pub struct DepthMeasurement {
    /// Measured depth in meters.
    pub depth: f64,
}

/// Dead reckoning: propagate AUV state using DVL and compass.
pub fn dead_reckon(state: &AuvState, dvl: &DvlMeasurement, heading: f64, dt: f64) -> AuvState {
    let cos_h = heading.cos();
    let sin_h = heading.sin();
    // Transform body-frame velocity to world frame
    let v_north = dvl.forward * cos_h - dvl.starboard * sin_h;
    let v_east = dvl.forward * sin_h + dvl.starboard * cos_h;

    AuvState {
        north: state.north + v_north * dt,
        east: state.east + v_east * dt,
        depth: state.depth + dvl.downward * dt,
        heading,
    }
}

/// Compute distance traveled between two states.
pub fn distance_2d(a: &AuvState, b: &AuvState) -> f64 {
    let dn = b.north - a.north;
    let de = b.east - a.east;
    (dn * dn + de * de).sqrt()
}

/// Compute 3D distance between two states.
pub fn distance_3d(a: &AuvState, b: &AuvState) -> f64 {
    let dn = b.north - a.north;
    let de = b.east - a.east;
    let dd = b.depth - a.depth;
    (dn * dn + de * de + dd * dd).sqrt()
}
