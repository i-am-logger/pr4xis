use pr4xis::category::laws::assert_category_laws;
use pr4xis::logic::Axiom;
use pr4xis::ontology::Ontology;

use crate::applied::localization::slam::engine::*;
use crate::applied::localization::slam::ontology::*;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn slam_category_laws() {
    assert_category_laws::<SlamCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn slam_ontology_validates() {
    SlamOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn constraint_reduces_uncertainty_holds() {
    assert!(ConstraintReducesUncertainty.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn loop_closure_connects_poses_holds() {
    assert!(LoopClosureConnectsPoses.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn pose_graph_construction() {
    let mut graph = PoseGraph::new();
    let id0 = graph.add_pose(Pose2D {
        x: 0.0,
        y: 0.0,
        theta: 0.0,
    });
    let id1 = graph.add_pose(Pose2D {
        x: 1.0,
        y: 0.0,
        theta: 0.0,
    });
    graph.add_odometry_edge(id0, id1, 1.0, 0.0, 0.0, 1.0);
    assert_eq!(graph.num_constraints(), 1);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn perfect_odometry_has_zero_error() {
    let mut graph = PoseGraph::new();
    let id0 = graph.add_pose(Pose2D {
        x: 0.0,
        y: 0.0,
        theta: 0.0,
    });
    let id1 = graph.add_pose(Pose2D {
        x: 1.0,
        y: 0.0,
        theta: 0.0,
    });
    graph.add_odometry_edge(id0, id1, 1.0, 0.0, 0.0, 1.0);
    assert!(
        graph.total_error() < 1e-12,
        "perfect odometry should have zero error"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn loop_closure_adds_constraint() {
    let mut graph = PoseGraph::new();
    let id0 = graph.add_pose(Pose2D {
        x: 0.0,
        y: 0.0,
        theta: 0.0,
    });
    let id1 = graph.add_pose(Pose2D {
        x: 1.0,
        y: 0.0,
        theta: 0.0,
    });
    let id2 = graph.add_pose(Pose2D {
        x: 1.0,
        y: 1.0,
        theta: core::f64::consts::FRAC_PI_2,
    });
    graph.add_odometry_edge(id0, id1, 1.0, 0.0, 0.0, 1.0);
    graph.add_odometry_edge(id1, id2, 1.0, 0.0, core::f64::consts::FRAC_PI_2, 1.0);
    let n_before = graph.num_constraints();
    graph.add_loop_closure(id2, id0, -1.0, -1.0, -core::f64::consts::FRAC_PI_2, 2.0);
    assert_eq!(graph.num_constraints(), n_before + 1);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn perfect_odometry_zero_error(
            dx in -10.0..10.0_f64,
            dy in -10.0..10.0_f64
        ) {
            let mut graph = PoseGraph::new();
            let id0 = graph.add_pose(Pose2D { x: 0.0, y: 0.0, theta: 0.0 });
            let id1 = graph.add_pose(Pose2D { x: dx, y: dy, theta: 0.0 });
            graph.add_odometry_edge(id0, id1, dx, dy, 0.0, 1.0);
            prop_assert!(graph.total_error() < 1e-10,
                "perfect odometry should have zero error, got {}", graph.total_error());
        }

        #[test]
        fn adding_edge_increases_constraint_count(n in 1..10_usize) {
            let mut graph = PoseGraph::new();
            let mut ids = Vec::new();
            for i in 0..=n {
                ids.push(graph.add_pose(Pose2D { x: i as f64, y: 0.0, theta: 0.0 }));
            }
            for i in 0..n {
                graph.add_odometry_edge(ids[i], ids[i + 1], 1.0, 0.0, 0.0, 1.0);
            }
            prop_assert_eq!(graph.num_constraints(), n);
        }
    }

    pr4xis::register_praxis_value!(perfect_odometry_zero_error, Verifiable);
    pr4xis::register_praxis_value!(adding_edge_increases_constraint_count, Verifiable);
}
