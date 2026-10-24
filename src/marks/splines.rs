//! `init_marks`' step 1 (IDA, 2026-10-16): the splines. `sub_4B96CD` reads
//! `files\m\spline.dat` once and, for each of the settings' objects, `sub_4B9A32` fills the
//! next of 8 places in four tables of `MarkSpline` records (0xE90 bytes): the points of the
//! object's faces of one kind, centred, turned, graded by height and ordered. These are the
//! leaves; every function takes the record in ebp.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;

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
pub const FOUND_RVA: Slot = Slot::at(0x7695B2); // g_marksSplineFaces: faces of the kind found
/// The four tables of 8 records (`g_marksSplinesA` .. `D`).
pub const TABLES: [Slot; 4] = [
    Slot::at(0x76B9B2),
    Slot::at(0x772E32),
    Slot::at(0x77A2B2),
    Slot::at(0x781732),
];
pub const OBJECT_RVA: Slot = Slot::at(0x76A202); // g_marksSplineObject
pub const PLACE_RVA: Slot = Slot::at(0x76A206); // g_marksSplinePlace: the next of 8
pub const OFFSET_RVA: Slot = Slot::at(0x76A20A); // g_marksSplineOffset: the place times 0xE90
/// Each table's face kind (`sub_4B96CD` sets them by `g_marksKind`).
pub const KINDS_RVA: Slot = Slot::at(0x788BB6); // g_marksSplineKinds
/// Each table's +0x24, +0x2C, +0x28 (in that order).
pub const ANGLES_RVA: Slot = Slot::at(0x788F39); // g_marksSplineAngles
/// Added to the third table's highest point for its +0x44.
pub const SHIFT_RVA: Slot = Slot::at(0x7695BE); // g_marksSplineShift
/// `bound_spline_points`' two vectors: the one the angle is measured from, and the size.
pub const UP_RVA: Slot = Slot::at(0x76A216); // g_marksSplineUp
pub const SIZE_RVA: Slot = Slot::at(0x76A222); // g_marksSplineSize
/// `turn_spline_points`' matrices: the three turns, then X x Y, then all three.
pub const TURN_X_RVA: Slot = Slot::at(0x76A28E); // g_marksTurnX
pub const TURN_Y_RVA: Slot = Slot::at(0x76A2BE); // g_marksTurnY
pub const TURN_Z_RVA: Slot = Slot::at(0x76A2EE); // g_marksTurnZ
pub const TURN_XY_RVA: Slot = Slot::at(0x76A22E); // g_marksTurnXY
pub const TURN_RVA: Slot = Slot::at(0x76A25E); // g_marksTurn
/// `order_spline_points`' work: the ids, then the points.
pub const ORDERED_IDS_RVA: Slot = Slot::at(0x76A31E); // g_marksOrderedIds
pub const ORDERED_COORDS_RVA: Slot = Slot::at(0x76A51E); // g_marksOrderedCoords

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) }
}

/// The record's point count, refused past `MOST_POINTS` (the loops would run past it).
unsafe fn count(spline: u32) -> Result<u32, Refused> {
    Ok(rc2k_core::marks::splines::points(unsafe { get(spline + COUNT) })? as u32)
}

/// `n` dwords at `at` (unaligned: the records are).
unsafe fn words(at: u32, n: u32) -> Vec<u32> {
    (0..n).map(|k| unsafe { get(at + 4 * k) }).collect()
}

unsafe fn put_words(at: u32, words: &[u32]) {
    for (k, &w) in words.iter().enumerate() {
        unsafe { put(at + 4 * k as u32, w) };
    }
}

/// `n` points (three dwords each) at `at`.
unsafe fn triples(at: u32, n: u32) -> Vec<[u32; 3]> {
    (0..n)
        .map(|k| std::array::from_fn(|c| unsafe { get(at + 12 * k + 4 * c as u32) }))
        .collect()
}

unsafe fn put_triples(at: u32, points: &[[u32; 3]]) {
    unsafe { put_words(at, points.as_flattened()) };
}

/// `marks::centre_spline_points` (0x4B9F95): record `s`'s points (+0x654, 12 bytes, the
/// last first) less its centre (+0xC54). Refuses more than 0x80 points. Converted:
/// `rc2k_core::marks::splines::centre`.
pub unsafe fn centre_spline_points(s: u32) -> Result<(), Refused> {
    unsafe {
        let n = count(s)?;
        let mut points = triples(s + COORDS, n);
        rc2k_core::marks::splines::centre(&mut points, triples(s + CENTRE, 1)[0]);
        put_triples(s + COORDS, &points);
    }
    Ok(())
}

/// `marks::grade_spline_points` (0x4BA316): record `s`'s points' grades (+0xC90, the last
/// first) by their heights (+0x658): 0 below 0x3830, 1 below 0x6720, else 2. Refuses more
/// than 0x80 points. Converted: `rc2k_core::marks::splines::grade`.
pub unsafe fn grade_spline_points(s: u32) -> Result<(), Refused> {
    unsafe {
        let n = count(s)?;
        let grades: Vec<u32> = triples(s + COORDS, n)
            .iter()
            .map(|p| rc2k_core::marks::splines::grade(p[1] as i32))
            .collect();
        put_words(s + GRADES, &grades);
    }
    Ok(())
}

/// `marks::find_spline_ends` (0x4BA4F1): of record `s`'s +0xC points from +0 the last
/// with the highest third value, into +0x30; of the +0x10 from +4 the last with the
/// lowest, into +0x34. The value is read at +0x65C + 0x30 a point, not 12 (section 16):
/// read as the original does. Refuses a negative start or count, or points past 0x80.
/// Converted: `rc2k_core::marks::splines::find_end`.
pub unsafe fn find_spline_ends(s: u32) -> Result<(), Refused> {
    use rc2k_core::marks::splines::{find_end, run_fits};
    unsafe {
        for (start, n) in [(0u32, 0xCu32), (4, 0x10)] {
            run_fits(get(s + start), get(s + n))?;
        }
        for (start, n, to, highest) in [(0u32, 0xCu32, 0x30u32, true), (4, 0x10, 0x34, false)] {
            let (from, c) = (get(s + start), get(s + n));
            let end = find_end(from, c, highest, |p| get(s + 0x30 * p + 0x65C));
            put(s + to, end.at);
        }
    }
    Ok(())
}

/// `marks::add_face_points` (0x4B9E29): face `face`'s points (+0 the count, from +0x10, 16
/// bytes each: the point and two values) added to record `s`'s (+0x50 the count, ids
/// +0x454, values +0x54 and +0x254) unless there already, up to 0x80. Refuses a record of
/// more than 0x80 points. Converted: `rc2k_core::marks::splines::FacePoints::add_face`.
pub unsafe fn add_face_points(s: u32, face: u32) -> Result<(), Refused> {
    unsafe {
        let most = MOST_POINTS as usize;
        let before = get(s + COUNT);
        // The ids the searches read; the rest are only written.
        let mut ids = words(s + IDS, before.min(MOST_POINTS));
        ids.resize(most, 0);
        let (mut firsts, mut seconds) = (vec![0; most], vec![0; most]);
        let mut points = rc2k_core::marks::splines::FacePoints {
            count: before,
            ids: &mut ids,
            firsts: &mut firsts,
            seconds: &mut seconds,
        };
        let face_points = get(face);
        let at = |k: u32| face + 0x10 + 0x10 * k;
        points.add_face(
            face_points,
            |k| get(at(k)),
            |k| [get(at(k) + 4), get(at(k) + 8)],
        )?;
        let after = points.count;
        let added = before as usize..after as usize;
        for (table, values) in [(IDS, &ids), (FIRSTS, &firsts), (SECONDS, &seconds)] {
            put_words(s + table + 4 * before, &values[added.clone()]);
        }
        put(s + COUNT, after);
    }
    Ok(())
}

/// `marks::reset_pong` (0x4B7E64): the eight floats from `flt_769182` to four (x, y)
/// pairs from `flt_768688`, `dword_768700` 5, `dword_7687D0` 0. Every register kept.
/// Converted: `rc2k_core::marks::pong::Court::reset` (exact at every precision: `fld;
/// fstp` of a float only makes a signalling NaN quiet).
pub unsafe fn reset_pong() {
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
pub unsafe fn gather_spline_points(s: u32, object: u32, kind: u32) -> Result<(), Refused> {
    use crate::scene::objects::{FACE_COUNTS_RVA, FACES, FACES_RVA, FIRST_FACE_RVA, POINTS_RVA};
    unsafe {
        let object = object as usize;
        if object >= 0x100 {
            return Err(BadInput("an object past 0xFF").into());
        }
        let first: u32 = game_read(FIRST_FACE_RVA + 4 * object);
        let faces: u32 = game_read(FACE_COUNTS_RVA + 4 * object);
        if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
            return Err(BadInput("an object's faces past g_faces").into());
        }
        put(s + COUNT, 0);
        game_write(FOUND_RVA, 0u32);
        for f in first..first + faces {
            let face: u32 = game_read(FACES_RVA + 4 * f as usize);
            if (*((face + 7) as *const u8) & 0x1F) as u32 == kind {
                game_write(FOUND_RVA, game_read::<u32>(FOUND_RVA).wrapping_add(1));
                add_face_points(s, face)?;
            }
        }
        let n = get(s + COUNT);
        let points: u32 = game_read(POINTS_RVA);
        for k in (0..n).rev() {
            put(
                s + IDS + 4 * k,
                get(s + IDS + 4 * k).wrapping_mul(12).wrapping_add(points),
            );
        }
        for k in (0..n).rev() {
            let from = get(s + IDS + 4 * k);
            for c in 0..3 {
                put(s + COORDS + k * 12 + 4 * c, get(from + 4 * c));
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
pub unsafe fn multiply(a: u32, b: u32, out: u32) {
    unsafe { crate::scene::matrix3::multiply_matrices3(a, b, out) };
}

/// `scene::transform_int_points`: `count` points from `from` turned by the matrix at
/// `matrix` into `to`.
pub unsafe fn transform(from: u32, to: u32, count: u32, matrix: u32) -> Result<(), Refused> {
    unsafe { crate::scene::matrix3::transform_int_points(from, to, count, matrix) }
}

/// The X x Y x Z products of `turn_spline_points` and `turn_record`.
unsafe fn multiply_turns() {
    let at = |r: Slot| game_addr(r) as u32;
    unsafe {
        for (a, b, out) in [
            (TURN_X_RVA, TURN_Y_RVA, TURN_XY_RVA),
            (TURN_XY_RVA, TURN_Z_RVA, TURN_RVA),
        ] {
            multiply(at(a), at(b), at(out));
        }
    }
}

/// `marks::turn_spline_points` (0x4B9FE6): the turns about X, Y, Z by minus record `s`'s
/// +0x20, +0x24, +0x28 into `g_marksTurnX` .. `Z`, X x Y into `g_marksTurnXY`, that x Z
/// into `g_marksTurn`, and the record's points turned by it in place. Refuses more than
/// 0x80 points or an angle `fsincos` cannot take, before writing anything.
pub unsafe fn turn_spline_points(s: u32) -> Result<(), Refused> {
    use crate::scene::matrix3::{Axis, turn_matrix3, turnable};
    unsafe {
        count(s)?;
        let angles = [0x20u32, 0x24, 0x28].map(|o| negated(get(s + o)));
        for a in &angles {
            turnable(a)?;
        }
        let at = |r: Slot| game_addr(r) as u32;
        for ((axis, to), angle) in [
            (Axis::X, TURN_X_RVA),
            (Axis::Y, TURN_Y_RVA),
            (Axis::Z, TURN_Z_RVA),
        ]
        .into_iter()
        .zip(angles)
        {
            turn_matrix3(axis, at(to), angle)?;
        }
        multiply_turns();
        transform(s + COORDS, s + COORDS, get(s + COUNT), at(TURN_RVA))?;
    }
    Ok(())
}

/// `marks::place_shape_points` (0x4BA0AE): the turns about X, Y, Z by record `s`'s +0x20,
/// +0x24, +0x28 (not negated, as `turn_spline_points` does) into `g_marksTurnX` .. `Z`, X x
/// Y into `g_marksTurnXY`, that x Z into `g_marksTurn`; the record's points turned by it
/// into `g_marksOrderedCoords`; then, from the last, each written plus the record's +0xC54
/// .. through the record's pointer at +0x454. Refuses more than 0x80 points, an angle
/// `fsincos` cannot take, and a null pointer among the points', before writing anything.
pub unsafe fn place_shape_points(s: u32) -> Result<(), Refused> {
    unsafe {
        let n = count(s)?;
        turnable_record(s, [0x20, 0x24, 0x28])?;
        if (0..n).any(|k| get(s + IDS + 4 * k) == 0) {
            return Err(BadInput("a null point pointer").into());
        }
        turn_record(s, [0x20, 0x24, 0x28])?;
        transform_int_points_from(s + COORDS, get(s + COUNT))?;
        for k in (0..n).rev() {
            write_point(s, k, k);
        }
    }
    Ok(())
}

/// Whether the record's three angles (at `offsets`, not negated) can be turned by.
pub unsafe fn turnable_record(s: u32, offsets: [u32; 3]) -> Result<(), Refused> {
    use crate::scene::matrix3::{extended, turnable};
    for o in offsets {
        unsafe { turnable(&extended(get(s + o)))? };
    }
    Ok(())
}

/// The turns about X, Y, Z by the record's floats at `offsets` (not negated) into
/// `g_marksTurnX` .. `Z`, X x Y into `g_marksTurnXY`, that x Z into `g_marksTurn`, as
/// `sub_4BA0AE` and `sub_4BA16C` do.
pub unsafe fn turn_record(s: u32, offsets: [u32; 3]) -> Result<(), Refused> {
    use crate::scene::matrix3::{Axis, extended, turn_matrix3};
    unsafe {
        let at = |r: Slot| game_addr(r) as u32;
        for ((axis, to), o) in [
            (Axis::X, TURN_X_RVA),
            (Axis::Y, TURN_Y_RVA),
            (Axis::Z, TURN_Z_RVA),
        ]
        .into_iter()
        .zip(offsets)
        {
            turn_matrix3(axis, at(to), extended(get(s + o)))?;
        }
        multiply_turns();
    }
    Ok(())
}

/// `count` points from `from` turned by `g_marksTurn` into `g_marksOrderedCoords`
/// (`scene::transform_int_points`).
pub unsafe fn transform_int_points_from(from: u32, count: u32) -> Result<(), Refused> {
    let at = |r: Slot| game_addr(r) as u32;
    unsafe { transform(from, at(ORDERED_COORDS_RVA), count, at(TURN_RVA)) }
}

/// Ordered point `k` plus record `s`'s +0xC54 .. written through the record's pointer
/// `pointer` (at +0x454).
pub unsafe fn write_point(s: u32, k: u32, pointer: u32) {
    unsafe {
        let to = get(s + IDS + 4 * pointer);
        let from = game_addr(ORDERED_COORDS_RVA) as u32 + 12 * k;
        for c in 0..3 {
            put(
                to + 4 * c,
                get(from + 4 * c).wrapping_add(get(s + CENTRE + 4 * c)),
            );
        }
    }
}

/// `marks::order_spline_points` (0x4BA388): record `s`'s ids and points put in grade
/// order (each grade's the last first) through `g_marksOrderedIds` and
/// `g_marksOrderedCoords`; +0 .. +8 each grade's start, +0xC .. +0x14 its count; then the
/// count's ids and points moved back (as `memory::move_bytes` would, as many as the count,
/// whatever their grade). Refuses more than 0x80 points or a negative count. Converted:
/// `rc2k_core::marks::splines::order`.
pub unsafe fn order_spline_points(s: u32) -> Result<(), Refused> {
    unsafe {
        let n = count(s)?;
        if (get(s + COUNT) as i32) < 0 {
            return Err(BadInput("a spline of a negative count: the moves would run away").into());
        }
        let (ids_at, coords_at) = (
            game_addr(ORDERED_IDS_RVA) as u32,
            game_addr(ORDERED_COORDS_RVA) as u32,
        );
        // What the moves back read: the ordered tables' first `n`, those not placed as
        // they were.
        let mut ordered_ids = words(ids_at, n);
        let mut ordered_coords = triples(coords_at, n);
        let o = rc2k_core::marks::splines::order(
            &words(s + GRADES, n),
            &words(s + IDS, n),
            &triples(s + COORDS, n),
            &mut ordered_ids,
            &mut ordered_coords,
        );
        let placed = o.placed as usize;
        put_words(ids_at, &ordered_ids[..placed]);
        put_triples(coords_at, &ordered_coords[..placed]);
        put_words(s, &o.starts);
        put_words(s + 0xC, &o.counts);
        put_words(s + IDS, &ordered_ids);
        put_triples(s + COORDS, &ordered_coords);
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
pub unsafe fn bound_spline_points(s: u32) -> Result<(), Refused> {
    unsafe {
        let n = count(s)?;
        let b = rc2k_core::marks::splines::bounds(&triples(s + COORDS, n));
        let size = b.size;
        let up = game_addr(UP_RVA) as u32;
        let size_at = game_addr(SIZE_RVA) as u32;
        put(size_at + 4, x87::from_int(size[1]));
        put(size_at + 8, x87::from_int(size[2]));
        for (c, &m) in b.middle.iter().enumerate() {
            put(s + CENTRE + 4 * c as u32, m as u32);
        }
        let angle = crate::scene::vector3::angle_between_vectors(up, size_at);
        put(s + 0x20, crate::scene::matrix3::rounded(angle));
    }
    Ok(())
}

/// What `make_spline` refuses for an object: past 0xFF, a negative face count or faces past
/// `g_faces`, an angle of its tables `fsincos` cannot take.
unsafe fn spline_checks(object: u32) -> Result<(), Refused> {
    use crate::scene::objects::{FACE_COUNTS_RVA, FACES, FIRST_FACE_RVA};
    unsafe {
        if object >= 0x100 {
            return Err(BadInput("an object past 0xFF").into());
        }
        let first: u32 = game_read(FIRST_FACE_RVA + 4 * object as usize);
        let faces: u32 = game_read(FACE_COUNTS_RVA + 4 * object as usize);
        if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
            return Err(BadInput("an object's faces past g_faces").into());
        }
        for n in 0..12 {
            crate::scene::matrix3::turnable(&negated(game_read(ANGLES_RVA + 4 * n)))?;
        }
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
pub unsafe fn make_spline(object: u32) -> Result<(), Refused> {
    unsafe {
        game_write(OBJECT_RVA, object);
        let place = game_read::<u32>(PLACE_RVA) as i32;
        if place >= 8 {
            return Ok(());
        }
        if place < 0 {
            return Err(BadInput("a negative spline place").into());
        }
        spline_checks(object)?;
        let offset = place as u32 * SPLINE_LEN;
        game_write(OFFSET_RVA, offset);
        let table = |t: usize| game_addr(TABLES[t]) as u32;
        for t in 0..4 {
            let record = table(t) + offset;
            gather_spline_points(record, object, game_read(KINDS_RVA + 4 * t))?;
            bound_spline_points(record)?;
            if t == 1 || t == 2 {
                for o in [CENTRE, CENTRE + 4, CENTRE + 8, 0x20] {
                    put(table(t) + o, get(table(0) + o));
                }
            }
            centre_spline_points(record)?;
            for (k, o) in [0x24u32, 0x2C, 0x28].into_iter().enumerate() {
                put(
                    record + o,
                    x87::loaded(game_read(ANGLES_RVA + 4 * (3 * t + k))),
                );
            }
            turn_spline_points(record)?;
            if t == 1 || t == 2 {
                put(record + 0x18, 0x58F0);
                put(record + 0x1C, 0x6F78);
                grade_spline_points(record)?;
                order_spline_points(record)?;
                find_spline_ends(record)?;
            }
            if t == 2 {
                let shift: [u32; 3] = game_read(SHIFT_RVA);
                let p = record
                    .wrapping_add(COORDS)
                    .wrapping_add(get(record + 0x30).wrapping_mul(12));
                for c in 0..3 {
                    put(
                        record + 0x44 + 4 * c,
                        shift[c as usize].wrapping_add(get(p + 4 * c)),
                    );
                }
            }
        }
        game_write(PLACE_RVA, place as u32 + 1);
    }
    Ok(())
}

pub const LOADED_RVA: Slot = Slot::at(0x788EF6); // g_marksSplinesLoaded: 0, or where to go on
pub const RESUME_RVA: usize = rva(0x4B9706); // loc_4B9706
const PATH_RVA: Slot = Slot::at(0x788EFA); // "files\m\spline.dat"
pub const FILE_RVA: Slot = Slot::at(0x7695D2); // g_marksSplineFile
pub const FILE_LEN: u32 = 0xC00;
/// The five values by kind (`dword_788BC6` .. `dword_788DE6`, 0x22 each) and where they go.
const BY_KIND_RVA: Slot = Slot::at(0x788BC6);
pub const CHOSEN_RVA: Slot = Slot::at(0x788BB2); // dword_788BB2, then `g_marksSplineKinds`
const KINDS: u32 = 0x22;
pub const MADE_RVA: Slot = Slot::at(0x7695CE); // dword_7695CE
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
pub unsafe fn load_splines(resume: u32) -> Result<(), Refused> {
    unsafe {
        let loaded: u32 = game_read(LOADED_RVA);
        if loaded != 0 && loaded != resume {
            return Err(BadInput("the splines' mark is neither 0 nor loc_4B9706").into());
        }
        let kind: u32 = game_read(crate::marks::kind_values::KIND_RVA);
        if kind >= KINDS {
            return Err(BadInput("a marks' kind of 0x22 or more").into());
        }
        let settings: u32 = game_read(crate::marks::lines::SETTINGS_RVA);
        let count = get(settings + OBJECTS_AT) as i32;
        if count > MOST_OBJECTS as i32 {
            return Err(BadInput("more than 19 spline objects").into());
        }
        let objects: Vec<u32> = (0..count.max(0) as u32)
            .rev()
            .map(|k| get(settings + OBJECTS_AT + 4 + 4 * k))
            .collect();
        // The place starts at 0: the first eight are made, the rest only noted.
        for &o in objects.iter().take(8) {
            spline_checks(o)?;
        }
        if loaded == 0 {
            let file = game_addr(FILE_RVA) as u32;
            crate::files::read_whole_file::read_whole_file(
                game_addr(PATH_RVA) as *const u8,
                file as *mut u8,
                FILE_LEN,
            )?;
            game_write(LOADED_RVA, resume);
        }
        for n in 0..5 {
            let v: u32 = game_read(BY_KIND_RVA + 4 * (n * KINDS + kind) as usize);
            game_write(CHOSEN_RVA + 4 * n as usize, v);
        }
        game_write(PLACE_RVA, 0u32);
        for o in objects {
            make_spline(o)?;
        }
        game_write(MADE_RVA, 0u32);
    }
    Ok(())
}
