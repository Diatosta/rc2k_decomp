//! `effects::copy_first_point` (0x4C1163): `dword_769306` .. `dword_76930E` into
//! `flt_811BDF` .. `flt_811BE7`, and `dword_7692F6` into `dword_811BCF` (`push`/`pop`).
//!
//! Caller: `effects::shapes_frame`.

use crate::effects::windscreen_drop_vars::WindscreenDrops;

/// The Rust version (see the header).
pub fn copy_first_point(drops: &WindscreenDrops, effects: &crate::effects::owner::Effects) {
    let frame = &effects.frame;
    for (n, to) in drops.tilt.point.iter().enumerate() {
        to.set_bits(frame.kept_view.vector[n].bits());
    }
    drops.tilt.length.set(frame.kept_view.length.get());
}
