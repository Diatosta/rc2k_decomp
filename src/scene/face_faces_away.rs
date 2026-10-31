//! `scene::face_faces_away` (0x45205B): whether a face of an object is seen from behind.
//!
//! In:  ebp = the face
//! Out: eax = 1 if it faces away (or has fewer than three vertices), 0 if not;
//!      everything else preserved
//!
//! `g_polygonVertexCount` = the face's number of vertices. Then, for the triangles of
//! the face's first point with each two that follow one another (`sub_4520A5`): the
//! three points' places in the view (their records 0x1000 + (number & 0xFFF), +0x10
//! .. +0x18) to `dword_6396A0` ..; the two sides from the first point to
//! `dword_6396C4` ..; their cross product to the work vector; and the first point's
//! place times that. A product above 0 (or no number) says this triangle faces away;
//! the first one that does not ends it with 0. The x87's arithmetic in `X` (plain Rust since
//! 2026-10-24; the original's instructions are `asm_reference` in the tests).
//! A point's number below 0 is fatal error 0x80600314.
//! One caller, `sub_45071B`, which turns the answer into the carry flag.
//! Refuses when there is no face or no records.

use crate::errors::{BadInput, FatalError, Refused};
use crate::math::x87_float::{X, compare};
use crate::physics::ground_store::GROUND_WORK;
use crate::race::scratch_store::FRAME_SCRATCH;
use crate::scene::draw_store::DRAW;
use crate::scene::x87_mem::{fld, fnstsw, fstp};

// ---- game data ----

const BAD_POINT: FatalError = FatalError {
    code: 0x8060_0314,
    message: "illegal point num in object",
};

// ---- Rust version ----

/// `sub_4520A5` from its first `fld`: the block is the three points (nine floats), the
/// six differences, the cross product, and the x87's status after `fldz; fcompp` (only the
/// condition bits are modelled).
fn triangle_at(w: *mut u8) {
    let t0 = fld(w, 0xC);
    let t1 = fld(w, 0x0);
    let t2 = t0 - t1;
    fstp(w, 0x24, t2);
    let t3 = fld(w, 0x18);
    let t4 = fld(w, 0x0);
    let t5 = t3 - t4;
    fstp(w, 0x28, t5);
    let t6 = fld(w, 0x10);
    let t7 = fld(w, 0x4);
    let t8 = t6 - t7;
    fstp(w, 0x2C, t8);
    let t9 = fld(w, 0x1C);
    let t10 = fld(w, 0x4);
    let t11 = t9 - t10;
    fstp(w, 0x30, t11);
    let t12 = fld(w, 0x14);
    let t13 = fld(w, 0x8);
    let t14 = t12 - t13;
    fstp(w, 0x34, t14);
    let t15 = fld(w, 0x20);
    let t16 = fld(w, 0x8);
    let t17 = t15 - t16;
    fstp(w, 0x38, t17);
    let t18 = fld(w, 0x38);
    let t19 = fld(w, 0x2C);
    let t20 = t18 * t19;
    let t21 = fld(w, 0x30);
    let t22 = fld(w, 0x34);
    let t23 = t21 * t22;
    let t24 = t20 - t23;
    fstp(w, 0x3C, t24);
    let t25 = fld(w, 0x28);
    let t26 = fld(w, 0x34);
    let t27 = t25 * t26;
    let t28 = fld(w, 0x38);
    let t29 = fld(w, 0x24);
    let t30 = t28 * t29;
    let t31 = t27 - t30;
    fstp(w, 0x40, t31);
    let t32 = fld(w, 0x30);
    let t33 = fld(w, 0x24);
    let t34 = t32 * t33;
    let t35 = fld(w, 0x28);
    let t36 = fld(w, 0x2C);
    let t37 = t35 * t36;
    let t38 = t34 - t37;
    fstp(w, 0x44, t38);
    let t39 = fld(w, 0x0);
    let t40 = fld(w, 0x3C);
    let t41 = t39 * t40;
    let t42 = fld(w, 0x4);
    let t43 = fld(w, 0x40);
    let t44 = t42 * t43;
    let t45 = fld(w, 0x8);
    let t46 = fld(w, 0x44);
    let t47 = t45 * t46;
    let t48 = t44 + t47;
    let t49 = t41 + t48;
    let t50 = X(0.0);
    let t51 = compare(t50, t49);
    fnstsw(w, 0x48, t51); // status word C3 C2 C0 (TOP not modelled)
}

fn triangle(w: &mut [u32; 19]) {
    triangle_at(w.as_mut_ptr().cast());
}

pub fn face_faces_away(face: usize) -> Result<bool, Refused> {
    if face == 0 || !crate::scene::point_records::held() {
        return Err(BadInput("no face or no records").into());
    }
    let count: u32 = crate::scene::object_faces::word(face as u32)?;
    DRAW.polygon_vertex_count.set_bits(count);
    if count < 3 {
        return Ok(true);
    }
    let len = (count as usize)
        .checked_mul(0x10)
        .and_then(|n| n.checked_add(0x10))
        .ok_or(BadInput("a face of too many vertices"))?;
    let words = crate::scene::object_faces::bytes(face as u32, len)?;
    let point = |n: u32| {
        let at = 0x10 + n as usize * 0x10;
        u32::from_le_bytes(words[at..at + 4].try_into().unwrap())
    };
    let table = crate::scene::reading::PointRecords::new()?;
    for n in 2..count {
        // The original looks at each triangle's numbers as it comes to it.
        if [0, n - 1, n].into_iter().any(|k| (point(k) as i32) < 0) {
            return Err(BAD_POINT.into());
        }
        let mut w = [0u32; 19];
        for (at, k) in [0, n - 1, n].into_iter().enumerate() {
            let place = table.place(point(k))?;
            w[at * 3..at * 3 + 3].copy_from_slice(&place);
        }
        triangle(&mut w);
        let work: [u32; 15] = w[..15].try_into().expect("15 dwords");
        let cross: [u32; 3] = w[15..18].try_into().expect("3 dwords");
        crate::session::write_bits(&GROUND_WORK.normal_work, work);
        crate::session::write_first(&FRAME_SCRATCH.work_vector, &cross);
        // The carry of the compare: 0 below the product, or no number.
        if w[18] & 0x100 == 0 {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential};

    /// The original's instructions for `triangle`.
    unsafe fn triangle_ref(w: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{w} + 12]", "fsub dword ptr [{w}]", "fstp dword ptr [{w} + 36]",
                "fld dword ptr [{w} + 24]", "fsub dword ptr [{w}]", "fstp dword ptr [{w} + 40]",
                "fld dword ptr [{w} + 16]", "fsub dword ptr [{w} + 4]", "fstp dword ptr [{w} + 44]",
                "fld dword ptr [{w} + 28]", "fsub dword ptr [{w} + 4]", "fstp dword ptr [{w} + 48]",
                "fld dword ptr [{w} + 20]", "fsub dword ptr [{w} + 8]", "fstp dword ptr [{w} + 52]",
                "fld dword ptr [{w} + 32]", "fsub dword ptr [{w} + 8]", "fstp dword ptr [{w} + 56]",
                "fld dword ptr [{w} + 56]", "fld dword ptr [{w} + 44]",
                ".byte 0xDE, 0xC9", // fmulp st(1), st
                "fld dword ptr [{w} + 48]", "fld dword ptr [{w} + 52]",
                ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9", // fsubp st(1), st
                "fstp dword ptr [{w} + 60]",
                "fld dword ptr [{w} + 40]", "fld dword ptr [{w} + 52]",
                ".byte 0xDE, 0xC9",
                "fld dword ptr [{w} + 56]", "fld dword ptr [{w} + 36]",
                ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9",
                "fstp dword ptr [{w} + 64]",
                "fld dword ptr [{w} + 48]", "fld dword ptr [{w} + 36]",
                ".byte 0xDE, 0xC9",
                "fld dword ptr [{w} + 40]", "fld dword ptr [{w} + 44]",
                ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xE9",
                "fstp dword ptr [{w} + 68]",
                "fld dword ptr [{w}]", "fld dword ptr [{w} + 60]",
                ".byte 0xDE, 0xC9",
                "fld dword ptr [{w} + 4]", "fld dword ptr [{w} + 64]",
                ".byte 0xDE, 0xC9",
                "fld dword ptr [{w} + 8]", "fld dword ptr [{w} + 68]",
                ".byte 0xDE, 0xC9",
                ".byte 0xDE, 0xC1", // faddp st(1), st
                ".byte 0xDE, 0xC1",
                ".byte 0xD9, 0xEE", // fldz
                ".byte 0xDE, 0xD9", // fcompp
                "fnstsw word ptr [{w} + 72]",
                w = in(reg) w,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    #[test]
    fn triangle_matches_the_assembly() {
        // 19 dwords: nine floats, ten written; the status keeps only its condition bits
        let kinds = {
            let mut k = [Kind::Float; 19];
            k[18] = Kind::Exact;
            k
        };
        differential::<19>(
            "face_faces_away triangle",
            8000,
            11,
            kinds,
            |rng, m| {
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().take(9) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
                m[9..].fill(0xAAAA_AAAA);
                m[18] = 0;
                if rng.below(6) == 0 {
                    // a flat triangle: the product is exactly zero
                    let first = [m[0], m[1], m[2]];
                    m[3..6].copy_from_slice(&first);
                }
            },
            |base| unsafe {
                triangle_ref(base);
                let s = base.add(72).cast::<u32>();
                s.write_unaligned(s.read_unaligned() & 0x4500);
            },
            |base| unsafe {
                triangle_at(base);
                let s = base.add(72).cast::<u32>();
                s.write_unaligned(s.read_unaligned() & 0x4500);
            },
        );
    }
}
