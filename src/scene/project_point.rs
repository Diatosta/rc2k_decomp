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
//! Converted 2026-10-21: the hook's Rust version is `project_point_plain`, through
//! `rc2k_core::scene::projection` (plain floats; the x87 version stays for the Rust callers
//! in unconverted code); `record_offset` is the core's for both. Shadow mode and the
//! self-test compare the record's floats and `dword_62400C` within `service::floats`'
//! tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{game_ptr, game_read, game_write, rva};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::polygon::{dword, put};

// ---- game data ----

pub const RECORDS_RVA: usize = rva(0x5C6FB8); // dword_5C6FB8
pub const RECORD_LEN: usize = 0x20;
pub const POINTS_RVA: usize = rva(0x5CCCC0); // dword_5CCCC0
pub const POINT_COUNT_RVA: usize = rva(0x5CCD20); // dword_5CCD20
pub const OBJECT_POINTS_RVA: usize = rva(0x5CD680); // dword_5CD680
pub const MATRIX_RVA: usize = rva(0x5EA2CC); // flt_5EA2CC
pub const FLAT_RVA: usize = rva(0x60932C); // dword_60932C
pub const FLAT_DEPTH_RVA: usize = rva(0x5CCD80); // dword_5CCD80
pub const SCALE_RVA: usize = rva(0x624088); // flt_624088
pub const CENTRE_X_RVA: usize = rva(0x62409C); // flt_62409C
pub const CENTRE_Y_RVA: usize = rva(0x6240A0); // flt_6240A0
pub const LOWERED_RVA: usize = rva(0x6365C4); // dword_6365C4
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

macro_rules! x87 {
    ($($line:literal),+; $($name:ident = $value:expr),+) => {
        unsafe {
            core::arch::asm!(
                $($line),+,
                $($name = in(reg) $value),+,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            )
        }
    };
}

/// `sub_44D110` as `project_point` uses it: the point's three whole numbers through the
/// matrix, into the record's +0x10, +0x14, +0x18.
pub(crate) unsafe fn rotate(point: *const i32, matrix: *const u32, record: *mut u32) {
    x87!(
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
        "fstp dword ptr [{o} + 0x18]";
        p = point, m = matrix, o = record
    );
}

/// The record's place on the screen from its place in the view. `v`: the scale, the
/// centre's x and y, and the whole number the flat depth is made from.
/// (`.byte 0xDE, 0xF1` is the original's `fdivrp st(1), st`.)
pub(crate) unsafe fn to_screen(record: *mut u32, v: &[u32; 4], flat: bool, checked: *mut u32) {
    if flat {
        x87!(
            "fld dword ptr [{o} + 0x10]", "fld dword ptr [{o} + 0x14]",
            "fld dword ptr [{o} + 0x18]",
            "fstp dword ptr [{w}]", "fild dword ptr [{v} + 12]", "fld1", "faddp st(1), st(0)",
            "fld dword ptr [{v}]", ".byte 0xDE, 0xF1",
            "fmul st(1), st(0)", "fmulp st(2), st(0)", "fchs",
            "fadd dword ptr [{v} + 8]", "fxch st(1)", "fadd dword ptr [{v} + 4]",
            "fstp dword ptr [{o} + 0xC]", "fstp dword ptr [{o} + 0x1C]";
            o = record, v = v.as_ptr(), w = checked
        );
    } else {
        x87!(
            "fld dword ptr [{o} + 0x10]", "fld dword ptr [{o} + 0x14]",
            "fld dword ptr [{o} + 0x18]",
            "fld dword ptr [{v}]", ".byte 0xDE, 0xF1",
            "fmul st(1), st(0)", "fmulp st(2), st(0)", "fchs",
            "fadd dword ptr [{v} + 8]", "fxch st(1)", "fadd dword ptr [{v} + 4]",
            "fstp dword ptr [{o} + 0xC]", "fstp dword ptr [{o} + 0x1C]";
            o = record, v = v.as_ptr()
        );
    }
}

/// The offset of a point's record, or the fatal error for a number too high.
// Converted: `rc2k_core::scene::projection::record_offset`
pub unsafe fn record_offset(point: u32) -> Result<u32, FatalError> {
    use rc2k_core::scene::projection::TooHigh;
    unsafe {
        rc2k_core::scene::projection::record_offset(
            point,
            game_read(OBJECT_POINTS_RVA),
            game_read(POINT_COUNT_RVA),
        )
        .map_err(|e| match e {
            TooHigh::Object => TOO_HIGH_OBJECT,
            TooHigh::Scene => TOO_HIGH,
        })
    }
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
pub(crate) unsafe fn make_scene_record(point: u32, record: u32) -> Result<u32, BadInput> {
    unsafe {
        let at = record as usize;
        let points: u32 = game_read(POINTS_RVA);
        if points == 0 {
            return Err(BadInput("no points"));
        }
        let from = points.wrapping_add(point.wrapping_mul(12));
        put(at, point);
        rotate(from as *const i32, game_ptr(MATRIX_RVA), record as *mut u32);
        put(at + 0xC, 0);
        put(at + 0x1C, 0);
        if f32::from_bits(dword(at + 0x18)) >= NEAR {
            let v = [
                game_read(SCALE_RVA),
                game_read(CENTRE_X_RVA),
                game_read(CENTRE_Y_RVA),
                game_read(FLAT_DEPTH_RVA),
            ];
            let flat = game_read::<u32>(FLAT_RVA) != 0;
            to_screen(record as *mut u32, &v, flat, game_ptr(CHECKED_RVA));
        }
        if game_read::<u32>(LOWERED_RVA) == 1 {
            put(at + 0x18, x87::sub(dword(at + 0x18), LOWERED_BY.to_bits()));
            game_write(CHECKED_RVA, LOWERED_BY.to_bits());
        }
        Ok(from)
    }
}

pub unsafe fn project_point(point: u32) -> Result<Projected, Refused> {
    unsafe {
        let offset = record_offset(point)?;
        let records: u32 = game_read(RECORDS_RVA);
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
                ecx: Some(game_read::<u32>(POINT_COUNT_RVA).wrapping_add(0x40)),
            });
        }
        let from = make_scene_record(point, record)?;
        Ok(Projected {
            record,
            eax: point,
            ecx: Some(from),
        })
    }
}
