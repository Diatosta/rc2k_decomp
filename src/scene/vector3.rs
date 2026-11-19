//! The three-float vector helpers used by the marks.
//!
//! Plain Rust (the x87's arithmetic in `X`, `math/x87_float.rs`); the tests compare against
//! recorded vectors (`data/vectors`).

use crate::math::cells::{Cells, load};
use crate::math::x87_float::{ONE, compare, ld, stf};
use crate::math::x87_mem::{Base, cells, fld, fnstsw, fstp80};

/// `scene::normalise_vector3`: the three floats each times 1 over their length (the root of the
/// squares, the last two added first).
pub fn normalise(v: &(impl Cells + ?Sized)) {
    let f = |k| ld(v.float(k));
    let (x, y, z) = (f(0), f(1), f(2));
    let length = (x * x + (y * y + z * z)).sqrt();
    let t10 = ONE / length;
    let (x, y, z) = (f(0) * t10, f(1) * t10, f(2) * t10);
    v.set_float(2, stf(z));
    v.set_float(1, stf(y));
    v.set_float(0, stf(x));
}

/// `scene::dot_product3`: the dot product x x' + (y y' + z z'), unrounded.
pub fn dot_product3(a: [u32; 3], b: [u32; 3]) -> [u8; 10] {
    let out = [std::cell::Cell::new([0u8; 10])];
    dot(&a[..], &b[..], &out[..]);
    out[0].get()
}

fn dot(a: impl Base, b: impl Base, o: impl Base) {
    let t0 = fld(a, 0x0);
    let t1 = fld(b, 0x0);
    let t2 = t0 * t1;
    let t3 = fld(a, 0x4);
    let t4 = fld(b, 0x4);
    let t5 = t3 * t4;
    let t6 = fld(a, 0x8);
    let t7 = fld(b, 0x8);
    let t8 = t6 * t7;
    let t9 = t5 + t8;
    let t10 = t2 + t9;
    fstp80(o, 0x0, t10);
}

/// `fld a; fcos; fcomp b; fstsw`: the status word's C0 .. C2 (0x700).
fn cosine_against(a: u32, b: u32) -> u32 {
    let mut status = [0u32];
    cosine(&[a][..], &[b][..], cells(&mut status));
    status[0] & 0x700
}

fn cosine(a: impl Base, b: impl Base, s: impl Base) {
    let t0 = fld(a, 0x0);
    let t1 = t0.cos();
    let t2 = fld(b, 0x0);
    let t3 = compare(t1, t2);
    fnstsw(s, 0x0, t3); // status word C3 C2 C0 (TOP not modelled)
}

/// `scene::angle_between_vectors`: the two vectors normalised in
/// place (in order), their dot product kept as a float, and the angle whose cosine it
/// is found by 16 halvings from pi / 2 by pi / 4 (floats each step; out when the compare
/// is unordered); the last compare's C0 .. C2 in the shared FPU status cell.
/// Returns the angle (a float, loaded).
pub fn angle_between_vectors(
    a: &(impl Cells + ?Sized),
    b: &(impl Cells + ?Sized),
    work: &crate::effects::work_store::MarksWork,
) -> [u8; 10] {
    let mut angle = crate::math::x87::loaded(work.cosine_start.get());
    let mut step = crate::math::x87::loaded(work.cosine_step.get());
    normalise(b);
    normalise(a);
    let dot = crate::math::matrix3::rounded(dot_product3(load(a), load(b)));
    let half: u32 = work.cosine_half.get();
    for _ in 0..16 {
        let status = cosine_against(angle.to_bits(), dot);
        work.fpu_status.set(status);
        match status {
            0x400 => break,
            0x100 => angle = crate::math::x87::sub(angle, step),
            _ => angle = crate::math::x87::add(angle, step),
        }
        step = crate::math::x87::mul(step, half);
    }
    crate::math::matrix3::extended(angle)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, differential};

    #[test]
    fn normalise_matches_the_assembly() {
        let test = Differential {
            name: "scene.vector3.normalise",
            cases: 8000,
            seed: 4,
            kinds: [Kind::Float; 3],
            fill: &|rng, m| {
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut() {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
            },
            plain: &|c| normalise(c),
        };
        differential(test);
    }

    #[test]
    fn dot_matches_the_assembly() {
        // a, b, the 80-bit result (3 dwords)
        let test = Differential {
            name: "scene.vector3.dot",
            cases: 8000,
            seed: 5,
            kinds: [Kind::Exact; 9],
            fill: &|rng, m| {
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().take(6) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
                m[6..].fill(0xAAAA_AAAA);
            },
            plain: &|c| dot(c, &c[3..], &c[6..]),
        };
        differential(test);
    }

    #[test]
    fn cosine_matches_the_assembly() {
        // the angle, the float it is compared with (often the cosine itself), the status
        let test = Differential {
            name: "scene.vector3.cosine",
            cases: 8000,
            seed: 6,
            kinds: [Kind::Exact; 3],
            fill: &|rng, m| {
                m[0] = rng.angle();
                m[1] = match rng.below(3) {
                    0 => rc2k_core::math::portable::cos(f32::from_bits(m[0]) as f64) as f32,
                    1 => rng.unit(),
                    _ => f32::from_bits(rng.fl()),
                }
                .to_bits();
                m[2] = 0;
            },
            plain: &|c| {
                cosine(c, &c[1..], &c[2..]);
                c[2].set(c[2].get() & 0x700);
            },
        };
        differential(test);
    }
}
