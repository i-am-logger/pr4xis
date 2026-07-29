#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::angle::Angle;
use crate::formal::math::geometry::point::Point2;
use crate::formal::math::geometry::vector::Vec2;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// A 2D pose in the SLAM graph.
#[derive(Debug, Clone)]
pub struct Pose2D {
    pub position: Point2,
    pub theta: Angle,
}

/// A 2D landmark position.
#[derive(Debug, Clone)]
pub struct Landmark2D {
    pub position: Point2,
}

/// An edge (constraint) in the pose graph.
#[derive(Debug, Clone)]
pub struct PoseGraphEdge {
    pub from_id: usize,
    pub to_id: usize,
    /// Relative pose measurement (dx, dy).
    pub delta: Vec2,
    pub dtheta: Angle,
    /// Information (inverse covariance) weight. Dimensionless (UNITLESS) —
    /// a normalised precision weight, the same dimensional treatment
    /// `total_error` already documents for the residual it scales.
    pub information_weight: Quantity,
}

/// A simple pose graph for 2D SLAM.
#[derive(Debug, Clone)]
pub struct PoseGraph {
    pub poses: Vec<Pose2D>,
    pub edges: Vec<PoseGraphEdge>,
}

impl Default for PoseGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl PoseGraph {
    pub fn new() -> Self {
        Self {
            poses: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a pose to the graph.
    pub fn add_pose(&mut self, pose: Pose2D) -> usize {
        let id = self.poses.len();
        self.poses.push(pose);
        id
    }

    /// Add an odometry edge between consecutive poses.
    pub fn add_odometry_edge(
        &mut self,
        from: usize,
        to: usize,
        delta: Vec2,
        dtheta: Angle,
        weight: Quantity,
    ) {
        self.edges.push(PoseGraphEdge {
            from_id: from,
            to_id: to,
            delta,
            dtheta,
            information_weight: weight,
        });
    }

    /// Add a loop closure edge.
    pub fn add_loop_closure(
        &mut self,
        from: usize,
        to: usize,
        delta: Vec2,
        dtheta: Angle,
        weight: Quantity,
    ) {
        // Loop closures are structurally the same as odometry edges,
        // but typically have higher information weight.
        self.edges.push(PoseGraphEdge {
            from_id: from,
            to_id: to,
            delta,
            dtheta,
            information_weight: weight,
        });
    }

    /// Compute total graph error (sum of squared weighted residuals).
    ///
    /// This is `F(x) = Σ e_ij^T Ω_ij e_ij` (Grisetti et al. 2010, "A
    /// Tutorial on Graph-Based SLAM", the NLLS graph cost the module's
    /// `information_weight` optimizes). Each residual `e_ij` mixes a
    /// translation term (`ex`, `ey`, meters) with a rotation term (`et`,
    /// radians — dimensionless in SI), so the weighted sum is not itself a
    /// length; it is the scale-free NLLS cost. Returns a dimensionless
    /// [`Quantity`] (`unit::UNITLESS`).
    pub fn total_error(&self) -> Quantity {
        let error: f64 = self
            .edges
            .iter()
            .map(|edge| {
                let pi = &self.poses[edge.from_id];
                let pj = &self.poses[edge.to_id];
                let cos_t = pi.theta.cos();
                let sin_t = pi.theta.sin();
                let dx = pj.position.x - pi.position.x;
                let dy = pj.position.y - pi.position.y;
                // Residual in local frame of pose i
                let dx_actual = cos_t * dx + sin_t * dy;
                let dy_actual = -sin_t * dx + cos_t * dy;
                let dtheta_actual = pj.theta.sub(&pi.theta);
                let ex = dx_actual - edge.delta.x;
                let ey = dy_actual - edge.delta.y;
                let et = dtheta_actual.sub(&edge.dtheta).radians();
                edge.information_weight.value * (ex * ex + ey * ey + et * et)
            })
            .sum();
        Quantity::from_unit(error, &unit::UNITLESS)
    }

    /// Number of constraints (edges) in the graph.
    ///
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`) — a
    /// cardinality, same as
    /// `formal::mereology::counting::ontology::cardinality`.
    pub fn num_constraints(&self) -> Quantity {
        Quantity::from_unit(self.edges.len() as f64, &unit::UNITLESS)
    }
}
