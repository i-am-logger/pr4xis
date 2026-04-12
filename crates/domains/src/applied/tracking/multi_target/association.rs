use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::probability::mahalanobis;

/// Data association: matching detections to tracks.
///
/// The fundamental problem of multi-target tracking:
/// given N tracks and M detections, which detection belongs to which track?
///
/// Source: Bar-Shalom et al. (2001), Chapter 7.
/// Nearest-neighbor association: each detection goes to the closest track
/// within the validation gate.
///
/// Returns: Vec<(track_index, detection_index)> of associations.
/// Unassociated detections should initiate new tracks.
/// Unassociated tracks should coast (predict-only).
pub fn nearest_neighbor(
    track_predictions: &[(Vector, Matrix)], // (predicted measurement, innovation covariance S)
    detections: &[Vector],
    gate_threshold: f64,
) -> Vec<(usize, usize)> {
    let mut associations = Vec::new();
    let mut used_detections = vec![false; detections.len()];

    for (t_idx, (z_pred, s)) in track_predictions.iter().enumerate() {
        let mut best_dist = f64::INFINITY;
        let mut best_det = None;

        for (d_idx, z) in detections.iter().enumerate() {
            if used_detections[d_idx] {
                continue;
            }
            if let Some(d2) = mahalanobis::mahalanobis_squared(z, z_pred, s)
                && d2 < gate_threshold
                && d2 < best_dist
            {
                best_dist = d2;
                best_det = Some(d_idx);
            }
        }

        if let Some(d_idx) = best_det {
            associations.push((t_idx, d_idx));
            used_detections[d_idx] = true;
        }
    }

    associations
}

/// Get indices of unassociated detections (potential new tracks).
pub fn unassociated_detections(
    num_detections: usize,
    associations: &[(usize, usize)],
) -> Vec<usize> {
    let associated: std::collections::HashSet<usize> =
        associations.iter().map(|(_, d)| *d).collect();
    (0..num_detections)
        .filter(|d| !associated.contains(d))
        .collect()
}

/// Get indices of unassociated tracks (should coast).
pub fn unassociated_tracks(num_tracks: usize, associations: &[(usize, usize)]) -> Vec<usize> {
    let associated: std::collections::HashSet<usize> =
        associations.iter().map(|(t, _)| *t).collect();
    (0..num_tracks)
        .filter(|t| !associated.contains(t))
        .collect()
}
