#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::angle::Angle;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::duration::Duration;

/// Object class label for a camera detection.
///
/// Source: Geiger, Lenz & Urtasun (2012), "Are We Ready for Autonomous
/// Driving? The KITTI Vision Benchmark Suite", CVPR — Table 1 defines the
/// eight object categories annotated for 2D/3D detection: Car, Van, Truck,
/// Pedestrian, Person (sitting), Cyclist, Tram, Misc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectClass {
    Car,
    Van,
    Truck,
    Pedestrian,
    PersonSitting,
    Cyclist,
    Tram,
    Misc,
}

/// A radar detection with range, Doppler velocity, and azimuth.
#[derive(Debug, Clone)]
pub struct RadarTarget {
    pub range: Quantity,
    pub doppler: Quantity,
    /// Bearing of the target relative to boresight (circle group S¹).
    pub azimuth: Angle,
    pub rcs: f64, // radar cross section (dBsm)
}

/// A camera object detection.
#[derive(Debug, Clone)]
pub struct CameraObject {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
    pub class_label: ObjectClass,
    pub confidence: f64,
}

/// Temporally aligned pair of radar and camera frames.
#[derive(Debug, Clone)]
pub struct AlignedFrame {
    pub radar_targets: Vec<RadarTarget>,
    pub camera_objects: Vec<CameraObject>,
    pub time_offset_s: Duration,
}

/// Fused radar-camera detection.
#[derive(Debug, Clone)]
pub struct FusedRadarCameraDetection {
    pub range: Quantity,
    pub doppler: Quantity,
    /// Bearing carried through from the radar target (circle group S¹).
    pub azimuth: Angle,
    pub class_label: ObjectClass,
    pub confidence: f64,
}

/// Project radar target to image column given sensor geometry.
///
/// Simplified model: azimuth maps linearly to image x-coordinate.
///
/// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`) — an image-space
/// pixel coordinate is a discrete count/position in pixel space, not an SI
/// unit, same as `formal::mereology::counting::ontology::cardinality`.
pub fn radar_azimuth_to_image_x(azimuth: Angle, image_width: f64, fov_rad: Angle) -> Quantity {
    Quantity::from_unit(
        (azimuth.radians() / fov_rad.radians() + 0.5) * image_width,
        &unit::UNITLESS,
    )
}

/// Associate radar targets with camera detections by azimuth-to-image projection.
pub fn associate_radar_camera(
    frame: &AlignedFrame,
    image_width: f64,
    fov_rad: Angle,
) -> Vec<FusedRadarCameraDetection> {
    let mut fused = Vec::new();
    for target in &frame.radar_targets {
        let proj_x = radar_azimuth_to_image_x(target.azimuth, image_width, fov_rad).value;
        // Find best matching camera detection
        if let Some(best) = frame
            .camera_objects
            .iter()
            .filter(|obj| proj_x >= obj.x_min && proj_x <= obj.x_max)
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
        {
            fused.push(FusedRadarCameraDetection {
                range: target.range.clone(),
                doppler: target.doppler.clone(),
                azimuth: target.azimuth,
                class_label: best.class_label,
                confidence: best.confidence,
            });
        }
    }
    fused
}
