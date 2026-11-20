//! The rated rolling particles' set.

/// Kind 0x15's emitter variables (`emit_rolling_particles_rated`).
pub struct RollingRatedSet {
    /// The rate's factors
    pub rate_k: [f32; 4],
    /// The side's random
    pub side_random: u32,
    /// The place's drop in y
    pub lift: f32,
    /// Per axis (x, y, z) the factors of the first row, the third row and the side
    pub motion_k: [f32; 9],
    /// The spin's random
    pub spin_random: u32,
    /// The size's factor
    pub size_k: f32,
}

impl RollingRatedSet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        RollingRatedSet {
            rate_k: [65.336, 0.00002, 1.0, 196608.0],
            side_random: 0x0,
            lift: 65536.0,
            motion_k: [
                49.152, 49.152, -49.152, 49.152, 49.152, -49.152, 49.152, 49.152, -49.152,
            ],
            spin_random: 0x0,
            size_k: 0.2,
        }
    }
}
