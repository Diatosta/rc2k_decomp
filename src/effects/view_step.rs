//! `effects::view_step` (0x4C1593): the view's step since the last frame, negated: for each
//! of x, y, z `flt_7692EA` .. = -(`flt_76927A` .. less `flt_769296` ..) (`fld; fsub;
//! fchs; fstp`), then `flt_769296` .. = `flt_76927A` .. (`push`/`pop`).
//!
//! Caller: `effects::view_frame`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::math::x87;

pub fn view_step(effects: &crate::effects::owner::Effects) {
    let k = &effects.frame.kept_view;
    // now: `kept_a`'s scaled vector (`flt_76927A` ..)
    for n in 0..3 {
        k.step[n].set(x87::negated(x87::sub(&k.kept_a.scaled[n], &k.last[n])));
    }
    for n in 0..3 {
        k.last[n].set_bits(k.kept_a.scaled[n].bits());
    }
}
