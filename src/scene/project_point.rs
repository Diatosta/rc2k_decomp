//! `scene::project_point`: a point's place in the view and on the screen, worked out once and
//! kept.
//!
//! The records are eight dwords each in the scene's point records: the point's number, the x
//! on the screen (after three unused dwords), the point in the view (three floats), the y on the
//! screen. A point n
//! of the scene has record n & 0xFFF; a point -n of an object has record 0x1000 + (n & 0xFFF),
//! which is made elsewhere and only found here. A record whose number is not the point's is made
//! anew: the point's three whole numbers (12 bytes each in the course's point table) through the
//! view matrix (`scene::rotate_point`: three rows of x, y, z factors and a fourth to add), the
//! screen's two set to 0, and, unless the depth is under 4096: x = x' * (view scale / depth) + the
//! view's centre x, y = the centre y - y' * (the same). With the stage's flat-depth switch set the
//! depth divided by is a fixed depth + 1 instead. With the lights' draw mode at 1 the record's
//! depth is then lowered by 262144. Fatal errors 0x80600902 and 0x80600903 for a number past the
//! object's or the scene's points.
//!
//! `record_offset` is the core's (`rc2k_core::scene::projection`); the arithmetic is the
//! x87's, in plain Rust over `X` (`math/x87_float.rs`); the tests compare against recorded
//! vectors (`reference_vectors`).

use crate::errors::{BadInput, FatalError, Refused};
use crate::math::x87;
use crate::math::x87_float::{ONE, Rounds, at_rounding, int, ld, st};
use crate::math::x87_mem::{Base, fstp};
use crate::scene::point_records;

// ---- game data ----

pub const RECORD_LEN: usize = 0x20;
/// A point nearer than this is not put on the screen.
pub const NEAR: f32 = 4096.0;
/// What the record's depth is lowered by.
const LOWERED_BY: f32 = 262144.0;

const TOO_HIGH_OBJECT: FatalError = FatalError {
    code: 0x8060_0902,
    message: "OBJPNTROT : point eax is too high",
};
const TOO_HIGH: FatalError = FatalError {
    code: 0x8060_0903,
    message: "CRSPNTROT : point eax is too high",
};

/// `scene::rotate_point` as `project_point` uses it: the point's three whole numbers through the
/// matrix, into the record's three view floats.
pub(crate) fn rotate(point: &[u32], matrix: &[f32; 16], record: &mut [u32]) {
    at_rounding!(|r| rotate_with(r, point, matrix, record));
}

/// `rotate` at the rounding `r`.
pub(crate) fn rotate_with<R: Rounds>(r: R, point: &[u32], matrix: &[f32; 16], record: &mut [u32]) {
    let place = rotate_at_with(r, [0, 1, 2].map(|k| point[k] as i32), matrix);
    record[4..7].copy_from_slice(&place.map(f32::to_bits));
}

/// `fild` x3, the three rows' sums of products plus the place, `fstp dword` x3.
pub(crate) fn rotate_at(p: [i32; 3], m: &[f32; 16]) -> [f32; 3] {
    at_rounding!(|r| rotate_at_with(r, p, m))
}

/// `rotate_at` at the rounding `r`.
fn rotate_at_with<R: Rounds>(r: R, p: [i32; 3], m: &[f32; 16]) -> [f32; 3] {
    let t0 = r.int(p[2]);
    let t1 = r.int(p[1]);
    let t2 = r.int(p[0]);
    let t10 = r.ld(m[0]) * t2 + (r.ld(m[4]) * t1 + r.ld(m[8]) * t0);
    let t12 = t10 + r.ld(m[12]);
    let t20 = r.ld(m[1]) * t2 + (r.ld(m[5]) * t1 + r.ld(m[9]) * t0);
    let t22 = t20 + r.ld(m[13]);
    let t30 = t2 * r.ld(m[2]) + (t0 * r.ld(m[10]) + t1 * r.ld(m[6]));
    let t32 = t30 + r.ld(m[14]);
    [r.stf(t12), r.stf(t22), r.stf(t32)]
}

/// The view window's values the screen's place is made with.
#[derive(Clone, Copy)]
struct Screen {
    /// The view's scale.
    scale: f32,
    /// The window's middle, x and y.
    centre: [f32; 2],
    /// The whole number the flat depth is made from.
    flat_depth: i32,
}

/// The record's place on the screen from its place in the view (`checked` gets the depth on
/// the flat way).
pub(crate) fn to_screen(
    record: &mut [u32],
    window: &crate::scene::view_window::ViewWindow,
    flat_depth: u32,
    flat: bool,
    checked: impl Base,
) {
    let screen = Screen {
        scale: window.borrow().view_scale,
        centre: [
            window.borrow().view_centre[0],
            window.borrow().view_centre[1],
        ],
        flat_depth: flat_depth as i32,
    };
    let view = [4, 5, 6].map(|k| f32::from_bits(record[k]));
    let [x, y] = if flat {
        screen_flat(view, screen, checked)
    } else {
        screen_plain(view, screen)
    };
    (record[3], record[7]) = (x, y);
}

/// With the flat depth: the depth goes to `checked`, and the one divided by is the whole
/// number plus one. The screen's x and y, as bits.
fn screen_flat(view: [f32; 3], s: Screen, checked: impl Base) -> [u32; 2] {
    let t0 = ld(view[0]);
    let t1 = ld(view[1]);
    fstp(checked, 0x0, ld(view[2]));
    let t5 = int(s.flat_depth) + ONE;
    let t7 = ld(s.scale) / t5;
    let t8 = t1 * t7;
    let t9 = t0 * t7;
    let t12 = -t8 + ld(s.centre[1]);
    let t14 = t9 + ld(s.centre[0]);
    [st(t14), st(t12)]
}

/// The usual way: x = x' * (scale / depth) + centre x, y = centre y - y' * (the same).
fn screen_plain(view: [f32; 3], s: Screen) -> [u32; 2] {
    at_rounding!(|r| screen_plain_with(r, view, s))
}

/// `screen_plain` at the rounding `r`.
fn screen_plain_with<R: Rounds>(r: R, view: [f32; 3], s: Screen) -> [u32; 2] {
    let t0 = r.ld(view[0]);
    let t1 = r.ld(view[1]);
    let t4 = r.ld(s.scale) / r.ld(view[2]);
    let t5 = t1 * t4;
    let t6 = t0 * t4;
    let t9 = -t5 + r.ld(s.centre[1]);
    let t11 = t6 + r.ld(s.centre[0]);
    [r.st(t11), r.st(t9)]
}

/// The offset of a point's record, or the fatal error for a number too high.
// Logic: `rc2k_core::scene::projection::record_offset`
pub fn record_offset(
    point: u32,
    reads: crate::scene::workers::DrawReads<'_>,
) -> Result<u32, FatalError> {
    use rc2k_core::scene::projection::TooHigh;
    rc2k_core::scene::projection::record_offset(
        point,
        reads.object_tables.point_count(),
        reads.course_work.get().scene_point_count,
    )
    .map_err(|e| match e {
        TooHigh::Object => TOO_HIGH_OBJECT,
        TooHigh::Scene => TOO_HIGH,
    })
}

/// What the function leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projected {
    /// The record (`scene::point_records`: its number + 1).
    pub record: u32,
}

/// A scene point's record made anew: the point through the view's matrix and onto the screen.
pub(crate) fn make_scene_record(
    part: &crate::scene::scratch_store::PartScratch,
    point: u32,
    record: u32,
    reads: crate::scene::workers::DrawReads<'_>,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), BadInput> {
    if !crate::race::scene_points::held(&reads.stage.course) {
        return Err(BadInput("no points"));
    }
    // Made on a copy of the record (`scene::point_records`), then stored.
    let mut r = point_records::get(part, record)?;
    r[0] = point;
    let place = crate::race::scene_points::point_number(reads.course(), &part.points, point)?;
    rotate(&place, &view.matrices.borrow().view_matrix, &mut r);
    r[3] = 0;
    r[7] = 0;
    if f32::from_bits(r[6]) >= NEAR {
        let flat = reads.stage_prefs.flat_depth.get() != 0;
        // The depth it stores is a work value nothing reads.
        to_screen(
            &mut r,
            &view.polygon.window,
            reads.course_work.get().flat_depth,
            flat,
            &crate::memory::session::Dword::new(0),
        );
    }
    if reads.light_work.draw_mode == 1 {
        r[6] = x87::sub(r[6], LOWERED_BY.to_bits()).to_bits();
    }
    point_records::set(part, record, r)?;
    Ok(())
}

pub fn project_point(
    part: &crate::scene::scratch_store::PartScratch,
    point: u32,
    reads: crate::scene::workers::DrawReads<'_>,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<Projected, Refused> {
    let offset = record_offset(point, reads)?;
    if !point_records::held(reads.course_lists) {
        return Err(BadInput("no records for the points").into());
    }
    let record = point_records::at_offset(offset);
    if (point as i32) >= 0 && point_records::field(part, record, 0)? != point {
        make_scene_record(part, point, record, reads, view)?;
    }
    Ok(Projected { record })
}

#[cfg(test)]
mod tests;
