//! The shape tilt's work.

use crate::memory::session::PlainFloat;

/// The shape tilt and the first view point (`effects::tilt_shape`, `effects::copy_first_point`,
/// `effects::edges`).
#[derive(rc2k_derive::Cells)]
pub struct ShapeTilt {
    /// The kind's float times 0.0175.
    pub tilt: PlainFloat,
    /// The sine of the tilt.
    pub sine: PlainFloat,
    /// The cosine of the tilt.
    pub cosine: PlainFloat,
    /// The point's length.
    pub length: PlainFloat,
    /// The point made of length 1.
    pub unit: [PlainFloat; 3],
    /// The first kept view point (copied by `effects::copy_first_point`, turned by the tilt, then
    /// also set to 0 by `effects::windscreen_drops_drift` and moved by the drift).
    pub point: [PlainFloat; 3],
}

impl ShapeTilt {
    pub const fn new() -> Self {
        ShapeTilt {
            tilt: PlainFloat::new(0.0),
            sine: PlainFloat::new(0.0),
            cosine: PlainFloat::new(0.0),
            length: PlainFloat::new(0.0),
            unit: [
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
            ],
            point: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
        }
    }
}
