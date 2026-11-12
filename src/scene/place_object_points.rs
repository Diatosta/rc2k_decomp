//! `scene::place_object_points` (0x45BE05): an object's points and normals into the
//! view, the points onto the screen.
//!
//! Takes: only matter for an object with normals and no points (refused)
//!
//! The object (the car state's `object_number`) has `point_counts[n]` points and then
//! `normal_counts[n]` normals, numbered from `first_point[n]`, twelve bytes each in the
//! course's `object_points` table: a point three whole numbers, a normal three floats.
//! Each gets the point record 0x1000 + (number & 0xFFF) (its +0 = the number). A point goes
//! through the object matrix (`made_object_matrix`; the same arithmetic as
//! `scene::rotate_point`) and onto the screen as `scene::project_point` does it (not if it
//! is nearer than 4096). With the object's `move_by` not 0 the point is first moved towards
//! the one that many bytes further on in the table, by `move_fraction` of the way, rounded
//! to whole numbers in `point_moved` (the original's store of that fraction to its shared
//! work dword is not kept). A normal goes through the matrix without its fourth row.
//! One caller, `scene::place_object_points_any`, which takes this way unless the 3DNow or
//! the SSE switch is set: `place_object_points_sse` is the SSE way; the 3DNow way is not
//! ported. Refuses an object's number of 0x100 or more, no records, no points' table, no
//! matrix, and normals without points (the original goes on from whatever its registers
//! hold).
//!
//! The arithmetic (the moved point, the rotation, the normals) is the x87's, in plain Rust
//! over `X` (`math/x87_float.rs`), rounding as the control word says.

use crate::errors::{BadInput, Refused};
use crate::scene::point_records;
use crate::scene::project_point::{NEAR, rotate, to_screen};
use crate::scene::x87_mem::{Base, cells, fild, fistp, fld, fstp};

// ---- game data ----
const OBJECTS: u32 = 0x100;

// ---- Rust version ----

/// The point at `from` moved towards the one `by` bytes on, into `point_moved` and the
/// next two cells.
fn moved(
    race: &crate::race::owner::Race,
    from: usize,
    by: u32,
    fraction: u32,
) -> Result<(), BadInput> {
    let course = &race.course;
    let other = from.wrapping_add(by as usize);
    let read = |at: usize| crate::scene::object_points::point(at as u32);
    let (a, b) = (read(from)?, read(other)?);
    // The fraction, the two points, the result, the fraction as it is stored.
    let mut w = [fraction, a[0], a[1], a[2], b[0], b[1], b[2], 0, 0, 0, 0];
    moved_at(cells(&mut w));
    // The original stores the three from `point_moved` on: y and z land over the first two
    // group masks (`notes/bugs.md`).
    course.course.point_moved.set(w[7]);
    course.course.group_masks[0].set(w[8]);
    course.course.group_masks[1].set(w[9]);
    Ok(())
}

/// The block of `moved`: the fraction, the two points, the result (three whole numbers) and the
/// fraction as it is stored.
fn moved_at(w: impl Base) {
    let t0 = fld(w, 0x0);
    let t1 = fild(w, 0xC);
    let t2 = fild(w, 0x18);
    let t3 = t2 - t1;
    let t4 = t3 * t0;
    let t5 = t1 + t4;
    let t6 = fild(w, 0x8);
    let t7 = fild(w, 0x14);
    let t8 = t7 - t6;
    let t9 = t8 * t0;
    let t10 = t6 + t9;
    let t11 = fild(w, 0x4);
    let t12 = fild(w, 0x10);
    let t13 = t12 - t11;
    let t14 = t13 * t0;
    let t15 = t11 + t14;
    fistp(w, 0x1C, t15);
    fistp(w, 0x20, t10);
    fistp(w, 0x24, t5);
    fstp(w, 0x28, t0);
}

/// A normal's three floats through the matrix's first three rows, into
/// the record's +0x10, +0x14, +0x18.
fn turn(normal: &[u32], matrix: &[u32], record: &mut [u32]) {
    turn_normal(normal, matrix, cells(record));
}

fn turn_normal(p: impl Base, m: impl Base, o: impl Base) {
    let t0 = fld(p, 0x8);
    let t1 = fld(p, 0x4);
    let t2 = fld(p, 0x0);
    let t3 = fld(m, 0x0);
    let t4 = t3 * t2;
    let t5 = fld(m, 0x10);
    let t6 = t5 * t1;
    let t7 = fld(m, 0x20);
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = fld(m, 0x4);
    let t12 = t11 * t2;
    let t13 = fld(m, 0x14);
    let t14 = t13 * t1;
    let t15 = fld(m, 0x24);
    let t16 = t15 * t0;
    let t17 = t14 + t16;
    let t18 = t12 + t17;
    let t19 = fld(m, 0x8);
    let t20 = t2 * t19;
    let t21 = fld(m, 0x18);
    let t22 = t1 * t21;
    let t23 = fld(m, 0x28);
    let t24 = t0 * t23;
    let t25 = t24 + t22;
    let t26 = t20 + t25;
    fstp(o, 0x10, t10);
    fstp(o, 0x14, t18);
    fstp(o, 0x18, t26);
}

/// What the object is made of, read and checked.
struct Object {
    first: u32,
    points: i32,
    normals: i32,
    table: u32,
}

fn object(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
    cars: &crate::race::car_draw_state::CarDrawState,
) -> Result<Object, BadInput> {
    let objects = &scene.objects;
    let number: u32 = cars.object_number.get();
    if number >= OBJECTS {
        return Err(BadInput("an object's number past the tables"));
    }
    let object = Object {
        first: objects.slots.first_point[number as usize].get(),
        points: objects.slots.point_counts[number as usize].get() as i32,
        normals: objects.slots.normal_counts[number as usize].get() as i32,
        table: race.course.lists.object_points.get(),
    };
    if object.points <= 0 && object.normals > 0 {
        return Err(BadInput("an object of normals and no points"));
    }
    if object.points > 0 && (object.table == 0 || !point_records::held(&race.course)) {
        return Err(BadInput("no points' table or no records"));
    }
    Ok(object)
}

/// The object's points placed and projected, its normals turned. What the original
/// leaves in the registers its one caller's callers never read (`draw_object`).
pub fn place_object_points(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let race = ctx.race;
    let object = object(race, ctx.scene, &ctx.view.cars)?;
    if object.points <= 0 {
        return Ok(());
    }
    // A copy: nothing the loop writes is the matrix (it is always the object's own,
    // `made_object_matrix`, which is never written here).
    let matrix: [u32; 16] = std::array::from_fn(|n| ctx.view.matrices.made_object_matrix[n].bits());
    let move_by: u32 = ctx.view.object.move_by.get();
    let fraction: u32 = ctx.view.object.move_fraction.get();
    let mut number = object.first;
    let mut from = object.table.wrapping_add(object.first.wrapping_mul(12));
    for _ in 0..object.points {
        // Made on a copy of the record (`scene::point_records`), then stored.
        let record = point_records::of_object_point(number);
        let mut r = point_records::get(ctx.part, record)?;
        r[0] = number;
        let point = if move_by != 0 {
            moved(race, from as usize, move_by, fraction)?;
            // x, y, z read on into `group_masks[0]`, `[1]`, as stored.
            [
                race.course.course.point_moved.get(),
                race.course.course.group_masks[0].get(),
                race.course.course.group_masks[1].get(),
            ]
        } else {
            crate::scene::object_points::point(from)?
        };
        rotate(&point, &matrix, &mut r);
        r[3] = 0;
        r[7] = 0;
        if f32::from_bits(r[6]) >= NEAR {
            let v = [
                ctx.view.polygon.window.view_scale.bits(),
                ctx.view.polygon.window.view_centre[0].bits(),
                ctx.view.polygon.window.view_centre[1].bits(),
                race.course.course.flat_depth.get(),
            ];
            let flat = race.course.stage_prefs.flat_depth.get() != 0;
            to_screen(&mut r, &v, flat, &crate::session::Dword::new(0));
        }
        point_records::set(ctx.part, record, r)?;
        from = from.wrapping_add(12);
        number = number.wrapping_add(1);
    }
    for _ in 0..object.normals.max(0) {
        let record = point_records::of_object_point(number);
        let mut r = point_records::get(ctx.part, record)?;
        r[0] = number;
        let normal = crate::scene::object_points::point(from)?;
        turn(&normal, &matrix, &mut r);
        point_records::set(ctx.part, record, r)?;
        from = from.wrapping_add(12);
        number = number.wrapping_add(1);
    }
    Ok(())
}

/// A point or normal through three rows of the matrix, plus the fourth when `place`: the
/// four lanes the SSE way stores at a record's +0x10 (`mulps`, `addps`, in its order).
fn sse_lanes(p: [f32; 3], m: &[f32; 16], place: bool) -> [u32; 4] {
    std::array::from_fn(|k| {
        let sum = p[0] * m[k] + p[1] * m[k + 4] + p[2] * m[k + 8];
        (if place { sum + m[k + 12] } else { sum }).to_bits()
    })
}

/// The SSE way, reached only with the developers' extensions switch (the debug UI's
/// Switches tab). Not as the x87 way: the matrix is always `made_object_matrix`, the
/// points are never moved, a record's +0x1C holds the fourth lane until the screen's two are
/// worked out, and the table's point 0 ends as a normal in record 0x1000.
pub fn place_object_points_sse(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
) -> Result<(), Refused> {
    let race = ctx.race;
    let object = object(race, ctx.scene, &ctx.view.cars)?;
    let m: [f32; 16] = std::array::from_fn(|n| ctx.view.matrices.made_object_matrix[n].get());
    let mut number = object.first;
    let mut from = object.table.wrapping_add(object.first.wrapping_mul(12));
    for _ in 0..object.points.max(0) {
        let record = point_records::of_object_point(number);
        let mut r = point_records::get(ctx.part, record)?;
        r[0] = number;
        let point = crate::scene::object_points::point(from)?;
        // The whole numbers rounded to floats.
        let lanes = sse_lanes(point.map(|v| v as i32 as f32), &m, true);
        r[4..8].copy_from_slice(&lanes);
        // The screen position.
        r[3] = 0;
        r[7] = 0;
        if f32::from_bits(r[6]) >= NEAR {
            let v = [
                ctx.view.polygon.window.view_scale.bits(),
                ctx.view.polygon.window.view_centre[0].bits(),
                ctx.view.polygon.window.view_centre[1].bits(),
                race.course.course.flat_depth.get(),
            ];
            let flat = race.course.stage_prefs.flat_depth.get() != 0;
            to_screen(&mut r, &v, flat, &crate::session::Dword::new(0));
        }
        point_records::set(ctx.part, record, r)?;
        from = from.wrapping_add(12);
        number = number.wrapping_add(1);
    }
    let normal = |from: u32, number: u32| -> Result<(), Refused> {
        let record = point_records::of_object_point(number);
        let mut r = point_records::get(ctx.part, record)?;
        r[0] = number;
        let n = crate::scene::object_points::point(from)?;
        let lanes = sse_lanes(n.map(f32::from_bits), &m, false);
        r[4..8].copy_from_slice(&lanes);
        point_records::set(ctx.part, record, r)?;
        Ok(())
    };
    for _ in 0..object.normals.max(0) {
        normal(from, number)?;
        from = from.wrapping_add(12);
        number = number.wrapping_add(1);
    }
    normal(object.table, 0)
}

#[cfg(all(test, any(target_arch = "x86", target_arch = "x86_64")))] // x86's instructions
mod sse_tests {
    use super::sse_lanes;
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    /// The lanes as the original's instructions make them.
    #[test]
    fn lanes_match_the_instructions() {
        let m: [f32; 16] = std::array::from_fn(|n| (n as f32 * 1.37 - 7.1).powi(3));
        let p = [123_456.0f32, -9_876_543.0, 0.125];
        let got = sse_lanes(p, &m, true);
        // SAFETY: SSE is on every x86 the game builds for; the loads are unaligned ones
        // of the arrays above.
        let want: [u32; 4] = unsafe {
            let row = |k: usize| _mm_loadu_ps(m[k..k + 4].as_ptr());
            let sum = _mm_add_ps(
                _mm_add_ps(
                    _mm_add_ps(
                        _mm_mul_ps(_mm_set1_ps(p[0]), row(0)),
                        _mm_mul_ps(_mm_set1_ps(p[1]), row(4)),
                    ),
                    _mm_mul_ps(_mm_set1_ps(p[2]), row(8)),
                ),
                row(12),
            );
            std::mem::transmute(sum)
        };
        assert_eq!(got, want);
    }
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, differential};

    #[test]
    fn moved_matches_the_assembly() {
        // dword 0 the fraction, 1 .. 4 the point, 4 .. 7 the other, 7 .. 10 the whole numbers,
        // 10 the fraction as stored
        let test = Differential {
            name: "scene.place_object_points.moved",
            cases: 8000,
            seed: 18,
            kinds: [Kind::Exact; 11],
            fill: &|rng, m| {
                m.fill(0xAAAA_AAAA);
                m[0] = match rng.below(3) {
                    0 => rng.unit().to_bits(),
                    1 => (rng.unit() * 2.0).to_bits(),
                    _ => rng.fl(),
                };
                for d in m.iter_mut().skip(1).take(6) {
                    *d = if rng.below(2) == 0 {
                        rng.int()
                    } else {
                        rng.small() as u32
                    };
                }
            },
            plain: &|c| moved_at(c),
        };
        differential(test);
    }

    #[test]
    fn turn_matches_the_assembly() {
        // dwords 0 .. 3 the normal, 4 .. 19 the matrix, 20 .. 28 the record
        let test = Differential {
            name: "scene.place_object_points.turn",
            cases: 8000,
            seed: 19,
            kinds: [Kind::Exact; 28],
            fill: &|rng, m| {
                m.fill(0xAAAA_AAAA);
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().take(3) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
                for d in m.iter_mut().skip(4).take(15) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
            },
            plain: &|c| turn_normal(c, &c[4..], &c[20..]),
        };
        differential(test);
    }
}
