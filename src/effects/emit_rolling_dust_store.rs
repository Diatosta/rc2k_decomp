//! The rolling dust's choice and set.

use crate::memory::session::{PlainDword, PlainFloat};

/// The wheel kind 1's choice.
#[derive(rc2k_derive::Cells)]
pub struct RollingChoice {
    /// `emit_rolling_particles_rated` when not 0 (`ring_frame` sets it from the rain's drops)
    pub choice: PlainDword,
}

impl RollingChoice {
    /// The values the game starts with.
    pub const fn new() -> Self {
        RollingChoice {
            choice: PlainDword::new(0x0),
        }
    }
}

/// Kind 1's emitter variables (`emit_particle_1`).
#[derive(rc2k_derive::Cells)]
pub struct RollingSet {
    /// The side's random
    pub side_random: PlainDword,
    /// Per axis (x, y, z) the factors of the first row, the third row and the side
    pub motion_k: [PlainFloat; 9],
    /// The spread `jitter_point` takes
    pub spread: PlainFloat,
    /// Added to y
    pub lift: PlainFloat,
    /// The spin's random
    pub spin_random: PlainDword,
    /// The size's factor
    pub size_k: PlainFloat,
}

impl RollingSet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        RollingSet {
            side_random: PlainDword::new(0x0),
            motion_k: [
                PlainFloat::new(65.536),
                PlainFloat::new(49.152),
                PlainFloat::new(12.152),
                PlainFloat::new(65.536),
                PlainFloat::new(49.152),
                PlainFloat::new(12.152),
                PlainFloat::new(65.536),
                PlainFloat::new(49.152),
                PlainFloat::new(12.152),
            ],
            spread: PlainFloat::new(96.0),
            lift: PlainFloat::new(64.0),
            spin_random: PlainDword::new(0x0),
            size_k: PlainFloat::new(0.2),
        }
    }
}
