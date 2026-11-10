//! `effects::aim_vector` (0x4C1294): for each of x, y, z `dword_769306` .. =
//! -`flt_7692B2` .. less `flt_76927A` .. plus `flt_7692CE` .. plus `flt_7692EA` ..; then
//! that vector made of length 1 into `unk_7692FA` .., its length (`fst`) in
//! `dword_7692F6` (`x87::unit_and_length`).
//!
//! Caller: `effects::view_frame`.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::math::x87;

pub fn aim_vector(effects: &crate::effects::owner::Effects) {
    let k = &effects.frame.kept_view;
    let v: [u32; 3] = std::array::from_fn(|n| {
        // Per axis: (negated, less, plus, plus).
        x87::negated_less_plus_plus(
            k.kept_b.scaled[n].bits(),
            k.kept_a.scaled[n].bits(),
            k.kept_c.scaled[n].bits(),
            k.step[n].bits(),
        )
    });
    for (to, v) in k.vector.iter().zip(v) {
        to.set_bits(v);
    }
    let (unit, length) = x87::unit_and_length(v);
    k.length.set(length);
    for (to, u) in k.unit.iter().zip(unit) {
        to.set_bits(u);
    }
}
