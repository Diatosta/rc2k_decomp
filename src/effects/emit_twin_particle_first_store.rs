//! The twin particles' state and tuning.

/// The twin car emitters' shared work (`emit_twin_particle_first`, `emit_twin_particle_second`).
pub struct TwinState {
    /// The point the entry's place starts at (turned and placed by the car's matrix)
    pub point: [f32; 3],
    /// The car's `motion`, kept (as bits)
    pub motion: [u32; 3],
    /// The car's level (its `levels[0]` or `levels[1]`)
    pub level: u32,
}

impl TwinState {
    /// The values the game starts with.
    pub const fn new() -> Self {
        TwinState {
            point: [0.0, 8192.0, 101072.0],
            motion: [0x0, 0x0, 0x0],
            level: 0x0,
        }
    }
}

/// One twin car emitter's variables (`emit_twin_particle_first`'s and
/// `emit_twin_particle_second`'s).
pub struct TwinTuning {
    /// The chance's random
    pub chance_random: u32,
    /// The place's lift
    pub lift: f32,
    /// The second row's factors (z, y, x)
    pub row_k: [f32; 3],
    /// The spread `jitter_point` takes
    pub spread: f32,
    /// The motion's factors (x, y, z)
    pub motion_k: [f32; 3],
    /// The spin's random
    pub spin_random: u32,
}

impl TwinTuning {
    /// The values the game starts with.
    pub(super) const fn nineteenth() -> Self {
        TwinTuning {
            chance_random: 0x0,
            lift: 65536.0,
            row_k: [256.0, 256.0, 256.0],
            spread: 128.0,
            motion_k: [0.8, 0.8, 0.8],
            spin_random: 0x0,
        }
    }
}
