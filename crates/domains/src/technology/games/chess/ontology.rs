use super::board::Board;
use super::piece::{Color, Piece, PieceKind};
use super::square::Square;
use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Quality};

// =============================================================================
// Relationship: SquareConnection (any square to any square — thin category)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SquareConnection {
    pub from: Square,
    pub to: Square,
}

impl Relationship for SquareConnection {
    type Object = Square;
    fn source(&self) -> Square {
        self.from
    }
    fn target(&self) -> Square {
        self.to
    }
}

// =============================================================================
// Category: ChessCategory (squares + connections)
// =============================================================================

pub struct ChessCategory;

impl Category for ChessCategory {
    type Object = Square;
    type Morphism = SquareConnection;

    fn identity(obj: &Square) -> SquareConnection {
        SquareConnection {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &SquareConnection, g: &SquareConnection) -> Option<SquareConnection> {
        if f.to != g.from {
            return None;
        }
        Some(SquareConnection {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<SquareConnection> {
        let squares = Square::variants();
        let mut m = Vec::new();
        for &a in &squares {
            for &b in &squares {
                m.push(SquareConnection { from: a, to: b });
            }
        }
        m
    }
}

// =============================================================================
// Qualities (context-dependent on a Board)
// =============================================================================

#[derive(Debug, Clone)]
pub struct PieceAt {
    pub board: Board,
}

impl Quality for PieceAt {
    type Individual = Square;
    type Value = Piece;

    fn get(&self, sq: &Square) -> Option<Piece> {
        self.board.get(*sq)
    }
}

#[derive(Debug, Clone)]
pub struct Mobility {
    pub board: Board,
}

impl Quality for Mobility {
    type Individual = Square;
    type Value = usize;

    fn get(&self, sq: &Square) -> Option<usize> {
        let moves = self.board.legal_moves(*sq);
        if moves.is_empty() {
            None
        } else {
            Some(moves.len())
        }
    }
}

#[derive(Debug, Clone)]
pub struct AttackedBy {
    pub board: Board,
    pub by_color: Color,
}

impl Quality for AttackedBy {
    type Individual = Square;
    type Value = bool;

    fn get(&self, sq: &Square) -> Option<bool> {
        Some(self.board.is_attacked_by(*sq, self.by_color))
    }
}

// =============================================================================
// Axioms: concrete for ChessCategory
// =============================================================================

pub struct KingSafety {
    pub board: Board,
}

impl Axiom for KingSafety {
    fn description(&self) -> &str {
        "no legal move leaves the king in check"
    }

    fn holds(&self) -> bool {
        let color = self.board.to_move;
        for sq in Square::variants() {
            for to in self.board.legal_moves(sq) {
                if let Some(new_board) = self.board.apply_move(sq, to)
                    && new_board.in_check(color)
                {
                    return false;
                }
            }
        }
        true
    }
}

pub struct OneKingPerSide {
    pub board: Board,
}

impl Axiom for OneKingPerSide {
    fn description(&self) -> &str {
        "each side has exactly one king"
    }

    fn holds(&self) -> bool {
        let wk = self
            .board
            .pieces(Color::White)
            .iter()
            .filter(|(_, p)| p.kind == PieceKind::King)
            .count();
        let bk = self
            .board
            .pieces(Color::Black)
            .iter()
            .filter(|(_, p)| p.kind == PieceKind::King)
            .count();
        wk == 1 && bk == 1
    }
}

pub struct MaxPieces {
    pub board: Board,
}

impl Axiom for MaxPieces {
    fn description(&self) -> &str {
        "at most 32 pieces on the board"
    }

    fn holds(&self) -> bool {
        let total = self.board.pieces(Color::White).len() + self.board.pieces(Color::Black).len();
        total <= 32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chess_category_has_64_squares() {
        assert_eq!(Square::variants().len(), 64);
    }

    #[test]
    fn test_piece_at_quality() {
        let quality = PieceAt {
            board: Board::starting(),
        };
        let king = quality.get(&Square::new(4, 0));
        assert_eq!(king.unwrap().kind, PieceKind::King);
        assert!(quality.get(&Square::new(4, 3)).is_none());
    }

    #[test]
    fn test_mobility_quality() {
        let quality = Mobility {
            board: Board::starting(),
        };
        assert_eq!(quality.get(&Square::new(1, 0)), Some(2)); // knight b1
        assert_eq!(quality.get(&Square::new(0, 0)), None); // rook blocked
    }

    #[test]
    fn test_attacked_by_quality() {
        let quality = AttackedBy {
            board: Board::starting(),
            by_color: Color::White,
        };
        assert_eq!(quality.get(&Square::new(3, 2)), Some(true)); // d3 attacked by e2 pawn
    }

    #[test]
    fn test_king_safety_axiom() {
        assert!(
            KingSafety {
                board: Board::starting()
            }
            .holds()
        );
    }

    #[test]
    fn test_one_king_axiom() {
        assert!(
            OneKingPerSide {
                board: Board::starting()
            }
            .holds()
        );
    }

    #[test]
    fn test_max_pieces_axiom() {
        assert!(
            MaxPieces {
                board: Board::starting()
            }
            .holds()
        );
    }

    #[test]
    fn test_occupied_squares() {
        let quality = PieceAt {
            board: Board::starting(),
        };
        let occupied = quality.individuals_with();
        assert_eq!(occupied.len(), 32); // 16 white + 16 black
    }
}
