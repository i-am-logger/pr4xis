use super::*;
use praxis::engine::EngineError;
use proptest::prelude::*;

fn arb_square() -> impl Strategy<Value = Square> {
    (0..8u8, 0..8u8).prop_map(|(f, r)| Square::new(f, r))
}

// =============================================================================
// Setup tests
// =============================================================================

#[test]
fn test_starting_position() {
    let board = Board::starting();
    assert_eq!(board.pieces(Color::White).len(), 16);
    assert_eq!(board.pieces(Color::Black).len(), 16);
    assert_eq!(board.to_move, Color::White);
    assert_eq!(board.castling, [true; 4]);
    assert_eq!(board.en_passant, None);
}

// =============================================================================
// Check detection tests
// =============================================================================

#[test]
fn test_not_in_check_at_start() {
    let board = Board::starting();
    assert!(!board.in_check(Color::White));
    assert!(!board.in_check(Color::Black));
}

#[test]
fn test_scholars_mate() {
    let board = Board::starting();
    let board = board
        .apply_move(Square::new(4, 1), Square::new(4, 3))
        .unwrap(); // e4
    let board = board
        .apply_move(Square::new(4, 6), Square::new(4, 4))
        .unwrap(); // e5
    let board = board
        .apply_move(Square::new(5, 0), Square::new(2, 3))
        .unwrap(); // Bc4
    let board = board
        .apply_move(Square::new(1, 7), Square::new(2, 5))
        .unwrap(); // Nc6
    let board = board
        .apply_move(Square::new(3, 0), Square::new(7, 4))
        .unwrap(); // Qh5
    let board = board
        .apply_move(Square::new(6, 7), Square::new(5, 5))
        .unwrap(); // Nf6
    let board = board
        .apply_move(Square::new(7, 4), Square::new(5, 6))
        .unwrap(); // Qxf7#
    assert!(board.is_checkmate());
}

#[test]
fn test_cant_move_into_check() {
    let mut board = Board::empty(Color::White);
    board.set(
        Square::new(4, 0),
        Some(Piece::new(PieceKind::King, Color::White)),
    );
    board.set(
        Square::new(4, 7),
        Some(Piece::new(PieceKind::Rook, Color::Black)),
    );
    board.set(
        Square::new(0, 7),
        Some(Piece::new(PieceKind::King, Color::Black)),
    );
    // King on e1, black rook on e8: king can't stay on e-file
    let moves = board.legal_moves(Square::new(4, 0));
    for sq in &moves {
        assert_ne!(
            sq.file, 4,
            "king should not move to e-file (attacked by rook)"
        );
    }
}

#[test]
fn test_must_escape_check() {
    let mut board = Board::empty(Color::White);
    board.set(
        Square::new(4, 0),
        Some(Piece::new(PieceKind::King, Color::White)),
    );
    board.set(
        Square::new(4, 7),
        Some(Piece::new(PieceKind::Rook, Color::Black)),
    );
    board.set(
        Square::new(0, 7),
        Some(Piece::new(PieceKind::King, Color::Black)),
    );
    assert!(board.in_check(Color::White));
    let moves = board.legal_moves(Square::new(4, 0));
    assert!(!moves.is_empty(), "king must have escape moves");
    // All moves must escape check
    for &to in &moves {
        let new_board = board.apply_move(Square::new(4, 0), to).unwrap();
        assert!(
            !new_board.in_check(Color::White),
            "move to {} doesn't escape check",
            to.name()
        );
    }
}

#[test]
fn test_pinned_piece_cant_move() {
    let mut board = Board::empty(Color::White);
    board.set(
        Square::new(4, 0),
        Some(Piece::new(PieceKind::King, Color::White)),
    );
    board.set(
        Square::new(4, 1),
        Some(Piece::new(PieceKind::Knight, Color::White)),
    );
    board.set(
        Square::new(4, 7),
        Some(Piece::new(PieceKind::Rook, Color::Black)),
    );
    board.set(
        Square::new(0, 7),
        Some(Piece::new(PieceKind::King, Color::Black)),
    );
    // Knight on e2 is pinned by rook on e8 to king on e1
    let moves = board.legal_moves(Square::new(4, 1));
    assert!(moves.is_empty(), "pinned knight should have no moves");
}

// =============================================================================
// Castling tests
// =============================================================================

#[test]
fn test_kingside_castling() {
    let mut board = Board::empty(Color::White);
    board.castling = [true, true, true, true];
    board.set(
        Square::new(4, 0),
        Some(Piece::new(PieceKind::King, Color::White)),
    );
    board.set(
        Square::new(7, 0),
        Some(Piece::new(PieceKind::Rook, Color::White)),
    );
    board.set(
        Square::new(4, 7),
        Some(Piece::new(PieceKind::King, Color::Black)),
    );
    let moves = board.legal_moves(Square::new(4, 0));
    assert!(
        moves.contains(&Square::new(6, 0)),
        "should be able to castle kingside"
    );
    let new_board = board
        .apply_move(Square::new(4, 0), Square::new(6, 0))
        .unwrap();
    assert_eq!(
        new_board.get(Square::new(6, 0)).unwrap().kind,
        PieceKind::King
    );
    assert_eq!(
        new_board.get(Square::new(5, 0)).unwrap().kind,
        PieceKind::Rook
    );
}

#[test]
fn test_queenside_castling() {
    let mut board = Board::empty(Color::White);
    board.castling = [true, true, true, true];
    board.set(
        Square::new(4, 0),
        Some(Piece::new(PieceKind::King, Color::White)),
    );
    board.set(
        Square::new(0, 0),
        Some(Piece::new(PieceKind::Rook, Color::White)),
    );
    board.set(
        Square::new(4, 7),
        Some(Piece::new(PieceKind::King, Color::Black)),
    );
    let moves = board.legal_moves(Square::new(4, 0));
    assert!(
        moves.contains(&Square::new(2, 0)),
        "should be able to castle queenside"
    );
}

#[test]
fn test_cant_castle_through_check() {
    let mut board = Board::empty(Color::White);
    board.castling = [true, true, true, true];
    board.set(
        Square::new(4, 0),
        Some(Piece::new(PieceKind::King, Color::White)),
    );
    board.set(
        Square::new(7, 0),
        Some(Piece::new(PieceKind::Rook, Color::White)),
    );
    board.set(
        Square::new(5, 7),
        Some(Piece::new(PieceKind::Rook, Color::Black)),
    ); // attacks f1
    board.set(
        Square::new(0, 7),
        Some(Piece::new(PieceKind::King, Color::Black)),
    );
    let moves = board.legal_moves(Square::new(4, 0));
    assert!(
        !moves.contains(&Square::new(6, 0)),
        "can't castle through attacked square"
    );
}

#[test]
fn test_cant_castle_out_of_check() {
    let mut board = Board::empty(Color::White);
    board.castling = [true, true, true, true];
    board.set(
        Square::new(4, 0),
        Some(Piece::new(PieceKind::King, Color::White)),
    );
    board.set(
        Square::new(7, 0),
        Some(Piece::new(PieceKind::Rook, Color::White)),
    );
    board.set(
        Square::new(4, 7),
        Some(Piece::new(PieceKind::Rook, Color::Black)),
    ); // checks king
    board.set(
        Square::new(0, 7),
        Some(Piece::new(PieceKind::King, Color::Black)),
    );
    let moves = board.legal_moves(Square::new(4, 0));
    assert!(
        !moves.contains(&Square::new(6, 0)),
        "can't castle out of check"
    );
}

#[test]
fn test_castling_rights_lost_on_king_move() {
    let board = Board::starting();
    let board = board
        .apply_move(Square::new(4, 1), Square::new(4, 3))
        .unwrap(); // e4
    let board = board
        .apply_move(Square::new(4, 6), Square::new(4, 4))
        .unwrap(); // e5
    let board = board
        .apply_move(Square::new(4, 0), Square::new(4, 1))
        .unwrap(); // Ke2
    assert!(
        !board.castling[0] && !board.castling[1],
        "white lost both castling rights"
    );
}

// =============================================================================
// En passant tests
// =============================================================================

#[test]
fn test_en_passant() {
    let mut board = Board::empty(Color::White);
    board.set(
        Square::new(4, 4),
        Some(Piece::new(PieceKind::Pawn, Color::White)),
    );
    board.set(
        Square::new(3, 6),
        Some(Piece::new(PieceKind::Pawn, Color::Black)),
    );
    board.set(
        Square::new(4, 0),
        Some(Piece::new(PieceKind::King, Color::White)),
    );
    board.set(
        Square::new(4, 7),
        Some(Piece::new(PieceKind::King, Color::Black)),
    );
    board.to_move = Color::Black;
    // Black pawn advances two squares
    let board = board
        .apply_move(Square::new(3, 6), Square::new(3, 4))
        .unwrap();
    assert_eq!(board.en_passant, Some(Square::new(3, 5)));
    // White can capture en passant
    let moves = board.legal_moves(Square::new(4, 4));
    assert!(
        moves.contains(&Square::new(3, 5)),
        "should allow en passant"
    );
    let board = board
        .apply_move(Square::new(4, 4), Square::new(3, 5))
        .unwrap();
    assert!(
        board.is_empty(Square::new(3, 4)),
        "captured pawn should be removed"
    );
}

#[test]
fn test_en_passant_expires() {
    let board = Board::starting();
    let board = board
        .apply_move(Square::new(4, 1), Square::new(4, 3))
        .unwrap(); // e4
    assert_eq!(board.en_passant, Some(Square::new(4, 2)));
    let board = board
        .apply_move(Square::new(0, 6), Square::new(0, 5))
        .unwrap(); // a6
    assert_eq!(
        board.en_passant, None,
        "en passant should expire after one move"
    );
}

// =============================================================================
// Promotion tests
// =============================================================================

#[test]
fn test_pawn_promotion() {
    let mut board = Board::empty(Color::White);
    board.set(
        Square::new(0, 6),
        Some(Piece::new(PieceKind::Pawn, Color::White)),
    );
    board.set(
        Square::new(4, 0),
        Some(Piece::new(PieceKind::King, Color::White)),
    );
    board.set(
        Square::new(4, 7),
        Some(Piece::new(PieceKind::King, Color::Black)),
    );
    let board = board
        .apply_move(Square::new(0, 6), Square::new(0, 7))
        .unwrap();
    assert_eq!(
        board.get(Square::new(0, 7)).unwrap().kind,
        PieceKind::Queen,
        "pawn should auto-promote to queen"
    );
}

// =============================================================================
// Stalemate and 50-move rule tests
// =============================================================================

#[test]
fn test_stalemate() {
    let mut board = Board::empty(Color::Black);
    board.to_move = Color::Black;
    board.set(
        Square::new(0, 7),
        Some(Piece::new(PieceKind::King, Color::Black)),
    );
    board.set(
        Square::new(1, 5),
        Some(Piece::new(PieceKind::Queen, Color::White)),
    );
    board.set(
        Square::new(2, 6),
        Some(Piece::new(PieceKind::King, Color::White)),
    );
    // Black king at a8, white queen at b6, white king at c7
    // Black has no legal moves but is not in check
    assert!(!board.in_check(Color::Black));
    assert!(board.is_stalemate());
}

#[test]
fn test_fifty_move_rule() {
    let mut board = Board::empty(Color::White);
    board.halfmove_clock = 100;
    assert!(board.is_fifty_move_rule());
    board.halfmove_clock = 99;
    assert!(!board.is_fifty_move_rule());
}

// =============================================================================
// Property-based tests — ontology enforcement
// =============================================================================

proptest! {
    /// No legal move leaves own king in check
    #[test]
    fn prop_no_move_leaves_king_in_check(from in arb_square()) {
        let board = Board::starting();
        for to in board.legal_moves(from) {
            let new_board = board.apply_move(from, to).unwrap();
            // The side that just moved should NOT be in check
            prop_assert!(!new_board.in_check(board.to_move),
                "move {} → {} leaves king in check", from.name(), to.name());
        }
    }

    /// apply_move and legal_moves agree
    #[test]
    fn prop_apply_enforces_legality(from in arb_square(), to in arb_square()) {
        let board = Board::starting();
        let legal = board.legal_moves(from);
        let result = board.apply_move(from, to);
        if legal.contains(&to) {
            prop_assert!(result.is_some());
        } else {
            prop_assert!(result.is_none());
        }
    }

    /// A move changes the side to move
    #[test]
    fn prop_move_changes_turn(from in arb_square(), to in arb_square()) {
        let board = Board::starting();
        if let Some(new_board) = board.apply_move(from, to) {
            prop_assert_ne!(board.to_move, new_board.to_move);
        }
    }

    /// Source is empty after any legal move
    #[test]
    fn prop_source_empty_after_move(from in arb_square(), to in arb_square()) {
        let board = Board::starting();
        if let Some(new_board) = board.apply_move(from, to) {
            prop_assert!(new_board.is_empty(from));
        }
    }

    /// Piece appears at target after move (with correct type)
    #[test]
    fn prop_piece_at_target(from in arb_square(), to in arb_square()) {
        let board = Board::starting();
        let piece = board.get(from);
        if let Some(new_board) = board.apply_move(from, to) {
            let target = new_board.get(to);
            prop_assert!(target.is_some());
            // Color matches (might be promoted but same color)
            prop_assert_eq!(target.unwrap().color, piece.unwrap().color);
        }
    }

    /// Piece count decreases by exactly 1 on capture, unchanged otherwise
    #[test]
    fn prop_piece_count_on_move(from in arb_square(), to in arb_square()) {
        let board = Board::starting();
        let before = board.pieces(Color::White).len() + board.pieces(Color::Black).len();
        if let Some(new_board) = board.apply_move(from, to) {
            let after = new_board.pieces(Color::White).len() + new_board.pieces(Color::Black).len();
            let was_capture = board.get(to).is_some()
                || (board.get(from).is_some_and(|p| p.kind == PieceKind::Pawn)
                    && board.en_passant == Some(to));
            if was_capture {
                prop_assert_eq!(after, before - 1);
            } else {
                prop_assert_eq!(after, before);
            }
        }
    }

    /// Cannot move opponent's pieces
    #[test]
    fn prop_cant_move_opponent(sq in arb_square()) {
        let board = Board::starting();
        if let Some(p) = board.get(sq)
            && p.color != board.to_move
        {
            prop_assert!(board.legal_moves(sq).is_empty());
        }
    }

    /// Empty square has no moves
    #[test]
    fn prop_empty_no_moves(sq in arb_square()) {
        let board = Board::empty(Color::White);
        prop_assert!(board.legal_moves(sq).is_empty());
    }

    /// No legal move captures own piece
    #[test]
    fn prop_no_self_capture(from in arb_square()) {
        let board = Board::starting();
        if let Some(piece) = board.get(from) {
            for to in board.legal_moves(from) {
                if let Some(target) = board.get(to) {
                    prop_assert_ne!(target.color, piece.color);
                }
            }
        }
    }

    /// ChessMove::new agrees with apply_move
    #[test]
    fn prop_chessmove_consistent(from in arb_square(), to in arb_square()) {
        let board = Board::starting();
        prop_assert_eq!(
            ChessMove::new(&board, from, to).is_some(),
            board.apply_move(from, to).is_some()
        );
    }

    /// Rook moves are always on same file or rank (on empty board)
    #[test]
    fn prop_rook_moves_straight(sq in arb_square()) {
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::Rook, Color::White)));
        for to in board.legal_moves(sq) {
            prop_assert!(to.file == sq.file || to.rank == sq.rank);
        }
    }

    /// Rook has exactly 14 moves on empty board from any square
    #[test]
    fn prop_rook_14_moves(sq in arb_square()) {
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::Rook, Color::White)));
        prop_assert_eq!(board.legal_moves(sq).len(), 14);
    }

    /// Bishop moves are always diagonal
    #[test]
    fn prop_bishop_diagonal(sq in arb_square()) {
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::Bishop, Color::White)));
        for to in board.legal_moves(sq) {
            let df = (to.file as i8 - sq.file as i8).abs();
            let dr = (to.rank as i8 - sq.rank as i8).abs();
            prop_assert_eq!(df, dr);
        }
    }

    /// Queen = rook + bishop moves on empty board
    #[test]
    fn prop_queen_is_rook_plus_bishop(sq in arb_square()) {
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::Queen, Color::White)));
        let queen: std::collections::HashSet<_> = board.legal_moves(sq).into_iter().collect();
        board.set(sq, Some(Piece::new(PieceKind::Rook, Color::White)));
        let rook: std::collections::HashSet<_> = board.legal_moves(sq).into_iter().collect();
        board.set(sq, Some(Piece::new(PieceKind::Bishop, Color::White)));
        let bishop: std::collections::HashSet<_> = board.legal_moves(sq).into_iter().collect();
        prop_assert_eq!(queen, rook.union(&bishop).cloned().collect());
    }

    /// Knight moves in L-shape
    #[test]
    fn prop_knight_l_shape(sq in arb_square()) {
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::Knight, Color::White)));
        for to in board.legal_moves(sq) {
            let df = (to.file as i8 - sq.file as i8).abs();
            let dr = (to.rank as i8 - sq.rank as i8).abs();
            prop_assert!((df == 1 && dr == 2) || (df == 2 && dr == 1));
        }
    }

    /// Knight has 2-8 moves on empty board
    #[test]
    fn prop_knight_2_to_8(sq in arb_square()) {
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::Knight, Color::White)));
        let n = board.legal_moves(sq).len();
        prop_assert!((2..=8).contains(&n));
    }

    /// King moves at most 1 step
    #[test]
    fn prop_king_one_step(sq in arb_square()) {
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::King, Color::White)));
        for to in board.legal_moves(sq) {
            let df = (to.file as i8 - sq.file as i8).abs();
            let dr = (to.rank as i8 - sq.rank as i8).abs();
            prop_assert!(df <= 1 && dr <= 1);
        }
    }

    /// King has 3-8 moves on empty board
    #[test]
    fn prop_king_3_to_8(sq in arb_square()) {
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::King, Color::White)));
        let n = board.legal_moves(sq).len();
        prop_assert!((3..=8).contains(&n));
    }

    /// White pawn only moves forward (increasing rank)
    #[test]
    fn prop_white_pawn_forward(file in 0..8u8, rank in 1..7u8) {
        let sq = Square::new(file, rank);
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::Pawn, Color::White)));
        for to in board.legal_moves(sq) {
            prop_assert!(to.rank > sq.rank);
        }
    }

    /// Black pawn only moves forward (decreasing rank)
    #[test]
    fn prop_black_pawn_forward(file in 0..8u8, rank in 1..7u8) {
        let sq = Square::new(file, rank);
        let mut board = Board::empty(Color::Black);
        board.to_move = Color::Black;
        board.set(sq, Some(Piece::new(PieceKind::Pawn, Color::Black)));
        for to in board.legal_moves(sq) {
            prop_assert!(to.rank < sq.rank);
        }
    }

    /// Pawn double move only from starting rank
    #[test]
    fn prop_pawn_double_from_start(file in 0..8u8) {
        let sq = Square::new(file, 1);
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::Pawn, Color::White)));
        let moves = board.legal_moves(sq);
        prop_assert!(moves.contains(&Square::new(file, 3)));
    }

    /// Pawn NOT from starting rank can't double move
    #[test]
    fn prop_pawn_no_double_not_start(file in 0..8u8, rank in 2..7u8) {
        let sq = Square::new(file, rank);
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::Pawn, Color::White)));
        let moves = board.legal_moves(sq);
        prop_assert_eq!(moves.len(), 1);
    }

    /// Sliding piece blocked by own piece
    #[test]
    fn prop_rook_blocked(sq in arb_square(), dist in 1..7i8) {
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::Rook, Color::White)));
        if let Some(blocker) = sq.offset(0, dist) {
            board.set(blocker, Some(Piece::new(PieceKind::Pawn, Color::White)));
            let moves = board.legal_moves(sq);
            prop_assert!(!moves.contains(&blocker));
            if let Some(past) = sq.offset(0, dist + 1) {
                prop_assert!(!moves.contains(&past));
            }
        }
    }

    /// Sliding piece can capture opponent
    #[test]
    fn prop_rook_captures(sq in arb_square(), dist in 1..7i8) {
        let mut board = Board::empty(Color::White);
        board.set(sq, Some(Piece::new(PieceKind::Rook, Color::White)));
        if let Some(target) = sq.offset(0, dist) {
            board.set(target, Some(Piece::new(PieceKind::Pawn, Color::Black)));
            prop_assert!(board.legal_moves(sq).contains(&target));
        }
    }

    /// En passant square is set after double pawn move
    #[test]
    fn prop_en_passant_set(file in 0..8u8) {
        let board = Board::starting();
        let from = Square::new(file, 1);
        let to = Square::new(file, 3);
        if let Some(new_board) = board.apply_move(from, to) {
            prop_assert_eq!(new_board.en_passant, Some(Square::new(file, 2)));
        }
    }

    /// En passant cleared after non-double-pawn move
    #[test]
    fn prop_en_passant_clears(from in arb_square(), to in arb_square()) {
        let board = Board::starting();
        if let Some(b1) = board.apply_move(Square::new(4, 1), Square::new(4, 3)) {
            // b1 has en passant set
            if let Some(b2) = b1.apply_move(from, to) {
                // After black moves, en passant should clear (unless another double pawn)
                let piece = b1.get(from);
                let is_double_pawn = piece.is_some_and(|p| {
                    p.kind == PieceKind::Pawn && (to.rank as i8 - from.rank as i8).abs() == 2
                });
                if !is_double_pawn {
                    prop_assert_eq!(b2.en_passant, None);
                }
            }
        }
    }

    /// Halfmove clock resets on pawn move
    #[test]
    fn prop_halfmove_resets_pawn(file in 0..8u8) {
        let mut board = Board::starting();
        board.halfmove_clock = 10;
        if let Some(new_board) = board.apply_move(Square::new(file, 1), Square::new(file, 2)) {
            prop_assert_eq!(new_board.halfmove_clock, 0);
        }
    }

    /// Each side has exactly 1 king at start
    #[test]
    fn prop_one_king(_sq in arb_square()) {
        let board = Board::starting();
        let wk = board.pieces(Color::White).iter().filter(|(_, p)| p.kind == PieceKind::King).count();
        let bk = board.pieces(Color::Black).iter().filter(|(_, p)| p.kind == PieceKind::King).count();
        prop_assert_eq!(wk, 1);
        prop_assert_eq!(bk, 1);
    }

    /// Each side has exactly 8 pawns at start
    #[test]
    fn prop_eight_pawns(_sq in arb_square()) {
        let board = Board::starting();
        let wp = board.pieces(Color::White).iter().filter(|(_, p)| p.kind == PieceKind::Pawn).count();
        let bp = board.pieces(Color::Black).iter().filter(|(_, p)| p.kind == PieceKind::Pawn).count();
        prop_assert_eq!(wp, 8);
        prop_assert_eq!(bp, 8);
    }

    /// All legal moves land on valid squares
    #[test]
    fn prop_moves_in_bounds(sq in arb_square()) {
        let board = Board::starting();
        for to in board.legal_moves(sq) {
            prop_assert!(to.file < 8 && to.rank < 8);
        }
    }

    /// 64 squares total
    #[test]
    fn prop_64_squares(_sq in arb_square()) {
        use praxis::category::Entity;
        prop_assert_eq!(Square::variants().len(), 64);
    }
}

// =============================================================================
// Engine tests — the .next() API
// =============================================================================

#[test]
fn test_engine_new_game() {
    let engine = new_game();
    assert!(!engine.is_terminal());
}

#[test]
fn test_engine_e4() {
    let engine = new_game()
        .next(ChessAction::new(Square::new(4, 1), Square::new(4, 3)))
        .unwrap(); // e4
    assert_eq!(engine.situation().to_move, Color::Black);
}

#[test]
fn test_engine_chain() {
    let engine = new_game()
        .next(ChessAction::new(Square::new(4, 1), Square::new(4, 3)))
        .unwrap() // e4
        .next(ChessAction::new(Square::new(4, 6), Square::new(4, 4)))
        .unwrap() // e5
        .next(ChessAction::new(Square::new(6, 0), Square::new(5, 2)))
        .unwrap(); // Nf3
    assert_eq!(engine.trace().successful_steps(), 3);
}

#[test]
fn test_engine_illegal_move_blocked() {
    let engine = new_game();
    // Try to move a rook through pawns
    let result = engine.next(ChessAction::new(Square::new(0, 0), Square::new(0, 3)));
    assert!(result.is_err());
}

#[test]
fn test_engine_empty_square_blocked() {
    let engine = new_game();
    let result = engine.next(ChessAction::new(Square::new(4, 4), Square::new(4, 5)));
    assert!(result.is_err());
}

#[test]
fn test_engine_opponents_piece_blocked() {
    let engine = new_game();
    // Try to move a black pawn (white's turn)
    let result = engine.next(ChessAction::new(Square::new(4, 6), Square::new(4, 5)));
    assert!(result.is_err());
}

#[test]
fn test_engine_violation_describes_rule() {
    let engine = new_game();
    let EngineError::Violated { engine, violations } = engine
        .next(ChessAction::new(Square::new(0, 0), Square::new(0, 5)))
        .unwrap_err()
    else {
        panic!("expected Violated")
    };
    // Should have a legal_move violation
    let has_legal_move_violation = violations.iter().any(|v| v.rule() == "legal_move");
    assert!(has_legal_move_violation, "should have legal_move violation");
    assert!(engine.trace().violations() > 0);
}

#[test]
fn test_engine_trace_dump() {
    let engine = new_game()
        .next(ChessAction::new(Square::new(4, 1), Square::new(4, 3)))
        .unwrap();
    let dump = engine.trace().dump();
    assert!(dump.contains("OK"));
    assert!(dump.contains("e2"));
}

#[test]
fn test_engine_scholars_mate() {
    let engine = new_game()
        .next(ChessAction::new(Square::new(4, 1), Square::new(4, 3)))
        .unwrap() // e4
        .next(ChessAction::new(Square::new(4, 6), Square::new(4, 4)))
        .unwrap() // e5
        .next(ChessAction::new(Square::new(5, 0), Square::new(2, 3)))
        .unwrap() // Bc4
        .next(ChessAction::new(Square::new(1, 7), Square::new(2, 5)))
        .unwrap() // Nc6
        .next(ChessAction::new(Square::new(3, 0), Square::new(7, 4)))
        .unwrap() // Qh5
        .next(ChessAction::new(Square::new(6, 7), Square::new(5, 5)))
        .unwrap() // Nf6
        .next(ChessAction::new(Square::new(7, 4), Square::new(5, 6)))
        .unwrap(); // Qxf7#
    assert!(engine.is_terminal());
    assert!(engine.situation().is_checkmate());
}

#[test]
fn test_engine_cant_move_after_checkmate() {
    let engine = new_game()
        .next(ChessAction::new(Square::new(4, 1), Square::new(4, 3)))
        .unwrap()
        .next(ChessAction::new(Square::new(4, 6), Square::new(4, 4)))
        .unwrap()
        .next(ChessAction::new(Square::new(5, 0), Square::new(2, 3)))
        .unwrap()
        .next(ChessAction::new(Square::new(1, 7), Square::new(2, 5)))
        .unwrap()
        .next(ChessAction::new(Square::new(3, 0), Square::new(7, 4)))
        .unwrap()
        .next(ChessAction::new(Square::new(6, 7), Square::new(5, 5)))
        .unwrap()
        .next(ChessAction::new(Square::new(7, 4), Square::new(5, 6)))
        .unwrap();
    // Try any move after checkmate
    let result = engine.next(ChessAction::new(Square::new(0, 6), Square::new(0, 5)));
    assert!(result.is_err());
}

proptest! {
    /// Engine blocks illegal moves
    #[test]
    fn prop_engine_blocks_illegal(from in arb_square(), to in arb_square()) {
        let engine = new_game();
        let board = engine.situation();
        let legal = board.legal_moves(from);
        let result = engine.next(ChessAction::new(from, to));
        if legal.contains(&to) {
            prop_assert!(result.is_ok());
        } else {
            prop_assert!(result.is_err());
        }
    }

    /// Engine trace grows on every attempt
    #[test]
    fn prop_trace_grows(from in arb_square(), to in arb_square()) {
        let engine = new_game();
        let result = engine.next(ChessAction::new(from, to));
        match result {
            Ok(e) => prop_assert_eq!(e.trace().entries().len(), 1),
            Err(EngineError::Violated { engine: e, .. }) => prop_assert_eq!(e.trace().entries().len(), 1),
            Err(_) => unreachable!(),
        }
    }

    /// Violations always carry the rule name
    #[test]
    fn prop_violations_have_rule(from in arb_square(), to in arb_square()) {
        let engine = new_game();
        if let Err(EngineError::Violated { violations, .. }) = engine.next(ChessAction::new(from, to)) {
            for v in &violations {
                prop_assert!(!v.rule().is_empty());
                prop_assert!(!v.reason().is_empty());
            }
        }
    }

    /// Successful moves always have all preconditions satisfied in trace
    #[test]
    fn prop_success_all_satisfied(from in arb_square(), to in arb_square()) {
        let engine = new_game();
        if let Ok(engine) = engine.next(ChessAction::new(from, to)) {
            let entry = engine.trace().last().unwrap();
            prop_assert!(entry.success);
            for r in &entry.precondition_results {
                prop_assert!(r.is_satisfied());
            }
        }
    }
}

// Famous chess games are tested via PGN files in pgn.rs — not manual move lists.
// See crates/domains/src/chess/games/*.pgn for authentic game records.
