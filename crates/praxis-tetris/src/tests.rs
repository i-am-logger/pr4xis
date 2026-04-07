use crate::board::{Board, HEIGHT, WIDTH};
use crate::*;
use proptest::prelude::*;

// =============================================================================
// Proptest strategies
// =============================================================================

fn arb_piece_kind() -> impl Strategy<Value = PieceKind> {
    (0..7usize).prop_map(|i| PieceKind::all()[i])
}

fn arb_action() -> impl Strategy<Value = GameAction> {
    prop_oneof![
        Just(GameAction::MoveLeft),
        Just(GameAction::MoveRight),
        Just(GameAction::MoveDown),
        Just(GameAction::HardDrop),
        Just(GameAction::RotateCW),
        Just(GameAction::RotateCCW),
    ]
}

fn arb_actions() -> impl Strategy<Value = Vec<GameAction>> {
    proptest::collection::vec(arb_action(), 0..50)
}

fn arb_seed() -> impl Strategy<Value = u64> {
    0..10000u64
}

// =============================================================================
// Board tests
// =============================================================================

#[test]
fn test_new_board_is_empty() {
    let board = Board::new();
    assert_eq!(board.filled_count(), 0);
}

#[test]
fn test_walls_are_filled() {
    let board = Board::new();
    assert!(board.is_filled(-1, 0)); // left wall
    assert!(board.is_filled(WIDTH as i32, 0)); // right wall
    assert!(board.is_filled(0, -1)); // floor
    assert!(board.is_filled(0, HEIGHT as i32)); // ceiling
}

#[test]
fn test_line_clear() {
    let mut board = Board::new();
    // Fill bottom row with I pieces: cols 0-3, then 4-7
    let _ = board.lock_piece(&Piece {
        kind: PieceKind::I,
        rotation: Rotation::R0,
        x: 0,
        y: 0,
    });
    let _ = board.lock_piece(&Piece {
        kind: PieceKind::I,
        rotation: Rotation::R0,
        x: 4,
        y: 0,
    });
    // 8 of 10 filled. Add an O piece at cols 8-9
    let _ = board.lock_piece(&Piece {
        kind: PieceKind::O,
        rotation: Rotation::R0,
        x: 8,
        y: 0,
    });
    // Row 0 should now be full (10 cells)
    assert!(board.row_full(0));
    let cleared = board.clear_lines();
    assert_eq!(cleared, 1);
    assert!(!board.row_full(0)); // row 0 is now empty (or shifted)
}

#[test]
fn test_piece_fits_on_empty_board() {
    let board = Board::new();
    for kind in PieceKind::all() {
        let piece = Piece::new(kind);
        assert!(
            board.piece_fits(&piece),
            "{:?} should fit on empty board at spawn",
            kind
        );
    }
}

#[test]
fn test_piece_doesnt_fit_out_of_bounds() {
    let board = Board::new();
    let piece = Piece {
        kind: PieceKind::I,
        rotation: Rotation::R0,
        x: -1,
        y: 0,
    };
    assert!(!board.piece_fits(&piece));
}

// =============================================================================
// Piece tests
// =============================================================================

#[test]
fn test_all_pieces_have_4_cells() {
    for kind in PieceKind::all() {
        let piece = Piece::new(kind);
        assert_eq!(piece.cells().len(), 4, "{:?} should have 4 cells", kind);
    }
}

#[test]
fn test_rotation_cycle() {
    let piece = Piece::new(PieceKind::T);
    let r1 = piece.rotated_cw();
    let r2 = r1.rotated_cw();
    let r3 = r2.rotated_cw();
    let r4 = r3.rotated_cw();
    assert_eq!(piece.rotation, r4.rotation); // 4 CW rotations = identity
}

#[test]
fn test_cw_ccw_inverse() {
    let piece = Piece::new(PieceKind::S);
    let rotated = piece.rotated_cw().rotated_ccw();
    assert_eq!(piece.rotation, rotated.rotation);
}

// =============================================================================
// Game tests
// =============================================================================

#[test]
fn test_new_game_has_piece() {
    let game = Game::new(42);
    assert!(game.current_piece.is_some());
    assert!(!game.game_over);
}

#[test]
fn test_move_left_right() {
    let mut game = Game::new(42);
    let before = game.current_piece.as_ref().unwrap().x;
    game.act(GameAction::MoveLeft);
    let after = game.current_piece.as_ref().unwrap().x;
    assert_eq!(after, before - 1);

    game.act(GameAction::MoveRight);
    let after2 = game.current_piece.as_ref().unwrap().x;
    assert_eq!(after2, before);
}

#[test]
fn test_hard_drop_locks() {
    let mut game = Game::new(42);
    let result = game.act(GameAction::HardDrop);
    assert!(matches!(
        result,
        ActionResult::Locked { .. } | ActionResult::GameOver
    ));
}

#[test]
fn test_blocked_at_left_wall() {
    let mut game = Game::new(42);
    // Move left many times — should eventually be blocked
    for _ in 0..20 {
        game.act(GameAction::MoveLeft);
    }
    let result = game.act(GameAction::MoveLeft);
    assert_eq!(result, ActionResult::Blocked);
}

#[test]
fn test_blocked_at_right_wall() {
    let mut game = Game::new(42);
    for _ in 0..20 {
        game.act(GameAction::MoveRight);
    }
    let result = game.act(GameAction::MoveRight);
    assert_eq!(result, ActionResult::Blocked);
}

// =============================================================================
// Property-based tests — ontology enforcement
// =============================================================================

proptest! {
    /// Every piece kind has exactly 4 cells in any rotation
    #[test]
    fn prop_all_rotations_have_4_cells(kind in arb_piece_kind()) {
        let piece = Piece::new(kind);
        prop_assert_eq!(piece.cells().len(), 4);
        prop_assert_eq!(piece.rotated_cw().cells().len(), 4);
        prop_assert_eq!(piece.rotated_cw().rotated_cw().cells().len(), 4);
        prop_assert_eq!(piece.rotated_ccw().cells().len(), 4);
    }

    /// 4 CW rotations = identity (rotation is cyclic)
    #[test]
    fn prop_four_rotations_identity(kind in arb_piece_kind()) {
        let p = Piece::new(kind);
        let p4 = p.rotated_cw().rotated_cw().rotated_cw().rotated_cw();
        prop_assert_eq!(p.rotation, p4.rotation);
        prop_assert_eq!(p.cells(), p4.cells());
    }

    /// CW then CCW = identity
    #[test]
    fn prop_cw_ccw_inverse(kind in arb_piece_kind()) {
        let p = Piece::new(kind);
        let roundtrip = p.rotated_cw().rotated_ccw();
        prop_assert_eq!(p.rotation, roundtrip.rotation);
    }

    /// New game always starts with a piece
    #[test]
    fn prop_new_game_has_piece(seed in arb_seed()) {
        let game = Game::new(seed);
        prop_assert!(game.current_piece.is_some());
        prop_assert!(!game.game_over);
    }

    /// No action sequence can place a piece out of bounds
    #[test]
    fn prop_piece_always_in_bounds(seed in arb_seed(), actions in arb_actions()) {
        let mut game = Game::new(seed);
        for action in actions {
            game.act(action);
            if game.game_over { break; }
            if let Some(piece) = &game.current_piece {
                for (x, y) in piece.cells() {
                    prop_assert!(x >= 0 && x < WIDTH as i32,
                        "piece cell x={} out of bounds", x);
                    prop_assert!(y >= 0 && y < HEIGHT as i32,
                        "piece cell y={} out of bounds", y);
                }
            }
        }
    }

    /// Board cell count never decreases (except via line clears which remove 10 per line)
    #[test]
    fn prop_filled_cells_monotonic_per_lock(seed in arb_seed()) {
        let mut game = Game::new(seed);
        let mut last_count = 0;
        // Play 10 pieces
        for _ in 0..10 {
            if game.game_over { break; }
            let result = game.act(GameAction::HardDrop);
            let count = game.board.filled_count();
            match result {
                ActionResult::Locked { lines_cleared } => {
                    // Each lock adds 4 cells, each line clear removes WIDTH cells
                    let expected_min = last_count + 4 - (lines_cleared as usize * WIDTH);
                    prop_assert_eq!(count, expected_min,
                        "filled count {} != expected {} (was {}, cleared {})",
                        count, expected_min, last_count, lines_cleared);
                }
                _ => {}
            }
            last_count = count;
        }
    }

    /// Hard drop always results in Locked or GameOver
    #[test]
    fn prop_hard_drop_always_locks(seed in arb_seed()) {
        let mut game = Game::new(seed);
        if !game.game_over {
            let result = game.act(GameAction::HardDrop);
            let is_locked_or_over = matches!(result, ActionResult::Locked { .. } | ActionResult::GameOver);
            prop_assert!(is_locked_or_over);
        }
    }

    /// Score never decreases
    #[test]
    fn prop_score_never_decreases(seed in arb_seed(), actions in arb_actions()) {
        let mut game = Game::new(seed);
        let mut last_score = 0;
        for action in actions {
            game.act(action);
            prop_assert!(game.score >= last_score, "score decreased from {} to {}", last_score, game.score);
            last_score = game.score;
            if game.game_over { break; }
        }
    }

    /// Game over state is permanent — no action can undo it
    #[test]
    fn prop_game_over_is_permanent(seed in arb_seed(), actions in arb_actions()) {
        let mut game = Game::new(seed);
        // Force game over by filling the board
        for _ in 0..200 {
            game.act(GameAction::HardDrop);
            if game.game_over { break; }
        }
        if game.game_over {
            for action in actions {
                let result = game.act(action);
                prop_assert_eq!(result, ActionResult::GameOver);
                prop_assert!(game.game_over);
            }
        }
    }

    /// Deterministic: same seed + same actions = same state
    #[test]
    fn prop_deterministic(seed in arb_seed(), actions in proptest::collection::vec(arb_action(), 0..20)) {
        let mut game1 = Game::new(seed);
        let mut game2 = Game::new(seed);
        for action in &actions {
            game1.act(*action);
            game2.act(*action);
        }
        prop_assert_eq!(game1.board, game2.board);
        prop_assert_eq!(game1.score, game2.score);
        prop_assert_eq!(game1.game_over, game2.game_over);
    }

    /// Walls block movement — piece x never goes negative or past WIDTH
    #[test]
    fn prop_walls_block(seed in arb_seed()) {
        let mut game = Game::new(seed);
        // Move left 20 times
        for _ in 0..20 {
            game.act(GameAction::MoveLeft);
        }
        if let Some(piece) = &game.current_piece {
            for (x, _) in piece.cells() {
                prop_assert!(x >= 0, "piece went through left wall: x={}", x);
            }
        }
        // Move right 20 times
        for _ in 0..20 {
            game.act(GameAction::MoveRight);
        }
        if let Some(piece) = &game.current_piece {
            for (x, _) in piece.cells() {
                prop_assert!(x < WIDTH as i32, "piece went through right wall: x={}", x);
            }
        }
    }
}

// =============================================================================
// Engine tests — Situation/Action/Precondition/Trace
// =============================================================================

use crate::engine::*;

#[test]
fn engine_move_and_drop() {
    let e = new_tetris(42);
    assert!(!e.is_terminal());
    let e = e.try_next(TetrisAction(GameAction::MoveLeft)).unwrap();
    let e = e.try_next(TetrisAction(GameAction::MoveRight)).unwrap();
    let e = e.try_next(TetrisAction(GameAction::HardDrop)).unwrap();
    assert_eq!(e.step(), 3);
}

#[test]
fn engine_back_forward() {
    let e = new_tetris(42);
    let e = e.try_next(TetrisAction(GameAction::MoveLeft)).unwrap();
    let e = e.try_next(TetrisAction(GameAction::RotateCW)).unwrap();
    let e = e.back().unwrap();
    assert_eq!(e.step(), 1);
    let e = e.forward().unwrap();
    assert_eq!(e.step(), 2);
}

#[test]
fn engine_trace_records() {
    let e = new_tetris(42);
    let e = e.try_next(TetrisAction(GameAction::MoveDown)).unwrap();
    let e = e.try_next(TetrisAction(GameAction::MoveDown)).unwrap();
    assert_eq!(e.trace().entries.len(), 2);
    assert!(e.trace().entries.iter().all(|entry| entry.success));
}
