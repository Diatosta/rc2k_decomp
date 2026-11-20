//! The exhaust smoke's set.

/// Kind 0xD's emitter variables (`emit_exhaust_smoke`).
pub struct ExhaustSet {
    /// The place's lift
    pub lift: f32,
    /// The strength's factor
    pub strength_k: f32,
    /// The motion's factors (x, y, z)
    pub motion_k: [f32; 3],
    /// The drift's random
    pub drift_random: u32,
    /// The spin's random
    pub spin_random: u32,
    /// The size's factor, base and scale
    pub size_k: [f32; 3],
}

impl ExhaustSet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        ExhaustSet {
            lift: 65536.0,
            strength_k: 0.0001,
            motion_k: [-2048.0, -2048.0, -2048.0],
            drift_random: 0x0,
            spin_random: 0x0,
            size_k: [65536.0, 2048.0, 0.2],
        }
    }
}
