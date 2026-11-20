//! The rated rolling particles' set.

use crate::memory::session::{PlainDword, PlainFloat};

/// Kind 0x15's emitter variables (`emit_rolling_particles_rated`).
#[derive(rc2k_derive::Cells)]
pub struct RollingRatedSet {
    /// The rate's factors
    pub rate_k: [PlainFloat; 4],
    /// The side's random
    pub side_random: PlainDword,
    /// The place's drop in y
    pub lift: PlainFloat,
    /// Per axis (x, y, z) the factors of the first row, the third row and the side
    pub motion_k: [PlainFloat; 9],
    /// The spin's random
    pub spin_random: PlainDword,
    /// The size's factor
    pub size_k: PlainFloat,
}

impl RollingRatedSet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        RollingRatedSet {
            rate_k: [
                PlainFloat::new(65.336),
                PlainFloat::new(0.00002),
                PlainFloat::new(1.0),
                PlainFloat::new(196608.0),
            ],
            side_random: PlainDword::new(0x0),
            lift: PlainFloat::new(65536.0),
            motion_k: [
                PlainFloat::new(49.152),
                PlainFloat::new(49.152),
                PlainFloat::new(-49.152),
                PlainFloat::new(49.152),
                PlainFloat::new(49.152),
                PlainFloat::new(-49.152),
                PlainFloat::new(49.152),
                PlainFloat::new(49.152),
                PlainFloat::new(-49.152),
            ],
            spin_random: PlainDword::new(0x0),
            size_k: PlainFloat::new(0.2),
        }
    }
}
