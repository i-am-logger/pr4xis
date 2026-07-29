use pr4xis::category::laws::assert_category_laws;
use pr4xis::logic::Axiom;
use pr4xis::ontology::Ontology;

use crate::applied::localization::slam::engine::*;
use crate::applied::localization::slam::ontology::*;
use crate::formal::math::angle::Angle;
use crate::formal::math::geometry::point::Point2;
use crate::formal::math::geometry::vector::Vec2;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

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
        position: Point2::new(0.0, 0.0),
        theta: Angle::ZERO,
    });
    let id1 = graph.add_pose(Pose2D {
        position: Point2::new(1.0, 0.0),
        theta: Angle::ZERO,
    });
    graph.add_odometry_edge(
        id0,
        id1,
        Vec2::new(1.0, 0.0),
        Angle::ZERO,
        Quantity::from_unit(1.0, &unit::UNITLESS),
    );
    assert_eq!(graph.num_constraints().value, 1.0);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn perfect_odometry_has_zero_error() {
    let mut graph = PoseGraph::new();
    let id0 = graph.add_pose(Pose2D {
        position: Point2::new(0.0, 0.0),
        theta: Angle::ZERO,
    });
    let id1 = graph.add_pose(Pose2D {
        position: Point2::new(1.0, 0.0),
        theta: Angle::ZERO,
    });
    graph.add_odometry_edge(
        id0,
        id1,
        Vec2::new(1.0, 0.0),
        Angle::ZERO,
        Quantity::from_unit(1.0, &unit::UNITLESS),
    );
    assert!(
        graph.total_error().value < 1e-12,
        "perfect odometry should have zero error"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn loop_closure_adds_constraint() {
    let mut graph = PoseGraph::new();
    let id0 = graph.add_pose(Pose2D {
        position: Point2::new(0.0, 0.0),
        theta: Angle::ZERO,
    });
    let id1 = graph.add_pose(Pose2D {
        position: Point2::new(1.0, 0.0),
        theta: Angle::ZERO,
    });
    let id2 = graph.add_pose(Pose2D {
        position: Point2::new(1.0, 1.0),
        theta: Angle::from_radians(core::f64::consts::FRAC_PI_2),
    });
    graph.add_odometry_edge(
        id0,
        id1,
        Vec2::new(1.0, 0.0),
        Angle::ZERO,
        Quantity::from_unit(1.0, &unit::UNITLESS),
    );
    graph.add_odometry_edge(
        id1,
        id2,
        Vec2::new(1.0, 0.0),
        Angle::from_radians(core::f64::consts::FRAC_PI_2),
        Quantity::from_unit(1.0, &unit::UNITLESS),
    );
    let n_before = graph.num_constraints().value;
    graph.add_loop_closure(
        id2,
        id0,
        Vec2::new(-1.0, -1.0),
        Angle::from_radians(-core::f64::consts::FRAC_PI_2),
        Quantity::from_unit(2.0, &unit::UNITLESS),
    );
    assert_eq!(graph.num_constraints().value, n_before + 1.0);
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
            let id0 = graph.add_pose(Pose2D { position: Point2::new(0.0, 0.0), theta: Angle::ZERO });
            let id1 = graph.add_pose(Pose2D { position: Point2::new(dx, dy), theta: Angle::ZERO });
            graph.add_odometry_edge(
                id0,
                id1,
                Vec2::new(dx, dy),
                Angle::ZERO,
                Quantity::from_unit(1.0, &unit::UNITLESS),
            );
            prop_assert!(graph.total_error().value < 1e-10,
                "perfect odometry should have zero error, got {}", graph.total_error().value);
        }

        #[test]
        fn adding_edge_increases_constraint_count(n in 1..10_usize) {
            let mut graph = PoseGraph::new();
            let mut ids = Vec::new();
            for i in 0..=n {
                ids.push(graph.add_pose(Pose2D { position: Point2::new(i as f64, 0.0), theta: Angle::ZERO }));
            }
            for i in 0..n {
                graph.add_odometry_edge(
                    ids[i],
                    ids[i + 1],
                    Vec2::new(1.0, 0.0),
                    Angle::ZERO,
                    Quantity::from_unit(1.0, &unit::UNITLESS),
                );
            }
            prop_assert_eq!(graph.num_constraints().value, n as f64);
        }
    }

    pr4xis::register_praxis_value!(perfect_odometry_zero_error, Verifiable);
    pr4xis::register_praxis_value!(adding_edge_increases_constraint_count, Verifiable);
}
