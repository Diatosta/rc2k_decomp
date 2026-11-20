//! The slip spray's set: the emitter variables of kind 5 and of kind 0xB (the same values, each
//! kind with its own random).

/// A slip emitter's variables.
pub struct SlipSpraySet {
    /// The place's random
    pub place_random: u32,
    /// The motion's x factor
    pub motion_x: f32,
    /// The motion's y factor
    pub motion_y: f32,
    /// Added to the motion's y
    pub lift: f32,
    /// The motion's z factor
    pub motion_z: f32,
    /// The spread `jitter_point` takes
    pub spread: f32,
    /// The third row's factors (z, y, x)
    pub roll_k: [f32; 3],
    /// The spin's factor
    pub spin: f32,
}

impl SlipSpraySet {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SlipSpraySet {
            place_random: 0x0,
            motion_x: 10.0,
            motion_y: 10.0,
            lift: 15.0,
            motion_z: 10.0,
            spread: 110.0,
            roll_k: [-12.0, -12.0, -12.0],
            spin: -64.0,
        }
    }
}
