//! `scene::project_point` (0x44D004): a point's place in the view and on the screen,
//! worked out once and kept.
//!
//! Takes: the point's number
//!
//! The records are 0x20 bytes each in the scene's point records: +0 the point's number, +0xC
//! the x on the screen, +0x10, +0x14, +0x18 the point in the view, +0x1C the y on the
//! screen. A point n of the scene has record n & 0xFFF; a point -n of an object has record
//! 0x1000 + (n & 0xFFF), which is made elsewhere and only found here.
//! A record whose number is not the point's is made anew: the point's three whole numbers
//! (12 bytes each in the course's point table) through the view matrix (`scene::rotate_point`:
//! three rows of x, y, z factors and a fourth to add; x87, inlined here with its own
//! instructions), the screen's two set to 0, and, unless the depth is under 4096:
//! x = x' * (view scale / depth) + the view's centre x, y = the centre y - y' * (the same).
//! With the stage's flat-depth switch set the depth divided by is a fixed depth + 1 instead
//! (the real one went to the shared work dword, which nothing read: not kept). With the
//! lights' draw mode at 1 the record's depth is then lowered by 262144.
//! Fatal errors 0x80600902 and 0x80600903 for a number past the object's or the scene's
//! points.
//! One caller, `scene::project_polygon`.
//!
//! `record_offset` is the core's (`rc2k_core::scene::projection`); the arithmetic is the
//! x87's, in plain Rust over `X` (`math/x87_float.rs`), as the original's instructions
//! are kept as recorded in the tests' `reference_vectors`.

use crate::errors::{BadInput, FatalError, Refused};
use crate::math::x87;
use crate::math::x87_float::{ONE, Rounds, at_rounding, int, ld, st};
use crate::scene::point_records;
use crate::scene::x87_mem::{Base, fstp};

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

// ---- Rust version ----

/// `scene::rotate_point` as `project_point` uses it: the point's three whole numbers through the
/// matrix, into the record's +0x10, +0x14, +0x18.
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

/// The record's place on the screen from its place in the view (`checked` gets the depth
/// on the flat way, a work value).
pub(crate) fn to_screen(
    record: &mut [u32],
    window: &crate::scene::view_window::ViewWindow,
    flat_depth: u32,
    flat: bool,
    checked: impl Base,
) {
    let screen = Screen {
        scale: window.view_scale.get(),
        centre: [window.view_centre[0].get(), window.view_centre[1].get()],
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
// Converted: `rc2k_core::scene::projection::record_offset`
pub fn record_offset(point: u32, reads: crate::workers::DrawReads<'_>) -> Result<u32, FatalError> {
    use rc2k_core::scene::projection::TooHigh;
    rc2k_core::scene::projection::record_offset(
        point,
        reads.object_tables.point_count.get(),
        reads.course_work.scene_point_count.get(),
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

/// A scene point's record made anew (the same code as in `project_point`): the point through the view's matrix and onto the screen.
pub(crate) fn make_scene_record(
    part: &crate::workers::PartScratch,
    point: u32,
    record: u32,
    reads: crate::workers::DrawReads<'_>,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), BadInput> {
    let points: u32 = reads.course_work.scene_points.get();
    if points == 0 {
        return Err(BadInput("no points"));
    }
    // Made on a copy of the record (`scene::point_records`), then stored.
    let mut r = point_records::get(part, record)?;
    r[0] = point;
    let place = crate::race::scene_points::point_number(reads.course(), &part.points, point)?;
    rotate(
        &place,
        &view
            .matrices
            .view_matrix
            .each_ref()
            .map(crate::session::PlainFloat::get),
        &mut r,
    );
    r[3] = 0;
    r[7] = 0;
    if f32::from_bits(r[6]) >= NEAR {
        let flat = reads.stage_prefs.flat_depth.get() != 0;
        // The depth it stores is a work value nothing reads.
        to_screen(
            &mut r,
            &view.polygon.window,
            reads.course_work.flat_depth.get(),
            flat,
            &crate::session::Dword::new(0),
        );
    }
    if reads.light_state.draw_mode.get() == 1 {
        r[6] = x87::sub(r[6], LOWERED_BY.to_bits()).to_bits();
    }
    point_records::set(part, record, r)?;
    Ok(())
}

pub fn project_point(
    part: &crate::workers::PartScratch,
    point: u32,
    reads: crate::workers::DrawReads<'_>,
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
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, differential};

    #[test]
    fn rotate_matches_the_assembly() {
        // dwords 0 .. 3 the point, 4 .. 19 the matrix, 20 .. 28 the record
        let test = Differential {
            name: "scene.project_point.rotate",
            cases: 8000,
            seed: 13,
            kinds: [Kind::Float; 28],
            fill: &|rng, m| {
                m.fill(0xAAAA_AAAA);
                for d in m.iter_mut().take(3) {
                    *d = rng.int();
                }
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().skip(4).take(15) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
            },
            plain: &|c| {
                let m: [f32; 16] = std::array::from_fn(|k| f32::from_bits(c[4 + k].get()));
                let place = rotate_at([0, 1, 2].map(|k| c[k].get() as i32), &m);
                for (k, v) in place.into_iter().enumerate() {
                    c[24 + k].set(v.to_bits());
                }
            },
        };
        differential(test);
    }

    /// The window's values and the place in the view of a test's block.
    fn screen_of(c: &[std::cell::Cell<u32>]) -> (Screen, [f32; 3]) {
        let f = |k: usize| f32::from_bits(c[k].get());
        let screen = Screen {
            scale: f(0),
            centre: [f(1), f(2)],
            flat_depth: c[3].get() as i32,
        };
        (screen, [f(12), f(13), f(14)])
    }

    /// The screen's x and y into the test's record (+0xC, +0x1C).
    fn put_screen(c: &[std::cell::Cell<u32>], [x, y]: [u32; 2]) {
        c[11].set(x);
        c[15].set(y);
    }

    /// dwords 0 .. 4 the scale, centre x, centre y and whole number, 4 the checked word, 8 ..
    /// 16 the record (+0x10 .. +0x18 the place in the view).
    fn screen_fill(rng: &mut crate::test_support::vectors::Rng, m: &mut [u32; 16]) {
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
        let test = Differential {
            name: "scene.project_point.screen_flat",
            cases: 8000,
            seed: 14,
            kinds: [Kind::Float; 16],
            fill: &screen_fill,
            plain: &|c| {
                let (screen, view) = screen_of(c);
                put_screen(c, screen_flat(view, screen, &c[4..]));
            },
        };
        differential(test);
    }

    #[test]
    fn screen_plain_matches_the_assembly() {
        let test = Differential {
            name: "scene.project_point.screen_plain",
            cases: 8000,
            seed: 15,
            kinds: [Kind::Float; 16],
            fill: &screen_fill,
            plain: &|c| {
                let (screen, view) = screen_of(c);
                put_screen(c, screen_plain(view, screen));
            },
        };
        differential(test);
    }
}
