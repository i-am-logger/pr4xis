#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::board::Board;
use pr4xis::engine::{Action, Engine, Precondition, Situation};
use pr4xis::logic::proof::{Counterexample, SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::square::Square;

// =============================================================================
// Situation: Board (Debug + Clone + PartialEq via derive on Board)
// =============================================================================

impl Situation for Board {}

// =============================================================================
// Action: ChessAction (from square, to square)
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ChessAction {
    pub from: Square,
    pub to: Square,
}

impl ChessAction {
    pub fn new(from: Square, to: Square) -> Self {
        Self { from, to }
    }
}

impl Action for ChessAction {
    type Sit = Board;
}

/// Helper: build typed Provenance for a chess precondition axiom.
fn chess_meta(name: &'static str, description: &'static str) -> Provenance {
    Provenance {
        name: OntologyName::new_static(name),
        description: Label::new_static(description),
        // Murray (1913) is the classical source on chess rules; FIDE codifies modern.
        citation: Citation::parse_static(
            "Murray (1913) A History of Chess; FIDE Laws of Chess (2023)",
        ),
        module_path: ModulePath::new_static(module_path!()),
    }
}

// =============================================================================
// Preconditions: chess rules
// =============================================================================

/// Piece must exist at the source square.
pub struct PieceExists;

impl Precondition<ChessAction> for PieceExists {
    fn check(&self, board: &Board, action: &ChessAction) -> Verdict {
        let meta = chess_meta("PieceExists", "a piece must exist at the source square");
        match board.get(action.from) {
            Some(_) => Ok(Box::new(SimpleProof::new(meta))),
            None => Err(Box::new(SimpleCounterexample::new(meta))),
        }
    }
}

/// Piece must belong to the side to move.
pub struct OwnPiece;

impl Precondition<ChessAction> for OwnPiece {
    fn check(&self, board: &Board, action: &ChessAction) -> Verdict {
        let meta = chess_meta("OwnPiece", "can only move your own pieces");
        match board.get(action.from) {
            Some(piece) if piece.color == board.to_move => Ok(Box::new(SimpleProof::new(meta))),
            _ => Err(Box::new(SimpleCounterexample::new(meta))),
        }
    }
}

/// Move must be legal (in the legal_moves list).
pub struct LegalMove;

impl Precondition<ChessAction> for LegalMove {
    fn check(&self, board: &Board, action: &ChessAction) -> Verdict {
        let meta = chess_meta(
            "LegalMove",
            "move must follow piece movement rules (including check)",
        );
        let legal = board.legal_moves(action.from);
        if legal.contains(&action.to) {
            Ok(Box::new(SimpleProof::new(meta)))
        } else {
            Err(Box::new(SimpleCounterexample::new(meta)))
        }
    }
}

/// Game must not be over.
pub struct GameNotOver;

impl Precondition<ChessAction> for GameNotOver {
    fn check(&self, board: &Board, _action: &ChessAction) -> Verdict {
        let meta = chess_meta(
            "GameNotOver",
            "game must not be over (checkmate, stalemate, or 50-move rule)",
        );
        if board.is_checkmate() || board.is_stalemate() || board.is_fifty_move_rule() {
            Err(Box::new(SimpleCounterexample::new(meta)))
        } else {
            Ok(Box::new(SimpleProof::new(meta)))
        }
    }
}

// =============================================================================
// Apply function
// =============================================================================

fn apply_chess_move(board: &Board, action: &ChessAction) -> Result<Board, Box<dyn Counterexample>> {
    board.apply_move(action.from, action.to).ok_or_else(|| {
        let meta = chess_meta(
            "ApplyMoveFailed",
            "preconditions passed but board.apply_move rejected the move",
        );
        Box::new(SimpleCounterexample::new(meta)) as Box<dyn Counterexample>
    })
}

// =============================================================================
// Engine constructor
// =============================================================================

pub type ChessEngine = Engine<ChessAction>;

/// Create a new chess game engine from the starting position.
pub fn new_game() -> ChessEngine {
    Engine::new(
        Board::starting(),
        vec![
            Box::new(GameNotOver),
            Box::new(PieceExists),
            Box::new(OwnPiece),
            Box::new(LegalMove),
        ],
        apply_chess_move,
    )
}
