//! Chess — piece taxonomy + position invariants.
//!
//! This ontology models the abstract concepts of a chess position: the
//! six piece kinds, the two colours, and the canonical position-level
//! invariants (one king per side, ≤32 pieces, no move leaves the mover
//! in check). The rich `Square` / `Piece` / `Board` types in the sibling
//! modules carry the 64-square coordinate system and move algebra; this
//! ontology is the upper-layer categorical view used by Praxis-level
//! reasoning.
//!
//! # Literature
//!
//! - **FIDE Laws of Chess** (current edition) — Articles 2 (initial
//!   position: 16 pieces per side, one king each), 3 (movement of
//!   pieces, including the prohibition on leaving one's king in check),
//!   and the prohibition on a side having more than 16 pieces.
//! - **Shannon (1950)** "Programming a Computer for Playing Chess"
//!   *Philosophical Magazine* 41(314) — foundational position
//!   representation (12-piece × 64-square) and the legal-move /
//!   check-safety formulation that all modern engines inherit.

use super::board::Board;
use super::piece::{Color, Piece, PieceKind};
use super::square::Square;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Chess",
    source: "FIDE Laws of Chess (current edition); Shannon (1950) Programming a Computer for Playing Chess, Philosophical Magazine 41(314)",

    concepts: [
        // The six piece kinds (FIDE Article 3).
        King, Queen, Rook, Bishop, Knight, Pawn,

        // Abstract piece grouping (FIDE Article 3: the six kinds).
        ChessPiece,

        // Movement classes (Shannon 1950): pieces grouped by their
        // line-of-action geometry.
        SlidingPiece,
        LeapingPiece,

        // Position-level concepts.
        ChessSquare,
        ChessBoard,
        ChessMove,
        WhiteSide,
        BlackSide,
    ],

    labels: {
        King: ("en", "King",
            "FIDE Article 3.8: the king moves one square in any direction; capture of the king ends the game (check / checkmate)."),
        Queen: ("en", "Queen",
            "FIDE Article 3.4: the queen moves any number of vacant squares along a rank, file, or diagonal — sliding in all eight directions."),
        Rook: ("en", "Rook",
            "FIDE Article 3.3: the rook moves any number of vacant squares along a rank or file — sliding orthogonally."),
        Bishop: ("en", "Bishop",
            "FIDE Article 3.2: the bishop moves any number of vacant squares along a diagonal — sliding diagonally."),
        Knight: ("en", "Knight",
            "FIDE Article 3.6: the knight moves in an L-shape (two squares along one axis, then one square perpendicular); it leaps over intervening pieces."),
        Pawn: ("en", "Pawn",
            "FIDE Article 3.7: the pawn moves one square forward (or two from its starting rank), captures diagonally, and promotes on the eighth rank."),

        ChessPiece: ("en", "Chess piece",
            "FIDE Article 3: any of the six piece kinds (king, queen, rook, bishop, knight, pawn)."),
        SlidingPiece: ("en", "Sliding piece",
            "Shannon (1950): a piece whose move generation follows a line of vacant squares until blocked (queen, rook, bishop)."),
        LeapingPiece: ("en", "Leaping piece",
            "Shannon (1950): a piece whose move is fixed-displacement and not blocked by intervening squares (knight, king, pawn — pawn within its limited displacement set)."),

        ChessSquare: ("en", "Square",
            "FIDE Article 2.1: one of the 64 squares (file × rank) on the board."),
        ChessBoard: ("en", "Board",
            "FIDE Article 2: the 8×8 grid of 64 squares plus the side-to-move, castling-rights, and en-passant context."),
        ChessMove: ("en", "Move",
            "FIDE Article 3: the displacement of one piece from a source square to a destination square (possibly with capture, promotion, castling, or en-passant)."),
        WhiteSide: ("en", "White",
            "FIDE Article 2.2: the side that moves first; conventionally assigned the lighter pieces."),
        BlackSide: ("en", "Black",
            "FIDE Article 2.2: the side that moves second."),
    },

    is_a: [
        // The six piece kinds subsume to ChessPiece (FIDE Article 3).
        (King, ChessPiece),
        (Queen, ChessPiece),
        (Rook, ChessPiece),
        (Bishop, ChessPiece),
        (Knight, ChessPiece),
        (Pawn, ChessPiece),

        // Shannon (1950) movement-class taxonomy.
        (Queen, SlidingPiece),
        (Rook, SlidingPiece),
        (Bishop, SlidingPiece),
        (Knight, LeapingPiece),
        (King, LeapingPiece),
        (Pawn, LeapingPiece),

        // Movement classes are themselves chess pieces.
        (SlidingPiece, ChessPiece),
        (LeapingPiece, ChessPiece),
    ],

    has_a: [
        // FIDE Article 2: a board has 64 squares.
        (ChessBoard, ChessSquare),

        // FIDE Article 3: a move involves two squares (source, destination)
        // and a piece.
        (ChessMove, ChessSquare),
        (ChessMove, ChessPiece),
    ],

    opposes: [
        // FIDE Article 2.2: the two sides oppose each other (alternate moves).
        (WhiteSide, BlackSide),
        (BlackSide, WhiteSide),

        // Shannon (1950) movement-class disjointness: a piece moves either
        // as a slider or as a leaper, not both.
        (SlidingPiece, LeapingPiece),
        (LeapingPiece, SlidingPiece),
    ],
}

/// Quality: position-level data attached to each square — piece occupancy
/// in a given `Board` context.
///
/// This is the bridge from the abstract ontology to the rich-type runtime:
/// the ontology says "ChessBoard has-a ChessSquare"; this quality reifies
/// that for a concrete `Board` by mapping each `Square` to its `Piece`
/// (if any).
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

/// Quality: number of legal destinations from a square on a given board.
/// None when the square is empty or has no legal move.
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

/// Quality: whether a square is attacked by a given colour.
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

/// Quality: which Shannon (1950) movement class each concept belongs to.
/// Total over piece-kind concepts; None on non-piece concepts.
#[derive(Debug, Clone)]
pub struct ShannonMovementClass;

impl Quality for ShannonMovementClass {
    type Individual = ChessConcept;
    type Value = &'static str;

    fn get(&self, c: &ChessConcept) -> Option<&'static str> {
        Some(match c {
            ChessConcept::Queen | ChessConcept::Rook | ChessConcept::Bishop => "sliding",
            ChessConcept::King | ChessConcept::Knight | ChessConcept::Pawn => "leaping",
            _ => return None,
        })
    }
}

impl Ontology for ChessOntology {
    type Cat = ChessCategory;
    type Qual = ShannonMovementClass;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        // Structural axioms are derived from the category. Domain axioms
        // are position-dependent (parameterised by `Board`) so they are
        // not pushed here — call them with a specific board:
        //   `KingSafety { board }.verify()`.
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

/// Axiom: in a legal position, no move available to the side to move
/// leaves that side's king in check (FIDE Article 3.9).
///
/// "It is illegal to leave one's own king attacked by an opponent piece
/// after one's own move." — FIDE Laws of Chess Article 3.9. Verified by
/// generating every legal move for the side to move and checking the
/// resulting position has the mover NOT in check.
pub struct KingSafety {
    pub board: Board,
}

impl Axiom for KingSafety {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::FinitelyGenerated;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let color = self.board.to_move;
        for sq in Square::variants() {
            for to in self.board.legal_moves(sq) {
                if let Some(new_board) = self.board.apply_move(sq, to)
                    && new_board.in_check(color)
                {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "KingSafety",
        "no legal move leaves the moving side's king in check",
        "FIDE Laws of Chess Article 3.9"
    );
}

pr4xis::register_axiom!(KingSafety, "FIDE Laws of Chess Article 3.9");

/// Axiom: each side has exactly one king (FIDE Article 2.1).
///
/// "Each player [...] sixteen pieces, of which one is a king." — FIDE
/// Article 2.1 (initial-position arrangement; the king is not capturable
/// in legal play, so this count is invariant throughout the game).
pub struct OneKingPerSide {
    pub board: Board,
}

impl Axiom for OneKingPerSide {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
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
        if wk == 1 && bk == 1 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "OneKingPerSide",
        "each side has exactly one king",
        "FIDE Laws of Chess Article 2.1"
    );
}

pr4xis::register_axiom!(OneKingPerSide, "FIDE Laws of Chess Article 2.1");

/// Axiom: at most 32 pieces on the board (FIDE Article 2.1).
///
/// 16 white pieces + 16 black pieces in the initial position; captures
/// reduce the count, so the total never exceeds 32.
pub struct MaxPieces {
    pub board: Board,
}

impl Axiom for MaxPieces {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let total = self.board.pieces(Color::White).len() + self.board.pieces(Color::Black).len();
        if total <= 32 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "MaxPieces",
        "at most 32 pieces on the board",
        "FIDE Laws of Chess Article 2.1"
    );
}

pr4xis::register_axiom!(MaxPieces, "FIDE Laws of Chess Article 2.1");

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ChessCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ChessOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fourteen_concepts() {
        // 6 piece kinds + ChessPiece + SlidingPiece + LeapingPiece +
        // ChessSquare + ChessBoard + ChessMove + WhiteSide + BlackSide.
        assert_eq!(ChessConcept::variants().len(), 14);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn piece_kinds_subsume_chess_piece() {
        let sub: Vec<_> = ChessCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ChessRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for kind in [
            ChessConcept::King,
            ChessConcept::Queen,
            ChessConcept::Rook,
            ChessConcept::Bishop,
            ChessConcept::Knight,
            ChessConcept::Pawn,
        ] {
            assert!(
                sub.contains(&(kind, ChessConcept::ChessPiece)),
                "{:?} should subsume ChessPiece",
                kind
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn shannon_sliding_vs_leaping() {
        // Shannon (1950): Q/R/B are sliders; N/K/P are leapers.
        let sub: Vec<_> = ChessCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ChessRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for slider in [
            ChessConcept::Queen,
            ChessConcept::Rook,
            ChessConcept::Bishop,
        ] {
            assert!(
                sub.contains(&(slider, ChessConcept::SlidingPiece)),
                "{:?} should be a sliding piece",
                slider
            );
        }
        for leaper in [ChessConcept::King, ChessConcept::Knight, ChessConcept::Pawn] {
            assert!(
                sub.contains(&(leaper, ChessConcept::LeapingPiece)),
                "{:?} should be a leaping piece",
                leaper
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn two_sides_oppose() {
        let opposed: Vec<_> = ChessCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ChessRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opposed.contains(&(ChessConcept::WhiteSide, ChessConcept::BlackSide)));
        assert!(opposed.contains(&(ChessConcept::BlackSide, ChessConcept::WhiteSide)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn piece_at_quality_works() {
        let q = PieceAt {
            board: Board::starting(),
        };
        let king = q.get(&Square::new(4, 0));
        assert_eq!(king.unwrap().kind, PieceKind::King);
        assert!(q.get(&Square::new(4, 3)).is_none());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mobility_quality_works() {
        let q = Mobility {
            board: Board::starting(),
        };
        assert_eq!(q.get(&Square::new(1, 0)), Some(2)); // knight b1
        assert_eq!(q.get(&Square::new(0, 0)), None); // rook blocked
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn attacked_by_quality_works() {
        let q = AttackedBy {
            board: Board::starting(),
            by_color: Color::White,
        };
        assert_eq!(q.get(&Square::new(3, 2)), Some(true)); // d3 attacked by e2 pawn
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn king_safety_holds_on_starting_position() {
        match (KingSafety {
            board: Board::starting(),
        })
        .verify()
        {
            Ok(_) => {}
            Err(c) => panic!(
                "KingSafety failed on starting position: {}",
                c.meta().name.as_str()
            ),
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn one_king_per_side_on_start() {
        assert!(
            OneKingPerSide {
                board: Board::starting()
            }
            .verify()
            .is_ok()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn max_pieces_on_start() {
        assert!(
            MaxPieces {
                board: Board::starting()
            }
            .verify()
            .is_ok()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn shannon_movement_class_total_on_pieces() {
        let q = ShannonMovementClass;
        for kind in [
            ChessConcept::King,
            ChessConcept::Queen,
            ChessConcept::Rook,
            ChessConcept::Bishop,
            ChessConcept::Knight,
            ChessConcept::Pawn,
        ] {
            assert!(q.get(&kind).is_some(), "{:?} missing movement class", kind);
        }
    }

    fn arb_concept() -> impl Strategy<Value = ChessConcept> {
        proptest::sample::select(ChessConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in ChessCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ChessOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = ChessConcept::variants();
            for m in ChessCategory::morphisms() {
                if m.kind() == ChessRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        #[test]
        fn prop_shannon_movement_class_total_on_pieces(c in arb_concept()) {
            // Total on the six piece kinds, None on the rest.
            let v = ShannonMovementClass.get(&c);
            let is_kind = matches!(
                c,
                ChessConcept::King | ChessConcept::Queen | ChessConcept::Rook
                | ChessConcept::Bishop | ChessConcept::Knight | ChessConcept::Pawn
            );
            prop_assert_eq!(v.is_some(), is_kind);
        }

        #[test]
        fn prop_one_king_per_side_invariant_under_starting_position(_seed in any::<u32>()) {
            let axiom = OneKingPerSide { board: Board::starting() };
            prop_assert!(axiom.verify().is_ok());
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_subsumption_targets_valid, Verifiable);
    pr4xis::register_praxis_value!(prop_shannon_movement_class_total_on_pieces, Verifiable);
    pr4xis::register_praxis_value!(
        prop_one_king_per_side_invariant_under_starting_position,
        Verifiable
    );
}
