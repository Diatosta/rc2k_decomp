//! `init_marks`' step 1 (IDA, 2026-10-16): the splines. `sub_4B96CD` reads
//! `files\m\spline.dat` once and, for each of the settings' objects, `sub_4B9A32` fills the
//! next of 8 places in four tables of `MarkSpline` records (0xE90 bytes): the points of the
//! object's faces of one kind, centred, turned, graded by height and ordered. These are the
//! leaves; every function takes the record in ebp.

#[cfg(test)]
use crate::effects::owner::effects;
use crate::effects::paths::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner_course::Course;
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

/// The bytes of one of the four tables (A .. D).
const TABLE_LEN: u32 = 29824;
/// The four tables' bytes, end to end as in the image.
const TABLES_LEN: u32 = 4 * TABLE_LEN;

/// A spline record: its first byte's place in the four tables taken end to end, as the
/// original's ebp. Fields are read by offset from it; a read past the record goes on into
/// the next ones, as the original's (`find_spline_ends`, section 16), and one past the
/// tables is refused.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Spline(u32);

impl Spline {
    /// Record `n` (0 .. 7) of table `t` (A .. D).
    pub fn record(t: usize, n: u32) -> Spline {
        Spline(t as u32 * TABLE_LEN + n * SPLINE_LEN)
    }

    /// The table byte `at` of the four tables is in.
    fn table_byte(paths: &Paths, at: u32) -> &Bytes<29824> {
        [
            &paths.spline_tables.a,
            &paths.spline_tables.b,
            &paths.spline_tables.c,
            &paths.spline_tables.d,
        ][(at / TABLE_LEN) as usize]
    }

    /// The four bytes at `o` from the record, refused past the tables.
    fn place(self, o: u32) -> Result<u32, BadInput> {
        match self.0.checked_add(o) {
            Some(at) if at.checked_add(4).is_some_and(|end| end <= TABLES_LEN) => Ok(at),
            _ => Err(BadInput("a spline field past the tables")),
        }
    }

    /// The dword at `o` (unaligned: the records are).
    pub fn get(self, paths: &Paths, o: u32) -> Result<u32, BadInput> {
        let at = self.place(o)?;
        let mut v = [0u8; 4];
        for (k, b) in v.iter_mut().enumerate() {
            let at = at + k as u32;
            *b = Self::table_byte(paths, at).get((at % TABLE_LEN) as usize);
        }
        Ok(u32::from_le_bytes(v))
    }

    /// The dword at `o` set to `v`.
    pub fn put(self, paths: &Paths, o: u32, v: u32) -> Result<(), BadInput> {
        let at = self.place(o)?;
        for (k, b) in v.to_le_bytes().into_iter().enumerate() {
            let at = at + k as u32;
            Self::table_byte(paths, at).set((at % TABLE_LEN) as usize, b);
        }
        Ok(())
    }

    /// `n` dwords from `o`.
    fn words(self, paths: &Paths, o: u32, n: u32) -> Result<Vec<u32>, BadInput> {
        (0..n).map(|k| self.get(paths, o + 4 * k)).collect()
    }

    fn put_words(self, paths: &Paths, o: u32, words: &[u32]) -> Result<(), Refused> {
        for (k, &w) in words.iter().enumerate() {
            self.put(paths, o + 4 * k as u32, w)?;
        }
        Ok(())
    }

    /// `n` points (three dwords each) from `o`.
    fn triples(self, paths: &Paths, o: u32, n: u32) -> Result<Vec<[u32; 3]>, BadInput> {
        (0..n)
            .map(|k| {
                Ok([
                    self.get(paths, o + 12 * k)?,
                    self.get(paths, o + 12 * k + 4)?,
                    self.get(paths, o + 12 * k + 8)?,
                ])
            })
            .collect()
    }

    fn put_triples(self, paths: &Paths, o: u32, points: &[[u32; 3]]) -> Result<(), Refused> {
        self.put_words(paths, o, points.as_flattened())
    }

    /// The record's point count, refused past `MOST_POINTS` (the loops would run past it).
    pub fn count(self, paths: &Paths) -> Result<u32, Refused> {
        Ok(rc2k_core::effects::splines::points(self.get(paths, COUNT)?)? as u32)
    }

    /// The `n` points from point `first` (+0x654), as cells for the matrix helpers.
    pub fn points(self, paths: &Paths, first: u32, n: u32) -> Result<Vec<Dword>, BadInput> {
        Ok(self
            .triples(paths, COORDS + 12 * first, n)?
            .as_flattened()
            .iter()
            .map(|&v| Dword::new(v))
            .collect())
    }
}

#[cfg(test)]
mod spline_handle_tests {
    use super::*;

    #[test]
    fn a_record_reads_on_into_the_next_table_and_not_past_the_last() {
        let paths = &effects().paths;
        let last_of_a = Spline::record(0, 7);
        assert_eq!(
            last_of_a.get(paths, SPLINE_LEN).unwrap(),
            paths.spline_tables.b.get_u32(0)
        );
        // A dword across the tables' seam, bytes from both.
        let across = last_of_a.get(paths, SPLINE_LEN - 2).unwrap().to_le_bytes();
        assert_eq!(across[2], paths.spline_tables.b.get(0));
        assert_eq!(across[0], paths.spline_tables.a.get(29822));
        let last = Spline::record(3, 7);
        assert!(last.get(paths, SPLINE_LEN - 4).is_ok());
        assert!(last.get(paths, SPLINE_LEN - 3).is_err());
        assert!(last.put(paths, u32::MAX, 0).is_err());
    }
}

/// `effects::centre_spline_points` (0x4B9F95): record `s`'s points (+0x654, 12 bytes, the
/// last first) less its centre (+0xC54). Refuses more than 0x80 points. Converted:
/// `rc2k_core::effects::splines::centre`.
pub fn centre_spline_points(paths: &Paths, s: Spline) -> Result<(), Refused> {
    {
        let n = s.count(paths)?;
        let mut points = s.triples(paths, COORDS, n)?;
        rc2k_core::effects::splines::centre(&mut points, s.triples(paths, CENTRE, 1)?[0]);
        s.put_triples(paths, COORDS, &points)?;
    }
    Ok(())
}

/// `effects::grade_spline_points` (0x4BA316): record `s`'s points' grades (+0xC90, the last
/// first) by their heights (+0x658): 0 below 0x3830, 1 below 0x6720, else 2. Refuses more
/// than 0x80 points. Converted: `rc2k_core::effects::splines::grade`.
pub fn grade_spline_points(paths: &Paths, s: Spline) -> Result<(), Refused> {
    {
        let n = s.count(paths)?;
        let grades: Vec<u32> = s
            .triples(paths, COORDS, n)?
            .iter()
            .map(|p| rc2k_core::effects::splines::grade(p[1] as i32))
            .collect();
        s.put_words(paths, GRADES, &grades)?;
    }
    Ok(())
}

/// `effects::find_spline_ends` (0x4BA4F1): of record `s`'s +0xC points from +0 the last
/// with the highest third value, into +0x30; of the +0x10 from +4 the last with the
/// lowest, into +0x34. The value is read at +0x65C + 0x30 a point, not 12 (section 16):
/// read as the original does. Refuses a negative start or count, or points past 0x80.
/// Converted: `rc2k_core::effects::splines::find_end`.
pub fn find_spline_ends(paths: &Paths, s: Spline) -> Result<(), Refused> {
    use rc2k_core::effects::splines::{find_end, run_fits};
    {
        for (start, n) in [(0u32, 0xCu32), (4, 0x10)] {
            run_fits(s.get(paths, start)?, s.get(paths, n)?)?;
        }
        for (start, n, to, highest) in [(0u32, 0xCu32, 0x30u32, true), (4, 0x10, 0x34, false)] {
            let (from, c) = (s.get(paths, start)?, s.get(paths, n)?);
            let bad = crate::effects::Sticky::default();
            let end = find_end(from, c, highest, |p| {
                bad.take(s.get(paths, 0x30 * p + 0x65C))
            });
            bad.done()?;
            s.put(paths, to, end.at)?;
        }
    }
    Ok(())
}

/// `effects::add_face_points` (0x4B9E29): face `face`'s points (+0 the count, from +0x10, 16
/// bytes each: the point and two values) added to record `s`'s (+0x50 the count, ids
/// +0x454, values +0x54 and +0x254) unless there already, up to 0x80. Refuses a record of
/// more than 0x80 points. Converted: `rc2k_core::effects::splines::FacePoints::add_face`.
pub fn add_face_points(paths: &Paths, s: Spline, face: u32) -> Result<(), Refused> {
    {
        let most = MOST_POINTS as usize;
        let before = s.get(paths, COUNT)?;
        // The ids the searches read; the rest are only written.
        let mut ids = s.words(paths, IDS, before.min(MOST_POINTS))?;
        ids.resize(most, 0);
        let (mut firsts, mut seconds) = (vec![0; most], vec![0; most]);
        let mut points = rc2k_core::effects::splines::FacePoints {
            count: before,
            ids: &mut ids,
            firsts: &mut firsts,
            seconds: &mut seconds,
        };
        let face_points = crate::scene::object_faces::word(face)?;
        let at = |k: u32| face + 0x10 + 0x10 * k;
        let bad = crate::effects::Sticky::default();
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
            s.put_words(paths, table + 4 * before, &values[added.clone()])?;
        }
        s.put(paths, COUNT, after)?;
    }
    Ok(())
}

/// `effects::reset_pong` (0x4B7E64): the eight floats from `flt_769182` to four (x, y)
/// pairs from `flt_768688`, `dword_768700` 5, `dword_7687D0` 0. Every register kept.
/// Converted: `rc2k_core::effects::pong::Court::reset` (exact at every precision: `fld;
/// fstp` of a float only makes a signalling NaN quiet).
pub fn reset_pong(pong: &crate::effects::pong::Pong, effects: &crate::effects::owner::Effects) {
    let rules = crate::effects::pong::rules(pong, effects);
    crate::effects::pong::with_court(pong, |c| c.reset(&rules));
}

/// `effects::gather_spline_points` (0x4B9D5F): record `s` emptied, then the points of object
/// `object`'s faces of kind `kind` (+7, low five bits) added (`add_face_points`; the faces
/// counted in `g_marksSplineFaces`); the ids made pointers into `g_objectPoints` and the
/// points copied to +0x654 (both the last first). Its last call,
/// `effects::find_farthest_points` (0x4BA578), works inside a `pusha` .. `popa` and keeps
/// nothing (section 16): not made. Refuses an object past 0xFF, a negative face count or
/// faces past `g_faces`.
pub fn gather_spline_points(
    course: &Course,
    paths: &Paths,
    s: Spline,
    object: u32,
    kind: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    use crate::scene::objects::FACES;
    {
        let object = object as usize;
        if object >= 0x100 {
            return Err(BadInput("an object past 0xFF").into());
        }
        let first: u32 = scene.objects.slots.first_face[object].get();
        let faces: u32 = scene.objects.slots.face_counts[object].get();
        if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
            return Err(BadInput("an object's faces past g_faces").into());
        }
        s.put(paths, COUNT, 0)?;
        paths.spline_head.found.set_bits(0u32);
        for f in first..first + faces {
            let face: u32 = crate::scene::objects::face_cell(f as usize).get();
            if (crate::scene::object_faces::byte(face + 7)? & 0x1F) as u32 == kind {
                paths
                    .spline_head
                    .found
                    .set(paths.spline_head.found.get().wrapping_add(1));
                add_face_points(paths, s, face)?;
            }
        }
        let n = s.get(paths, COUNT)?;
        let points: u32 = course.lists.object_points.bits();
        for k in (0..n).rev() {
            let id = s.get(paths, IDS + 4 * k)?;
            s.put(paths, IDS + 4 * k, id.wrapping_mul(12).wrapping_add(points))?;
        }
        for k in (0..n).rev() {
            let from = s.get(paths, IDS + 4 * k)?;
            for c in 0..3 {
                let v = crate::scene::object_points::word(from + 4 * c)?;
                s.put(paths, COORDS + k * 12 + 4 * c, v)?;
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

/// The X x Y x Z products of `turn_spline_points` and `turn_record`.
fn multiply_turns(paths: &Paths) {
    for (a, b, out) in [
        (
            &paths.spline_work.turn_x,
            &paths.spline_work.turn_y,
            &paths.spline_work.turn_xy,
        ),
        (
            &paths.spline_work.turn_xy,
            &paths.spline_work.turn_z,
            &paths.spline_work.turn,
        ),
    ] {
        crate::scene::matrix3::multiply_matrices3(a, b, out);
    }
}

/// `effects::turn_spline_points` (0x4B9FE6): the turns about X, Y, Z by minus record `s`'s
/// +0x20, +0x24, +0x28 into `g_marksTurnX` .. `Z`, X x Y into `g_marksTurnXY`, that x Z
/// into `g_marksTurn`, and the record's points turned by it in place. Refuses more than
/// 0x80 points or an angle `fsincos` cannot take, before writing anything.
pub fn turn_spline_points(paths: &Paths, s: Spline) -> Result<(), Refused> {
    use crate::scene::matrix3::{Axis, turn_matrix3, turnable};
    s.count(paths)?;
    let mut angles = [negated(0); 3];
    for (a, o) in angles.iter_mut().zip([0x20u32, 0x24, 0x28]) {
        *a = negated(s.get(paths, o)?);
    }
    for a in &angles {
        turnable(a)?;
    }
    for ((axis, to), angle) in [
        (Axis::X, &paths.spline_work.turn_x),
        (Axis::Y, &paths.spline_work.turn_y),
        (Axis::Z, &paths.spline_work.turn_z),
    ]
    .into_iter()
    .zip(angles)
    {
        turn_matrix3(axis, to, angle)?;
    }
    multiply_turns(paths);
    let n = s.count(paths)?;
    let points = s.points(paths, 0, n)?;
    crate::scene::matrix3::transform_int_points(
        &points[..],
        &points[..],
        n,
        &paths.spline_work.turn,
    )?;
    let turned: Vec<u32> = points
        .iter()
        .map(super::super::session::Dword::get)
        .collect();
    s.put_words(paths, COORDS, &turned)?;
    Ok(())
}

/// `effects::place_shape_points` (0x4BA0AE): the turns about X, Y, Z by record `s`'s +0x20,
/// +0x24, +0x28 (not negated, as `turn_spline_points` does) into `g_marksTurnX` .. `Z`, X x
/// Y into `g_marksTurnXY`, that x Z into `g_marksTurn`; the record's points turned by it
/// into `g_marksOrderedCoords`; then, from the last, each written plus the record's +0xC54
/// .. through the record's pointer at +0x454. Refuses more than 0x80 points, an angle
/// `fsincos` cannot take, and a null pointer among the points', before writing anything.
pub fn place_shape_points(paths: &Paths, s: Spline) -> Result<(), Refused> {
    {
        let n = s.count(paths)?;
        turnable_record(paths, s, [0x20, 0x24, 0x28])?;
        for k in 0..n {
            if s.get(paths, IDS + 4 * k)? == 0 {
                return Err(BadInput("a null point pointer").into());
            }
        }
        turn_record(paths, s, [0x20, 0x24, 0x28])?;
        transform_int_points_from(paths, s, 0, n)?;
        for k in (0..n).rev() {
            write_point(paths, s, k, k)?;
        }
    }
    Ok(())
}

/// Whether the record's three angles (at `offsets`, not negated) can be turned by.
pub fn turnable_record(paths: &Paths, s: Spline, offsets: [u32; 3]) -> Result<(), Refused> {
    use crate::scene::matrix3::{extended, turnable};
    for o in offsets {
        turnable(&extended(s.get(paths, o)?))?;
    }
    Ok(())
}

/// The turns about X, Y, Z by the record's floats at `offsets` (not negated) into
/// `g_marksTurnX` .. `Z`, X x Y into `g_marksTurnXY`, that x Z into `g_marksTurn`, as
/// `sub_4BA0AE` and `sub_4BA16C` do.
pub fn turn_record(paths: &Paths, s: Spline, offsets: [u32; 3]) -> Result<(), Refused> {
    use crate::scene::matrix3::{Axis, extended, turn_matrix3};
    for ((axis, to), o) in [
        (Axis::X, &paths.spline_work.turn_x),
        (Axis::Y, &paths.spline_work.turn_y),
        (Axis::Z, &paths.spline_work.turn_z),
    ]
    .into_iter()
    .zip(offsets)
    {
        turn_matrix3(axis, to, extended(s.get(paths, o)?))?;
    }
    multiply_turns(paths);
    Ok(())
}

/// Record `s`'s `count` points from point `first` turned by `g_marksTurn` into
/// `g_marksOrderedCoords` (`scene::transform_int_points`). Refuses points past the
/// record's 0x80.
pub fn transform_int_points_from(
    paths: &Paths,
    s: Spline,
    first: u32,
    count: u32,
) -> Result<(), Refused> {
    if count > 0 && first.checked_add(count).is_none_or(|end| end > MOST_POINTS) {
        return Err(BadInput("points past the record's 0x80").into());
    }
    let points = s.points(paths, first, count)?;
    crate::scene::matrix3::transform_int_points(
        &points[..],
        &paths.spline_work.ordered_coords,
        count,
        &paths.spline_work.turn,
    )
}

/// Ordered point `k` plus record `s`'s +0xC54 .. written through the record's pointer
/// `pointer` (at +0x454).
pub fn write_point(paths: &Paths, s: Spline, k: u32, pointer: u32) -> Result<(), Refused> {
    {
        let to = s.get(paths, IDS + 4 * pointer)?;
        let ordered = &paths.spline_work.ordered_coords;
        for c in 0..3 {
            let from = ordered
                .get((3 * k + c) as usize)
                .ok_or(BadInput("an ordered point past 0x80"))?;
            // `to` is a place of `scene::object_points` (phase 13).
            let v = from.get().wrapping_add(s.get(paths, CENTRE + 4 * c)?);
            crate::scene::object_points::write(to + 4 * c, &v.to_le_bytes())?;
        }
    }
    Ok(())
}

/// `effects::order_spline_points` (0x4BA388): record `s`'s ids and points put in grade
/// order (each grade's the last first) through `g_marksOrderedIds` and
/// `g_marksOrderedCoords`; +0 .. +8 each grade's start, +0xC .. +0x14 its count; then the
/// count's ids and points moved back (as `memory::move_bytes` would, as many as the count,
/// whatever their grade). Refuses more than 0x80 points or a negative count. Converted:
/// `rc2k_core::effects::splines::order`.
pub fn order_spline_points(paths: &Paths, s: Spline) -> Result<(), Refused> {
    {
        let n = s.count(paths)?;
        if (s.get(paths, COUNT)? as i32) < 0 {
            return Err(BadInput("a spline of a negative count: the moves would run away").into());
        }
        let (ids, coords) = (
            &paths.spline_work.ordered_ids,
            &paths.spline_work.ordered_coords,
        );
        let n_us = n as usize;
        // What the moves back read: the ordered tables' first `n`, those not placed as
        // they were.
        let mut ordered_ids: Vec<u32> = ids[..n_us]
            .iter()
            .map(super::super::session::Dword::get)
            .collect();
        let mut ordered_coords: Vec<[u32; 3]> = coords[..3 * n_us]
            .chunks(3)
            .map(|p| [p[0].get(), p[1].get(), p[2].get()])
            .collect();
        let o = rc2k_core::effects::splines::order(
            &s.words(paths, GRADES, n)?,
            &s.words(paths, IDS, n)?,
            &s.triples(paths, COORDS, n)?,
            &mut ordered_ids,
            &mut ordered_coords,
        );
        let placed = o.placed as usize;
        for (cell, &v) in ids.iter().zip(&ordered_ids[..placed]) {
            cell.set(v);
        }
        for (cell, &v) in coords.iter().zip(ordered_coords[..placed].as_flattened()) {
            cell.set(v);
        }
        s.put_words(paths, 0, &o.starts)?;
        s.put_words(paths, 0xC, &o.counts)?;
        s.put_words(paths, IDS, &ordered_ids)?;
        s.put_triples(paths, COORDS, &ordered_coords)?;
    }
    Ok(())
}

/// `effects::bound_spline_points` (0x4B9E7D): record `s`'s points' smallest and largest in
/// each part; the y and z sizes as floats into `g_marksSplineSize` + 4, + 8 (its x is
/// left as it was: section 16); the box's middle (half the size, `sar`, plus the smallest)
/// into the centre (+0xC54); the angle between `g_marksSplineUp` and the size
/// (`scene::angle_between_vectors`, both normalised) into +0x20. Refuses more than 0x80
/// points. Converted: `rc2k_core::effects::splines::bounds` (the floats, the sizes made
/// floats and the angle, stay the x87's: the first round as its control word says).
pub fn bound_spline_points(paths: &Paths, s: Spline) -> Result<(), Refused> {
    let n = s.count(paths)?;
    let b = rc2k_core::effects::splines::bounds(&s.triples(paths, COORDS, n)?);
    let size = b.size;
    let (up, size_at) = (&paths.spline_work.up, &paths.spline_work.size);
    size_at[1].set_bits(x87::from_int(size[1]));
    size_at[2].set_bits(x87::from_int(size[2]));
    for (c, &m) in b.middle.iter().enumerate() {
        s.put(paths, CENTRE + 4 * c as u32, m as u32)?;
    }
    let angle = crate::scene::vector3::angle_between_vectors(up, size_at);
    s.put(paths, 0x20, crate::scene::matrix3::rounded(angle))?;
    Ok(())
}

/// What `make_spline` refuses for an object: past 0xFF, a negative face count or faces past
/// `g_faces`, an angle of its tables `fsincos` cannot take.
fn spline_checks(
    paths: &Paths,
    object: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    use crate::scene::objects::FACES;
    if object >= 0x100 {
        return Err(BadInput("an object past 0xFF").into());
    }
    let first: u32 = scene.objects.slots.first_face[object as usize].get();
    let faces: u32 = scene.objects.slots.face_counts[object as usize].get();
    if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
        return Err(BadInput("an object's faces past g_faces").into());
    }
    for n in 0..12 {
        crate::scene::matrix3::turnable(&negated(paths.spline_constants.angles[n].bits()))?;
    }
    Ok(())
}

/// `effects::make_spline` (0x4B9A32): `object` into `g_marksSplineObject`; unless 8 are
/// made (`g_marksSplinePlace`), the next record of each table for it: of the faces of that
/// table's kind gathered, bounded, centred, given the table's angles and turned; the second
/// and third also given the first table's record 0's centre and +0x20 first (record 0's
/// whatever the place: section 16) and then graded (+0x18, +0x1C 0x58F0, 0x6F78), ordered
/// and their ends found; the third's +0x44 .. the highest point plus `g_marksSplineShift`.
/// Then the place counted. Refuses a negative place, an object past 0xFF or faces past
/// `g_faces`, an angle `fsincos` cannot take, before writing anything else.
pub fn make_spline(
    course: &Course,
    paths: &Paths,
    object: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    {
        paths.spline_work.object.set_bits(object);
        let place = paths.spline_work.place.bits() as i32;
        if place >= 8 {
            return Ok(());
        }
        if place < 0 {
            return Err(BadInput("a negative spline place").into());
        }
        spline_checks(paths, object, scene)?;
        let offset = place as u32 * SPLINE_LEN;
        paths.spline_work.offset.set_bits(offset);
        for t in 0..4 {
            let record = Spline::record(t, place as u32);
            gather_spline_points(
                course,
                paths,
                record,
                object,
                paths.spline_kinds.chosen[1 + t].bits(),
                scene,
            )?;
            bound_spline_points(paths, record)?;
            if t == 1 || t == 2 {
                for o in [CENTRE, CENTRE + 4, CENTRE + 8, 0x20] {
                    Spline::record(t, 0).put(paths, o, Spline::record(0, 0).get(paths, o)?)?;
                }
            }
            centre_spline_points(paths, record)?;
            for (k, o) in [0x24u32, 0x2C, 0x28].into_iter().enumerate() {
                record.put(
                    paths,
                    o,
                    x87::loaded(paths.spline_constants.angles[3 * t + k].bits()),
                )?;
            }
            turn_spline_points(paths, record)?;
            if t == 1 || t == 2 {
                record.put(paths, 0x18, 0x58F0)?;
                record.put(paths, 0x1C, 0x6F78)?;
                grade_spline_points(paths, record)?;
                order_spline_points(paths, record)?;
                find_spline_ends(paths, record)?;
            }
            if t == 2 {
                let shift: [u32; 3] = read_bits(&paths.spline_head.shift);
                let p = COORDS.wrapping_add(record.get(paths, 0x30)?.wrapping_mul(12));
                for c in 0..3 {
                    let v = record.get(paths, p.wrapping_add(4 * c))?;
                    record.put(paths, 0x44 + 4 * c, shift[c as usize].wrapping_add(v))?;
                }
            }
        }
        paths.spline_work.place.set_bits(place as u32 + 1);
    }
    Ok(())
}
pub const RESUME: u32 = 0x4B9706; // loc_4B9706
const KINDS: u32 = 0x22;
/// The settings' objects: a count at +0x26B8, then the objects.
const OBJECTS_AT: u32 = 0x26B8;
const MOST_OBJECTS: u32 = (0x2708 - 0x26BC) / 4;

/// `effects::load_splines` (0x4B96CD), in step 1: once (`g_marksSplinesLoaded`, then
/// `resume`) `files\m\spline.dat` (0xC00 bytes at most, `files::read_whole_file`) into
/// `g_marksSplineFile`; the five values of `g_marksKind` into 0x788BB2 ..;
/// `g_marksSplinePlace` 0; `make_spline` for each of the settings' objects (+0x26BC, the last
/// first); `dword_7695CE` 0. Refuses a mark neither 0 nor `resume`, a kind of 0x22 or
/// more, more than 19 objects, and what `make_spline` would refuse for any of them, before
/// writing anything.
pub fn load_splines(
    course: &Course,
    paths: &Paths,
    resume: u32,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let loaded: u32 = paths.spline_kinds.loaded.bits();
    if loaded != 0 && loaded != resume {
        return Err(BadInput("the splines' mark is neither 0 nor loc_4B9706").into());
    }
    let kind: u32 = effects.kinds.marks_kind.kind.bits();
    if kind >= KINDS {
        return Err(BadInput("a marks' kind of 0x22 or more").into());
    }
    let marks = crate::effects::setup::marks()?;
    let count = marks.object_count.get() as i32;
    if count > MOST_OBJECTS as i32 {
        return Err(BadInput("more than 19 spline objects").into());
    }
    let objects: Vec<u32> = (0..count.max(0) as u32)
        .rev()
        .map(|k| {
            marks
                .at(OBJECTS_AT + 4 + 4 * k)
                .read::<u32>("a spline object")
        })
        .collect::<Result<_, _>>()?;
    // The place starts at 0: the first eight are made, the rest only noted.
    for &o in objects.iter().take(8) {
        spline_checks(paths, o, scene)?;
    }
    if loaded == 0 {
        crate::files::read_whole_file::read_whole_file_bytes(
            &crate::files::cstring(&paths.spline_path.path, "a spline file's path")?,
            &paths.spline_head.file,
        )?;
        paths.spline_kinds.loaded.set_bits(resume);
    }
    for n in 0..5 {
        let v = paths.spline_kinds.by_kind[(n * KINDS + kind) as usize].bits();
        paths.spline_kinds.chosen[n as usize].set(v);
    }
    paths.spline_work.place.set_bits(0u32);
    for o in objects {
        make_spline(course, paths, o, scene)?;
    }
    paths.spline_head.made.set_bits(0u32);
    Ok(())
}

/// The car models' tables and the splines' head: the points and looks by car model (`effects::emit_particle_13`, `effects::draw_mark_slots`, `effects::cars_particles`), then the spline variables the looks table runs into (the original reads two dwords past it) and the spline file (IDA 0x769312 .. 0x76A1D2, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SplineHead {
    /// 0x769312: `g_marksPoints`: three floats by car model (34 models), a mark slot's point is an index into it (12 bytes each).
    pub points: [Float; 102],
    /// 0x7694AA: `dword_7694AA`, `dword_7694B2` ..: the looks by the car's model (two dwords 8 bytes apart, 33 pairs; models 32 and 33 read past the end: `found`, `wave`).
    pub looks: [Dword; 66],
    /// 0x7695B2: `g_marksSplineFaces`: how many faces of the kind `gather_spline_points` found.
    pub found: Dword,
    /// 0x7695B6: `flt_7695B6`: the wave's angle (`effects::run_splines`).
    pub angle: Float,
    /// 0x7695BA: `flt_7695BA`: the wave (`effects::run_splines`).
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

impl SplineHead {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        SplineHead {
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
        }
    }
    pub const START: usize = 0x769312;
    pub const LEN: usize = 0xEC0;
}

#[cfg(test)]
mod spline_head_tests {
    use super::*;

    #[test]
    fn spline_head_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplineHead>(), SplineHead::LEN);
        let p = (&raw const crate::effects::owner::EFFECTS.paths.spline_head).cast::<u8>();
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

/// The splines' work places (`effects::bend_points`, `effects::splines`, `effects::place_spline`) (IDA 0x76A1D2 .. 0x76B9B2, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SplineWork {
    /// 0x76A1D2: `dword_76A1D2` ..: four points of three whole numbers: A, B, C and D (`effects::bend_points`).
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

impl SplineWork {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        SplineWork {
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
                    0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xE0,
                    0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                ],
            )])),
        }
    }
    pub const START: usize = 0x76A1D2;
    pub const LEN: usize = 0x17E0;
}

#[cfg(test)]
mod spline_work_tests {
    use super::*;

    #[test]
    fn spline_work_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplineWork>(), SplineWork::LEN);
        let p = (&raw const crate::effects::owner::EFFECTS.paths.spline_work).cast::<u8>();
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

/// The four tables of eight spline records of 0xE90 bytes (`effects::splines`) (IDA 0x76B9B2 .. 0x788BB2, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
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

impl SplineTables {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        SplineTables {
            a: Bytes::new(crate::session::with_runs::<29824>(&[
                (
                    0x38,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0xEC8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x1D58,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x2BE8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x3A78,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x4908,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x5798,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x6628,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
            ])),
            b: Bytes::new(crate::session::with_runs::<29824>(&[
                (
                    0x38,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0xEC8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x1D58,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x2BE8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x3A78,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x4908,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x5798,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x6628,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
            ])),
            c: Bytes::new(crate::session::with_runs::<29824>(&[
                (
                    0x38,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0xEC8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x1D58,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x2BE8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x3A78,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x4908,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x5798,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x6628,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
            ])),
            d: Bytes::new(crate::session::with_runs::<29824>(&[
                (
                    0x38,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0xEC8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x1D58,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x2BE8,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x3A78,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x4908,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x5798,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
                (
                    0x6628,
                    &[
                        0xC0, 0xE0, 0xFF, 0xFF, 0x00, 0x80, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                        0xE0, 0x2E, 0x00, 0x00, 0x00, 0x80, 0xFF, 0xFF,
                    ],
                ),
            ])),
        }
    }
    pub const START: usize = 0x76B9B2;
    pub const LEN: usize = 0x1D200;
}

#[cfg(test)]
mod spline_tables_tests {
    use super::*;

    #[test]
    fn spline_tables_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplineTables>(), SplineTables::LEN);
        let p = (&raw const crate::effects::owner::EFFECTS.paths.spline_tables).cast::<u8>();
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

/// The splines' values by kind (`effects::splines`) (IDA 0x788BB2 .. 0x788EFA, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
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

impl SplineKinds {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        SplineKinds {
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
                0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC,
                0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F,
                0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC,
                0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F,
                0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC,
                0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F,
                0xCD, 0xCC, 0x4C, 0x3F, 0xCD, 0xCC, 0x4C, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xCD, 0xCC, 0x4C, 0x3F,
            ]),
            loaded: Dword::new(0x0),
        }
    }
    pub const START: usize = 0x788BB2;
    pub const LEN: usize = 0x348;
}

#[cfg(test)]
mod spline_kinds_tests {
    use super::*;

    #[test]
    fn spline_kinds_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplineKinds>(), SplineKinds::LEN);
        let p = (&raw const crate::effects::owner::EFFECTS.paths.spline_kinds).cast::<u8>();
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

/// The splines' file name (`effects::splines`) (IDA 0x788EFA .. 0x788F0D, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SplinePath {
    /// 0x788EFA: `files\m\spline.dat`, the file's name.
    pub path: Bytes<19>,
}

impl SplinePath {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        SplinePath {
            path: Bytes::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x73, 0x70, 0x6C, 0x69, 0x6E, 0x65,
                0x2E, 0x64, 0x61, 0x74, 0x00,
            ]),
        }
    }
    pub const START: usize = 0x788EFA;
    pub const LEN: usize = 0x13;
}

#[cfg(test)]
mod spline_path_tests {
    use super::*;

    #[test]
    fn spline_path_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplinePath>(), SplinePath::LEN);
        let p = (&raw const crate::effects::owner::EFFECTS.paths.spline_path).cast::<u8>();
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

/// The splines' constants (`effects::run_splines`, `effects::place_spline`, `effects::splines`) (IDA 0x788F0D .. 0x788F69, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SplineConstants {
    /// 0x788F0D: `flt_788F0D`, `flt_788F11`, `flt_788F15`: the wave's constants.
    pub wave_k: [Float; 3],
    /// 0x788F19: `flt_788F19` .. `flt_788F35`: `place_spline`'s factors and limits.
    pub place_k: [Float; 8],
    /// 0x788F39: `g_marksSplineAngles`: each table's +0x24, +0x2C, +0x28 (four times three).
    pub angles: [Float; 12],
}

impl SplineConstants {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        SplineConstants {
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
        }
    }
    pub const START: usize = 0x788F0D;
    pub const LEN: usize = 0x5C;
}

#[cfg(test)]
mod spline_constants_tests {
    use super::*;

    #[test]
    fn spline_constants_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<SplineConstants>(), SplineConstants::LEN);
        let p = (&raw const crate::effects::owner::EFFECTS.paths.spline_constants).cast::<u8>();
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

impl SplineHead {
    /// Dword `i` of the looks table, read as the original reads it: model 32 and 33 of the
    /// cars (34 models, pairs 8 bytes apart) run two and three dwords past the table's end
    /// into `found`, `angle` and `wave` (`notes/bugs.md`).
    pub fn look(&self, i: usize) -> u32 {
        match i {
            0..66 => self.looks[i].get(),
            66 => self.found.get(),
            67 => self.angle.bits(),
            68 => self.wave.bits(),
            _ => panic!("the looks table is read at most up to `wave`"),
        }
    }
}
