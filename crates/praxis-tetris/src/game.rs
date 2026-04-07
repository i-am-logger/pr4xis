use crate::board::Board;
use crate::piece::{Piece, PieceKind};

/// Actions a player can take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameAction {
    MoveLeft,
    MoveRight,
    MoveDown, // soft drop
    HardDrop, // instant drop to bottom
    RotateCW,
    RotateCCW,
}

/// Result of an action — the ontology's enforcement response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionResult {
    /// Action succeeded, piece moved/rotated.
    Moved,
    /// Piece locked (hit bottom or another piece). Lines cleared.
    Locked { lines_cleared: u32 },
    /// Action rejected — would cause collision or out of bounds.
    Blocked,
    /// Game over — new piece can't spawn.
    GameOver,
}

/// A Tetris game with full rule enforcement.
#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    pub board: Board,
    pub current_piece: Option<Piece>,
    pub score: u32,
    pub level: u32,
    piece_count: u32,
    seed: u64,
    pub game_over: bool,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut game = Self {
            board: Board::new(),
            current_piece: None,
            score: 0,
            level: 1,
            piece_count: 0,
            seed,
            game_over: false,
        };
        game.spawn_piece();
        game
    }

    /// Apply a player action. Returns the enforcement result.
    pub fn act(&mut self, action: GameAction) -> ActionResult {
        if self.game_over {
            return ActionResult::GameOver;
        }

        let piece = match &self.current_piece {
            Some(p) => p.clone(),
            None => return ActionResult::GameOver,
        };

        match action {
            GameAction::MoveLeft => self.try_move(&piece, -1, 0),
            GameAction::MoveRight => self.try_move(&piece, 1, 0),
            GameAction::MoveDown => {
                let moved = piece.moved(0, -1);
                if self.board.piece_fits(&moved) {
                    self.current_piece = Some(moved);
                    ActionResult::Moved
                } else {
                    self.lock_current()
                }
            }
            GameAction::HardDrop => {
                let mut p = piece;
                while self.board.piece_fits(&p.moved(0, -1)) {
                    p = p.moved(0, -1);
                }
                self.current_piece = Some(p);
                self.lock_current()
            }
            GameAction::RotateCW => {
                let rotated = piece.rotated_cw();
                if self.board.piece_fits(&rotated) {
                    self.current_piece = Some(rotated);
                    ActionResult::Moved
                } else {
                    // Wall kick: try shifting left/right
                    self.try_wall_kick(&rotated)
                }
            }
            GameAction::RotateCCW => {
                let rotated = piece.rotated_ccw();
                if self.board.piece_fits(&rotated) {
                    self.current_piece = Some(rotated);
                    ActionResult::Moved
                } else {
                    self.try_wall_kick(&rotated)
                }
            }
        }
    }

    /// Apply gravity (one step down). Call this on each tick.
    pub fn tick(&mut self) -> ActionResult {
        self.act(GameAction::MoveDown)
    }

    fn try_move(&mut self, piece: &Piece, dx: i32, dy: i32) -> ActionResult {
        let moved = piece.moved(dx, dy);
        if self.board.piece_fits(&moved) {
            self.current_piece = Some(moved);
            ActionResult::Moved
        } else {
            ActionResult::Blocked
        }
    }

    fn try_wall_kick(&mut self, rotated: &Piece) -> ActionResult {
        for dx in [-1, 1, -2, 2] {
            let kicked = rotated.moved(dx, 0);
            if self.board.piece_fits(&kicked) {
                self.current_piece = Some(kicked);
                return ActionResult::Moved;
            }
        }
        ActionResult::Blocked
    }

    fn lock_current(&mut self) -> ActionResult {
        if let Some(piece) = self.current_piece.take() {
            let _ = self.board.lock_piece(&piece);
            let lines = self.board.clear_lines();
            self.score += match lines {
                1 => 100 * self.level,
                2 => 300 * self.level,
                3 => 500 * self.level,
                4 => 800 * self.level, // Tetris!
                _ => 0,
            };
            self.spawn_piece();
            if self.game_over {
                ActionResult::GameOver
            } else {
                ActionResult::Locked {
                    lines_cleared: lines,
                }
            }
        } else {
            ActionResult::GameOver
        }
    }

    fn spawn_piece(&mut self) {
        self.piece_count += 1;
        let kind = self.next_piece_kind();
        let piece = Piece::new(kind);
        if self.board.piece_fits(&piece) {
            self.current_piece = Some(piece);
        } else {
            self.game_over = true;
            self.current_piece = None;
        }
    }

    fn next_piece_kind(&self) -> PieceKind {
        let hash = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.piece_count as u64);
        PieceKind::all()[(hash % 7) as usize]
    }
}
