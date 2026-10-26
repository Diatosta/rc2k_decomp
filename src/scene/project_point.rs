//! `scene::project_point` (0x44D004): a point's place in the view and on the screen,
//! worked out once and kept.
//!
//! In:  eax = the point's number
//! Out: edx = its record; a point of the scene: eax unchanged, ecx = the point's address
//!      (or the number of points + 0x40 if the record was already made); a point of an
//!      object (a negative number): eax = the record's offset; everything else preserved
//!
//! The records are 0x20 bytes each at `dword_5C6FB8`: +0 the point's number, +0xC the
//! x on the screen, +0x10, +0x14, +0x18 the point in the view, +0x1C the y on the
//! screen. A point n of the scene has record n & 0xFFF; a point -n of an object has
//! record 0x1000 + (n & 0xFFF), which is made elsewhere and only found here.
//! A record whose number is not the point's is made anew: the point's three whole
//! numbers (12 bytes each at `dword_5CCCC0`) through the matrix `flt_5EA2CC`
//! (`sub_44D110`: three rows of x, y, z factors and a fourth to add; x87, inlined here
//! with its own instructions), the screen's two set to 0, and, unless the depth is
//! under 4096: x = x' * (`flt_624088` / depth) + `flt_62409C`, y = `flt_6240A0` - y' *
//! (the same). With `dword_60932C` set the depth divided by is `dword_5CCD80` + 1
//! instead (and the real one goes to `dword_62400C`). With `dword_6365C4` at 1 the
//! record's depth is then lowered by 262144.
//! Fatal errors 0x80600902 and 0x80600903 for a number past the object's or the
//! scene's points.
//! One caller, `scene::project_polygon`.
//!
//! `record_offset` is the core's (`rc2k_core::scene::projection`); the arithmetic is the
//! x87's, in plain Rust over `X` (`math/x87_float.rs`), as the original's instructions
//! are kept as `asm_reference` in the tests (2026-10-24).

use crate::errors::{BadInput, FatalError, Refused};
use crate::math::x87;
use crate::math::x87_float::ONE;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::matrices_store::MATRICES;
use crate::race::prefs_store::STAGE_PREFS;
use crate::scene::draw_store::DRAW;
use crate::scene::light_store::LIGHT_STATE;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::scene::polygon::{dword, put};
use crate::scene::x87_mem::{fild, fld, fstp};

// ---- game data ----

pub const RECORD_LEN: usize = 0x20;
/// `flt_5C5E18`: a point nearer than this is not put on the screen.
pub const NEAR: f32 = 4096.0;
/// `flt_5C5E1C`.
const LOWERED_BY: f32 = 262144.0;

const TOO_HIGH_OBJECT: FatalError = FatalError {
    code: 0x8060_0902,
    message: "OBJPNTROT : point eax is too high",
};
const TOO_HIGH: FatalError = FatalError {
    code: 0x8060_0903,
    message: "CRSPNTROT : point eax is too high",
};

// ---- Rust version ----

/// `sub_44D110` as `project_point` uses it: the point's three whole numbers through the
/// matrix, into the record's +0x10, +0x14, +0x18.
pub(crate) fn rotate(point: *const i32, matrix: *const u32, record: *mut u32) {
    rotate_at(point.cast(), matrix.cast(), record.cast())
}

/// `fild` x3, the three rows' sums of products plus the place, `fstp dword` x3.
fn rotate_at(p: *const u8, m: *const u8, o: *mut u8) {
    let t0 = fild(p, 0x8);
    let t1 = fild(p, 0x4);
    let t2 = fild(p, 0x0);
    let t3 = fld(m, 0x0);
    let t4 = t3 * t2;
    let t5 = fld(m, 0x10);
    let t6 = t5 * t1;
    let t7 = fld(m, 0x20);
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = fld(m, 0x30);
    let t12 = t10 + t11;
    let t13 = fld(m, 0x4);
    let t14 = t13 * t2;
    let t15 = fld(m, 0x14);
    let t16 = t15 * t1;
    let t17 = fld(m, 0x24);
    let t18 = t17 * t0;
    let t19 = t16 + t18;
    let t20 = t14 + t19;
    let t21 = fld(m, 0x34);
    let t22 = t20 + t21;
    let t23 = fld(m, 0x8);
    let t24 = t2 * t23;
    let t25 = fld(m, 0x18);
    let t26 = t1 * t25;
    let t27 = fld(m, 0x28);
    let t28 = t0 * t27;
    let t29 = t28 + t26;
    let t30 = t24 + t29;
    let t31 = fld(m, 0x38);
    let t32 = t30 + t31;
    fstp(o, 0x10, t12);
    fstp(o, 0x14, t22);
    fstp(o, 0x18, t32);
}

/// The record's place on the screen from its place in the view. `v`: the scale, the
/// centre's x and y, and the whole number the flat depth is made from.
pub(crate) fn to_screen(record: *mut u32, v: &[u32; 4], flat: bool, checked: *mut u32) {
    if flat {
        screen_flat(record.cast(), v.as_ptr().cast(), checked.cast());
    } else {
        screen_plain(record.cast(), v.as_ptr().cast());
    }
}

/// With the flat depth: the depth goes to `checked`, and the one divided by is the whole
/// number plus one.
fn screen_flat(o: *mut u8, v: *const u8, w: *mut u8) {
    let t0 = fld(o, 0x10);
    let t1 = fld(o, 0x14);
    let t2 = fld(o, 0x18);
    fstp(w, 0x0, t2);
    let t3 = fild(v, 0xC);
    let t4 = ONE;
    let t5 = t3 + t4;
    let t6 = fld(v, 0x0);
    let t7 = t6 / t5;
    let t8 = t1 * t7;
    let t9 = t0 * t7;
    let t10 = -t8;
    let t11 = fld(v, 0x8);
    let t12 = t10 + t11;
    let t13 = fld(v, 0x4);
    let t14 = t9 + t13;
    fstp(o, 0xC, t14);
    fstp(o, 0x1C, t12);
}

/// The usual way: x = x' * (scale / depth) + centre x, y = centre y - y' * (the same).
fn screen_plain(o: *mut u8, v: *const u8) {
    let t0 = fld(o, 0x10);
    let t1 = fld(o, 0x14);
    let t2 = fld(o, 0x18);
    let t3 = fld(v, 0x0);
    let t4 = t3 / t2;
    let t5 = t1 * t4;
    let t6 = t0 * t4;
    let t7 = -t5;
    let t8 = fld(v, 0x8);
    let t9 = t7 + t8;
    let t10 = fld(v, 0x4);
    let t11 = t6 + t10;
    fstp(o, 0xC, t11);
    fstp(o, 0x1C, t9);
}

/// The offset of a point's record, or the fatal error for a number too high.
// Converted: `rc2k_core::scene::projection::record_offset`
pub fn record_offset(point: u32) -> Result<u32, FatalError> {
    use rc2k_core::scene::projection::TooHigh;
    rc2k_core::scene::projection::record_offset(
        point,
        OBJECT_TABLES.point_count.get(),
        COURSE.scene_point_count.bits(),
    )
    .map_err(|e| match e {
        TooHigh::Object => TOO_HIGH_OBJECT,
        TooHigh::Scene => TOO_HIGH,
    })
}

/// What the function leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projected {
    /// The record's address.
    pub record: u32,
    pub eax: u32,
    /// None: unchanged.
    pub ecx: Option<u32>,
}

/// A scene point's record made anew (`sub_44CF4E`, and the same code in
/// `project_point`): the point through the view's matrix and onto the screen.
/// Returns the point's address.
pub(crate) fn make_scene_record(point: u32, record: u32) -> Result<u32, BadInput> {
    let at = record as usize;
    let points: u32 = COURSE.scene_points.bits();
    if points == 0 {
        return Err(BadInput("no points"));
    }
    let from = points.wrapping_add(point.wrapping_mul(12));
    put(at, point);
    rotate(
        crate::pointers::host(from) as *const i32,
        crate::pointers::host(MATRICES.view_matrix[0].addr()) as *const u32,
        crate::pointers::host(record) as *mut u32,
    );
    put(at + 0xC, 0);
    put(at + 0x1C, 0);
    if f32::from_bits(dword(at + 0x18)) >= NEAR {
        let v = [
            DRAW.view_scale.bits(),
            DRAW.view_centre[0].bits(),
            DRAW.view_centre[1].bits(),
            COURSE.flat_depth.bits(),
        ];
        let flat = STAGE_PREFS.flat_depth.bits() != 0;
        to_screen(
            crate::pointers::host(record) as *mut u32,
            &v,
            flat,
            DRAW.scratch.as_ptr(),
        );
    }
    if LIGHT_STATE.draw_mode.bits() == 1 {
        put(at + 0x18, x87::sub(dword(at + 0x18), LOWERED_BY.to_bits()));
        DRAW.scratch.set_bits(LOWERED_BY.to_bits());
    }
    Ok(from)
}

pub fn project_point(point: u32) -> Result<Projected, Refused> {
    let offset = record_offset(point)?;
    let records: u32 = COURSE_LISTS.point_records.bits();
    if records == 0 {
        return Err(BadInput("no records for the points").into());
    }
    let record = records.wrapping_add(offset);
    if (point as i32) < 0 {
        return Ok(Projected {
            record,
            eax: offset,
            ecx: None,
        });
    }
    let at = record as usize;
    if dword(at) == point {
        return Ok(Projected {
            record,
            eax: point,
            ecx: Some(COURSE.scene_point_count.bits().wrapping_add(0x40)),
        });
    }
    let from = make_scene_record(point, record)?;
    Ok(Projected {
        record,
        eax: point,
        ecx: Some(from),
    })
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential};

    /// The original's instructions for `rotate`.
    unsafe fn rotate_at_ref(p: *mut u8, m: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fild dword ptr [{p} + 8]", "fild dword ptr [{p} + 4]", "fild dword ptr [{p}]",
                "fld dword ptr [{m}]", "fmul st(0), st(1)",
                "fld dword ptr [{m} + 0x10]", "fmul st(0), st(3)",
                "fld dword ptr [{m} + 0x20]", "fmul st(0), st(5)",
                "faddp st(1), st(0)", "faddp st(1), st(0)",
                "fadd dword ptr [{m} + 0x30]", "fxch st(3)",
                "fld dword ptr [{m} + 4]", "fmul st(0), st(2)",
                "fld dword ptr [{m} + 0x14]", "fmul st(0), st(4)",
                "fld dword ptr [{m} + 0x24]", "fmul st(0), st(3)",
                "faddp st(1), st(0)", "faddp st(1), st(0)",
                "fadd dword ptr [{m} + 0x34]", "fxch st(3)",
                "fld dword ptr [{m} + 8]", "fmulp st(3), st(0)",
                "fld dword ptr [{m} + 0x18]", "fmulp st(1), st(0)",
                "fld dword ptr [{m} + 0x28]", "fmulp st(2), st(0)",
                "faddp st(1), st(0)", "faddp st(1), st(0)",
                "fadd dword ptr [{m} + 0x38]", "fxch st(2)",
                "fstp dword ptr [{o} + 0x10]", "fstp dword ptr [{o} + 0x14]",
                "fstp dword ptr [{o} + 0x18]",
                p = in(reg) p,
                m = in(reg) m,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `screen_flat`.
    unsafe fn screen_flat_ref(o: *mut u8, v: *mut u8, w: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{o} + 0x10]", "fld dword ptr [{o} + 0x14]",
                "fld dword ptr [{o} + 0x18]",
                "fstp dword ptr [{w}]", "fild dword ptr [{v} + 12]", "fld1", "faddp st(1), st(0)",
                "fld dword ptr [{v}]", ".byte 0xDE, 0xF1",
                "fmul st(1), st(0)", "fmulp st(2), st(0)", "fchs",
                "fadd dword ptr [{v} + 8]", "fxch st(1)", "fadd dword ptr [{v} + 4]",
                "fstp dword ptr [{o} + 0xC]", "fstp dword ptr [{o} + 0x1C]",
                o = in(reg) o,
                v = in(reg) v,
                w = in(reg) w,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `screen_plain`.
    unsafe fn screen_plain_ref(o: *mut u8, v: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{o} + 0x10]", "fld dword ptr [{o} + 0x14]",
                "fld dword ptr [{o} + 0x18]",
                "fld dword ptr [{v}]", ".byte 0xDE, 0xF1",
                "fmul st(1), st(0)", "fmulp st(2), st(0)", "fchs",
                "fadd dword ptr [{v} + 8]", "fxch st(1)", "fadd dword ptr [{v} + 4]",
                "fstp dword ptr [{o} + 0xC]", "fstp dword ptr [{o} + 0x1C]",
                o = in(reg) o,
                v = in(reg) v,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    #[test]
    fn rotate_matches_the_assembly() {
        // dwords 0 .. 3 the point, 4 .. 19 the matrix, 20 .. 28 the record
        differential::<28>(
            "project_point rotate",
            8000,
            13,
            [Kind::Float; 28],
            |rng, m| {
                m.fill(0xAAAA_AAAA);
                for d in m.iter_mut().take(3) {
                    *d = rng.int();
                }
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().skip(4).take(15) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
            },
            |base| unsafe { rotate_at_ref(base, base.add(16), base.add(80)) },
            |base| unsafe { rotate_at(base, base.add(16), base.add(80)) },
        );
    }

    /// dwords 0 .. 4 the scale, centre x, centre y and whole number, 4 the checked word, 8 ..
    /// 16 the record (+0x10 .. +0x18 the place in the view).
    fn screen_fill(rng: &mut crate::scene::asm_test::Rng, m: &mut [u32; 16]) {
        m.fill(0xAAAA_AAAA);
        let moderate = rng.below(2) == 0;
        for d in m.iter_mut().take(3) {
            *d = if moderate { rng.moderate() } else { rng.fl() };
        }
        m[3] = rng.int();
        for d in m.iter_mut().skip(12).take(3) {
            *d = if moderate { rng.moderate() } else { rng.fl() };
        }
    }

    #[test]
    fn screen_flat_matches_the_assembly() {
        differential::<16>(
            "project_point flat",
            8000,
            14,
            [Kind::Float; 16],
            screen_fill,
            |base| unsafe { screen_flat_ref(base.add(32), base, base.add(16)) },
            |base| unsafe { screen_flat(base.add(32), base, base.add(16)) },
        );
    }

    #[test]
    fn screen_plain_matches_the_assembly() {
        differential::<16>(
            "project_point not flat",
            8000,
            15,
            [Kind::Float; 16],
            screen_fill,
            |base| unsafe { screen_plain_ref(base.add(32), base) },
            |base| unsafe { screen_plain(base.add(32), base) },
        );
    }
}
