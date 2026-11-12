//! `scene::reflect_vector` (0x451A9D): a direction mirrored at a surface and turned by
//! a matrix.
//!
//! Returns: the matrix `SceneMatrices.view_back_matrix[0]`
//!
//! v = the three floats at `SceneMatrices.reflect_vector[0]`, n = the three at `SceneMatrices.reflect_normal[0]`:
//! d = v . n (the first and second products added first, then the third); v = v * d, each rounded to a
//! float; v = v + v - n; then v through the matrix `SceneMatrices.view_back_matrix[0]`
//! (`scene::rotate_point`, inlined) and back to its place. d stays unrounded between its uses. The original's store of d to `g_scratch` (nothing read it)
//! is not kept.
//! Plain Rust (the x87's arithmetic in `X`, `math/x87_float.rs`); the
//! original's instructions' results are recorded vectors in the tests (`data/vectors`).
//! One caller, `scene::reflection_coordinates` (the reflection's texture coordinates).
use crate::scene::x87_mem::{Base, cells, fld, fstp};

// ---- Rust version ----

pub fn reflect_vector(
    m: &crate::scene::view_matrices::ViewMatrices,
    view_back: &[crate::session::Float; 16],
) {
    let (v, n): ([u32; 3], [u32; 3]) = (
        crate::session::read_bits(&m.reflect_vector),
        crate::session::read_bits(&m.reflect_normal),
    );
    // The vector, the normal, d.
    let mut w = [v[0], v[1], v[2], n[0], n[1], n[2], 0];
    // The matrix is only read: a copy of its cells.
    let back = view_back.each_ref().map(crate::session::Float::bits);
    reflect(cells(&mut w), &back[..]);
    crate::session::write_bits(&m.reflect_vector, [w[0], w[1], w[2]]);
}

/// The block's arithmetic: `w` the vector, the normal, d; `m` the matrix.
fn reflect(w: impl Base, m: impl Base) {
    let t0 = fld(w, 0x8);
    let t1 = fld(w, 0x14);
    let t2 = t0 * t1;
    let t3 = fld(w, 0x4);
    let t4 = fld(w, 0x10);
    let t5 = t3 * t4;
    let t6 = fld(w, 0x0);
    let t7 = fld(w, 0xC);
    let t8 = t6 * t7;
    let t9 = t5 + t8;
    let t10 = t2 + t9;
    let t11 = fld(w, 0x8);
    let t12 = t11 * t10;
    fstp(w, 0x8, t12);
    let t13 = fld(w, 0x4);
    let t14 = t13 * t10;
    fstp(w, 0x4, t14);
    let t15 = fld(w, 0x0);
    let t16 = t15 * t10;
    fstp(w, 0x0, t16);
    fstp(w, 0x18, t10);
    let t17 = fld(w, 0x0);
    let t18 = t17 + t17;
    let t19 = fld(w, 0xC);
    let t20 = t18 - t19;
    fstp(w, 0x0, t20);
    let t21 = fld(w, 0x4);
    let t22 = t21 + t21;
    let t23 = fld(w, 0x10);
    let t24 = t22 - t23;
    fstp(w, 0x4, t24);
    let t25 = fld(w, 0x8);
    let t26 = t25 + t25;
    let t27 = fld(w, 0x14);
    let t28 = t26 - t27;
    fstp(w, 0x8, t28);
    let t29 = fld(w, 0x8);
    let t30 = fld(w, 0x4);
    let t31 = fld(w, 0x0);
    let t32 = fld(m, 0x0);
    let t33 = t32 * t31;
    let t34 = fld(m, 0x10);
    let t35 = t34 * t30;
    let t36 = fld(m, 0x20);
    let t37 = t36 * t29;
    let t38 = t35 + t37;
    let t39 = t33 + t38;
    let t40 = fld(m, 0x30);
    let t41 = t39 + t40;
    let t42 = fld(m, 0x4);
    let t43 = t42 * t31;
    let t44 = fld(m, 0x14);
    let t45 = t44 * t30;
    let t46 = fld(m, 0x24);
    let t47 = t46 * t29;
    let t48 = t45 + t47;
    let t49 = t43 + t48;
    let t50 = fld(m, 0x34);
    let t51 = t49 + t50;
    let t52 = fld(m, 0x8);
    let t53 = t31 * t52;
    let t54 = fld(m, 0x18);
    let t55 = t30 * t54;
    let t56 = fld(m, 0x28);
    let t57 = t29 * t56;
    let t58 = t57 + t55;
    let t59 = t53 + t58;
    let t60 = fld(m, 0x38);
    let t61 = t59 + t60;
    fstp(w, 0x0, t41);
    fstp(w, 0x4, t51);
    fstp(w, 0x8, t61);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, differential};

    #[test]
    fn reflect_matches_the_assembly() {
        // dwords 0 .. 7 the vector, the normal, d; 8 .. 23 the matrix
        let test = Differential {
            name: "scene.reflect_vector.reflect",
            cases: 8000,
            seed: 12,
            kinds: [Kind::Float; 23],
            fill: &|rng, m| {
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut() {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
                m[6] = 0xAAAA_AAAA;
                m[7] = 0xAAAA_AAAA;
            },
            plain: &|c| reflect(c, &c[8..]),
        };
        differential(test);
    }
}
