//! `scene::place_object_points`: an object's points and normals into the view, the points onto
//! the screen.
//!
//! The object (the car state's `object_number`) has `point_counts[n]` points and then
//! `normal_counts[n]` normals, numbered from `first_point[n]`, twelve bytes each in the
//! course's `object_points` table: a point three whole numbers, a normal three floats. Each
//! gets the point record 0x1000 + (number & 0xFFF) (its +0 = the number). A point goes through
//! the object matrix (`made_object_matrix`; the same arithmetic as `scene::rotate_point`) and
//! onto the screen as `scene::project_point` does it (not if it is nearer than 4096). With the
//! object's `move_by` not 0 the point is first moved towards the one that many bytes further on
//! in the table, by `move_fraction` of the way, rounded to whole numbers (the moved point and
//! the fraction are the drawing's own values). A normal goes through the matrix without its
//! fourth row. Refuses an object's number of 0x100 or more, no records, no points' table, no
//! matrix, and normals without points.
//!
//! The arithmetic (the moved point, the rotation, the normals) is the x87's, in plain Rust over
//! `X` (`math/x87_float.rs`), rounding as the control word says.

use crate::errors::{BadInput, Refused};
use crate::math::x87_float::{Rounds, at_rounding, int, ld, st, to_int};
use crate::scene::point_records;
use crate::scene::project_point::{NEAR, rotate, to_screen};

// ---- game data ----
const OBJECTS: u32 = 0x100;

/// The point at `from` moved towards the one `by` bytes on, as whole numbers.
fn moved(
    object_points: &crate::scene::object_points_store::ObjectPoints,
    from: usize,
    by: u32,
    fraction: f32,
) -> Result<[u32; 3], BadInput> {
    let other = from.wrapping_add(by as usize);
    let read = |at: usize| object_points.point(at as u32);
    let (a, b) = (read(from)?, read(other)?);
    let whole = |p: [u32; 3]| p.map(|v| v as i32);
    let ([x, y, z], _) = moved_at(fraction, whole(a), whole(b));
    // The three are stored from `point_moved` on: y and z land over the first two group masks,
    // which `scene::load_object` writes before it reads them.
    Ok([x as u32, y as u32, z as u32])
}

/// The block of `moved`: the two points' whole numbers, each `a + (b - a) fraction` rounded to
/// a whole number (z worked out first, then y, then x), and the fraction as it is stored.
fn moved_at(fraction: f32, a: [i32; 3], b: [i32; 3]) -> ([i32; 3], u32) {
    let t0 = ld(fraction);
    let along = |k: usize| {
        let from = int(a[k]);
        from + (int(b[k]) - from) * t0
    };
    let (z, y, x) = (along(2), along(1), along(0));
    ([to_int(x), to_int(y), to_int(z)], st(t0))
}

/// A normal's three floats through the matrix's first three rows, into
/// the record's three view floats.
fn turn(normal: &[u32; 3], matrix: &[f32; 16], record: &mut [u32]) {
    record[4..7].copy_from_slice(&turn_normal(normal.map(f32::from_bits), matrix));
}

/// `fld` x3, the three rows' sums of products, `fstp dword` x3 (the floats' bits, as the
/// record holds them).
fn turn_normal(p: [f32; 3], m: &[f32; 16]) -> [u32; 3] {
    at_rounding!(|r| turn_normal_with(r, p, m))
}

/// `turn_normal` at the rounding `r`.
fn turn_normal_with<R: Rounds>(r: R, p: [f32; 3], m: &[f32; 16]) -> [u32; 3] {
    let (t0, t1, t2) = (r.ld(p[2]), r.ld(p[1]), r.ld(p[0]));
    let t10 = r.ld(m[0]) * t2 + (r.ld(m[4]) * t1 + r.ld(m[8]) * t0);
    let t18 = r.ld(m[1]) * t2 + (r.ld(m[5]) * t1 + r.ld(m[9]) * t0);
    let t26 = t2 * r.ld(m[2]) + (t0 * r.ld(m[10]) + t1 * r.ld(m[6]));
    [r.st(t10), r.st(t18), r.st(t26)]
}

/// What the object is made of, read and checked.
struct Object {
    first: u32,
    points: i32,
    normals: i32,
    table: u32,
}

fn object(
    lists: crate::race::course_lists_store::CourseListsRef<'_>,
    object_slots: crate::scene::object_slots_store::SlotShapesRef<'_>,
    cars: &crate::race::car_draw_state::CarDrawState,
) -> Result<Object, BadInput> {
    let number: u32 = cars.borrow().object_number;
    if number >= OBJECTS {
        return Err(BadInput("an object's number past the tables"));
    }
    let object = Object {
        first: object_slots.first_point(number as usize),
        points: object_slots.point_count(number as usize) as i32,
        normals: object_slots.normal_count(number as usize) as i32,
        table: lists.object_points(),
    };
    if object.points <= 0 && object.normals > 0 {
        return Err(BadInput("an object of normals and no points"));
    }
    if object.points > 0 && (object.table == 0 || !point_records::held(lists)) {
        return Err(BadInput("no points' table or no records"));
    }
    Ok(object)
}

/// The object's points placed and projected, its normals turned.
pub fn place_object_points(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let object = object(
        ctx.reads.course_lists,
        ctx.reads.object_slots,
        &ctx.view.cars,
    )?;
    if object.points <= 0 {
        return Ok(());
    }
    // A copy: nothing the loop writes is the matrix (it is always the object's own,
    // `made_object_matrix`, which is never written here).
    let matrix: [f32; 16] = ctx.view.matrices.borrow().made_object_matrix;
    let move_by: u32 = ctx.view.object.borrow().move_by;
    let fraction = f32::from_bits(ctx.view.object.borrow().move_fraction);
    let mut number = object.first;
    let mut from = object.table.wrapping_add(object.first.wrapping_mul(12));
    for _ in 0..object.points {
        // Made on a copy of the record (`scene::point_records`), then stored.
        let record = point_records::of_object_point(number);
        let mut r = point_records::get(ctx.part, record)?;
        r[0] = number;
        let point = if move_by != 0 {
            moved(
                &ctx.reads.stage.object_points,
                from as usize,
                move_by,
                fraction,
            )?
        } else {
            ctx.reads.stage.object_points.point(from)?
        };
        rotate(&point, &matrix, &mut r);
        r[3] = 0;
        r[7] = 0;
        if f32::from_bits(r[6]) >= NEAR {
            let flat = ctx.reads.stage_prefs.flat_depth.get() != 0;
            to_screen(
                &mut r,
                &ctx.view.polygon.window,
                ctx.reads.course_work.get().flat_depth,
                flat,
                &crate::memory::session::Dword::new(0),
            );
        }
        point_records::set(ctx.part, record, r)?;
        from = from.wrapping_add(12);
        number = number.wrapping_add(1);
    }
    for _ in 0..object.normals.max(0) {
        let record = point_records::of_object_point(number);
        let mut r = point_records::get(ctx.part, record)?;
        r[0] = number;
        let normal = ctx.reads.stage.object_points.point(from)?;
        turn(&normal, &matrix, &mut r);
        point_records::set(ctx.part, record, r)?;
        from = from.wrapping_add(12);
        number = number.wrapping_add(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
