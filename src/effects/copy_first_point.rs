//! `effects::copy_first_point` (0x4C1163): `KeptView.vector[0]` .. `KeptView.vector[2]` into
//! `ShapeTilt.point[0]` .. `ShapeTilt.point[2]`, and `KeptView.length` into `ShapeTilt.length` (`push`/`pop`).
//!
//! Caller: `effects::shapes_frame`.

use crate::effects::windscreen_drop_vars::WindscreenDrops;

pub fn copy_first_point(drops: &WindscreenDrops, effects: &crate::effects::owner::Effects) {
    let frame = &effects.frame;
    for (n, to) in drops.tilt.point.iter().enumerate() {
        to.set(frame.kept_view.vector[n].get());
    }
    drops.tilt.length.set(frame.kept_view.length.get());
}
