use super::board::Board;
use super::piece::PieceKind;
use super::square::Square;

/// A chess move with full context.
/// Only constructable through Board::apply_move, which enforces legality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChessMove {
    pub piece: PieceKind,
    pub from: Square,
    pub to: Square,
    pub captured: Option<PieceKind>,
    pub is_castling: bool,
    pub is_en_passant: bool,
    pub is_promotion: bool,
    pub gives_check: bool,
}

impl ChessMove {
    /// Create a move from a board context. Returns None if illegal.
    pub fn new(board: &Board, from: Square, to: Square) -> Option<Self> {
        let piece = board.get(from)?;
        if piece.color != board.to_move {
            return None;
        }
        if !board.legal_moves(from).contains(&to) {
            return None;
        }

        let captured = if piece.kind == PieceKind::Pawn && board.en_passant == Some(to) {
            Some(PieceKind::Pawn)
        } else {
            board.get(to).map(|p| p.kind)
        };

        let is_castling =
            piece.kind == PieceKind::King && (to.file as i8 - from.file as i8).abs() == 2;

        let is_en_passant = piece.kind == PieceKind::Pawn && board.en_passant == Some(to);

        let is_promotion = piece.kind == PieceKind::Pawn && (to.rank == 0 || to.rank == 7);

        let new_board = board.apply_move(from, to).unwrap();
        let gives_check = new_board.in_check(new_board.to_move);

        Some(Self {
            piece: piece.kind,
            from,
            to,
            captured,
            is_castling,
            is_en_passant,
            is_promotion,
            gives_check,
        })
    }

    pub fn name(&self) -> String {
        let piece_char = match self.piece {
            PieceKind::Pawn => String::new(),
            PieceKind::Knight => "N".to_string(),
            PieceKind::Bishop => "B".to_string(),
            PieceKind::Rook => "R".to_string(),
            PieceKind::Queen => "Q".to_string(),
            PieceKind::King => "K".to_string(),
        };
        let capture = if self.captured.is_some() { "x" } else { "" };
        let check = if self.gives_check { "+" } else { "" };
        let promo = if self.is_promotion { "=Q" } else { "" };
        if self.is_castling {
            if self.to.file > self.from.file {
                "O-O".to_string()
            } else {
                "O-O-O".to_string()
            }
        } else {
            format!(
                "{}{}{}{}{}",
                piece_char,
                capture,
                self.to.name(),
                promo,
                check
            )
        }
    }
}
