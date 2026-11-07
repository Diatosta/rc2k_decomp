//! `scene::reflect_vector` (0x451A9D): a direction mirrored at a surface and turned by
//! a matrix.
//!
//! In:  nothing
//! Out: esi = the matrix `flt_5EA20C`; everything else preserved
//!
//! v = the three floats at `dword_5EA428`, n = the three at `qword_5EA438`:
//! d = v . n (the first and second products added first, then the third); v = v * d, each rounded to a
//! float; `dword_62400C` = d; v = v + v - n; then v through the matrix `flt_5EA20C`
//! (`scene::rotate_point`, inlined) and back to its place. d stays unrounded between its uses.
//! Plain Rust since 2026-10-24 (the x87's arithmetic in `X`, `math/x87_float.rs`); the
//! original's instructions are `asm_reference` in the tests.
//! One caller, `sub_45198C` (the reflection's texture coordinates).
use crate::scene::x87_mem::{Base, cells, fld, fstp};

// ---- Rust version ----

pub fn reflect_vector(frame: &crate::race::owner_frame::Frame, scene: &crate::scene::owner::Scene) {
    let (v, n): ([u32; 3], [u32; 3]) = (
        crate::session::read_bits(&frame.matrices.reflect_vector),
        crate::session::read_bits(&frame.matrices.reflect_normal),
    );
    // The vector, the normal, d.
    let mut w = [v[0], v[1], v[2], n[0], n[1], n[2], 0];
    // The matrix is only read: a copy of its cells.
    let m = frame
        .matrices
        .view_back_matrix
        .each_ref()
        .map(super::super::session::Float::bits);
    reflect(cells(&mut w), &m[..]);
    crate::session::write_bits(&frame.matrices.reflect_vector, [w[0], w[1], w[2]]);
    scene.drawing.scene.scratch.set_bits(w[6]);
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

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential};

    /// The original's instructions (the block of `reflect_vector`).
    unsafe fn reflect_ref(w: *mut u8, m: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{w} + 8]",
                "fmul dword ptr [{w} + 20]",
                "fld dword ptr [{w} + 4]",
                "fmul dword ptr [{w} + 16]",
                "fld dword ptr [{w}]",
                "fmul dword ptr [{w} + 12]",
                ".byte 0xDE, 0xC1", // faddp st(1), st
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fld dword ptr [{w} + 8]",
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                "fstp dword ptr [{w} + 8]",
                "fld dword ptr [{w} + 4]",
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                "fstp dword ptr [{w} + 4]",
                "fld dword ptr [{w}]",
                ".byte 0xD8, 0xC9", // fmul st, st(1)
                "fstp dword ptr [{w}]",
                "fstp dword ptr [{w} + 24]",
                "fld dword ptr [{w}]",
                ".byte 0xD8, 0xC0", // fadd st, st
                "fsub dword ptr [{w} + 12]",
                "fstp dword ptr [{w}]",
                "fld dword ptr [{w} + 4]",
                ".byte 0xD8, 0xC0", // fadd st, st
                "fsub dword ptr [{w} + 16]",
                "fstp dword ptr [{w} + 4]",
                "fld dword ptr [{w} + 8]",
                ".byte 0xD8, 0xC0", // fadd st, st
                "fsub dword ptr [{w} + 20]",
                "fstp dword ptr [{w} + 8]",
                "fld dword ptr [{w} + 8]",
                "fld dword ptr [{w} + 4]",
                "fld dword ptr [{w}]",
                // `scene::rotate_point` (0x44D110), as in `project_point.rs`.
                "fld dword ptr [{m}]",
                "fmul st(0), st(1)",
                "fld dword ptr [{m} + 0x10]",
                "fmul st(0), st(3)",
                "fld dword ptr [{m} + 0x20]",
                "fmul st(0), st(5)",
                "faddp st(1), st(0)",
                "faddp st(1), st(0)",
                "fadd dword ptr [{m} + 0x30]",
                "fxch st(3)",
                "fld dword ptr [{m} + 4]",
                "fmul st(0), st(2)",
                "fld dword ptr [{m} + 0x14]",
                "fmul st(0), st(4)",
                "fld dword ptr [{m} + 0x24]",
                "fmul st(0), st(3)",
                "faddp st(1), st(0)",
                "faddp st(1), st(0)",
                "fadd dword ptr [{m} + 0x34]",
                "fxch st(3)",
                "fld dword ptr [{m} + 8]",
                "fmulp st(3), st(0)",
                "fld dword ptr [{m} + 0x18]",
                "fmulp st(1), st(0)",
                "fld dword ptr [{m} + 0x28]",
                "fmulp st(2), st(0)",
                "faddp st(1), st(0)",
                "faddp st(1), st(0)",
                "fadd dword ptr [{m} + 0x38]",
                "fxch st(2)",
                "fstp dword ptr [{w}]",
                "fstp dword ptr [{w} + 4]",
                "fstp dword ptr [{w} + 8]",
                w = in(reg) w,
                m = in(reg) m,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    #[test]
    fn reflect_matches_the_assembly() {
        // dwords 0 .. 7 the vector, the normal, d; 8 .. 23 the matrix
        differential::<23>(
            "reflect_vector",
            8000,
            12,
            [Kind::Float; 23],
            |rng, m| {
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut() {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
                m[6] = 0xAAAA_AAAA;
                m[7] = 0xAAAA_AAAA;
            },
            |base| unsafe { reflect_ref(base, base.add(32)) },
            |base| unsafe { reflect(base, base.add(32)) },
        );
    }
}
