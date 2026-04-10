/// A radar detection with range, Doppler velocity, and azimuth.
#[derive(Debug, Clone)]
pub struct RadarTarget {
    pub range: f64,
    pub doppler: f64,
    pub azimuth: f64,
    pub rcs: f64, // radar cross section (dBsm)
}

/// A camera object detection.
#[derive(Debug, Clone)]
pub struct CameraObject {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
    pub class_label: &'static str,
    pub confidence: f64,
}

/// Temporally aligned pair of radar and camera frames.
#[derive(Debug, Clone)]
pub struct AlignedFrame {
    pub radar_targets: Vec<RadarTarget>,
    pub camera_objects: Vec<CameraObject>,
    pub time_offset_s: f64,
}

/// Fused radar-camera detection.
#[derive(Debug, Clone)]
pub struct FusedRadarCameraDetection {
    pub range: f64,
    pub doppler: f64,
    pub azimuth: f64,
    pub class_label: &'static str,
    pub confidence: f64,
}

/// Project radar target to image column given sensor geometry.
///
/// Simplified model: azimuth maps linearly to image x-coordinate.
pub fn radar_azimuth_to_image_x(azimuth: f64, image_width: f64, fov_rad: f64) -> f64 {
    (azimuth / fov_rad + 0.5) * image_width
}

/// Associate radar targets with camera detections by azimuth-to-image projection.
pub fn associate_radar_camera(
    frame: &AlignedFrame,
    image_width: f64,
    fov_rad: f64,
) -> Vec<FusedRadarCameraDetection> {
    let mut fused = Vec::new();
    for target in &frame.radar_targets {
        let proj_x = radar_azimuth_to_image_x(target.azimuth, image_width, fov_rad);
        // Find best matching camera detection
        if let Some(best) = frame
            .camera_objects
            .iter()
            .filter(|obj| proj_x >= obj.x_min && proj_x <= obj.x_max)
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            fused.push(FusedRadarCameraDetection {
                range: target.range,
                doppler: target.doppler,
                azimuth: target.azimuth,
                class_label: best.class_label,
                confidence: best.confidence,
            });
        }
    }
    fused
}
