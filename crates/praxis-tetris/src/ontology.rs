use crate::piece::PieceKind;
use praxis_category::Entity;
use praxis_ontology::Quality;

/// Tetris piece kinds as entities.
impl Entity for PieceKind {
    fn variants() -> Vec<Self> {
        PieceKind::all().to_vec()
    }
}

/// Quality: number of cells (always 4 for tetrominoes).
#[derive(Debug, Clone)]
pub struct CellCount;

impl Quality for CellCount {
    type Individual = PieceKind;
    type Value = usize;

    fn get(&self, kind: &PieceKind) -> Option<usize> {
        let piece = crate::piece::Piece::new(*kind);
        Some(piece.cells().len())
    }
}

/// Quality: number of distinct rotations.
#[derive(Debug, Clone)]
pub struct RotationCount;

impl Quality for RotationCount {
    type Individual = PieceKind;
    type Value = usize;

    fn get(&self, kind: &PieceKind) -> Option<usize> {
        let p = crate::piece::Piece::new(*kind);
        let mut distinct = std::collections::HashSet::new();
        let mut current = p.clone();
        for _ in 0..4 {
            let cells = current.cells();
            // Normalize: translate to origin
            let min_x = cells.iter().map(|c| c.0).min().unwrap();
            let min_y = cells.iter().map(|c| c.1).min().unwrap();
            let normalized: Vec<(i32, i32)> =
                cells.iter().map(|c| (c.0 - min_x, c.1 - min_y)).collect();
            let mut sorted = normalized;
            sorted.sort();
            distinct.insert(sorted);
            current = current.rotated_cw();
        }
        Some(distinct.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_7_piece_kinds() {
        assert_eq!(PieceKind::variants().len(), 7);
    }

    #[test]
    fn test_all_tetrominoes_have_4_cells() {
        let quality = CellCount;
        for kind in PieceKind::all() {
            assert_eq!(quality.get(&kind), Some(4));
        }
    }

    #[test]
    fn test_o_piece_has_1_rotation() {
        assert_eq!(RotationCount.get(&PieceKind::O), Some(1));
    }

    #[test]
    fn test_i_piece_has_2_rotations() {
        assert_eq!(RotationCount.get(&PieceKind::I), Some(2));
    }

    #[test]
    fn test_t_piece_has_4_rotations() {
        assert_eq!(RotationCount.get(&PieceKind::T), Some(4));
    }
}
