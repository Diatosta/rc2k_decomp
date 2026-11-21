//! The shape tilt's work.

/// The shape tilt and the first view point (`effects::tilt_shape`, `effects::copy_first_point`,
/// `effects::edges`).
pub struct ShapeTilt {
    /// The kind's float times 0.0175.
    pub tilt: f32,
    /// The sine of the tilt.
    pub sine: f32,
    /// The cosine of the tilt.
    pub cosine: f32,
    /// The point's length.
    pub length: f32,
    /// The point made of length 1.
    pub unit: [f32; 3],
    /// The first kept view point (copied by `effects::copy_first_point`, turned by the tilt, then
    /// also set to 0 by `effects::windscreen_drops_drift` and moved by the drift).
    pub point: [f32; 3],
}

impl ShapeTilt {
    pub const fn new() -> Self {
        ShapeTilt {
            tilt: 0.0,
            sine: 0.0,
            cosine: 0.0,
            length: 0.0,
            unit: [0.0, 1.0, 0.0],
            point: [0.0, 0.0, 1.0],
        }
    }
}
