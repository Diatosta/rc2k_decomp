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

use crate::math::x87_float::{ld, to_int};

// ---- Rust version ----

pub fn light_dot(light: &crate::scene::object_light::ObjectLight) -> i32 {
    let [p0, p1, p2] = light
        .light_dot_point
        .each_ref()
        .map(crate::session::PlainFloat::get);
    let [l0, l1, l2] = light
        .light_dot_light
        .each_ref()
        .map(crate::session::PlainFloat::get);
    dot([p0, p1, p2], [l0, l1, l2], light.light_dot_factor.get())
}

/// `fld; fmul; fld; fmul; fld; fmul; faddp; faddp; fmul; fistp`: the point times the light,
/// the second and third products added first, times the factor.
fn dot(point: [f32; 3], light: [f32; 3], factor: f32) -> i32 {
    let t2 = ld(point[0]) * ld(light[0]);
    let t5 = ld(point[1]) * ld(light[1]);
    let t8 = ld(point[2]) * ld(light[2]);
    let t9 = t5 + t8;
    let t10 = t2 + t9;
    to_int(t10 * ld(factor))
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
            plain: &|c| {
                let f = |k: usize| f32::from_bits(c[k].get());
                c[7].set(dot([f(0), f(1), f(2)], [f(3), f(4), f(5)], f(6)) as u32);
            },
        };
        differential(test);
    }
}
