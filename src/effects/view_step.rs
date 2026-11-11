//! `effects::view_step` (0x4C1593): the view's step since the last frame, negated: for each
//! of x, y, z `KeptView.step[0]` .. = -(`KeptView.kept_a.scaled[0]` .. less `KeptView.last[0]` ..) (`fld; fsub;
//! fchs; fstp`), then `KeptView.last[0]` .. = `KeptView.kept_a.scaled[0]` .. (`push`/`pop`).
//!
//! Caller: `effects::view_frame`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::math::x87;

pub fn view_step(effects: &crate::effects::owner::Effects) {
    let k = &effects.frame.kept_view;
    // now: `kept_a`'s scaled vector (`KeptView.kept_a.scaled[0]` ..)
    for n in 0..3 {
        k.step[n].set(x87::negated(x87::sub(&k.kept_a.scaled[n], &k.last[n])));
    }
    for n in 0..3 {
        k.last[n].set_bits(k.kept_a.scaled[n].bits());
    }
}
