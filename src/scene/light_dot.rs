//! `scene::light_dot` (0x44EC18): how much a light falls on a point, as a whole number.
//!
//! Returns: the number
//!
//! The three floats at `dword_62DAA8` times the three at `dword_62DAB4`, added (the second and
//! third products first), times `flt_62DAC0`, rounded as the x87 is set to (`fistp`). The original's store to `g_scratch`
//! (nothing read it) is not kept. Callers: the two lighting functions 0x4511F1 and 0x451296, which
//! put a point's place in the first three, a light's direction in the second and its strength
//! in the factor.
//!
//! Plain Rust since 2026-10-24 (the x87's arithmetic in `X`, `math/x87_float.rs`); the
//! original's instructions are `asm_reference` in the tests.

use crate::scene::x87_mem::{Base, cells, fistp, fld};

// ---- Rust version ----

pub fn light_dot(scene: &crate::scene::owner::Scene) -> i32 {
    let [p0, p1, p2] = crate::session::read_bits(&scene.lights.work.light_dot_point);
    let [l0, l1, l2] = crate::session::read_bits(&scene.lights.work.light_dot_light);
    let v = [
        p0,
        p1,
        p2,
        l0,
        l1,
        l2,
        scene.lights.work.light_dot_factor.bits(),
    ];
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

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential};

    /// The original's instructions.
    unsafe fn dot_ref(v: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{v}]",
                "fmul dword ptr [{v} + 12]",
                "fld dword ptr [{v} + 4]",
                "fmul dword ptr [{v} + 16]",
                "fld dword ptr [{v} + 8]",
                "fmul dword ptr [{v} + 20]",
                ".byte 0xDE, 0xC1", // faddp st(1), st
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fmul dword ptr [{v} + 24]",
                "fistp dword ptr [{o}]",
                v = in(reg) v,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    #[test]
    fn light_dot_matches_the_assembly() {
        // seven floats, then the whole number
        differential::<8>(
            "light_dot",
            8000,
            3,
            [Kind::Exact; 8],
            |rng, m| {
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().take(7) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
                m[7] = 0xAAAA_AAAA;
            },
            |base| unsafe { dot_ref(base, base.add(28)) },
            |base| unsafe { dot(base, base.add(28)) },
        );
    }
}
