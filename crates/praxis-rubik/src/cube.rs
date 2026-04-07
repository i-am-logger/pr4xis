use crate::face::{Color, Face};
use crate::moves::Move;

/// A 3×3 Rubik's cube. Each face has 9 stickers (3×3 grid).
/// Positions are indexed [row][col] where (0,0) is top-left when facing that side.
///
/// The cube state is the sticker colors. The ontology enforces that
/// only valid face rotations can change the state.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cube {
    /// faces[face][row][col] = Color
    pub faces: [[Color; 9]; 6],
}

impl Cube {
    /// Solved cube: each face is a solid color.
    pub fn solved() -> Self {
        let mut faces = [[Color::White; 9]; 6];
        for face in Face::all() {
            let color = Color::of_face(face);
            for cell in &mut faces[face as usize] {
                *cell = color;
            }
        }
        Self { faces }
    }

    /// Is this cube in the solved state?
    pub fn is_solved(&self) -> bool {
        for face in Face::all() {
            let color = self.faces[face as usize][0];
            for i in 1..9 {
                if self.faces[face as usize][i] != color {
                    return false;
                }
            }
        }
        true
    }

    /// Get the sticker at a position.
    pub fn get(&self, face: Face, pos: usize) -> Color {
        self.faces[face as usize][pos]
    }

    /// Get a face as a 3×3 array (row-major).
    pub fn face_colors(&self, face: Face) -> [Color; 9] {
        self.faces[face as usize]
    }

    /// Apply a move to this cube. Returns the new state.
    /// This is the only way to change the cube — the ontology enforcement.
    pub fn apply(&self, m: Move) -> Self {
        let mut cube = self.clone();
        let times = m.rotation_count();
        for _ in 0..times {
            cube = cube.rotate_face_cw(m.face());
        }
        cube
    }

    /// Apply a sequence of moves.
    pub fn apply_sequence(&self, moves: &[Move]) -> Self {
        moves.iter().fold(self.clone(), |c, m| c.apply(*m))
    }

    /// Rotate a face 90° clockwise and cycle the adjacent edge stickers.
    fn rotate_face_cw(&self, face: Face) -> Self {
        let mut cube = self.clone();

        // Rotate the face itself (3×3 grid, 90° CW)
        let f = face as usize;
        let old = self.faces[f];
        cube.faces[f][0] = old[6];
        cube.faces[f][1] = old[3];
        cube.faces[f][2] = old[0];
        cube.faces[f][3] = old[7];
        cube.faces[f][4] = old[4]; // center stays
        cube.faces[f][5] = old[1];
        cube.faces[f][6] = old[8];
        cube.faces[f][7] = old[5];
        cube.faces[f][8] = old[2];

        // Cycle the 4 edge strips adjacent to this face.
        // Each face rotation cycles 12 stickers on neighboring faces.
        let (a, b, c, d) = adjacent_strips(face);
        // Read all strips before writing any (borrow checker)
        let sa = Self::read_strip(&cube, &a);
        let sb = Self::read_strip(&cube, &b);
        let sc = Self::read_strip(&cube, &c);
        let sd = Self::read_strip(&cube, &d);
        // Cycle: a←d, d←c, c←b, b←a
        Self::write_strip(&mut cube, &a, &sd);
        Self::write_strip(&mut cube, &d, &sc);
        Self::write_strip(&mut cube, &c, &sb);
        Self::write_strip(&mut cube, &b, &sa);

        cube
    }

    fn read_strip(&self, strip: &[(Face, usize); 3]) -> [Color; 3] {
        [
            self.faces[strip[0].0 as usize][strip[0].1],
            self.faces[strip[1].0 as usize][strip[1].1],
            self.faces[strip[2].0 as usize][strip[2].1],
        ]
    }

    fn write_strip(cube: &mut Cube, strip: &[(Face, usize); 3], colors: &[Color; 3]) {
        cube.faces[strip[0].0 as usize][strip[0].1] = colors[0];
        cube.faces[strip[1].0 as usize][strip[1].1] = colors[1];
        cube.faces[strip[2].0 as usize][strip[2].1] = colors[2];
    }

    /// Count the total number of each color (should always be 9 of each).
    pub fn color_counts(&self) -> [u8; 6] {
        let mut counts = [0u8; 6];
        for face in Face::all() {
            for i in 0..9 {
                let c = self.faces[face as usize][i];
                counts[c as usize] += 1;
            }
        }
        counts
    }
}

/// Returns 4 strips of 3 sticker positions adjacent to the given face.
/// The strips cycle CW when the face is rotated CW: a ← d ← c ← b ← a.
#[allow(clippy::type_complexity)]
fn adjacent_strips(
    face: Face,
) -> (
    [(Face, usize); 3],
    [(Face, usize); 3],
    [(Face, usize); 3],
    [(Face, usize); 3],
) {
    // Position indices in a 3×3 face (row-major):
    // 0 1 2
    // 3 4 5
    // 6 7 8
    match face {
        Face::U => (
            [(Face::F, 0), (Face::F, 1), (Face::F, 2)],
            [(Face::L, 0), (Face::L, 1), (Face::L, 2)],
            [(Face::B, 0), (Face::B, 1), (Face::B, 2)],
            [(Face::R, 0), (Face::R, 1), (Face::R, 2)],
        ),
        Face::D => (
            [(Face::F, 6), (Face::F, 7), (Face::F, 8)],
            [(Face::R, 6), (Face::R, 7), (Face::R, 8)],
            [(Face::B, 6), (Face::B, 7), (Face::B, 8)],
            [(Face::L, 6), (Face::L, 7), (Face::L, 8)],
        ),
        Face::F => (
            [(Face::U, 6), (Face::U, 7), (Face::U, 8)],
            [(Face::R, 0), (Face::R, 3), (Face::R, 6)],
            [(Face::D, 2), (Face::D, 1), (Face::D, 0)],
            [(Face::L, 8), (Face::L, 5), (Face::L, 2)],
        ),
        Face::B => (
            [(Face::U, 2), (Face::U, 1), (Face::U, 0)],
            [(Face::L, 0), (Face::L, 3), (Face::L, 6)],
            [(Face::D, 6), (Face::D, 7), (Face::D, 8)],
            [(Face::R, 8), (Face::R, 5), (Face::R, 2)],
        ),
        Face::L => (
            [(Face::U, 0), (Face::U, 3), (Face::U, 6)],
            [(Face::F, 0), (Face::F, 3), (Face::F, 6)],
            [(Face::D, 0), (Face::D, 3), (Face::D, 6)],
            [(Face::B, 8), (Face::B, 5), (Face::B, 2)],
        ),
        Face::R => (
            [(Face::U, 8), (Face::U, 5), (Face::U, 2)],
            [(Face::B, 0), (Face::B, 3), (Face::B, 6)],
            [(Face::D, 8), (Face::D, 5), (Face::D, 2)],
            [(Face::F, 8), (Face::F, 5), (Face::F, 2)],
        ),
    }
}
