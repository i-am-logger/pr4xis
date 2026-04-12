use super::board::Board;
use super::piece::{Color, PieceKind};
use super::square::Square;
use pr4xis::category::Entity;

/// Parse a PGN game string into a sequence of (from, to) coordinate moves.
/// Handles SAN notation: e4, Nf3, Bxc4, O-O, O-O-O, Qh5+, Rd8#, etc.
pub fn parse_pgn_moves(pgn: &str) -> Result<Vec<(Square, Square)>, String> {
    let mut board = Board::starting();
    let mut moves = Vec::new();

    // Extract just the move text (skip headers in [brackets])
    let move_text = pgn
        .lines()
        .filter(|line| !line.starts_with('[') && !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    // Tokenize: split on whitespace, filter out move numbers and results
    let tokens: Vec<&str> = move_text
        .split_whitespace()
        .filter(|t| !t.ends_with('.') && *t != "1-0" && *t != "0-1" && *t != "1/2-1/2" && *t != "*")
        .collect();

    for token in tokens {
        let san = token
            .trim_end_matches('+')
            .trim_end_matches('#')
            .trim_end_matches('!')
            .trim_end_matches('?');

        if san.is_empty() {
            continue;
        }

        let (from, to) = resolve_san(&board, san)?;
        moves.push((from, to));

        board = board
            .apply_move(from, to)
            .ok_or_else(|| format!("illegal move: {} ({}→{})", san, from.name(), to.name()))?;
    }

    Ok(moves)
}

/// Resolve SAN notation to (from, to) squares given current board.
fn resolve_san(board: &Board, san: &str) -> Result<(Square, Square), String> {
    // Castling
    if san == "O-O" || san == "0-0" {
        let rank = if board.to_move == Color::White { 0 } else { 7 };
        return Ok((Square::new(4, rank), Square::new(6, rank)));
    }
    if san == "O-O-O" || san == "0-0-0" {
        let rank = if board.to_move == Color::White { 0 } else { 7 };
        return Ok((Square::new(4, rank), Square::new(2, rank)));
    }

    let bytes = san.as_bytes();
    let len = bytes.len();

    // Strip promotion suffix (=Q, =R, etc.)
    let san_clean: &str = if len >= 2 && bytes[len - 2] == b'=' {
        &san[..len - 2]
    } else {
        san
    };

    let bytes = san_clean.as_bytes();
    let len = bytes.len();

    if len < 2 {
        return Err(format!("move too short: {}", san));
    }

    // Determine target square (always the last two chars)
    let target_file = bytes[len - 2] - b'a';
    let target_rank = bytes[len - 1] - b'1';
    if target_file >= 8 || target_rank >= 8 {
        return Err(format!("invalid target square in: {}", san));
    }
    let to = Square::new(target_file, target_rank);

    // Determine piece kind
    let first = bytes[0];
    let (piece_kind, disambig_start) = if first.is_ascii_uppercase() {
        let kind = match first {
            b'K' => PieceKind::King,
            b'Q' => PieceKind::Queen,
            b'R' => PieceKind::Rook,
            b'B' => PieceKind::Bishop,
            b'N' => PieceKind::Knight,
            _ => return Err(format!("unknown piece: {}", first as char)),
        };
        (kind, 1)
    } else {
        (PieceKind::Pawn, 0)
    };

    // Parse disambiguation (file, rank, or both) and capture marker
    let middle = &san_clean[disambig_start..len - 2];
    let middle = middle.trim_start_matches('x'); // remove capture marker
    let disambig_file: Option<u8> = middle
        .bytes()
        .find(|b| b.is_ascii_lowercase())
        .map(|b| b - b'a');
    let disambig_rank: Option<u8> = middle
        .bytes()
        .find(|b| b.is_ascii_digit())
        .map(|b| b - b'1');

    // For pawn captures, the first char is the source file
    let pawn_source_file: Option<u8> =
        if piece_kind == PieceKind::Pawn && disambig_start == 0 && san_clean.contains('x') {
            Some(bytes[0] - b'a')
        } else {
            disambig_file
        };

    // Find the piece that can legally move to the target
    let candidates: Vec<Square> = Square::variants()
        .into_iter()
        .filter(|&sq| {
            if let Some(piece) = board.get(sq) {
                if piece.kind != piece_kind || piece.color != board.to_move {
                    return false;
                }
                // Check disambiguation
                if piece_kind == PieceKind::Pawn {
                    if let Some(f) = pawn_source_file
                        && sq.file != f
                    {
                        return false;
                    }
                } else {
                    if let Some(f) = disambig_file
                        && sq.file != f
                    {
                        return false;
                    }
                    if let Some(r) = disambig_rank
                        && sq.rank != r
                    {
                        return false;
                    }
                }
                // Check if the piece can legally reach the target
                board.legal_moves(sq).contains(&to)
            } else {
                false
            }
        })
        .collect();

    match candidates.len() {
        0 => Err(format!(
            "no {:?} can move to {} (san: {}, to_move: {:?})",
            piece_kind,
            to.name(),
            san,
            board.to_move
        )),
        1 => Ok((candidates[0], to)),
        _ => Err(format!(
            "ambiguous: {} {:?}s can move to {} (san: {})",
            candidates.len(),
            piece_kind,
            to.name(),
            san
        )),
    }
}

/// Replay a PGN game through the engine. Returns the final board or error.
pub fn replay_pgn(pgn: &str) -> Result<Board, String> {
    let moves = parse_pgn_moves(pgn)?;
    let mut board = Board::starting();
    for (i, (from, to)) in moves.iter().enumerate() {
        board = board
            .apply_move(*from, *to)
            .ok_or_else(|| format!("move {} illegal: {}→{}", i + 1, from.name(), to.name()))?;
    }
    Ok(board)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_game() {
        let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bc4";
        let moves = parse_pgn_moves(pgn).unwrap();
        assert_eq!(moves.len(), 5);
    }

    #[test]
    fn test_parse_scholars_mate() {
        let pgn = "1. e4 e5 2. Bc4 Nc6 3. Qh5 Nf6 4. Qxf7#";
        let board = replay_pgn(pgn).unwrap();
        assert!(board.is_checkmate());
    }

    #[test]
    fn test_parse_fools_mate() {
        let pgn = "1. f3 e5 2. g4 Qh4#";
        let board = replay_pgn(pgn).unwrap();
        assert!(board.is_checkmate());
    }

    #[test]
    fn test_parse_castling() {
        let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bc4 Nf6 4. O-O";
        let moves = parse_pgn_moves(pgn).unwrap();
        assert_eq!(moves.len(), 7);
    }

    #[test]
    fn test_parse_with_headers() {
        let pgn = r#"[Event "Test"]
[Site "Internet"]
[Date "2024.01.01"]

1. e4 e5 2. Nf3 Nc6"#;
        let moves = parse_pgn_moves(pgn).unwrap();
        assert_eq!(moves.len(), 4);
    }

    #[test]
    fn test_parse_with_result() {
        let pgn = "1. e4 e5 2. Bc4 Nc6 3. Qh5 Nf6 4. Qxf7# 1-0";
        let board = replay_pgn(pgn).unwrap();
        assert!(board.is_checkmate());
    }

    // =========================================================================
    // Famous games from PGN files
    // =========================================================================

    fn load_game(filename: &str) -> String {
        let path = format!(
            "{}/src/social/games/chess/games/{}",
            env!("CARGO_MANIFEST_DIR"),
            filename
        );
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {}", path, e))
    }

    #[test]
    fn test_pgn_fools_mate() {
        let pgn = load_game("fools_mate.pgn");
        let board = replay_pgn(&pgn).unwrap();
        assert!(board.is_checkmate());
    }

    #[test]
    fn test_pgn_scholars_mate() {
        let pgn = load_game("scholars_mate.pgn");
        let board = replay_pgn(&pgn).unwrap();
        assert!(board.is_checkmate());
    }

    #[test]
    fn test_pgn_opera_game() {
        let pgn = load_game("opera_game.pgn");
        let board = replay_pgn(&pgn).unwrap();
        assert!(
            board.is_checkmate(),
            "Opera Game should end in checkmate (Rd8#)"
        );
    }

    #[test]
    fn test_pgn_immortal_game() {
        let pgn = load_game("immortal_game.pgn");
        let board = replay_pgn(&pgn).unwrap();
        assert!(
            board.is_checkmate(),
            "Immortal Game should end in checkmate (Be7#)"
        );
    }

    #[test]
    fn test_pgn_evergreen_game() {
        let pgn = load_game("evergreen_game.pgn");
        let board = replay_pgn(&pgn).unwrap();
        assert!(
            board.is_checkmate(),
            "Evergreen Game should end in checkmate (Bxe7#)"
        );
    }
}
