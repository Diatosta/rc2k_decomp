//! `scene::light_dot`: how much a light falls on a point, as a whole number.
//!
//! The three floats at `LightWork.light_dot_point` times the three at `LightWork.light_dot_light`,
//! added (the second and third products first), times `LightWork.light_dot_factor`, rounded as the
//! x87 is set to. The point holds a point's place, the light a light's direction and the factor
//! its strength.
//!
//! Plain Rust (the x87's arithmetic in `X`, `math/x87_float.rs`); the tests compare against
//! recorded vectors (`data/vectors`).

use crate::math::x87_float::{Rounds, at_rounding};

pub fn light_dot(light: &crate::scene::object_light::ObjectLight) -> i32 {
    let [p0, p1, p2] = light
        .light_dot_point
        .each_ref()
        .map(crate::memory::session::PlainFloat::get);
    let [l0, l1, l2] = light
        .light_dot_light
        .each_ref()
        .map(crate::memory::session::PlainFloat::get);
    dot([p0, p1, p2], [l0, l1, l2], light.light_dot_factor.get())
}

/// `fld; fmul; fld; fmul; fld; fmul; faddp; faddp; fmul; fistp`: the point times the light,
/// the second and third products added first, times the factor.
fn dot(point: [f32; 3], light: [f32; 3], factor: f32) -> i32 {
    at_rounding!(|r| dot_with(r, point, light, factor))
}

/// `dot` at the rounding `r`.
fn dot_with<R: Rounds>(r: R, point: [f32; 3], light: [f32; 3], factor: f32) -> i32 {
    let t2 = r.ld(point[0]) * r.ld(light[0]);
    let t5 = r.ld(point[1]) * r.ld(light[1]);
    let t8 = r.ld(point[2]) * r.ld(light[2]);
    let t9 = t5 + t8;
    let t10 = t2 + t9;
    r.to_int(t10 * r.ld(factor))
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
