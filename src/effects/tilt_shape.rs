//! `effects::tilt_shape`: the sine and cosine of the shape tilt into `ShapeTilt.sine` /
//! `ShapeTilt.cosine`; the point's y and z (`ShapeTilt.point[1]`, `ShapeTilt.point[2]`) turned by
//! them (`x87::turn_yz`); then the point `ShapeTilt.point[0]` .. made of length 1 into
//! `ShapeTilt.unit[0]` .., its length in `ShapeTilt.length` (`x87::unit_and_length`).

use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::math::x87;

pub fn tilt_shape(drops: &WindscreenDrops) {
    let mut t = drops.tilt.borrow_mut();
    let (s, c) = x87::sine_and_cosine(t.tilt);
    t.cosine = c;
    t.sine = s;
    let (y, z) = x87::turn_yz(t.point[1], t.point[2], s, c);
    t.point[1] = y;
    t.point[2] = z;
    let (unit, length) = x87::unit_and_length(t.point.map(f32::to_bits));
    t.length = length;
    t.unit = unit;
}
