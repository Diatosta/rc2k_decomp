//! `scene::light_dot` (0x44EC18): how much a light falls on a point, as a whole number.
//!
//! Returns: the number
//!
//! The three floats at `LightWork.light_dot_point[0]` times the three at `LightWork.light_dot_light[0]`, added (the second and
//! third products first), times `LightWork.light_dot_factor`, rounded as the x87 is set to (`fistp`). The original's store to `g_scratch`
//! (nothing read it) is not kept. Callers: the two lighting functions 0x4511F1 and 0x451296, which
//! put a point's place in the first three, a light's direction in the second and its strength
//! in the factor.
//!
//! Plain Rust (the x87's arithmetic in `X`, `math/x87_float.rs`); the
//! original's instructions' results are recorded vectors in the tests (`data/vectors`).

use crate::scene::x87_mem::{Base, cells, fistp, fld};

// ---- Rust version ----

pub fn light_dot(light: &crate::scene::object_light::ObjectLight) -> i32 {
    let [p0, p1, p2] = crate::session::read_bits(&light.light_dot_point);
    let [l0, l1, l2] = crate::session::read_bits(&light.light_dot_light);
    let v = [p0, p1, p2, l0, l1, l2, light.light_dot_factor.bits()];
    let mut out = [0u32];
    dot(&v[..], cells(&mut out));
    out[0] as i32
}

/// `fld; fmul; fld; fmul; fld; fmul; faddp; faddp; fmul; fistp` over the seven floats at `v`.
fn dot(v: impl Base, o: impl Base) {
    let t0 = fld(v, 0x0);
    let t1 = fld(v, 0xC);
    let t2 = t0 * t1;
    let t3 = fld(v, 0x4);
    let t4 = fld(v, 0x10);
    let t5 = t3 * t4;
    let t6 = fld(v, 0x8);
    let t7 = fld(v, 0x14);
    let t8 = t6 * t7;
    let t9 = t5 + t8;
    let t10 = t2 + t9;
    let t11 = fld(v, 0x18);
    let t12 = t10 * t11;
    fistp(o, 0x0, t12);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, differential};

    #[test]
    fn light_dot_matches_the_assembly() {
        // seven floats, then the whole number
        let test = Differential {
            name: "scene.light_dot.light_dot",
            cases: 8000,
            seed: 3,
            kinds: [Kind::Exact; 8],
            fill: &|rng, m| {
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().take(7) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
                m[7] = 0xAAAA_AAAA;
            },
            plain: &|c| dot(c, &c[7..]),
        };
        differential(test);
    }
}
