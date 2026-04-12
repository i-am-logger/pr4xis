/// A 2D pose in the SLAM graph.
#[derive(Debug, Clone)]
pub struct Pose2D {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

/// A 2D landmark position.
#[derive(Debug, Clone)]
pub struct Landmark2D {
    pub x: f64,
    pub y: f64,
}

/// An edge (constraint) in the pose graph.
#[derive(Debug, Clone)]
pub struct PoseGraphEdge {
    pub from_id: usize,
    pub to_id: usize,
    /// Relative pose measurement (dx, dy, dtheta).
    pub dx: f64,
    pub dy: f64,
    pub dtheta: f64,
    /// Information (inverse covariance) weight.
    pub information_weight: f64,
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
        dx: f64,
        dy: f64,
        dtheta: f64,
        weight: f64,
    ) {
        self.edges.push(PoseGraphEdge {
            from_id: from,
            to_id: to,
            dx,
            dy,
            dtheta,
            information_weight: weight,
        });
    }

    /// Add a loop closure edge.
    pub fn add_loop_closure(
        &mut self,
        from: usize,
        to: usize,
        dx: f64,
        dy: f64,
        dtheta: f64,
        weight: f64,
    ) {
        // Loop closures are structurally the same as odometry edges,
        // but typically have higher information weight.
        self.edges.push(PoseGraphEdge {
            from_id: from,
            to_id: to,
            dx,
            dy,
            dtheta,
            information_weight: weight,
        });
    }

    /// Compute total graph error (sum of squared weighted residuals).
    pub fn total_error(&self) -> f64 {
        self.edges
            .iter()
            .map(|edge| {
                let pi = &self.poses[edge.from_id];
                let pj = &self.poses[edge.to_id];
                let cos_t = pi.theta.cos();
                let sin_t = pi.theta.sin();
                // Residual in local frame of pose i
                let dx_actual = cos_t * (pj.x - pi.x) + sin_t * (pj.y - pi.y);
                let dy_actual = -sin_t * (pj.x - pi.x) + cos_t * (pj.y - pi.y);
                let dtheta_actual = pj.theta - pi.theta;
                let ex = dx_actual - edge.dx;
                let ey = dy_actual - edge.dy;
                let et = dtheta_actual - edge.dtheta;
                edge.information_weight * (ex * ex + ey * ey + et * et)
            })
            .sum()
    }

    /// Number of constraints (edges) in the graph.
    pub fn num_constraints(&self) -> usize {
        self.edges.len()
    }
}
