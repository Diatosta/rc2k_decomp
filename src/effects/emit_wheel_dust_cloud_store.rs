//! The wheel dust cloud's set.

/// Kind 6's emitter variables (`emit_wheel_dust_cloud`).
pub struct WheelFadeSet {
    /// The rate's factors
    pub rate_k: [f32; 4],
    /// The side's random
    pub side_random: u32,
    /// The place's drop in y
    pub lift: f32,
    /// Per axis (x, y, z) the scales of the three products (the wheel's `local_velocity_x`,
    /// `local_velocity_z`, `driven_speed`)
    pub motion_k: [f32; 9],
    /// The spin's random
    pub spin_random: u32,
    /// The size's factors
    pub size_k: [f32; 2],
}

impl WheelFadeSet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        WheelFadeSet {
            rate_k: [65.336, 0.0002, 1.0, 196608.0],
            side_random: 0x0,
            lift: 65536.0,
            motion_k: [
                65.536, -49.152, 26.152, 65.536, -49.152, 26.152, 65.536, -49.152, 26.152,
            ],
            spin_random: 0x0,
            size_k: [0.3, 8192.0],
        }
    }
}
