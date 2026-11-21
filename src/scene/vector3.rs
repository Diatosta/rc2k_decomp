//! The three-float vector helpers used by the marks.
//!
//! Plain Rust (the x87's arithmetic in `X`, `math/x87_float.rs`); the tests compare against
//! recorded vectors (`data/vectors`).

use crate::math::cells::Cells;
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

/// `normalise` on three plain floats, the same steps.
pub fn normalise_plain(v: &mut [f32; 3]) {
    let f = |k: usize| ld(v[k]);
    let (x, y, z) = (f(0), f(1), f(2));
    let length = (x * x + (y * y + z * z)).sqrt();
    let t10 = ONE / length;
    let (x, y, z) = (f(0) * t10, f(1) * t10, f(2) * t10);
    v[2] = stf(z);
    v[1] = stf(y);
    v[0] = stf(x);
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

/// `angle_between_vectors` on plain float vectors.
pub fn angle_between_vectors_plain(
    a: &mut [f32; 3],
    b: &mut [f32; 3],
    work: &std::cell::RefCell<crate::effects::work_store::MarksWork>,
) -> [u8; 10] {
    let mut angle = crate::math::x87::loaded(work.borrow().cosine_start);
    let mut step = crate::math::x87::loaded(work.borrow().cosine_step);
    normalise_plain(b);
    normalise_plain(a);
    let dot = crate::math::matrix3::rounded(dot_product3(a.map(f32::to_bits), b.map(f32::to_bits)));
    let half: u32 = work.borrow().cosine_half;
    for _ in 0..16 {
        let status = cosine_against(angle.to_bits(), dot);
        work.borrow_mut().fpu_status = status;
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
mod tests;
