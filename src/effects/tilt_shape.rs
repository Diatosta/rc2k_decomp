//! `effects::tilt_shape`: the sine and cosine of the shape tilt into `ShapeTilt.sine` /
//! `ShapeTilt.cosine`; the point's y and z (`ShapeTilt.point[1]`, `ShapeTilt.point[2]`) turned by
//! them (`x87::turn_yz`); then the point `ShapeTilt.point[0]` .. made of length 1 into
//! `ShapeTilt.unit[0]` .., its length in `ShapeTilt.length` (`x87::unit_and_length`).

use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::math::x87;
use crate::memory::session::{read_bits, write_bits};

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
