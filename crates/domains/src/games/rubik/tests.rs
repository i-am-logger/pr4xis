use super::*;
use proptest::prelude::*;

// =============================================================================
// Proptest strategies
// =============================================================================

fn arb_move() -> impl Strategy<Value = Move> {
    (0..18usize).prop_map(|i| Move::all()[i])
}

fn arb_move_sequence() -> impl Strategy<Value = Vec<Move>> {
    proptest::collection::vec(arb_move(), 0..20)
}

// =============================================================================
// Solved state tests
// =============================================================================

#[test]
fn test_solved_is_solved() {
    assert!(Cube::solved().is_solved());
}

#[test]
fn test_solved_has_9_of_each_color() {
    let counts = Cube::solved().color_counts();
    for c in counts {
        assert_eq!(c, 9);
    }
}

#[test]
fn test_single_move_unsolved() {
    let cube = Cube::solved().apply(Move::R);
    assert!(!cube.is_solved());
}

// =============================================================================
// Group theory enforcement — the ontology guarantees
// =============================================================================

#[test]
fn test_move_then_inverse_is_identity() {
    for m in Move::all() {
        let cube = Cube::solved().apply(m).apply(m.inverse());
        assert!(
            cube.is_solved(),
            "{} then {} should restore solved state",
            m.notation(),
            m.inverse().notation()
        );
    }
}

#[test]
fn test_four_cw_rotations_is_identity() {
    for m in [Move::U, Move::D, Move::F, Move::B, Move::L, Move::R] {
        let cube = Cube::solved().apply(m).apply(m).apply(m).apply(m);
        assert!(
            cube.is_solved(),
            "4× {} should restore solved state",
            m.notation()
        );
    }
}

#[test]
fn test_double_move_is_two_singles() {
    let cube_double = Cube::solved().apply(Move::R2);
    let cube_two = Cube::solved().apply(Move::R).apply(Move::R);
    assert_eq!(cube_double, cube_two);
}

#[test]
fn test_prime_is_three_cw() {
    let cube_prime = Cube::solved().apply(Move::Ri);
    let cube_three = Cube::solved().apply(Move::R).apply(Move::R).apply(Move::R);
    assert_eq!(cube_prime, cube_three);
}

#[test]
fn test_18_valid_moves() {
    assert_eq!(Move::all().len(), 18);
}

#[test]
fn test_inverse_of_inverse_is_self() {
    for m in Move::all() {
        assert_eq!(m.inverse().inverse(), m);
    }
}

#[test]
fn test_double_move_is_self_inverse() {
    for m in [Move::U2, Move::D2, Move::F2, Move::B2, Move::L2, Move::R2] {
        assert_eq!(m.inverse(), m, "{} should be its own inverse", m.notation());
    }
}

// =============================================================================
// Property-based tests — ontology enforcement
// =============================================================================

proptest! {
    /// Every move preserves the color count (9 of each, always)
    #[test]
    fn prop_move_preserves_colors(m in arb_move()) {
        let cube = Cube::solved().apply(m);
        let counts = cube.color_counts();
        for (i, &c) in counts.iter().enumerate() {
            prop_assert_eq!(c, 9, "color {} has {} stickers after {}", i, c, m.notation());
        }
    }

    /// Any sequence of moves preserves color counts
    #[test]
    fn prop_sequence_preserves_colors(moves in arb_move_sequence()) {
        let cube = Cube::solved().apply_sequence(&moves);
        let counts = cube.color_counts();
        for (i, &c) in counts.iter().enumerate() {
            prop_assert_eq!(c, 9, "color {} has {} stickers after {} moves", i, c, moves.len());
        }
    }

    /// Move then inverse always restores original state
    #[test]
    fn prop_move_inverse_identity(m in arb_move()) {
        let original = Cube::solved();
        let restored = original.apply(m).apply(m.inverse());
        prop_assert_eq!(original, restored);
    }

    /// Inverse of any sequence restores original
    #[test]
    fn prop_sequence_inverse_identity(moves in proptest::collection::vec(arb_move(), 1..10)) {
        let original = Cube::solved();
        let scrambled = original.apply_sequence(&moves);
        let inverse_seq: Vec<Move> = moves.iter().rev().map(|m| m.inverse()).collect();
        let restored = scrambled.apply_sequence(&inverse_seq);
        prop_assert_eq!(original, restored);
    }

    /// 4 CW rotations of any face = identity
    #[test]
    fn prop_four_rotations_identity(face_idx in 0..6usize) {
        let m = [Move::U, Move::D, Move::F, Move::B, Move::L, Move::R][face_idx];
        let cube = Cube::solved().apply(m).apply(m).apply(m).apply(m);
        prop_assert!(cube.is_solved());
    }

    /// Double move = two single moves
    #[test]
    fn prop_double_is_two_singles(face_idx in 0..6usize) {
        let single = [Move::U, Move::D, Move::F, Move::B, Move::L, Move::R][face_idx];
        let double = [Move::U2, Move::D2, Move::F2, Move::B2, Move::L2, Move::R2][face_idx];
        let cube_double = Cube::solved().apply(double);
        let cube_two = Cube::solved().apply(single).apply(single);
        prop_assert_eq!(cube_double, cube_two);
    }

    /// Prime = three CW rotations
    #[test]
    fn prop_prime_is_three_cw(face_idx in 0..6usize) {
        let cw = [Move::U, Move::D, Move::F, Move::B, Move::L, Move::R][face_idx];
        let prime = [Move::Ui, Move::Di, Move::Fi, Move::Bi, Move::Li, Move::Ri][face_idx];
        let cube_prime = Cube::solved().apply(prime);
        let cube_three = Cube::solved().apply(cw).apply(cw).apply(cw);
        prop_assert_eq!(cube_prime, cube_three);
    }

    /// Center stickers never move (fundamental Rubik's invariant)
    #[test]
    fn prop_centers_fixed(moves in arb_move_sequence()) {
        let cube = Cube::solved().apply_sequence(&moves);
        for face in Face::all() {
            let center = cube.get(face, 4); // position 4 = center
            let expected = Color::of_face(face);
            prop_assert_eq!(center, expected,
                "center of {:?} moved from {:?} to {:?} after {} moves",
                face, expected, center, moves.len());
        }
    }

    /// Inverse of inverse = original move
    #[test]
    fn prop_double_inverse(m in arb_move()) {
        prop_assert_eq!(m.inverse().inverse(), m);
    }

    /// Applying a scramble to two identical cubes gives identical results (deterministic)
    #[test]
    fn prop_deterministic(moves in arb_move_sequence()) {
        let a = Cube::solved().apply_sequence(&moves);
        let b = Cube::solved().apply_sequence(&moves);
        prop_assert_eq!(a, b);
    }

    /// Total stickers is always 54 (6 faces × 9 stickers)
    #[test]
    fn prop_54_stickers(moves in arb_move_sequence()) {
        let cube = Cube::solved().apply_sequence(&moves);
        let total: u8 = cube.color_counts().iter().sum();
        prop_assert_eq!(total, 54);
    }

    /// Each color always has exactly 9 stickers
    #[test]
    fn prop_9_per_color(moves in arb_move_sequence()) {
        let cube = Cube::solved().apply_sequence(&moves);
        for (i, &count) in cube.color_counts().iter().enumerate() {
            prop_assert_eq!(count, 9, "color {} has {} stickers", i, count);
        }
    }

    /// A single move always changes the cube (never identity)
    #[test]
    fn prop_single_move_changes(m in arb_move()) {
        let solved = Cube::solved();
        let after = solved.apply(m);
        prop_assert_ne!(solved, after, "{} should change the cube", m.notation());
    }

    /// Opposite faces don't affect each other: U doesn't move D stickers, etc.
    #[test]
    fn prop_opposite_face_independent(face_idx in 0..6usize) {
        let m = [Move::U, Move::D, Move::F, Move::B, Move::L, Move::R][face_idx];
        let opposite = m.face().opposite();
        let solved = Cube::solved();
        let after = solved.apply(m);
        // The opposite face should be unchanged
        prop_assert_eq!(
            solved.face_colors(opposite),
            after.face_colors(opposite),
            "rotating {:?} changed opposite face {:?}", m.face(), opposite
        );
    }

    /// The rotated face's center stays the same (redundant with centers_fixed but specific)
    #[test]
    fn prop_rotated_face_center_stays(m in arb_move()) {
        let solved = Cube::solved();
        let after = solved.apply(m);
        let face = m.face();
        prop_assert_eq!(solved.get(face, 4), after.get(face, 4),
            "{} should not move center of {:?}", m.notation(), face);
    }

    /// Applying the same move 2 times then its inverse 2 times = identity
    #[test]
    fn prop_double_then_double_inverse(m in arb_move()) {
        let cube = Cube::solved().apply(m).apply(m).apply(m.inverse()).apply(m.inverse());
        prop_assert!(cube.is_solved());
    }

    /// Order doesn't matter for moves on opposite faces (they commute)
    #[test]
    fn prop_opposite_faces_commute(face_idx in 0..3usize) {
        let pairs = [(Move::U, Move::D), (Move::F, Move::B), (Move::L, Move::R)];
        let (a, b) = pairs[face_idx];
        let ab = Cube::solved().apply(a).apply(b);
        let ba = Cube::solved().apply(b).apply(a);
        prop_assert_eq!(ab, ba, "{} and {} should commute", a.notation(), b.notation());
    }

    /// Superflip: applying 20 specific moves produces a non-solved state
    /// (less specific — just test that long sequences don't corrupt)
    #[test]
    fn prop_long_sequence_valid(moves in proptest::collection::vec(arb_move(), 50..100)) {
        let cube = Cube::solved().apply_sequence(&moves);
        let counts = cube.color_counts();
        for &c in &counts {
            prop_assert_eq!(c, 9);
        }
        // Centers still fixed
        for face in Face::all() {
            prop_assert_eq!(cube.get(face, 4), Color::of_face(face));
        }
    }

    /// Inverse of a double move is itself
    #[test]
    fn prop_double_self_inverse(face_idx in 0..6usize) {
        let double = [Move::U2, Move::D2, Move::F2, Move::B2, Move::L2, Move::R2][face_idx];
        prop_assert_eq!(double.inverse(), double);
    }

    /// CW inverse is CCW, CCW inverse is CW
    #[test]
    fn prop_cw_ccw_are_inverses(face_idx in 0..6usize) {
        let cw = [Move::U, Move::D, Move::F, Move::B, Move::L, Move::R][face_idx];
        let ccw = [Move::Ui, Move::Di, Move::Fi, Move::Bi, Move::Li, Move::Ri][face_idx];
        prop_assert_eq!(cw.inverse(), ccw);
        prop_assert_eq!(ccw.inverse(), cw);
    }

    /// 6 half-turn moves applied to solved cube: each face is 180° rotated
    #[test]
    fn prop_half_turn_is_involution(face_idx in 0..6usize) {
        let double = [Move::U2, Move::D2, Move::F2, Move::B2, Move::L2, Move::R2][face_idx];
        let cube = Cube::solved().apply(double).apply(double);
        prop_assert!(cube.is_solved(), "double {} twice should be identity", double.notation());
    }

    /// Scramble then reverse-unscramble from any starting state
    #[test]
    fn prop_reverse_from_scrambled(
        initial_scramble in proptest::collection::vec(arb_move(), 1..10),
        extra_moves in proptest::collection::vec(arb_move(), 1..10),
    ) {
        let start = Cube::solved().apply_sequence(&initial_scramble);
        let scrambled = start.apply_sequence(&extra_moves);
        let inverse_seq: Vec<Move> = extra_moves.iter().rev().map(|m| m.inverse()).collect();
        let restored = scrambled.apply_sequence(&inverse_seq);
        prop_assert_eq!(start, restored);
    }
}

// =============================================================================
// Engine tests — Situation/Action/Precondition/Trace
// =============================================================================

use praxis::engine::*;

#[test]
fn engine_scramble_and_undo() {
    let e = new_cube();
    assert!(e.situation().is_solved());
    let e = e.try_next(RubikAction(Move::R)).unwrap();
    assert!(!e.situation().is_solved());
    let e = e.back().unwrap();
    assert!(e.situation().is_solved());
}

#[test]
fn engine_sequence_preserves_invariant() {
    let mut e = new_cube();
    let moves = [Move::R, Move::U, Move::F, Move::L, Move::D, Move::B];
    for m in moves {
        e = e.try_next(RubikAction(m)).unwrap();
    }
    assert_eq!(e.step(), 6);
    // Color invariant holds throughout (precondition checked each step)
    assert!(e.trace().entries.iter().all(|entry| entry.success));
}

#[test]
fn engine_inverse_solves() {
    let mut e = new_cube();
    let moves = [Move::R, Move::U, Move::F];
    for m in &moves {
        e = e.try_next(RubikAction(*m)).unwrap();
    }
    assert!(!e.situation().is_solved());
    // Apply inverse sequence
    for m in moves.iter().rev() {
        e = e.try_next(RubikAction(m.inverse())).unwrap();
    }
    assert!(e.situation().is_solved());
}

#[test]
fn engine_back_forward_cycle() {
    let e = new_cube();
    let e = e.try_next(RubikAction(Move::R)).unwrap();
    let e = e.try_next(RubikAction(Move::U)).unwrap();
    let e = e.back().unwrap();
    let e = e.back().unwrap();
    assert!(e.situation().is_solved());
    let e = e.forward().unwrap();
    let e = e.forward().unwrap();
    assert_eq!(e.step(), 2);
}
