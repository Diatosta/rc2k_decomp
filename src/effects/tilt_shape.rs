//! `effects::tilt_shape`: the sine and cosine of the shape tilt into `ShapeTilt.sine` /
//! `ShapeTilt.cosine`; the point's y and z (`ShapeTilt.point[1]`, `ShapeTilt.point[2]`) turned by
//! them (`x87::turn_yz`); then the point `ShapeTilt.point[0]` .. made of length 1 into
//! `ShapeTilt.unit[0]` .., its length in `ShapeTilt.length` (`x87::unit_and_length`).

use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::math::x87;
use crate::memory::session::{PlainFloat, read_bits, write_bits};

pub fn tilt_shape(drops: &WindscreenDrops) {
    let t = &drops.tilt;
    let (s, c) = x87::sine_and_cosine(&t.tilt);
    t.cosine.set(c);
    t.sine.set(s);
    let (y, z) = x87::turn_yz(&t.point[1], &t.point[2], s, c);
    t.point[1].set(y);
    t.point[2].set(z);
    let (unit, length) = x87::unit_and_length(read_bits(&t.point));
    t.length.set(length);
    write_bits(&t.unit, unit.map(f32::to_bits));
}

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
