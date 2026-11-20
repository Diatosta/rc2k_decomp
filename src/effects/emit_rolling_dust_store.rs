//! The rolling dust's choice and set.

use crate::memory::session::PlainDword;

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
pub struct RollingSet {
    /// The side's random
    pub side_random: u32,
    /// Per axis (x, y, z) the factors of the first row, the third row and the side
    pub motion_k: [f32; 9],
    /// The spread `jitter_point` takes
    pub spread: f32,
    /// Added to y
    pub lift: f32,
    /// The spin's random
    pub spin_random: u32,
    /// The size's factor
    pub size_k: f32,
}

impl RollingSet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        RollingSet {
            side_random: 0x0,
            motion_k: [
                65.536, 49.152, 12.152, 65.536, 49.152, 12.152, 65.536, 49.152, 12.152,
            ],
            spread: 96.0,
            lift: 64.0,
            spin_random: 0x0,
            size_k: 0.2,
        }
    }
}
