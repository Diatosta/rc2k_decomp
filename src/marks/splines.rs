//! `init_marks`' step 1 (IDA, 2026-10-16): the splines. `sub_4B96CD` reads
//! `files\m\spline.dat` once and, for each of the settings' objects, `sub_4B9A32` fills the
//! next of 8 places in four tables of `MarkSpline` records (0xE90 bytes): the points of the
//! object's faces of one kind, centred, turned, graded by height and ordered. These are the
//! leaves; every function takes the record in ebp.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::kind_values::MARKS_KIND;
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::session::{Bytes, Dword, Float, read_bits};

pub const SPLINE_LEN: u32 = 0xE90;
/// The most points a record holds.
pub const MOST_POINTS: u32 = 0x80;
const COUNT: u32 = 0x50;
const IDS: u32 = 0x454;
const FIRSTS: u32 = 0x54;
const SECONDS: u32 = 0x254;
const COORDS: u32 = 0x654;
const CENTRE: u32 = 0xC54;
const GRADES: u32 = 0xC90;
// dword_768700
// dword_7687D0

/// A cell's address, as the game's routines are handed it.
fn at<T>(cell: &T) -> u32 {
    crate::session::address_of(cell)
}

fn get(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read("a game field")
}

fn put(at: u32, v: u32) -> Result<(), BadInput> {
    GamePtr::new(at).write(v, "a game field")
}

/// The record's point count, refused past `MOST_POINTS` (the loops would run past it).
fn count(spline: u32) -> Result<u32, Refused> {
    Ok(rc2k_core::marks::splines::points(get(spline + COUNT)?)? as u32)
}

/// `n` dwords at `at` (unaligned: the records are).
fn words(at: u32, n: u32) -> Result<Vec<u32>, BadInput> {
    (0..n).map(|k| get(at + 4 * k)).collect()
}

fn put_words(at: u32, words: &[u32]) -> Result<(), Refused> {
    for (k, &w) in words.iter().enumerate() {
        put(at + 4 * k as u32, w)?;
    }
    Ok(())
}

/// `n` points (three dwords each) at `at`.
fn triples(at: u32, n: u32) -> Result<Vec<[u32; 3]>, BadInput> {
    (0..n)
        .map(|k| {
            Ok([
                get(at + 12 * k)?,
                get(at + 12 * k + 4)?,
                get(at + 12 * k + 8)?,
            ])
        })
        .collect()
}

fn put_triples(at: u32, points: &[[u32; 3]]) -> Result<(), Refused> {
    put_words(at, points.as_flattened())?;
    Ok(())
}

/// `marks::centre_spline_points` (0x4B9F95): record `s`'s points (+0x654, 12 bytes, the
/// last first) less its centre (+0xC54). Refuses more than 0x80 points. Converted:
/// `rc2k_core::marks::splines::centre`.
pub fn centre_spline_points(s: u32) -> Result<(), Refused> {
    {
        let n = count(s)?;
        let mut points = triples(s + COORDS, n)?;
        rc2k_core::marks::splines::centre(&mut points, triples(s + CENTRE, 1)?[0]);
        put_triples(s + COORDS, &points)?;
    }
    Ok(())
}

/// `marks::grade_spline_points` (0x4BA316): record `s`'s points' grades (+0xC90, the last
/// first) by their heights (+0x658): 0 below 0x3830, 1 below 0x6720, else 2. Refuses more
/// than 0x80 points. Converted: `rc2k_core::marks::splines::grade`.
pub fn grade_spline_points(s: u32) -> Result<(), Refused> {
    {
        let n = count(s)?;
        let grades: Vec<u32> = triples(s + COORDS, n)?
            .iter()
            .map(|p| rc2k_core::marks::splines::grade(p[1] as i32))
            .collect();
        put_words(s + GRADES, &grades)?;
    }
    Ok(())
}

/// `marks::find_spline_ends` (0x4BA4F1): of record `s`'s +0xC points from +0 the last
/// with the highest third value, into +0x30; of the +0x10 from +4 the last with the
/// lowest, into +0x34. The value is read at +0x65C + 0x30 a point, not 12 (section 16):
/// read as the original does. Refuses a negative start or count, or points past 0x80.
/// Converted: `rc2k_core::marks::splines::find_end`.
pub fn find_spline_ends(s: u32) -> Result<(), Refused> {
    use rc2k_core::marks::splines::{find_end, run_fits};
    {
        for (start, n) in [(0u32, 0xCu32), (4, 0x10)] {
            run_fits(get(s + start)?, get(s + n)?)?;
        }
        for (start, n, to, highest) in [(0u32, 0xCu32, 0x30u32, true), (4, 0x10, 0x34, false)] {
            let (from, c) = (get(s + start)?, get(s + n)?);
            let bad = crate::marks::Sticky::default();
            let end = find_end(from, c, highest, |p| bad.take(get(s + 0x30 * p + 0x65C)));
            bad.done()?;
            put(s + to, end.at)?;
        }
    }
    Ok(())
}

/// `marks::add_face_points` (0x4B9E29): face `face`'s points (+0 the count, from +0x10, 16
/// bytes each: the point and two values) added to record `s`'s (+0x50 the count, ids
/// +0x454, values +0x54 and +0x254) unless there already, up to 0x80. Refuses a record of
/// more than 0x80 points. Converted: `rc2k_core::marks::splines::FacePoints::add_face`.
pub fn add_face_points(s: u32, face: u32) -> Result<(), Refused> {
    {
        let most = MOST_POINTS as usize;
        let before = get(s + COUNT)?;
        // The ids the searches read; the rest are only written.
        let mut ids = words(s + IDS, before.min(MOST_POINTS))?;
        ids.resize(most, 0);
        let (mut firsts, mut seconds) = (vec![0; most], vec![0; most]);
        let mut points = rc2k_core::marks::splines::FacePoints {
            count: before,
            ids: &mut ids,
            firsts: &mut firsts,
            seconds: &mut seconds,
        };
        let face_points = crate::scene::object_faces::word(face)?;
        let at = |k: u32| face + 0x10 + 0x10 * k;
        let bad = crate::marks::Sticky::default();
        points.add_face(
            face_points,
            |k| bad.take(crate::scene::object_faces::word(at(k))),
            |k| {
                [
                    bad.take(crate::scene::object_faces::word(at(k) + 4)),
                    bad.take(crate::scene::object_faces::word(at(k) + 8)),
                ]
            },
        )?;
        bad.done()?;
        let after = points.count;
        let added = before as usize..after as usize;
        for (table, values) in [(IDS, &ids), (FIRSTS, &firsts), (SECONDS, &seconds)] {
            put_words(s + table + 4 * before, &values[added.clone()])?;
        }
        put(s + COUNT, after)?;
    }
    Ok(())
}

/// `marks::reset_pong` (0x4B7E64): the eight floats from `flt_769182` to four (x, y)
/// pairs from `flt_768688`, `dword_768700` 5, `dword_7687D0` 0. Every register kept.
/// Converted: `rc2k_core::marks::pong::Court::reset` (exact at every precision: `fld;
/// fstp` of a float only makes a signalling NaN quiet).
pub fn reset_pong() {
    let rules = crate::marks::pong::rules();
    crate::marks::pong::with_court(|c| c.reset(&rules));
}

/// `marks::gather_spline_points` (0x4B9D5F): record `s` emptied, then the points of object
/// `object`'s faces of kind `kind` (+7, low five bits) added (`add_face_points`; the faces
/// counted in `g_marksSplineFaces`); the ids made pointers into `g_objectPoints` and the
/// points copied to +0x654 (both the last first). Its last call,
/// `marks::find_farthest_points` (0x4BA578), works inside a `pusha` .. `popa` and keeps
/// nothing (section 16): not made. Refuses an object past 0xFF, a negative face count or
/// faces past `g_faces`.
pub fn gather_spline_points(s: u32, object: u32, kind: u32) -> Result<(), Refused> {
    use crate::scene::object_slots_store::OBJECT_SLOTS;
    use crate::scene::objects::FACES;
    {
        let object = object as usize;
        if object >= 0x100 {
            return Err(BadInput("an object past 0xFF").into());
        }
        let first: u32 = OBJECT_SLOTS.first_face[object].get();
        let faces: u32 = OBJECT_SLOTS.face_counts[object].get();
        if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
            return Err(BadInput("an object's faces past g_faces").into());
        }
        put(s + COUNT, 0)?;
        SPLINE_HEAD.found.set_bits(0u32);
        for f in first..first + faces {
            let face: u32 = crate::scene::objects::face_cell(f as usize).get();
            if (crate::scene::object_faces::byte(face + 7)? & 0x1F) as u32 == kind {
                SPLINE_HEAD
                    .found
                    .set(SPLINE_HEAD.found.get().wrapping_add(1));
                add_face_points(s, face)?;
            }
        }
        let n = get(s + COUNT)?;
        let points: u32 = COURSE_LISTS.object_points.bits();
        for k in (0..n).rev() {
            put(
                s + IDS + 4 * k,
                get(s + IDS + 4 * k)?.wrapping_mul(12).wrapping_add(points),
            )?;
        }
        for k in (0..n).rev() {
            let from = get(s + IDS + 4 * k)?;
            for c in 0..3 {
                put(
                    s + COORDS + k * 12 + 4 * c,
                    crate::scene::object_points::word(from + 4 * c)?,
                )?;
            }
        }
    }
    Ok(())
}

/// An angle as `fld dword [x]; fchs` leaves it.
fn negated(f: u32) -> [u8; 10] {
    let mut a = crate::scene::matrix3::extended(f);
    a[9] ^= 0x80;
    a
}

/// `scene::multiply_matrices3` (`out` = `a` x `b`).
pub fn multiply(a: u32, b: u32, out: u32) {
    crate::scene::matrix3::multiply_matrices3(a, b, out);
}

/// `scene::transform_int_points`: `count` points from `from` turned by the matrix at
/// `matrix` into `to`.
pub fn transform(from: u32, to: u32, count: u32, matrix: u32) -> Result<(), Refused> {
    crate::scene::matrix3::transform_int_points(from, to, count, matrix)
}

/// The X x Y x Z products of `turn_spline_points` and `turn_record`.
fn multiply_turns() {
    for (a, b, out) in [
        (
            &SPLINE_WORK.turn_x[0],
            &SPLINE_WORK.turn_y[0],
            &SPLINE_WORK.turn_xy[0],
        ),
        (
            &SPLINE_WORK.turn_xy[0],
            &SPLINE_WORK.turn_z[0],
            &SPLINE_WORK.turn[0],
        ),
    ] {
        multiply(at(a), at(b), at(out));
    }
}

/// `marks::turn_spline_points` (0x4B9FE6): the turns about X, Y, Z by minus record `s`'s
/// +0x20, +0x24, +0x28 into `g_marksTurnX` .. `Z`, X x Y into `g_marksTurnXY`, that x Z
/// into `g_marksTurn`, and the record's points turned by it in place. Refuses more than
/// 0x80 points or an angle `fsincos` cannot take, before writing anything.
pub fn turn_spline_points(s: u32) -> Result<(), Refused> {
    use crate::scene::matrix3::{Axis, turn_matrix3, turnable};
    count(s)?;
    let mut angles = [negated(0); 3];
    for (a, o) in angles.iter_mut().zip([0x20u32, 0x24, 0x28]) {
        *a = negated(get(s + o)?);
    }
    for a in &angles {
        turnable(a)?;
    }
    for ((axis, to), angle) in [
        (Axis::X, &SPLINE_WORK.turn_x[0]),
        (Axis::Y, &SPLINE_WORK.turn_y[0]),
        (Axis::Z, &SPLINE_WORK.turn_z[0]),
    ]
    .into_iter()
    .zip(angles)
    {
        turn_matrix3(axis, at(to), angle)?;
    }
    multiply_turns();
    transform(
        s + COORDS,
        s + COORDS,
        get(s + COUNT)?,
        at(&SPLINE_WORK.turn[0]),
    )?;
    Ok(())
}

/// `marks::place_shape_points` (0x4BA0AE): the turns about X, Y, Z by record `s`'s +0x20,
/// +0x24, +0x28 (not negated, as `turn_spline_points` does) into `g_marksTurnX` .. `Z`, X x
/// Y into `g_marksTurnXY`, that x Z into `g_marksTurn`; the record's points turned by it
/// into `g_marksOrderedCoords`; then, from the last, each written plus the record's +0xC54
/// .. through the record's pointer at +0x454. Refuses more than 0x80 points, an angle
/// `fsincos` cannot take, and a null pointer among the points', before writing anything.
pub fn place_shape_points(s: u32) -> Result<(), Refused> {
    {
        let n = count(s)?;
        turnable_record(s, [0x20, 0x24, 0x28])?;
        for k in 0..n {
            if get(s + IDS + 4 * k)? == 0 {
                return Err(BadInput("a null point pointer").into());
            }
        }
        turn_record(s, [0x20, 0x24, 0x28])?;
        transform_int_points_from(s + COORDS, get(s + COUNT)?)?;
        for k in (0..n).rev() {
            write_point(s, k, k)?;
        }
    }
    Ok(())
}

/// Whether the record's three angles (at `offsets`, not negated) can be turned by.
pub fn turnable_record(s: u32, offsets: [u32; 3]) -> Result<(), Refused> {
    use crate::scene::matrix3::{extended, turnable};
    for o in offsets {
        turnable(&extended(get(s + o)?))?;
    }
    Ok(())
}

/// The turns about X, Y, Z by the record's floats at `offsets` (not negated) into
/// `g_marksTurnX` .. `Z`, X x Y into `g_marksTurnXY`, that x Z into `g_marksTurn`, as
/// `sub_4BA0AE` and `sub_4BA16C` do.
pub fn turn_record(s: u32, offsets: [u32; 3]) -> Result<(), Refused> {
    use crate::scene::matrix3::{Axis, extended, turn_matrix3};
    for ((axis, to), o) in [
        (Axis::X, &SPLINE_WORK.turn_x[0]),
        (Axis::Y, &SPLINE_WORK.turn_y[0]),
        (Axis::Z, &SPLINE_WORK.turn_z[0]),
    ]
    .into_iter()
    .zip(offsets)
    {
        turn_matrix3(axis, at(to), extended(get(s + o)?))?;
    }
    multiply_turns();
    Ok(())
}

/// `count` points from `from` turned by `g_marksTurn` into `g_marksOrderedCoords`
/// (`scene::transform_int_points`).
pub fn transform_int_points_from(from: u32, count: u32) -> Result<(), Refused> {
    transform(
        from,
        at(&SPLINE_WORK.ordered_coords[0]),
        count,
        at(&SPLINE_WORK.turn[0]),
    )
}

/// Ordered point `k` plus record `s`'s +0xC54 .. written through the record's pointer
/// `pointer` (at +0x454).
pub fn write_point(s: u32, k: u32, pointer: u32) -> Result<(), Refused> {
    {
        let to = get(s + IDS + 4 * pointer)?;
        let from = SPLINE_WORK.ordered_coords[0].addr() + 12 * k;
        for c in 0..3 {
            // `to` is a place of `scene::object_points` (phase 13).
            let v = get(from + 4 * c)?.wrapping_add(get(s + CENTRE + 4 * c)?);
            crate::scene::object_points::write(to + 4 * c, &v.to_le_bytes())?;
        }
    }
    Ok(())
}

/// `marks::order_spline_points` (0x4BA388): record `s`'s ids and points put in grade
/// order (each grade's the last first) through `g_marksOrderedIds` and
/// `g_marksOrderedCoords`; +0 .. +8 each grade's start, +0xC .. +0x14 its count; then the
/// count's ids and points moved back (as `memory::move_bytes` would, as many as the count,
/// whatever their grade). Refuses more than 0x80 points or a negative count. Converted:
/// `rc2k_core::marks::splines::order`.
pub fn order_spline_points(s: u32) -> Result<(), Refused> {
    {
        let n = count(s)?;
        if (get(s + COUNT)? as i32) < 0 {
            return Err(BadInput("a spline of a negative count: the moves would run away").into());
        }
        let (ids_at, coords_at) = (
            SPLINE_WORK.ordered_ids[0].addr(),
            SPLINE_WORK.ordered_coords[0].addr(),
        );
        // What the moves back read: the ordered tables' first `n`, those not placed as
        // they were.
        let mut ordered_ids = words(ids_at, n)?;
        let mut ordered_coords = triples(coords_at, n)?;
        let o = rc2k_core::marks::splines::order(
            &words(s + GRADES, n)?,
            &words(s + IDS, n)?,
            &triples(s + COORDS, n)?,
            &mut ordered_ids,
            &mut ordered_coords,
        );
        let placed = o.placed as usize;
        put_words(ids_at, &ordered_ids[..placed])?;
        put_triples(coords_at, &ordered_coords[..placed])?;
        put_words(s, &o.starts)?;
        put_words(s + 0xC, &o.counts)?;
        put_words(s + IDS, &ordered_ids)?;
        put_triples(s + COORDS, &ordered_coords)?;
    }
    Ok(())
}

/// `marks::bound_spline_points` (0x4B9E7D): record `s`'s points' smallest and largest in
/// each part; the y and z sizes as floats into `g_marksSplineSize` + 4, + 8 (its x is
/// left as it was: section 16); the box's middle (half the size, `sar`, plus the smallest)
/// into the centre (+0xC54); the angle between `g_marksSplineUp` and the size
/// (`scene::angle_between_vectors`, both normalised) into +0x20. Refuses more than 0x80
/// points. Converted: `rc2k_core::marks::splines::bounds` (the floats, the sizes made
/// floats and the angle, stay the x87's: the first round as its control word says).
pub fn bound_spline_points(s: u32) -> Result<(), Refused> {
    let n = count(s)?;
    let b = rc2k_core::marks::splines::bounds(&triples(s + COORDS, n)?);
    let size = b.size;
    let up = SPLINE_WORK.up[0].addr();
    let size_at = SPLINE_WORK.size[0].addr();
    put(size_at + 4, x87::from_int(size[1]))?;
    put(size_at + 8, x87::from_int(size[2]))?;
    for (c, &m) in b.middle.iter().enumerate() {
        put(s + CENTRE + 4 * c as u32, m as u32)?;
    }
    let angle = crate::scene::vector3::angle_between_vectors(up, size_at);
    put(s + 0x20, crate::scene::matrix3::rounded(angle))?;
    Ok(())
}

/// What `make_spline` refuses for an object: past 0xFF, a negative face count or faces past
/// `g_faces`, an angle of its tables `fsincos` cannot take.
fn spline_checks(object: u32) -> Result<(), Refused> {
    use crate::scene::object_slots_store::OBJECT_SLOTS;
    use crate::scene::objects::FACES;
    if object >= 0x100 {
        return Err(BadInput("an object past 0xFF").into());
    }
    let first: u32 = OBJECT_SLOTS.first_face[object as usize].get();
    let faces: u32 = OBJECT_SLOTS.face_counts[object as usize].get();
    if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
        return Err(BadInput("an object's faces past g_faces").into());
    }
    for n in 0..12 {
        crate::scene::matrix3::turnable(&negated(SPLINE_CONSTANTS.angles[n].bits()))?;
    }
    Ok(())
}

/// `marks::make_spline` (0x4B9A32): `object` into `g_marksSplineObject`; unless 8 are
/// made (`g_marksSplinePlace`), the next record of each table for it: of the faces of that
/// table's kind gathered, bounded, centred, given the table's angles and turned; the second
/// and third also given the first table's record 0's centre and +0x20 first (record 0's
/// whatever the place: section 16) and then graded (+0x18, +0x1C 0x58F0, 0x6F78), ordered
/// and their ends found; the third's +0x44 .. the highest point plus `g_marksSplineShift`.
/// Then the place counted. Refuses a negative place, an object past 0xFF or faces past
/// `g_faces`, an angle `fsincos` cannot take, before writing anything else.
pub fn make_spline(object: u32) -> Result<(), Refused> {
    {
        SPLINE_WORK.object.set_bits(object);
        let place = SPLINE_WORK.place.bits() as i32;
        if place >= 8 {
            return Ok(());
        }
        if place < 0 {
            return Err(BadInput("a negative spline place").into());
        }
        spline_checks(object)?;
        let offset = place as u32 * SPLINE_LEN;
        SPLINE_WORK.offset.set_bits(offset);
        let table = |t: usize| (SPLINE_TABLES.table(t)) as u32;
        for t in 0..4 {
            let record = table(t) + offset;
            gather_spline_points(record, object, SPLINE_KINDS.chosen[1 + t].bits())?;
            bound_spline_points(record)?;
            if t == 1 || t == 2 {
                for o in [CENTRE, CENTRE + 4, CENTRE + 8, 0x20] {
                    put(table(t) + o, get(table(0) + o)?)?;
                }
            }
            centre_spline_points(record)?;
            for (k, o) in [0x24u32, 0x2C, 0x28].into_iter().enumerate() {
                put(
                    record + o,
                    x87::loaded(SPLINE_CONSTANTS.angles[3 * t + k].bits()),
                )?;
            }
            turn_spline_points(record)?;
            if t == 1 || t == 2 {
                put(record + 0x18, 0x58F0)?;
                put(record + 0x1C, 0x6F78)?;
                grade_spline_points(record)?;
                order_spline_points(record)?;
                find_spline_ends(record)?;
            }
            if t == 2 {
                let shift: [u32; 3] = read_bits(&SPLINE_HEAD.shift);
                let p = record
                    .wrapping_add(COORDS)
                    .wrapping_add(get(record + 0x30)?.wrapping_mul(12));
                for c in 0..3 {
                    put(
                        record + 0x44 + 4 * c,
                        shift[c as usize].wrapping_add(get(p + 4 * c)?),
                    )?;
                }
            }
        }
        SPLINE_WORK.place.set_bits(place as u32 + 1);
    }
    Ok(())
}
pub const RESUME_RVA: usize = rva(0x4B9706); // loc_4B9706
pub const FILE_LEN: u32 = 0xC00;
const KINDS: u32 = 0x22;
/// The settings' objects: a count at +0x26B8, then the objects.
const OBJECTS_AT: u32 = 0x26B8;
const MOST_OBJECTS: u32 = (0x2708 - 0x26BC) / 4;

/// `marks::load_splines` (0x4B96CD), in step 1: once (`g_marksSplinesLoaded`, then
/// `resume`) `files\m\spline.dat` (0xC00 bytes at most, `files::read_whole_file`) into
/// `g_marksSplineFile`; the five values of `g_marksKind` into 0x788BB2 ..;
/// `g_marksSplinePlace` 0; `make_spline` for each of the settings' objects (+0x26BC, the last
/// first); `dword_7695CE` 0. Refuses a mark neither 0 nor `resume`, a kind of 0x22 or
/// more, more than 19 objects, and what `make_spline` would refuse for any of them, before
/// writing anything.
pub fn load_splines(resume: u32) -> Result<(), Refused> {
    let loaded: u32 = SPLINE_KINDS.loaded.bits();
    if loaded != 0 && loaded != resume {
        return Err(BadInput("the splines' mark is neither 0 nor loc_4B9706").into());
    }
    let kind: u32 = MARKS_KIND.kind.bits();
    if kind >= KINDS {
        return Err(BadInput("a marks' kind of 0x22 or more").into());
    }
    let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
    let count = get(settings + OBJECTS_AT)? as i32;
    if count > MOST_OBJECTS as i32 {
        return Err(BadInput("more than 19 spline objects").into());
    }
    let objects: Vec<u32> = (0..count.max(0) as u32)
        .rev()
        .map(|k| get(settings + OBJECTS_AT + 4 + 4 * k))
        .collect::<Result<_, _>>()?;
    // The place starts at 0: the first eight are made, the rest only noted.
    for &o in objects.iter().take(8) {
        spline_checks(o)?;
    }
    if loaded == 0 {
        let file = SPLINE_HEAD.file.addr();
        crate::files::read_whole_file::read_whole_file(
            crate::pointers::host(SPLINE_PATH.path.addr()) as *const u8,
            crate::pointers::host(file),
            FILE_LEN,
        )?;
        SPLINE_KINDS.loaded.set_bits(resume);
    }
    for n in 0..5 {
        let v = SPLINE_KINDS.by_kind[(n * KINDS + kind) as usize].bits();
        SPLINE_KINDS.chosen[n as usize].set(v);
    }
    SPLINE_WORK.place.set_bits(0u32);
    for o in objects {
        make_spline(o)?;
    }
    SPLINE_HEAD.made.set_bits(0u32);
    Ok(())
}

/// The car models' tables and the splines' head: the points and looks by car model (`marks::emit_particle_13`, `marks::draw_mark_slots`, `marks::cars_particles`), then the spline variables the looks table runs into (the original reads two dwords past it) and the spline file (IDA 0x769312 .. 0x76A1D2, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SplineHead {
    /// 0x769312: `g_marksPoints`: three floats by car model (34 models), a mark slot's point is an index into it (12 bytes each).
    pub points: [Float; 102],
    /// 0x7694AA: `dword_7694AA`, `dword_7694B2` ..: the looks by the car's model (two dwords 8 bytes apart, 33 pairs; models 32 and 33 read past the end: `found`, `wave`).
    pub looks: [Dword; 66],
    /// 0x7695B2: `g_marksSplineFaces`: how many faces of the kind `gather_spline_points` found.
    pub found: Dword,
    /// 0x7695B6: `flt_7695B6`: the wave's angle (`marks::run_splines`).
    pub angle: Float,
    /// 0x7695BA: `flt_7695BA`: the wave (`marks::run_splines`).
    pub wave: Float,
    /// 0x7695BE: `g_marksSplineShift`: added to the third table's highest point for its +0x44.
    pub shift: [Dword; 3],
    /// 0x7695CA: `dword_7695CA`: 0 or 1, which of the two graded records is being placed.
    pub switch: Dword,
    /// 0x7695CE: `g_marksSplinesMade`: the splines' fade-in counter, 0 .. 0xFF.
    pub made: Dword,
    /// 0x7695D2: `g_marksSplineFile`: `files\m\spline.dat` (0xC00 bytes at most): 12 bytes an entry, the blend weights.
    pub file: Bytes<3072>,
}

pub static SPLINE_HEAD: SplineHead = SplineHead {
    points: [
        Float::from_bits(0x4501619A),
        Float::from_bits(0x4384CCCD),
        Float::from_bits(0xC40F2000),
        Float::from_bits(0x45007333),
        Float::from_bits(0x4387B333),
        Float::from_bits(0xC3E2B333),
        Float::from_bits(0x45077800),
        Float::from_bits(0x4365E666),
        Float::from_bits(0xC3E7C000),
        Float::from_bits(0x450B8333),
        Float::from_bits(0x4384F333),
        Float::from_bits(0x43EC4000),
        Float::from_bits(0x4509C000),
        Float::from_bits(0x4389C000),
        Float::from_bits(0x43D5C000),
        Float::from_bits(0x44FB899A),
        Float::from_bits(0x43363333),
        Float::from_bits(0xC3F30CCD),
        Float::from_bits(0x45054E66),
        Float::from_bits(0x43940000),
        Float::from_bits(0x43EE4000),
        Float::from_bits(0x45150666),
        Float::from_bits(0x436C4CCD),
        Float::from_bits(0xC408CCCD),
        Float::from_bits(0x45088666),
        Float::from_bits(0x432D8000),
        Float::from_bits(0xC3CDE666),
        Float::from_bits(0x4509819A),
        Float::from_bits(0x437C0000),
        Float::from_bits(0x43C60CCD),
        Float::from_bits(0x4518CE66),
        Float::from_bits(0x4381E666),
        Float::from_bits(0xC3E57333),
        Float::from_bits(0x44FC7333),
        Float::from_bits(0x43703333),
        Float::from_bits(0xC4007333),
        Float::from_bits(0x45050000),
        Float::from_bits(0x4368CCCD),
        Float::from_bits(0x43EE4000),
        Float::from_bits(0x44EA5000),
        Float::from_bits(0x4374199A),
        Float::from_bits(0x43C4199A),
        Float::from_bits(0x44FEB99A),
        Float::from_bits(0x434B999A),
        Float::from_bits(0xC406ACCD),
        Float::from_bits(0x450C3333),
        Float::from_bits(0x4383D99A),
        Float::from_bits(0xC406A666),
        Float::from_bits(0x4512719A),
        Float::from_bits(0x439F8CCD),
        Float::from_bits(0xC3FAC000),
        Float::from_bits(0x450574CD),
        Float::from_bits(0x435C0000),
        Float::from_bits(0xC3CF0000),
        Float::from_bits(0x44F3699A),
        Float::from_bits(0x43AA2666),
        Float::from_bits(0xC4091333),
        Float::from_bits(0x44FDB000),
        Float::from_bits(0x435C0000),
        Float::from_bits(0xC3E30CCD),
        Float::from_bits(0x450BE99A),
        Float::from_bits(0x4395A666),
        Float::from_bits(0xC384599A),
        Float::from_bits(0x44FD1000),
        Float::from_bits(0x43608000),
        Float::from_bits(0x43CAD99A),
        Float::from_bits(0x4513D4CD),
        Float::from_bits(0x436C199A),
        Float::from_bits(0xC4078CCD),
        Float::from_bits(0x44E74000),
        Float::from_bits(0x43AF0000),
        Float::from_bits(0xC3C80000),
        Float::from_bits(0x4500C000),
        Float::from_bits(0x438C0000),
        Float::from_bits(0x43F00000),
        Float::from_bits(0x45048000),
        Float::from_bits(0x43820000),
        Float::from_bits(0xC4020000),
        Float::from_bits(0x448C0000),
        Float::from_bits(0x43340000),
        Float::from_bits(0x0),
        Float::from_bits(0x448C0000),
        Float::from_bits(0x448C0000),
        Float::from_bits(0x0),
        Float::from_bits(0x446D8000),
        Float::from_bits(0x44C80000),
        Float::from_bits(0xC3960000),
        Float::from_bits(0x44480000),
        Float::from_bits(0x447A0000),
        Float::from_bits(0xC3960000),
        Float::from_bits(0x451C4000),
        Float::from_bits(0x437A0000),
        Float::from_bits(0x0),
        Float::from_bits(0x44B54000),
        Float::from_bits(0x452F0000),
        Float::from_bits(0x80000000),
        Float::from_bits(0x44B54000),
        Float::from_bits(0x452F0000),
        Float::from_bits(0x80000000),
        Float::from_bits(0x44F12000),
        Float::from_bits(0x43998000),
        Float::from_bits(0xC32E0000),
    ],
    looks: [
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
        Dword::new(0xD800),
        Dword::new(0x14000),
    ],
    found: Dword::new(0x0),
    angle: Float::from_bits(0x0),
    wave: Float::from_bits(0x0),
    shift: [Dword::new(0x4000), Dword::new(0xFFFF8000), Dword::new(0x0)],
    switch: Dword::new(0x0),
    made: Dword::new(0x0),
    file: Bytes::new([0; 3072]),
};

impl SplineHead {
    pub const START: usize = 0x769312;
    pub const LEN: usize = 0xEC0;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&SPLINE_HEAD as *const SplineHead) as usize
    }
}

#[cfg(test)]
mod spline_head_tests {
    use super::*;

    #[test]
    fn spline_head_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplineHead>(), SplineHead::LEN);
        let p = &SPLINE_HEAD as *const SplineHead as *const u8;
        let ours: Vec<u8> = (0..size_of::<SplineHead>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(SplineHead::START, SplineHead::LEN),
            "SPLINE_HEAD starts differently from the image"
        );
        assert_eq!(offset_of!(SplineHead, points), 0x769312 - SplineHead::START);
        assert_eq!(offset_of!(SplineHead, looks), 0x7694AA - SplineHead::START);
        assert_eq!(offset_of!(SplineHead, found), 0x7695B2 - SplineHead::START);
        assert_eq!(offset_of!(SplineHead, angle), 0x7695B6 - SplineHead::START);
        assert_eq!(offset_of!(SplineHead, wave), 0x7695BA - SplineHead::START);
        assert_eq!(offset_of!(SplineHead, shift), 0x7695BE - SplineHead::START);
        assert_eq!(offset_of!(SplineHead, switch), 0x7695CA - SplineHead::START);
        assert_eq!(offset_of!(SplineHead, made), 0x7695CE - SplineHead::START);
        assert_eq!(offset_of!(SplineHead, file), 0x7695D2 - SplineHead::START);
    }
}

/// The splines' work places (`marks::bend_points`, `marks::splines`, `marks::place_spline`) (IDA 0x76A1D2 .. 0x76B9B2, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SplineWork {
    /// 0x76A1D2: `dword_76A1D2` ..: four points of three whole numbers: A, B, C and D (`marks::bend_points`).
    pub bend: [Dword; 12],
    /// 0x76A202: `g_marksSplineObject`: the object being made.
    pub object: Dword,
    /// 0x76A206: `g_marksSplinePlace`: the next of 8 places.
    pub place: Dword,
    /// 0x76A20A: `g_marksSplineOffset`: the place times 0xE90.
    pub offset: Dword,
    /// 0x76A20E: Not named: 8 bytes (a gap, or unreferenced data).
    pub _gap_76a20e: Bytes<8>,
    /// 0x76A216: `g_marksSplineUp`: the vector the angle is measured from.
    pub up: [Float; 3],
    /// 0x76A222: `g_marksSplineSize`: the points' size, as floats.
    pub size: [Float; 3],
    /// 0x76A22E: `g_marksTurnXY`: the X turn times the Y turn (a matrix of 12 floats).
    pub turn_xy: [Float; 12],
    /// 0x76A25E: `g_marksTurn`: all three turns.
    pub turn: [Float; 12],
    /// 0x76A28E: `g_marksTurnX`.
    pub turn_x: [Float; 12],
    /// 0x76A2BE: `g_marksTurnY`.
    pub turn_y: [Float; 12],
    /// 0x76A2EE: `g_marksTurnZ`.
    pub turn_z: [Float; 12],
    /// 0x76A31E: `g_marksOrderedIds`: `order_spline_points`' work, one id a point (at most 0x80 points).
    pub ordered_ids: [Dword; 128],
    /// 0x76A51E: `g_marksOrderedCoords`: the ordered points, three whole numbers each.
    pub ordered_coords: [Dword; 384],
    /// 0x76AB1E: Not named: 3732 bytes (a gap, or unreferenced data).
    pub _gap_76ab1e: Bytes<3732>,
}

pub static SPLINE_WORK: SplineWork = SplineWork {
    bend: [const { Dword::new(0x0) }; 12],
    object: Dword::new(0x0),
    place: Dword::new(0x0),
    offset: Dword::new(0x0),
    _gap_76a20e: Bytes::new([0; 8]),
    up: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x3F800000),
    ],
    size: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
    turn_xy: [const { Float::from_bits(0x0) }; 12],
    turn: [const { Float::from_bits(0x0) }; 12],
    turn_x: [const { Float::from_bits(0x0) }; 12],
    turn_y: [const { Float::from_bits(0x0) }; 12],
    turn_z: [const { Float::from_bits(0x0) }; 12],
    ordered_ids: [const { Dword::new(0x0) }; 128],
    ordered_coords: [const { Dword::new(0x0) }; 384],
    _gap_76ab1e: Bytes::new(crate::session::with_runs::<3732>(&[(
        0x3C,
        &[
            0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
            0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
        ],
    )])),
};

impl SplineWork {
    pub const START: usize = 0x76A1D2;
    pub const LEN: usize = 0x17E0;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&SPLINE_WORK as *const SplineWork) as usize
    }
}

#[cfg(test)]
mod spline_work_tests {
    use super::*;

    #[test]
    fn spline_work_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplineWork>(), SplineWork::LEN);
        let p = &SPLINE_WORK as *const SplineWork as *const u8;
        let ours: Vec<u8> = (0..size_of::<SplineWork>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(SplineWork::START, SplineWork::LEN),
            "SPLINE_WORK starts differently from the image"
        );
        assert_eq!(offset_of!(SplineWork, bend), 0x76A1D2 - SplineWork::START);
        assert_eq!(offset_of!(SplineWork, object), 0x76A202 - SplineWork::START);
        assert_eq!(offset_of!(SplineWork, place), 0x76A206 - SplineWork::START);
        assert_eq!(offset_of!(SplineWork, offset), 0x76A20A - SplineWork::START);
        assert_eq!(offset_of!(SplineWork, up), 0x76A216 - SplineWork::START);
        assert_eq!(offset_of!(SplineWork, size), 0x76A222 - SplineWork::START);
        assert_eq!(
            offset_of!(SplineWork, turn_xy),
            0x76A22E - SplineWork::START
        );
        assert_eq!(offset_of!(SplineWork, turn), 0x76A25E - SplineWork::START);
        assert_eq!(offset_of!(SplineWork, turn_x), 0x76A28E - SplineWork::START);
        assert_eq!(offset_of!(SplineWork, turn_y), 0x76A2BE - SplineWork::START);
        assert_eq!(offset_of!(SplineWork, turn_z), 0x76A2EE - SplineWork::START);
        assert_eq!(
            offset_of!(SplineWork, ordered_ids),
            0x76A31E - SplineWork::START
        );
        assert_eq!(
            offset_of!(SplineWork, ordered_coords),
            0x76A51E - SplineWork::START
        );
    }
}

/// The four tables of eight spline records of 0xE90 bytes (`marks::splines`) (IDA 0x76B9B2 .. 0x788BB2, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SplineTables {
    /// 0x76B9B2: `g_marksSplinesA`: eight `MarkSpline` records.
    pub a: Bytes<29824>,
    /// 0x772E32: `g_marksSplinesB`.
    pub b: Bytes<29824>,
    /// 0x77A2B2: `g_marksSplinesC`.
    pub c: Bytes<29824>,
    /// 0x781732: `g_marksSplinesD`.
    pub d: Bytes<29824>,
}

pub static SPLINE_TABLES: SplineTables = SplineTables {
    a: Bytes::new(crate::session::with_runs::<29824>(&[
        (
            0x38,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0xEC8,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x1D58,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x2BE8,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x3A78,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x4908,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x5798,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x6628,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
    ])),
    b: Bytes::new(crate::session::with_runs::<29824>(&[
        (
            0x38,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0xEC8,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x1D58,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x2BE8,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x3A78,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x4908,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x5798,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x6628,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
    ])),
    c: Bytes::new(crate::session::with_runs::<29824>(&[
        (
            0x38,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0xEC8,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x1D58,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x2BE8,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x3A78,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x4908,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x5798,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x6628,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
    ])),
    d: Bytes::new(crate::session::with_runs::<29824>(&[
        (
            0x38,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0xEC8,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x1D58,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x2BE8,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x3A78,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x4908,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x5798,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
        (
            0x6628,
            &[
                0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x2E,
                0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
            ],
        ),
    ])),
};

impl SplineTables {
    pub const START: usize = 0x76B9B2;
    pub const LEN: usize = 0x1D200;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&SPLINE_TABLES as *const SplineTables) as usize
    }
}

#[cfg(test)]
mod spline_tables_tests {
    use super::*;

    #[test]
    fn spline_tables_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplineTables>(), SplineTables::LEN);
        let p = &SPLINE_TABLES as *const SplineTables as *const u8;
        let ours: Vec<u8> = (0..size_of::<SplineTables>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(SplineTables::START, SplineTables::LEN),
            "SPLINE_TABLES starts differently from the image"
        );
        assert_eq!(offset_of!(SplineTables, a), 0x76B9B2 - SplineTables::START);
        assert_eq!(offset_of!(SplineTables, b), 0x772E32 - SplineTables::START);
        assert_eq!(offset_of!(SplineTables, c), 0x77A2B2 - SplineTables::START);
        assert_eq!(offset_of!(SplineTables, d), 0x781732 - SplineTables::START);
    }
}

/// The splines' values by kind (`marks::splines`) (IDA 0x788BB2 .. 0x788EFA, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SplineKinds {
    /// 0x788BB2: `g_marksSplineChosen`, then `g_marksSplineKinds` (four): the five values of `g_marksKind`.
    pub chosen: [Dword; 5],
    /// 0x788BC6: `dword_788BC6` .. `dword_788DE6`: five tables of 0x22 values by kind.
    pub by_kind: [Dword; 170],
    /// 0x788E6E: Not named: 136 bytes (a gap, or unreferenced data).
    pub _gap_788e6e: Bytes<136>,
    /// 0x788EF6: `g_marksSplinesLoaded`: 0, or where to go on.
    pub loaded: Dword,
}

pub static SPLINE_KINDS: SplineKinds = SplineKinds {
    chosen: [
        Dword::new(0x0),
        Dword::new(0x13),
        Dword::new(0x19),
        Dword::new(0x18),
        Dword::new(0x1C),
    ],
    by_kind: [
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x1),
        Dword::new(0x0),
        Dword::new(0x1),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x1),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x13),
        Dword::new(0x13),
        Dword::new(0xC),
        Dword::new(0x12),
        Dword::new(0x13),
        Dword::new(0x13),
        Dword::new(0x15),
        Dword::new(0x10),
        Dword::new(0x4),
        Dword::new(0x15),
        Dword::new(0x10),
        Dword::new(0xE),
        Dword::new(0x13),
        Dword::new(0x1C),
        Dword::new(0x13),
        Dword::new(0x15),
        Dword::new(0x11),
        Dword::new(0xE),
        Dword::new(0xA),
        Dword::new(0xB),
        Dword::new(0x11),
        Dword::new(0x16),
        Dword::new(0x10),
        Dword::new(0xE),
        Dword::new(0x15),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x13),
        Dword::new(0x18),
        Dword::new(0x18),
        Dword::new(0x1A),
        Dword::new(0x19),
        Dword::new(0x1C),
        Dword::new(0x18),
        Dword::new(0x1B),
        Dword::new(0x1C),
        Dword::new(0x18),
        Dword::new(0x1A),
        Dword::new(0x19),
        Dword::new(0x19),
        Dword::new(0x1A),
        Dword::new(0x12),
        Dword::new(0x18),
        Dword::new(0x18),
        Dword::new(0x1A),
        Dword::new(0x12),
        Dword::new(0xD),
        Dword::new(0xE),
        Dword::new(0x14),
        Dword::new(0x19),
        Dword::new(0x1C),
        Dword::new(0x11),
        Dword::new(0x1B),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x18),
        Dword::new(0x19),
        Dword::new(0x19),
        Dword::new(0x1B),
        Dword::new(0x1A),
        Dword::new(0x1B),
        Dword::new(0x19),
        Dword::new(0x1A),
        Dword::new(0x1B),
        Dword::new(0x19),
        Dword::new(0x19),
        Dword::new(0x18),
        Dword::new(0x1A),
        Dword::new(0x19),
        Dword::new(0x11),
        Dword::new(0x19),
        Dword::new(0x19),
        Dword::new(0x19),
        Dword::new(0x11),
        Dword::new(0xE),
        Dword::new(0xF),
        Dword::new(0x15),
        Dword::new(0x1A),
        Dword::new(0x1B),
        Dword::new(0x12),
        Dword::new(0x1A),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x19),
        Dword::new(0x1C),
        Dword::new(0x1C),
        Dword::new(0x1C),
        Dword::new(0x1B),
        Dword::new(0x1F),
        Dword::new(0x1C),
        Dword::new(0x1C),
        Dword::new(0x1F),
        Dword::new(0x1B),
        Dword::new(0x1B),
        Dword::new(0x1A),
        Dword::new(0x1B),
        Dword::new(0x1B),
        Dword::new(0x8),
        Dword::new(0x1A),
        Dword::new(0x1B),
        Dword::new(0x1C),
        Dword::new(0x1C),
        Dword::new(0x1B),
        Dword::new(0x1A),
        Dword::new(0x17),
        Dword::new(0x1C),
        Dword::new(0x1F),
        Dword::new(0x14),
        Dword::new(0x1C),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0xFFFFFFFF),
        Dword::new(0x1C),
    ],
    _gap_788e6e: Bytes::new([
        0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C,
        0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC,
        0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD,
        0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F,
        0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C,
        0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC,
        0x4C, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCD, 0xCC, 0x4C,
        0x3F,
    ]),
    loaded: Dword::new(0x0),
};

impl SplineKinds {
    pub const START: usize = 0x788BB2;
    pub const LEN: usize = 0x348;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&SPLINE_KINDS as *const SplineKinds) as usize
    }
}

#[cfg(test)]
mod spline_kinds_tests {
    use super::*;

    #[test]
    fn spline_kinds_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplineKinds>(), SplineKinds::LEN);
        let p = &SPLINE_KINDS as *const SplineKinds as *const u8;
        let ours: Vec<u8> = (0..size_of::<SplineKinds>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(SplineKinds::START, SplineKinds::LEN),
            "SPLINE_KINDS starts differently from the image"
        );
        assert_eq!(
            offset_of!(SplineKinds, chosen),
            0x788BB2 - SplineKinds::START
        );
        assert_eq!(
            offset_of!(SplineKinds, by_kind),
            0x788BC6 - SplineKinds::START
        );
        assert_eq!(
            offset_of!(SplineKinds, loaded),
            0x788EF6 - SplineKinds::START
        );
    }
}

/// The splines' file name (`marks::splines`) (IDA 0x788EFA .. 0x788F0D, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SplinePath {
    /// 0x788EFA: `files\m\spline.dat`, the file's name.
    pub path: Bytes<19>,
}

pub static SPLINE_PATH: SplinePath = SplinePath {
    path: Bytes::new([
        0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x73, 0x70, 0x6C, 0x69, 0x6E, 0x65, 0x2E,
        0x64, 0x61, 0x74, 0x00,
    ]),
};

impl SplinePath {
    pub const START: usize = 0x788EFA;
    pub const LEN: usize = 0x13;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&SPLINE_PATH as *const SplinePath) as usize
    }
}

#[cfg(test)]
mod spline_path_tests {
    use super::*;

    #[test]
    fn spline_path_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplinePath>(), SplinePath::LEN);
        let p = &SPLINE_PATH as *const SplinePath as *const u8;
        let ours: Vec<u8> = (0..size_of::<SplinePath>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(SplinePath::START, SplinePath::LEN),
            "SPLINE_PATH starts differently from the image"
        );
        assert_eq!(offset_of!(SplinePath, path), 0x788EFA - SplinePath::START);
    }
}

/// The splines' constants (`marks::run_splines`, `marks::place_spline`, `marks::splines`) (IDA 0x788F0D .. 0x788F69, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct SplineConstants {
    /// 0x788F0D: `flt_788F0D`, `flt_788F11`, `flt_788F15`: the wave's constants.
    pub wave_k: [Float; 3],
    /// 0x788F19: `flt_788F19` .. `flt_788F35`: `place_spline`'s factors and limits.
    pub place_k: [Float; 8],
    /// 0x788F39: `g_marksSplineAngles`: each table's +0x24, +0x2C, +0x28 (four times three).
    pub angles: [Float; 12],
}

pub static SPLINE_CONSTANTS: SplineConstants = SplineConstants {
    wave_k: [
        Float::from_bits(0x43000000),
        Float::from_bits(0x3E4CCCCD),
        Float::from_bits(0x3F800000),
    ],
    place_k: [
        Float::from_bits(0x3F4CCCCD),
        Float::from_bits(0x3ECCCCCD),
        Float::from_bits(0x0),
        Float::from_bits(0x3F19999A),
        Float::from_bits(0x3DCCCCCD),
        Float::from_bits(0x0),
        Float::from_bits(0x3F19999A),
        Float::from_bits(0x3DCCCCCD),
    ],
    angles: [const { Float::from_bits(0x0) }; 12],
};

impl SplineConstants {
    pub const START: usize = 0x788F0D;
    pub const LEN: usize = 0x5C;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&SPLINE_CONSTANTS as *const SplineConstants) as usize
    }
}

#[cfg(test)]
mod spline_constants_tests {
    use super::*;

    #[test]
    fn spline_constants_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplineConstants>(), SplineConstants::LEN);
        let p = &SPLINE_CONSTANTS as *const SplineConstants as *const u8;
        let ours: Vec<u8> = (0..size_of::<SplineConstants>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(SplineConstants::START, SplineConstants::LEN),
            "SPLINE_CONSTANTS starts differently from the image"
        );
        assert_eq!(
            offset_of!(SplineConstants, wave_k),
            0x788F0D - SplineConstants::START
        );
        assert_eq!(
            offset_of!(SplineConstants, place_k),
            0x788F19 - SplineConstants::START
        );
        assert_eq!(
            offset_of!(SplineConstants, angles),
            0x788F39 - SplineConstants::START
        );
    }
}

impl SplineTables {
    /// The address of table `t` (A .. D) of eight records.
    pub fn table(&self, t: usize) -> usize {
        ([self.a.addr(), self.b.addr(), self.c.addr(), self.d.addr()][t]) as usize
    }
}

impl SplineHead {
    /// Dword `i` of the looks table, read as the original reads it: model 32 and 33 of the
    /// cars (34 models, pairs 8 bytes apart) run two and three dwords past the table's end
    /// into `found`, `angle` and `wave` (`notes/bugs.md`).
    pub fn look(&self, i: usize) -> u32 {
        assert!(i < 69, "the looks table is read at most up to `wave`");
        unsafe {
            let first = (self as *const Self as *const u8)
                .add(core::mem::offset_of!(Self, looks))
                .cast::<Dword>();
            (*first.add(i)).get()
        }
    }
}
