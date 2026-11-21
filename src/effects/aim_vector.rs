//! `effects::aim_vector`: for each of x, y, z `KeptView.vector[0]` .. =
//! -`KeptView.kept_b.scaled[0]` .. less `KeptView.kept_a.scaled[0]` .. plus
//! `KeptView.kept_c.scaled[0]` .. plus `KeptView.step[0]` ..; then that vector made of length 1
//! into `KeptView.unit[0]` .., its length in `KeptView.length` (`x87::unit_and_length`).

use crate::math::x87;
use crate::math::x87_float as bh;

pub fn aim_vector(effects: &crate::effects::owner::Effects) {
    let mut k = effects.frame.kept_view.borrow_mut();
    let v: [f32; 3] = std::array::from_fn(|n| {
        // Per axis: (negated, less, plus, plus).
        bh::stf(
            -bh::ld(k.kept_b.scaled[n]) - bh::ld(k.kept_a.scaled[n])
                + bh::ld(k.kept_c.scaled[n])
                + bh::ld(k.step[n]),
        )
    });
    k.vector = v;
    let (unit, length) = x87::unit_and_length(v.map(f32::to_bits));
    k.length = length;
    k.unit = unit;
}
