//! The block lines' work.

/// The factors a settings block's morph starts from, by slot.
pub const BLOCK_START: [f32; 8] = [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0];

/// The block lines' flags (`effects::lines`).
pub struct LinesWork {
    /// Whether every block is measured
    pub every: u32,
    /// Whether the faces' sums are shared
    pub shares: u32,
    /// A marked block was measured
    pub any: u32,
}

impl LinesWork {
    /// The values the game starts with.
    pub const fn new() -> Self {
        LinesWork {
            every: 0,
            shares: 0,
            any: 0,
        }
    }
}
