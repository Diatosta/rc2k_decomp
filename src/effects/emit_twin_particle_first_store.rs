//! The twin particles' state and tuning.

use crate::memory::session::{PlainDword, PlainFloat};

/// The twin car emitters' shared work (`emit_twin_particle_first`, `emit_twin_particle_second`).
#[derive(rc2k_derive::Cells)]
pub struct TwinState {
    /// The point the entry's place starts at (turned and placed by the car's matrix)
    pub point: [PlainFloat; 3],
    /// The car's `motion`, kept
    pub motion: [PlainDword; 3],
    /// The car's level (its `levels[0]` or `levels[1]`)
    pub level: PlainDword,
}

impl TwinState {
    /// The values the game starts with.
    pub const fn new() -> Self {
        TwinState {
            point: [
                PlainFloat::new(0.0),
                PlainFloat::new(8192.0),
                PlainFloat::new(101072.0),
            ],
            motion: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            level: PlainDword::new(0x0),
        }
    }
}

/// One twin car emitter's variables (`emit_twin_particle_first`'s and
/// `emit_twin_particle_second`'s).
#[derive(rc2k_derive::Cells)]
pub struct TwinTuning {
    /// The chance's random
    pub chance_random: PlainDword,
    /// The place's lift
    pub lift: PlainFloat,
    /// The second row's factors (z, y, x)
    pub row_k: [PlainFloat; 3],
    /// The spread `jitter_point` takes
    pub spread: PlainFloat,
    /// The motion's factors (x, y, z)
    pub motion_k: [PlainFloat; 3],
    /// The spin's random
    pub spin_random: PlainDword,
}

impl TwinTuning {
    /// The values the game starts with.
    pub(super) const fn nineteenth() -> Self {
        TwinTuning {
            chance_random: PlainDword::new(0x0),
            lift: PlainFloat::new(65536.0),
            row_k: [
                PlainFloat::new(256.0),
                PlainFloat::new(256.0),
                PlainFloat::new(256.0),
            ],
            spread: PlainFloat::new(128.0),
            motion_k: [
                PlainFloat::new(0.8),
                PlainFloat::new(0.8),
                PlainFloat::new(0.8),
            ],
            spin_random: PlainDword::new(0x0),
        }
    }
}
