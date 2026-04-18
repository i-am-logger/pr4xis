#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;

/// Reference frames used in sensor fusion systems.
///
/// Each frame defines a coordinate system with an origin and orientation.
/// Frames are the objects in the FrameCategory — transforms between them
/// are the morphisms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum ReferenceFrame {
    /// Earth-Centered Earth-Fixed: origin at Earth's center, rotates with Earth
    ECEF,
    /// Earth-Centered Inertial: origin at Earth's center, fixed to stars
    ECI,
    /// North-East-Down: local tangent plane, origin at a geodetic reference point
    NED,
    /// East-North-Up: local tangent plane (right-handed, Z up)
    ENU,
    /// Body: fixed to the vehicle (forward-right-down or forward-left-up)
    Body,
    /// IMU: aligned with the inertial measurement unit measurement axes
    IMU,
    /// Camera: optical frame (right-down-forward, Z along optical axis)
    Camera,
    /// LiDAR: sensor frame (typically forward-left-up)
    LiDAR,
    /// Radar: sensor frame (boresight along principal axis)
    Radar,
    /// GNSS: antenna phase center
    GNSS,
}
