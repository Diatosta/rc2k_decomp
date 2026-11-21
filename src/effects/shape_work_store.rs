//! The shape sets' work.

/// A shape set (0xAB8 bytes): the working block and each set copied from it. Each field holds
/// the two lists, A (0) and B (1).
#[derive(Clone)]
pub struct MarkShapes {
    /// Each list's farthest pair (inner, outer)
    pub farthest: [[u32; 2]; 2],
    /// A, b, c, d scaled, two zeros, the two tilts
    pub steps: [u32; 8],
    /// Each list's point count
    pub count: [u32; 2],
    /// Each list's point ids (then pointers into the object's points)
    pub ids: [[u32; 64]; 2],
    /// The face of each point of each list
    pub faces: [[u32; 64]; 2],
    /// Each list's 64 points of 12 bytes, turned
    pub turned: [[u8; 768]; 2],
    /// Each list's lowest point
    pub low: [[u32; 3]; 2],
    /// Each list's 3 x 3 turn (X times Z)
    pub matrix: [[f32; 9]; 2],
}

impl MarkShapes {
    pub const fn new() -> Self {
        MarkShapes {
            farthest: [[0; 2]; 2],
            steps: [0; 8],
            count: [0; 2],
            ids: [[0; 64]; 2],
            faces: [[0; 64]; 2],
            turned: [[0; 768]; 2],
            low: [[0; 3]; 2],
            matrix: [[0.0; 9]; 2],
        }
    }
}

pub struct ShapeWork {
    /// The turn about X (3 x 3)
    pub turn_x: [f32; 9],
    /// The turns about Z by list A's angle and by list B's
    pub turn_z: [[f32; 9]; 2],
    /// How many sets are made
    pub sets_made: u32,
    /// The working block a set is made in
    pub work: MarkShapes,
    /// The made sets (the game makes at most 19; the nominal 0x22nd would run past the range)
    pub sets: [MarkShapes; 33],
}

impl ShapeWork {
    pub const fn new() -> Self {
        ShapeWork {
            turn_x: [0.0; 9],
            turn_z: [[0.0; 9]; 2],
            sets_made: 0,
            work: MarkShapes::new(),
            sets: [const { MarkShapes::new() }; 33],
        }
    }
}
