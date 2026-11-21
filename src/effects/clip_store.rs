//! The clip's and the glow's variables:
//! `ClipScreen` and `GlowState`.

/// The clip's screen edges.
pub struct ClipScreen {
    /// The screen's middle x, middle x less one, middle y and middle y less one
    /// (`effects::init_marks` sets them; `effects::clip_quad` clips against the settings' edge).
    pub edges: [f32; 4],
}

impl ClipScreen {
    pub const fn new() -> Self {
        Self {
            edges: [400.0, 399.0, 300.0, 299.0],
        }
    }
}

/// The glow's variables (and the clip's count).
pub struct GlowState {
    /// 1 = no glow (initially 1).
    pub off: u32,
    /// The glow's place (as the sprite's x, y, z are taken: z, y, x).
    pub place: [f32; 3],
    /// The vertex count `effects::clip_quad` leaves (nothing reads it).
    pub clip_count: u32,
    /// The glow sprite's angle.
    pub angle: u32,
    /// Added to the place's z and y.
    pub offset: [f32; 2],
}

impl GlowState {
    pub const fn new() -> Self {
        Self {
            off: 0x1,
            place: [262144.0, -65536.0, -65536.0],
            clip_count: 0x0,
            angle: 0x0,
            offset: [0.0, 0.0],
        }
    }
}
