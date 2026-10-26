//! `marks::view_step` (0x4C1593): the view's step since the last frame, negated: for each
//! of x, y, z `flt_7692EA` .. = -(`flt_76927A` .. less `flt_769296` ..) (`fld; fsub;
//! fchs; fstp`), then `flt_769296` .. = `flt_76927A` .. (`push`/`pop`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1270`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::marks::view_vectors::KEPT;
use crate::math::x87;

/// The Rust version (see the header).
pub fn view_step() {
    let k = &KEPT;
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
