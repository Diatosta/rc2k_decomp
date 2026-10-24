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
//! the first one that does not ends it with 0. x87, with the original's instructions.
//! A point's number below 0 is fatal error 0x80600314.
//! One caller, `sub_45071B`, which turns the answer into the carry flag.
//! Refuses when there is no face or no records.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::math::normalise_vector::VECTOR_RVA;
use crate::scene::polygon::dword;
use crate::scene::project_point::RECORDS_RVA;

// ---- game data ----

/// Three points and the six differences: fifteen floats from `dword_6396A0`.
const WORK_RVA: usize = rva(0x6396A0);

const BAD_POINT: FatalError = FatalError {
    code: 0x8060_0314,
    message: "illegal point num in object",
};

// ---- Rust version ----

/// `sub_4520A5` from its first `fld`: the block is the three points (nine floats), the
/// six differences, the cross product, and the x87's status after `fldz; fcompp`.
unsafe fn triangle(w: &mut [u32; 19]) {
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
            w = in(reg) w.as_mut_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

pub unsafe fn face_faces_away(face: usize) -> Result<bool, Refused> {
    unsafe {
        let records: u32 = game_read(RECORDS_RVA);
        if face == 0 || records == 0 {
            return Err(BadInput("no face or no records").into());
        }
        let count = dword(face);
        let point = |n: u32| dword(face + 0x10 + n as usize * 0x10);
        game_write(COUNT_RVA, count);
        if count < 3 {
            return Ok(true);
        }
        for n in 2..count {
            // The original looks at each triangle's numbers as it comes to it.
            if [0, n - 1, n].into_iter().any(|k| (point(k) as i32) < 0) {
                return Err(BAD_POINT.into());
            }
            let mut w = [0u32; 19];
            for (at, k) in [0, n - 1, n].into_iter().enumerate() {
                let record = records.wrapping_add(((point(k) & 0xFFF) + 0x1000) << 5) as usize;
                let place = ((record + 0x10) as *const [u32; 3]).read_unaligned();
                w[at * 3..at * 3 + 3].copy_from_slice(&place);
            }
            triangle(&mut w);
            let work: [u32; 15] = w[..15].try_into().expect("15 dwords");
            let cross: [u32; 3] = w[15..18].try_into().expect("3 dwords");
            game_write(WORK_RVA, work);
            game_write(VECTOR_RVA, cross);
            // The carry of the compare: 0 below the product, or no number.
            if w[18] & 0x100 == 0 {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
