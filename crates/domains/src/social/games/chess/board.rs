#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::piece::{Color, Piece, PieceKind};
use super::square::Square;
use pr4xis::category::Concept;

/// A chess board with full rule enforcement.
///
/// Tracks: piece positions, castling rights, en passant target,
/// halfmove clock (50-move rule), and move history for threefold repetition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Board {
    squares: [[Option<Piece>; 8]; 8],
    pub to_move: Color,
    /// Castling rights: [white_kingside, white_queenside, black_kingside, black_queenside]
    pub castling: [bool; 4],
    /// En passant target square (the square a pawn can capture to).
    pub en_passant: Option<Square>,
    /// Halfmove clock: incremented after each move, reset on pawn move or capture.
    /// 50-move rule triggers at 100 (50 moves per side).
    pub halfmove_clock: u32,
    /// Full move number (starts at 1, incremented after black's move).
    pub fullmove: u32,
}

/// Index into castling rights array.
const WK: usize = 0; // white kingside
const WQ: usize = 1; // white queenside
const BK: usize = 2; // black kingside
const BQ: usize = 3; // black queenside

impl Board {
    pub fn empty(to_move: Color) -> Self {
        Self {
            squares: [[None; 8]; 8],
            to_move,
            castling: [false; 4],
            en_passant: None,
            halfmove_clock: 0,
            fullmove: 1,
        }
    }

    pub fn starting() -> Self {
        let mut board = Self::empty(Color::White);
        board.castling = [true; 4];

        let back_rank = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];
        for (f, &kind) in back_rank.iter().enumerate() {
            board.set(
                Square::new(f as u8, 0),
                Some(Piece::new(kind, Color::White)),
            );
            board.set(
                Square::new(f as u8, 7),
                Some(Piece::new(kind, Color::Black)),
            );
            board.set(
                Square::new(f as u8, 1),
                Some(Piece::new(PieceKind::Pawn, Color::White)),
            );
            board.set(
                Square::new(f as u8, 6),
                Some(Piece::new(PieceKind::Pawn, Color::Black)),
            );
        }

        board
    }

    pub fn get(&self, sq: Square) -> Option<Piece> {
        self.squares[sq.file as usize][sq.rank as usize]
    }

    pub fn set(&mut self, sq: Square, piece: Option<Piece>) {
        self.squares[sq.file as usize][sq.rank as usize] = piece;
    }

    pub fn is_occupied_by(&self, sq: Square, color: Color) -> bool {
        self.get(sq).is_some_and(|p| p.color == color)
    }

    pub fn is_empty(&self, sq: Square) -> bool {
        self.get(sq).is_none()
    }

    pub fn path_clear(&self, from: Square, to: Square) -> bool {
        let df = (to.file as i8 - from.file as i8).signum();
        let dr = (to.rank as i8 - from.rank as i8).signum();
        let mut f = from.file as i8 + df;
        let mut r = from.rank as i8 + dr;
        while (f, r) != (to.file as i8, to.rank as i8) {
            if self.get(Square::new(f as u8, r as u8)).is_some() {
                return false;
            }
            f += df;
            r += dr;
        }
        true
    }

    /// Find the king's square for a given color.
    pub fn king_square(&self, color: Color) -> Option<Square> {
        for sq in Square::variants() {
            if let Some(p) = self.get(sq)
                && p.kind == PieceKind::King
                && p.color == color
            {
                return Some(sq);
            }
        }
        None
    }

    /// Is the given square attacked by any piece of `attacker_color`?
    pub fn is_attacked_by(&self, sq: Square, attacker_color: Color) -> bool {
        for attacker_sq in Square::variants() {
            if let Some(p) = self.get(attacker_sq) {
                if p.color != attacker_color {
                    continue;
                }
                if self.attacks(attacker_sq, p, sq) {
                    return true;
                }
            }
        }
        false
    }

    /// Does piece `p` at `from` attack square `to`?
    /// Note: for pawns, attacking is diagonal (different from moving).
    fn attacks(&self, from: Square, p: Piece, to: Square) -> bool {
        if from == to {
            return false;
        }
        match p.kind {
            PieceKind::Pawn => {
                let d = p.color.pawn_direction();
                let df = (to.file as i8 - from.file as i8).abs();
                let dr = to.rank as i8 - from.rank as i8;
                df == 1 && dr == d
            }
            PieceKind::Knight => {
                let df = (to.file as i8 - from.file as i8).abs();
                let dr = (to.rank as i8 - from.rank as i8).abs();
                (df == 1 && dr == 2) || (df == 2 && dr == 1)
            }
            PieceKind::King => {
                let df = (to.file as i8 - from.file as i8).abs();
                let dr = (to.rank as i8 - from.rank as i8).abs();
                df <= 1 && dr <= 1
            }
            PieceKind::Rook => {
                let on_line = from.file == to.file || from.rank == to.rank;
                on_line && self.path_clear(from, to)
            }
            PieceKind::Bishop => {
                let df = (to.file as i8 - from.file as i8).abs();
                let dr = (to.rank as i8 - from.rank as i8).abs();
                df == dr && df > 0 && self.path_clear(from, to)
            }
            PieceKind::Queen => {
                let df = (to.file as i8 - from.file as i8).abs();
                let dr = (to.rank as i8 - from.rank as i8).abs();
                let on_line = from.file == to.file || from.rank == to.rank;
                let on_diag = df == dr && df > 0;
                (on_line || on_diag) && self.path_clear(from, to)
            }
        }
    }

    /// Is the given color's king in check?
    pub fn in_check(&self, color: Color) -> bool {
        if let Some(king_sq) = self.king_square(color) {
            self.is_attacked_by(king_sq, color.opponent())
        } else {
            false
        }
    }

    /// Generate pseudo-legal moves (before filtering for check).
    fn pseudo_legal_moves(&self, from: Square) -> Vec<Square> {
        let piece = match self.get(from) {
            Some(p) if p.color == self.to_move => p,
            _ => return vec![],
        };

        let mut moves = Vec::new();

        match piece.kind {
            PieceKind::Pawn => {
                let d = piece.color.pawn_direction();

                // Forward one
                if let Some(fwd) = from.offset(0, d)
                    && self.is_empty(fwd)
                {
                    moves.push(fwd);
                    // Forward two from starting rank
                    if from.rank == piece.color.pawn_rank()
                        && let Some(fwd2) = from.offset(0, 2 * d)
                        && self.is_empty(fwd2)
                    {
                        moves.push(fwd2);
                    }
                }

                // Diagonal captures
                for df in [-1i8, 1] {
                    if let Some(diag) = from.offset(df, d) {
                        if self.is_occupied_by(diag, piece.color.opponent()) {
                            moves.push(diag);
                        }
                        // En passant
                        if self.en_passant == Some(diag) {
                            moves.push(diag);
                        }
                    }
                }
            }

            PieceKind::Knight => {
                for target in piece.reachable_squares(from) {
                    if !self.is_occupied_by(target, piece.color) {
                        moves.push(target);
                    }
                }
            }

            PieceKind::King => {
                for target in piece.reachable_squares(from) {
                    if !self.is_occupied_by(target, piece.color) {
                        moves.push(target);
                    }
                }
                // Castling
                self.add_castling_moves(from, piece.color, &mut moves);
            }

            // Sliding pieces: Rook, Bishop, Queen
            _ => {
                for target in piece.reachable_squares(from) {
                    if !self.is_occupied_by(target, piece.color) && self.path_clear(from, target) {
                        moves.push(target);
                    }
                }
            }
        }

        moves
    }

    /// Add castling moves if legal.
    fn add_castling_moves(&self, _king_sq: Square, color: Color, moves: &mut Vec<Square>) {
        let (rank, ki, qi) = match color {
            Color::White => (0u8, WK, WQ),
            Color::Black => (7u8, BK, BQ),
        };

        // Can't castle out of check
        if self.in_check(color) {
            return;
        }

        // Kingside: king moves to g-file
        if self.castling[ki] {
            let f_sq = Square::new(5, rank);
            let g_sq = Square::new(6, rank);
            if self.is_empty(f_sq)
                && self.is_empty(g_sq)
                && !self.is_attacked_by(f_sq, color.opponent())
                && !self.is_attacked_by(g_sq, color.opponent())
            {
                moves.push(g_sq);
            }
        }

        // Queenside: king moves to c-file
        if self.castling[qi] {
            let d_sq = Square::new(3, rank);
            let c_sq = Square::new(2, rank);
            let b_sq = Square::new(1, rank);
            if self.is_empty(d_sq)
                && self.is_empty(c_sq)
                && self.is_empty(b_sq)
                && !self.is_attacked_by(d_sq, color.opponent())
                && !self.is_attacked_by(c_sq, color.opponent())
            {
                moves.push(c_sq);
            }
        }
    }

    /// All legal moves for the piece at `from`.
    /// Filters pseudo-legal moves to exclude those that leave own king in check.
    pub fn legal_moves(&self, from: Square) -> Vec<Square> {
        self.pseudo_legal_moves(from)
            .into_iter()
            .filter(|&to| {
                let mut test = self.clone();
                test.make_move_unchecked(from, to);
                !test.in_check(self.to_move)
            })
            .collect()
    }

    /// Apply a move without legality check (used internally for testing check).
    fn make_move_unchecked(&mut self, from: Square, to: Square) {
        let piece = match self.get(from) {
            Some(p) => p,
            None => return,
        };

        // En passant capture: remove the captured pawn
        if piece.kind == PieceKind::Pawn && self.en_passant == Some(to) {
            let captured_rank = from.rank; // the pawn being captured is on the same rank
            self.set(Square::new(to.file, captured_rank), None);
        }

        // Castling: move the rook too
        if piece.kind == PieceKind::King {
            let df = to.file as i8 - from.file as i8;
            if df.abs() == 2 {
                let rank = from.rank;
                if df > 0 {
                    // Kingside
                    let rook_from = Square::new(7, rank);
                    let rook_to = Square::new(5, rank);
                    let rook = self.get(rook_from);
                    self.set(rook_from, None);
                    self.set(rook_to, rook);
                } else {
                    // Queenside
                    let rook_from = Square::new(0, rank);
                    let rook_to = Square::new(3, rank);
                    let rook = self.get(rook_from);
                    self.set(rook_from, None);
                    self.set(rook_to, rook);
                }
            }
        }

        // Move the piece
        self.set(from, None);

        // Pawn promotion: auto-promote to queen
        if piece.kind == PieceKind::Pawn && (to.rank == 0 || to.rank == 7) {
            self.set(to, Some(Piece::new(PieceKind::Queen, piece.color)));
        } else {
            self.set(to, Some(piece));
        }

        // Switch turn
        self.to_move = self.to_move.opponent();
    }

    /// Apply a move with full rule enforcement. Returns None if illegal.
    pub fn apply_move(&self, from: Square, to: Square) -> Option<Board> {
        if !self.legal_moves(from).contains(&to) {
            return None;
        }

        let piece = self.get(from).unwrap();
        let is_capture = self.get(to).is_some()
            || (piece.kind == PieceKind::Pawn && self.en_passant == Some(to));
        let is_pawn_move = piece.kind == PieceKind::Pawn;

        let mut new_board = self.clone();
        new_board.make_move_unchecked(from, to);

        // Update en passant
        new_board.en_passant = None;
        if piece.kind == PieceKind::Pawn {
            let dr = (to.rank as i8 - from.rank as i8).abs();
            if dr == 2 {
                // Set en passant target to the square the pawn passed through
                let ep_rank = (from.rank as i8 + piece.color.pawn_direction()) as u8;
                new_board.en_passant = Some(Square::new(from.file, ep_rank));
            }
        }

        // Update castling rights
        // King moves: lose both castling rights
        if piece.kind == PieceKind::King {
            match piece.color {
                Color::White => {
                    new_board.castling[WK] = false;
                    new_board.castling[WQ] = false;
                }
                Color::Black => {
                    new_board.castling[BK] = false;
                    new_board.castling[BQ] = false;
                }
            }
        }
        // Rook moves or is captured: lose that side's castling
        if from == Square::new(0, 0) || to == Square::new(0, 0) {
            new_board.castling[WQ] = false;
        }
        if from == Square::new(7, 0) || to == Square::new(7, 0) {
            new_board.castling[WK] = false;
        }
        if from == Square::new(0, 7) || to == Square::new(0, 7) {
            new_board.castling[BQ] = false;
        }
        if from == Square::new(7, 7) || to == Square::new(7, 7) {
            new_board.castling[BK] = false;
        }

        // Update halfmove clock
        if is_pawn_move || is_capture {
            new_board.halfmove_clock = 0;
        } else {
            new_board.halfmove_clock = self.halfmove_clock + 1;
        }

        // Update fullmove number
        if piece.color == Color::Black {
            new_board.fullmove = self.fullmove + 1;
        }

        Some(new_board)
    }

    /// Find all pieces of a given color.
    pub fn pieces(&self, color: Color) -> Vec<(Square, Piece)> {
        let mut result = Vec::new();
        for sq in Square::variants() {
            if let Some(p) = self.get(sq)
                && p.color == color
            {
                result.push((sq, p));
            }
        }
        result
    }

    /// Is the current side in checkmate?
    pub fn is_checkmate(&self) -> bool {
        self.in_check(self.to_move) && !self.has_legal_moves(self.to_move)
    }

    /// Is the position stalemate (no legal moves but not in check)?
    pub fn is_stalemate(&self) -> bool {
        !self.in_check(self.to_move) && !self.has_legal_moves(self.to_move)
    }

    /// Is the 50-move rule triggered?
    pub fn is_fifty_move_rule(&self) -> bool {
        self.halfmove_clock >= 100
    }

    /// Does the given color have any legal moves?
    fn has_legal_moves(&self, color: Color) -> bool {
        for sq in Square::variants() {
            if let Some(p) = self.get(sq)
                && p.color == color
                && !self.legal_moves(sq).is_empty()
            {
                return true;
            }
        }
        false
    }
}
