use crate::board::Board;
use crate::piece::Color;
use crate::square::Square;
use praxis_engine::{Action, Engine, Precondition, PreconditionResult, Situation};

// =============================================================================
// Situation: Board
// =============================================================================

impl Situation for Board {
    fn describe(&self) -> String {
        let pieces_white = self.pieces(Color::White).len();
        let pieces_black = self.pieces(Color::Black).len();
        let check = if self.in_check(self.to_move) {
            " CHECK"
        } else {
            ""
        };
        format!(
            "move {} | white:{} black:{} | to_move:{:?}{}",
            self.fullmove, pieces_white, pieces_black, self.to_move, check
        )
    }

    fn is_terminal(&self) -> bool {
        self.is_checkmate() || self.is_stalemate() || self.is_fifty_move_rule()
    }
}

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

    fn describe(&self) -> String {
        format!("{} → {}", self.from.name(), self.to.name())
    }
}

// =============================================================================
// Preconditions: chess rules
// =============================================================================

/// Piece must exist at the source square.
pub struct PieceExists;

impl Precondition<ChessAction> for PieceExists {
    fn check(&self, board: &Board, action: &ChessAction) -> PreconditionResult {
        match board.get(action.from) {
            Some(_) => PreconditionResult::satisfied(
                "piece_exists",
                &format!("piece found at {}", action.from.name()),
            ),
            None => PreconditionResult::violated(
                "piece_exists",
                &format!("no piece at {}", action.from.name()),
                &board.describe(),
                &action.describe(),
            ),
        }
    }

    fn describe(&self) -> &str {
        "a piece must exist at the source square"
    }
}

/// Piece must belong to the side to move.
pub struct OwnPiece;

impl Precondition<ChessAction> for OwnPiece {
    fn check(&self, board: &Board, action: &ChessAction) -> PreconditionResult {
        match board.get(action.from) {
            Some(piece) if piece.color == board.to_move => PreconditionResult::satisfied(
                "own_piece",
                &format!(
                    "{:?} {:?} belongs to {:?}",
                    piece.color, piece.kind, board.to_move
                ),
            ),
            Some(piece) => PreconditionResult::violated(
                "own_piece",
                &format!(
                    "{:?} piece at {} but {:?} to move",
                    piece.color,
                    action.from.name(),
                    board.to_move
                ),
                &board.describe(),
                &action.describe(),
            ),
            None => PreconditionResult::violated(
                "own_piece",
                &format!("no piece at {}", action.from.name()),
                &board.describe(),
                &action.describe(),
            ),
        }
    }

    fn describe(&self) -> &str {
        "can only move your own pieces"
    }
}

/// Move must be legal (in the legal_moves list).
pub struct LegalMove;

impl Precondition<ChessAction> for LegalMove {
    fn check(&self, board: &Board, action: &ChessAction) -> PreconditionResult {
        let legal = board.legal_moves(action.from);
        if legal.contains(&action.to) {
            PreconditionResult::satisfied(
                "legal_move",
                &format!("{} → {} is legal", action.from.name(), action.to.name()),
            )
        } else {
            let piece = board
                .get(action.from)
                .map(|p| format!("{:?}", p.kind))
                .unwrap_or("?".into());
            let legal_targets: Vec<String> = legal.iter().map(|s| s.name()).collect();
            PreconditionResult::violated(
                "legal_move",
                &format!(
                    "{} at {} cannot move to {} (legal: [{}])",
                    piece,
                    action.from.name(),
                    action.to.name(),
                    legal_targets.join(", ")
                ),
                &board.describe(),
                &action.describe(),
            )
        }
    }

    fn describe(&self) -> &str {
        "move must follow piece movement rules (including check)"
    }
}

/// Game must not be over.
pub struct GameNotOver;

impl Precondition<ChessAction> for GameNotOver {
    fn check(&self, board: &Board, action: &ChessAction) -> PreconditionResult {
        if board.is_checkmate() {
            PreconditionResult::violated(
                "game_not_over",
                "game is over: checkmate",
                &board.describe(),
                &action.describe(),
            )
        } else if board.is_stalemate() {
            PreconditionResult::violated(
                "game_not_over",
                "game is over: stalemate",
                &board.describe(),
                &action.describe(),
            )
        } else if board.is_fifty_move_rule() {
            PreconditionResult::violated(
                "game_not_over",
                "game is over: 50-move rule",
                &board.describe(),
                &action.describe(),
            )
        } else {
            PreconditionResult::satisfied("game_not_over", "game in progress")
        }
    }

    fn describe(&self) -> &str {
        "game must not be over (checkmate, stalemate, or 50-move rule)"
    }
}

// =============================================================================
// Apply function
// =============================================================================

fn apply_chess_move(board: &Board, action: &ChessAction) -> Board {
    board
        .apply_move(action.from, action.to)
        .expect("LegalMove precondition should guarantee legality")
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
