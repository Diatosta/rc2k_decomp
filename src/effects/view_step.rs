//! `effects::view_step` (0x4C1593): the view's step since the last frame, negated: for each
//! of x, y, z `flt_7692EA` .. = -(`flt_76927A` .. less `flt_769296` ..) (`fld; fsub;
//! fchs; fstp`), then `flt_769296` .. = `flt_76927A` .. (`push`/`pop`).
//!
//! Caller: `effects::view_frame`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::math::x87;

/// The Rust version (see the header).
pub fn view_step(effects: &crate::effects::owner::Effects) {
    let k = &effects.frame.kept_view;
    // now: `kept_a`'s scaled vector (`flt_76927A` ..)
    for n in 0..3 {
        k.step[n].set_bits(x87::negated_sub(
            k.kept_a.scaled[n].bits(),
            k.last[n].bits(),
        ));
    }
    for n in 0..3 {
        k.last[n].set_bits(k.kept_a.scaled[n].bits());
    }
}
