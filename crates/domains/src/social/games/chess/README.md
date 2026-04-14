# Chess -- Chessboard Ontology

Models a chess position as a thin category over the 64 board squares with board-context qualities for piece placement, mobility, and attack. Axioms encode the invariants of a legal position: exactly one king per side, at most 32 pieces, and no move that leaves the mover's king in check.

Key references:
- FIDE Laws of Chess
- Shannon 1950: *Programming a Computer for Playing Chess* (position representation)

## Entities

| Category | Entities |
|---|---|
| Board squares (64) | `Square` (file, rank) |
| Pieces (6 kinds × 2 colors) | King, Queen, Rook, Bishop, Knight, Pawn for White and Black |

## Category

`ChessCategory` has squares as objects and `SquareConnection` (any-to-any) as morphisms — a thin complete category. Composition is ordinary path composition. The board itself is a separate rich type carried through qualities; legality of moves is checked by those qualities rather than encoded in the morphism set.

## Qualities

| Quality | Type | Description |
|---|---|---|
| PieceAt | Option<Piece> | Piece occupying a square on a given `Board` |
| Mobility | Option<usize> | Number of legal destinations from a square (None if no legal move) |
| AttackedBy | bool | Whether a square is attacked by a given color |

## Axioms (3)

| Axiom | Description | Source |
|---|---|---|
| KingSafety | No legal move leaves the moving king in check | FIDE Laws of Chess |
| OneKingPerSide | Each side has exactly one king | FIDE Laws of Chess |
| MaxPieces | At most 32 pieces on the board | FIDE Laws of Chess |

Plus the auto-generated structural axioms from category laws.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `ChessCategory`, `SquareConnection`, `PieceAt`/`Mobility`/`AttackedBy` qualities, `KingSafety`/`OneKingPerSide`/`MaxPieces` axioms, tests
- `square.rs` -- `Square` entity (file × rank)
- `piece.rs` -- `Piece`, `PieceKind`, `Color`
- `board.rs` -- `Board` with piece placement, move generation, check detection
- `moves.rs` -- `ChessMove` rich type
- `pgn.rs` -- PGN notation read/write
- `engine.rs` -- runtime chess engine / action loop
- `games/` -- reference game data
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
