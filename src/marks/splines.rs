//! `init_marks`' step 1 (IDA, 2026-10-16): the splines. `sub_4B96CD` reads
//! `files\m\spline.dat` once and, for each of the settings' objects, `sub_4B9A32` fills the
//! next of 8 places in four tables of `MarkSpline` records (0xE90 bytes): the points of the
//! object's faces of one kind, centred, turned, graded by height and ordered. These are the
//! leaves; every function takes the record in ebp.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
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
/// Where `reset_pong` puts the eight floats: four (x, y) pairs 0x18 apart.
const PONG_TO: [usize; 8] = [
    0x768688, 0x76868C, 0x7686A0, 0x7686A4, 0x7686B8, 0x7686BC, 0x7686D0, 0x7686D4,
];
pub const OUTLINE_COUNT_RVA: usize = rva(0x768700); // dword_768700
pub const OUTLINE_MARK_RVA: usize = rva(0x7687D0); // dword_7687D0
pub const FOUND_RVA: usize = rva(0x7695B2); // g_marksSplineFaces: faces of the kind found
/// The four tables of 8 records (`g_marksSplinesA` .. `D`).
pub const TABLES: [usize; 4] = [0x76B9B2, 0x772E32, 0x77A2B2, 0x781732];
pub const TABLE_LEN: u32 = 8 * SPLINE_LEN;
pub const OBJECT_RVA: usize = rva(0x76A202); // g_marksSplineObject
pub const PLACE_RVA: usize = rva(0x76A206); // g_marksSplinePlace: the next of 8
pub const OFFSET_RVA: usize = rva(0x76A20A); // g_marksSplineOffset: the place times 0xE90
/// Each table's face kind (`sub_4B96CD` sets them by `g_marksKind`).
pub const KINDS_RVA: usize = rva(0x788BB6); // g_marksSplineKinds
/// Each table's +0x24, +0x2C, +0x28 (in that order).
pub const ANGLES_RVA: usize = rva(0x788F39); // g_marksSplineAngles
/// Added to the third table's highest point for its +0x44.
pub const SHIFT_RVA: usize = rva(0x7695BE); // g_marksSplineShift
/// `bound_spline_points`' two vectors: the one the angle is measured from, and the size.
pub const UP_RVA: usize = rva(0x76A216); // g_marksSplineUp
pub const SIZE_RVA: usize = rva(0x76A222); // g_marksSplineSize
/// `turn_spline_points`' matrices: the three turns, then X x Y, then all three.
pub const TURN_X_RVA: usize = rva(0x76A28E); // g_marksTurnX
pub const TURN_Y_RVA: usize = rva(0x76A2BE); // g_marksTurnY
pub const TURN_Z_RVA: usize = rva(0x76A2EE); // g_marksTurnZ
pub const TURN_XY_RVA: usize = rva(0x76A22E); // g_marksTurnXY
pub const TURN_RVA: usize = rva(0x76A25E); // g_marksTurn
/// `order_spline_points`' work: the ids, then the points.
pub const ORDERED_IDS_RVA: usize = rva(0x76A31E); // g_marksOrderedIds
pub const ORDERED_COORDS_RVA: usize = rva(0x76A51E); // g_marksOrderedCoords

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

/// `marks::centre_spline_points` (0x4B9F95): each point (+0x654, 12 bytes, the last first)
/// less the centre (+0xC54). Leaves eax, ebx, ecx the first point, edi 0 (as they were with
/// none). Refuses more than 0x80 points. Converted:
/// `rc2k_core::marks::splines::centre`.
pub unsafe fn centre_spline_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        let n = count(s)?;
        let mut points = triples(s + COORDS, n);
        rc2k_core::marks::splines::centre(&mut points, triples(s + CENTRE, 1)[0]);
        put_triples(s + COORDS, &points);
        if let Some(&first) = points.first() {
            [g.eax, g.ebx, g.ecx] = first;
            g.edi = 0;
        }
    }
    Ok(g)
}

/// `marks::grade_spline_points` (0x4BA316): each point's grade (+0xC90, the last first) by
/// its height (+0x658): 0 below 0x3830, 1 below 0x6720, else 2. Leaves eax the first
/// point's height, esi and edi 0 (eax 0x6720 with none). Refuses more than 0x80 points.
/// Converted: `rc2k_core::marks::splines::grade`.
pub unsafe fn grade_spline_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        let n = count(s)?;
        g.eax = rc2k_core::marks::splines::HIGH as u32;
        let heights: Vec<u32> = triples(s + COORDS, n).iter().map(|p| p[1]).collect();
        let grades: Vec<u32> = heights
            .iter()
            .map(|&h| rc2k_core::marks::splines::grade(h as i32))
            .collect();
        put_words(s + GRADES, &grades);
        if let Some(&first) = heights.first() {
            g.eax = first;
            g.esi = 0;
            g.edi = 0;
        }
    }
    Ok(g)
}

/// `marks::find_spline_ends` (0x4BA4F1): of the +0xC points from +0 the last with the
/// highest third value, into +0x30; of the +0x10 from +4 the last with the lowest, into
/// +0x34. The value is read at +0x65C + 0x30 a point, not 12 (section 16): read as the
/// original does. Leaves eax the second count, ecx the second found, edx its value, edi
/// the last point read times 12. Refuses a negative start or count, or points past 0x80.
/// Converted: `rc2k_core::marks::splines::find_end`.
pub unsafe fn find_spline_ends(entry: &Regs) -> Result<Regs, Refused> {
    use rc2k_core::marks::splines::{find_end, run_fits};
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        for (start, n) in [(0u32, 0xCu32), (4, 0x10)] {
            run_fits(get(s + start), get(s + n))?;
        }
        for (start, n, to, highest) in [(0u32, 0xCu32, 0x30u32, true), (4, 0x10, 0x34, false)] {
            let (from, c) = (get(s + start), get(s + n));
            let end = find_end(from, c, highest, |p| get(s + 0x30 * p + 0x65C));
            if let Some(last) = end.last {
                g.edi = last * 12;
            }
            g.eax = c;
            g.ecx = end.at;
            g.edx = end.value;
            put(s + to, g.ecx);
        }
    }
    Ok(g)
}

/// `marks::add_face_points` (0x4B9E29): face edi's points (+0 the count, from +0x10, 16
/// bytes each: the point and two values) added to the record's (+0x50 the count, ids
/// +0x454, values +0x54 and +0x254) unless there already, up to 0x80. Leaves eax the
/// count, ecx where the last search stopped, edx the last point read. Refuses a record of
/// more than 0x80 points. Converted: `rc2k_core::marks::splines::FacePoints::add_face`.
pub unsafe fn add_face_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
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
        let face_points = get(g.edi);
        let at = |k: u32| g.edi + 0x10 + 0x10 * k;
        let c = points.add_face(
            face_points,
            |k| get(at(k)),
            |k| [get(at(k) + 4), get(at(k) + 8)],
        )?;
        let after = points.count;
        let added = before as usize..after as usize;
        for (table, values) in [(IDS, &ids), (FIRSTS, &firsts), (SECONDS, &seconds)] {
            put_words(s + table + 4 * before, &values[added.clone()]);
        }
        g.eax = after;
        g.ecx = c.stopped.unwrap_or(face_points);
        g.edx = c.last_point.unwrap_or(g.edx);
        put(s + COUNT, g.eax);
    }
    Ok(g)
}

/// `marks::find_farthest_points` (0x4BA578): looks for the two farthest apart of eax points
/// at esi, all inside a `pusha` .. `popa`, so nothing is kept (section 16): it does
/// nothing seen.
pub fn find_farthest_points(entry: &Regs) -> Regs {
    *entry
}

/// `marks::reset_pong` (0x4B7E64): the eight floats from `flt_769182` to four (x, y)
/// pairs from `flt_768688`, `dword_768700` 5, `dword_7687D0` 0. Every register kept.
/// Converted: `rc2k_core::marks::pong::Court::reset` (exact at every precision: `fld;
/// fstp` of a float only makes a signalling NaN quiet).
pub unsafe fn reset_pong() {
    let rules = crate::marks::pong::rules();
    crate::marks::pong::with_court(|c| c.reset(&rules));
}

/// What `reset_pong` writes.
pub fn outline_regions() -> Vec<crate::service::harness::Region> {
    use crate::service::harness::at;
    let mut r: Vec<_> = PONG_TO.into_iter().map(|a| at(rva(a))).collect();
    r.extend([at(OUTLINE_COUNT_RVA), at(OUTLINE_MARK_RVA)]);
    r
}

/// `marks::gather_spline_points` (0x4B9D5F): record ebp emptied, then the points of object
/// eax's faces of kind ebx (+7, low five bits) added (`add_face_points`; the faces counted
/// in `g_marksSplineFaces`); the ids made pointers into `g_objectPoints` and the points
/// copied to +0x654 (both the last first); then `find_farthest_points`. Leaves eax the
/// count, esi +0x654. Refuses an object past 0xFF, a negative face count or faces past
/// `g_faces`.
pub unsafe fn gather_spline_points(entry: &Regs) -> Result<Regs, Refused> {
    use crate::scene::objects::{
        FACE_COUNTS_RVA, FACES, FACES_RVA, FIRST_FACE_RVA, FIRST_POINT_RVA, POINTS_RVA,
    };
    let mut g = *entry;
    unsafe {
        let object = g.eax as usize;
        if object >= 0x100 {
            return Err(BadInput("an object past 0xFF").into());
        }
        let first: u32 = game_read(FIRST_FACE_RVA + 4 * object);
        let faces: u32 = game_read(FACE_COUNTS_RVA + 4 * object);
        if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
            return Err(BadInput("an object's faces past g_faces").into());
        }
        let s = g.ebp;
        put(s + COUNT, 0);
        g.esi = first;
        g.edx = game_read(FIRST_POINT_RVA + 4 * object);
        game_write(FOUND_RVA, 0u32);
        for _ in 0..faces {
            g.edi = game_read(FACES_RVA + 4 * g.esi as usize);
            g.eax = (*((g.edi + 7) as *const u8) & 0x1F) as u32;
            if g.eax == g.ebx {
                game_write(FOUND_RVA, game_read::<u32>(FOUND_RVA).wrapping_add(1));
                g = add_face_points(&g)?;
            }
            g.esi += 1;
        }
        g.eax = faces;
        let n = get(s + COUNT);
        let points: u32 = game_read(POINTS_RVA);
        for k in (0..n).rev() {
            g.ebx = k;
            g.ecx = get(s + IDS + 4 * k).wrapping_mul(12).wrapping_add(points);
            put(s + IDS + 4 * k, g.ecx);
        }
        for k in (0..n).rev() {
            g.esi = get(s + IDS + 4 * k);
            g.edi = k * 12;
            for c in 0..3 {
                put(s + COORDS + g.edi + 4 * c, get(g.esi + 4 * c));
            }
        }
        g.esi = s + COORDS;
        g.eax = n;
        g = find_farthest_points(&g);
    }
    Ok(g)
}

/// An angle as `fld dword [x]; fchs` leaves it.
fn negated(f: u32) -> [u8; 10] {
    let mut a = crate::scene::matrix3::extended(f);
    a[9] ^= 0x80;
    a
}

/// `marks::turn_spline_points` (0x4B9FE6): the turns about X, Y, Z by minus record ebp's
/// +0x20, +0x24, +0x28 into `g_marksTurnX` .. `Z`, X x Y into `g_marksTurnXY`, that x Z
/// into `g_marksTurn`, and the record's points turned by it in place. Leaves ecx -1, edx
/// 0, esi and edi past the points, ebp `g_marksTurn`. Refuses more than 0x80 points or an
/// angle `fsincos` cannot take, before writing anything.
pub unsafe fn turn_spline_points(entry: &Regs) -> Result<Regs, Refused> {
    use crate::scene::matrix3::{
        Axis, multiply_matrices3, transform_int_points, turn_matrix3, turnable,
    };
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        count(s)?;
        let angles = [0x20u32, 0x24, 0x28].map(|o| negated(get(s + o)));
        for a in &angles {
            turnable(a)?;
        }
        let at = |r: usize| game_addr(r) as u32;
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
        g.esi = at(TURN_Z_RVA);
        for (a, b, out) in [
            (TURN_X_RVA, TURN_Y_RVA, TURN_XY_RVA),
            (TURN_XY_RVA, TURN_Z_RVA, TURN_RVA),
        ] {
            g = multiply_matrices3(&Regs {
                esi: at(a),
                edi: at(b),
                ebp: at(out),
                ..g
            });
        }
        g.esi = s + COORDS;
        g.edi = s + COORDS;
        g.ecx = get(s + COUNT);
        g.ebp = at(TURN_RVA);
        g = transform_int_points(&g)?;
    }
    Ok(g)
}

/// `marks::place_shape_points` (0x4BA0AE): the turns about X, Y, Z by record ebp's +0x20,
/// +0x24, +0x28 (not negated, as `turn_spline_points` does) into `g_marksTurnX` .. `Z`, X x
/// Y into `g_marksTurnXY`, that x Z into `g_marksTurn`; the record's points turned by it
/// into `g_marksOrderedCoords`; then, from the last, each written plus the record's +0xC54
/// .. through the record's pointer at +0x454. Leaves ebp the record; with points, esi the
/// first point's pointer, edi 0, eax .. ecx what was written there; without, as
/// `transform_int_points` leaves them. Refuses more than 0x80 points, an angle `fsincos`
/// cannot take, and a null pointer among the points', before writing anything.
pub unsafe fn place_shape_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        let n = count(s)?;
        turnable_record(s, [0x20, 0x24, 0x28])?;
        if (0..n).any(|k| get(s + IDS + 4 * k) == 0) {
            return Err(BadInput("a null point pointer").into());
        }
        g = turn_record(&g, s, [0x20, 0x24, 0x28])?;
        g = transform_int_points_from(&g, s + COORDS, get(s + COUNT))?;
        g.ebp = s;
        for k in (0..n).rev() {
            g = write_point(&g, s, k, k);
        }
    }
    Ok(g)
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
/// `sub_4BA0AE` and `sub_4BA16C` do (ebp pushed round the products: left as it was).
pub unsafe fn turn_record(entry: &Regs, s: u32, offsets: [u32; 3]) -> Result<Regs, Refused> {
    use crate::scene::matrix3::{Axis, extended, multiply_matrices3, turn_matrix3};
    let mut g = *entry;
    unsafe {
        let at = |r: usize| game_addr(r) as u32;
        for ((axis, to), o) in [
            (Axis::X, TURN_X_RVA),
            (Axis::Y, TURN_Y_RVA),
            (Axis::Z, TURN_Z_RVA),
        ]
        .into_iter()
        .zip(offsets)
        {
            g.esi = at(to);
            turn_matrix3(axis, at(to), extended(get(s + o)))?;
        }
        let ebp = g.ebp;
        for (a, b, out) in [
            (TURN_X_RVA, TURN_Y_RVA, TURN_XY_RVA),
            (TURN_XY_RVA, TURN_Z_RVA, TURN_RVA),
        ] {
            g = multiply_matrices3(&Regs {
                esi: at(a),
                edi: at(b),
                ebp: at(out),
                ..g
            });
        }
        g.ebp = ebp;
    }
    Ok(g)
}

/// `count` points from `from` turned by `g_marksTurn` into `g_marksOrderedCoords`
/// (`scene::transform_int_points`; ebp left as it was).
pub unsafe fn transform_int_points_from(
    entry: &Regs,
    from: u32,
    count: u32,
) -> Result<Regs, Refused> {
    use crate::scene::matrix3::transform_int_points;
    let at = |r: usize| game_addr(r) as u32;
    let mut g = unsafe {
        transform_int_points(&Regs {
            esi: from,
            edi: at(ORDERED_COORDS_RVA),
            ecx: count,
            ebp: at(TURN_RVA),
            ..*entry
        })?
    };
    g.ebp = entry.ebp;
    Ok(g)
}

/// Ordered point `k` plus the record's +0xC54 .. written through the record's pointer
/// `pointer` (at +0x454); esi that pointer, edi `k` times 12, eax .. ecx the values.
pub unsafe fn write_point(entry: &Regs, s: u32, k: u32, pointer: u32) -> Regs {
    let mut g = *entry;
    unsafe {
        g.esi = get(s + IDS + 4 * pointer);
        g.edi = 12 * k;
        let from = game_addr(ORDERED_COORDS_RVA) as u32 + 12 * k;
        let v: [u32; 3] = std::array::from_fn(|c| {
            get(from + 4 * c as u32).wrapping_add(get(s + CENTRE + 4 * c as u32))
        });
        for (c, x) in v.iter().enumerate() {
            put(g.esi + 4 * c as u32, *x);
        }
        [g.eax, g.ebx, g.ecx] = v;
    }
    g
}

/// `marks::order_spline_points` (0x4BA388): record ebp's ids and points put in grade order
/// (each grade's the last first) through `g_marksOrderedIds` and `g_marksOrderedCoords`;
/// +0 .. +8 each grade's start, +0xC .. +0x14 its count; then the count's ids and points
/// moved back (as `memory::move_bytes` would, as many as the count, whatever their
/// grade). Leaves eax +0x654, ebx `g_marksOrderedCoords`, ecx the count times 12, esi
/// the last taken times 12, edi 0. Refuses more than 0x80 points or a negative count.
/// Converted: `rc2k_core::marks::splines::order`.
pub unsafe fn order_spline_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
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
        if n > 0 {
            g.edi = 0;
        }
        if let Some(k) = o.last_taken {
            g.esi = k * 12;
        }
        g.eax = s + COORDS;
        g.ebx = coords_at;
        g.ecx = n * 12;
    }
    Ok(g)
}

/// `marks::bound_spline_points` (0x4B9E7D): record ebp's points' smallest and largest in
/// each part; the y and z sizes as floats into `g_marksSplineSize` + 4, + 8 (its x is
/// left as it was: section 16); the box's middle (half the size, `sar`, plus the smallest)
/// into the centre (+0xC54); the angle between `g_marksSplineUp` and the size
/// (`scene::angle_between_vectors`, both normalised) into +0x20. Leaves eax, ebx, ecx the
/// middle, esi `g_marksSplineUp`, edi `g_marksSplineSize`. Refuses more than 0x80 points.
/// Converted: `rc2k_core::marks::splines::bounds` (the floats, the sizes made floats and the
/// angle, stay the x87's: the first round as its control word says).
pub unsafe fn bound_spline_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        let n = count(s)?;
        let b = rc2k_core::marks::splines::bounds(&triples(s + COORDS, n));
        let size = b.size;
        g.esi = game_addr(UP_RVA) as u32;
        g.edi = game_addr(SIZE_RVA) as u32;
        put(g.edi + 4, x87::from_int(size[1]));
        put(g.edi + 8, x87::from_int(size[2]));
        let middle = b.middle.map(|m| m as u32);
        for (c, &m) in middle.iter().enumerate() {
            put(s + CENTRE + 4 * c as u32, m);
        }
        [g.eax, g.ebx, g.ecx] = middle;
        let angle = crate::scene::vector3::angle_between_vectors(g.esi, g.edi);
        put(s + 0x20, crate::scene::matrix3::rounded(angle));
    }
    Ok(g)
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

/// `marks::make_spline` (0x4B9A32): object eax into `g_marksSplineObject`; unless 8 are
/// made (`g_marksSplinePlace`), the next record of each table for it: of the faces of that
/// table's kind gathered, bounded, centred, given the table's angles and turned; the second
/// and third also given the first table's record 0's centre and +0x20 first (record 0's
/// whatever the place: section 16) and then graded (+0x18, +0x1C 0x58F0, 0x6F78), ordered
/// and their ends found; the third's +0x44 .. the highest point plus `g_marksSplineShift`.
/// Then the place counted. Leaves the registers as the last steps do (ebp the first
/// table's record). Refuses a negative place, an object past 0xFF or faces past `g_faces`,
/// an angle `fsincos` cannot take, before writing anything else.
pub unsafe fn make_spline(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        game_write(OBJECT_RVA, g.eax);
        let place = game_read::<u32>(PLACE_RVA) as i32;
        if place >= 8 {
            return Ok(g);
        }
        if place < 0 {
            return Err(BadInput("a negative spline place").into());
        }
        let object = g.eax;
        spline_checks(object)?;
        let offset = place as u32 * SPLINE_LEN;
        game_write(OFFSET_RVA, offset);
        let table = |t: usize| game_addr(rva(TABLES[t])) as u32;
        for t in 0..4 {
            let record = table(t) + offset;
            g.eax = object;
            g.ebx = game_read(KINDS_RVA + 4 * t);
            g.ebp = record;
            g = gather_spline_points(&g)?;
            g.ebp = record;
            g = bound_spline_points(&g)?;
            if t == 1 || t == 2 {
                g.esi = table(0);
                g.edi = table(t);
                for o in [CENTRE, CENTRE + 4, CENTRE + 8, 0x20] {
                    put(g.edi + o, get(g.esi + o));
                }
            }
            g.ebp = record;
            g = centre_spline_points(&g)?;
            g.ebp = record;
            for (k, o) in [0x24u32, 0x2C, 0x28].into_iter().enumerate() {
                put(
                    record + o,
                    x87::loaded(game_read(ANGLES_RVA + 4 * (3 * t + k))),
                );
            }
            g = turn_spline_points(&g)?;
            if t == 1 || t == 2 {
                g.ebp = record;
                put(record + 0x18, 0x58F0);
                put(record + 0x1C, 0x6F78);
                g = grade_spline_points(&g)?;
                g.ebp = record;
                g = order_spline_points(&g)?;
                g.ebp = record;
                g = find_spline_ends(&g)?;
            }
            if t == 2 {
                g.ebp = record;
                g.esi = get(record + 0x30).wrapping_mul(12);
                let shift: [u32; 3] = game_read(SHIFT_RVA);
                let p = record.wrapping_add(COORDS).wrapping_add(g.esi);
                g.eax = shift[0].wrapping_add(get(p));
                g.ebx = shift[1].wrapping_add(get(p + 4));
                g.ecx = shift[2].wrapping_add(get(p + 8));
                put(record + 0x44, g.eax);
                put(record + 0x48, g.ebx);
                put(record + 0x4C, g.ecx);
            }
        }
        g.ebp = table(0) + offset;
        for k in (0..(get(g.ebp + COUNT) as i32).max(0) as u32).rev() {
            g.ebx = k;
            g.ecx = get(g.ebp + IDS + 4 * k);
        }
        game_write(PLACE_RVA, place as u32 + 1);
    }
    Ok(g)
}

pub const LOADED_RVA: usize = rva(0x788EF6); // g_marksSplinesLoaded: 0, or where to go on
pub const RESUME_RVA: usize = rva(0x4B9706); // loc_4B9706
const PATH_RVA: usize = rva(0x788EFA); // "files\m\spline.dat"
pub const FILE_RVA: usize = rva(0x7695D2); // g_marksSplineFile
pub const FILE_LEN: u32 = 0xC00;
/// The five values by kind (`dword_788BC6` .. `dword_788DE6`, 0x22 each) and where they go.
const BY_KIND_RVA: usize = rva(0x788BC6);
pub const CHOSEN_RVA: usize = rva(0x788BB2); // dword_788BB2, then `g_marksSplineKinds`
const KINDS: u32 = 0x22;
pub const MADE_RVA: usize = rva(0x7695CE); // dword_7695CE
/// The settings' objects: a count at +0x26B8, then the objects.
const OBJECTS_AT: u32 = 0x26B8;
const MOST_OBJECTS: u32 = (0x2708 - 0x26BC) / 4;

/// `marks::load_splines` (0x4B96CD), in step 1: once (`g_marksSplinesLoaded`, then
/// `resume`) `files\m\spline.dat` (0xC00 bytes at most, place 1 of the part, inside
/// `pusha`) into `g_marksSplineFile`; the five values of `g_marksKind` into 0x788BB2 ..;
/// `g_marksSplinePlace` 0; `make_spline` for each of the settings' objects (+0x26BC, the last
/// first); `dword_7695CE` 0. Leaves esi the settings, the rest as the last `make_spline`
/// (eax the kind with none). Refuses a mark neither 0 nor `resume`, a kind of 0x22 or
/// more, more than 19 objects, and what `make_spline` would refuse for any of them, before
/// writing anything.
pub unsafe fn load_splines(
    entry: &Regs,
    resume: u32,
    part: crate::race::level_init::Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
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
            let mut r = Regs {
                eax: game_addr(PATH_RVA) as u32,
                ebx: game_addr(FILE_RVA) as u32,
                ecx: FILE_LEN,
                ..g
            };
            part(1, &mut r)?;
            g.eax = game_addr(LOADED_RVA) as u32;
            game_write(LOADED_RVA, resume);
        }
        g.eax = kind;
        for n in 0..5 {
            let v: u32 = game_read(BY_KIND_RVA + 4 * (n * KINDS + kind) as usize);
            game_write(CHOSEN_RVA + 4 * n as usize, v);
        }
        game_write(PLACE_RVA, 0u32);
        g.esi = settings;
        for o in objects {
            g.eax = o;
            g = make_spline(&g)?;
            g.esi = settings;
        }
        game_write(MADE_RVA, 0u32);
    }
    Ok(g)
}

/// The four tables, whole.
pub fn table_regions() -> Vec<crate::service::harness::Region> {
    TABLES
        .iter()
        .map(|&t| (game_addr(rva(t)) as u32, TABLE_LEN as usize))
        .collect()
}

/// What the record's functions write besides the record.
pub fn work_regions() -> Vec<crate::service::harness::Region> {
    use crate::service::harness::at;
    let a = |r: usize| game_addr(r) as u32;
    vec![
        (a(TURN_XY_RVA), 0xF0),
        (a(UP_RVA), 24),
        at(crate::scene::vector3::STATUS_RVA),
        (a(ORDERED_IDS_RVA), 4 * MOST_POINTS as usize),
        (a(ORDERED_COORDS_RVA), 12 * MOST_POINTS as usize),
        at(FOUND_RVA),
    ]
}

// ---- for the self-tests ----

/// A record of the test's (with room past it for `find_spline_ends`' reads), filled by lot.
pub struct TestSpline {
    pub bytes: Vec<u8>,
}

/// How far past a record `find_spline_ends` can read.
const READ_REACH: u32 = 0x65C + 0x30 * MOST_POINTS;

impl TestSpline {
    pub fn new() -> Self {
        Self {
            bytes: vec![0; READ_REACH as usize],
        }
    }

    pub fn addr(&self) -> u32 {
        self.bytes.as_ptr() as u32
    }

    pub fn region(&self) -> crate::service::harness::Region {
        (self.addr(), SPLINE_LEN as usize)
    }

    /// Everything by lot from `seed`, `points` points (heights about the grades' bounds),
    /// the grades' starts and counts a split of the points.
    pub fn fill(&self, seed: u32, points: u32) {
        let mut s = seed;
        let mut next = move || {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            s >> 4
        };
        let a = self.addr();
        unsafe {
            for n in 0..READ_REACH / 4 {
                put(a + 4 * n, next());
            }
            put(a + COUNT, points);
            for k in 0..MOST_POINTS {
                let p = a + COORDS + 12 * k;
                for c in 0..3 {
                    put(p + 4 * c, (next() % 0x2_0000).wrapping_sub(0x1_0000));
                }
                put(p + 4, next() % 0x9000);
                put(a + IDS + 4 * k, next() % 0x60);
            }
            let first = next() % (points + 1);
            let second = next() % (points - first + 1);
            for k in 0..MOST_POINTS {
                put(a + GRADES + 4 * k, next() % 4);
            }
            for o in [0x20u32, 0x24, 0x28] {
                put(
                    a + o,
                    (((next() % 0x8000) as f32 - 16384.0) / 2048.0).to_bits(),
                );
            }
            put(a, 0);
            put(a + 0xC, first);
            put(a + 4, first);
            put(a + 0x10, second);
        }
    }
}

impl Default for TestSpline {
    fn default() -> Self {
        Self::new()
    }
}
