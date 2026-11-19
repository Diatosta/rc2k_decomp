//! `init_marks`' step 1: the splines. `effects::load_splines` reads `files\m\spline.dat` once and,
//! for each of the settings' objects, `effects::make_spline` fills the next of 8 places in four
//! tables of spline records (0xE90 bytes): the points of the object's faces of one kind,
//! centred, turned, graded by height and ordered. These are the leaves; every function takes the
//! record.

use crate::effects::paths::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::memory::session::{PlainBytes, PlainDword, PlainFlag, PlainFloat, read_bits};
use crate::race::owner_course::Course;

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

/// The bytes of one of the four tables (A .. D).
const TABLE_LEN: u32 = 29824;
/// The four tables' bytes, end to end.
const TABLES_LEN: u32 = 4 * TABLE_LEN;

/// A spline record: its first byte's place in the four tables taken end to end. Fields are
/// read by offset from it; a read past the record goes on into the next ones (in
/// `find_spline_ends`, kept as is), and one past the tables is refused.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Spline(u32);

impl Spline {
    /// Record `n` (0 .. 7) of table `t` (A .. D).
    pub fn record(t: usize, n: u32) -> Spline {
        Spline(t as u32 * TABLE_LEN + n * SPLINE_LEN)
    }

    /// The table byte `at` of the four tables is in.
    fn table_byte(paths: &Paths, at: u32) -> &PlainBytes<29824> {
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
    pub fn points(self, paths: &Paths, first: u32, n: u32) -> Result<Vec<PlainDword>, BadInput> {
        Ok(self
            .triples(paths, COORDS + 12 * first, n)?
            .as_flattened()
            .iter()
            .map(|&v| PlainDword::new(v))
            .collect())
    }
}

#[cfg(test)]
mod spline_handle_tests {
    use super::*;

    #[test]
    fn a_record_reads_on_into_the_next_table_and_not_past_the_last() {
        let paths = &crate::test_support::own::effects().paths;
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

/// `effects::centre_spline_points`: record `s`'s points (+0x654, 12 bytes, the
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

/// `effects::grade_spline_points`: record `s`'s points' grades (+0xC90, the last
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

/// `effects::find_spline_ends`: of record `s`'s +0xC points from +0 the last with the
/// highest third value, into +0x30; of the +0x10 from +4 the last with the lowest, into
/// +0x34. The value is read at +0x65C + 0x30 a point, not 12 (a slip kept as is).
/// Refuses a negative start or count, or points past 0x80.
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

/// `effects::add_face_points`: face `face`'s points (+0 the count, from +0x10, 16
/// bytes each: the point and two values) added to record `s`'s (+0x50 the count, ids
/// +0x454, values +0x54 and +0x254) unless there already, up to 0x80. Refuses a record of
/// more than 0x80 points. Converted: `rc2k_core::effects::splines::FacePoints::add_face`.
pub fn add_face_points(
    object_faces: &crate::scene::object_faces::ObjectFaces,
    paths: &Paths,
    s: Spline,
    face: u32,
) -> Result<(), Refused> {
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
        let face_points = object_faces.word(face)?;
        let at = |k: u32| face + 0x10 + 0x10 * k;
        let bad = crate::effects::Sticky::default();
        points.add_face(
            face_points,
            |k| bad.take(object_faces.word(at(k))),
            |k| {
                [
                    bad.take(object_faces.word(at(k) + 4)),
                    bad.take(object_faces.word(at(k) + 8)),
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

/// `effects::reset_pong`: the eight floats from `PongRules.start[0]` to four (x, y)
/// pairs from `PongState.ball[0]`, `PongState.steps` 5, `PongState.hits` 0.
/// Converted: `rc2k_core::effects::pong::Court::reset` (exact at every precision: copying a
/// float only makes a signalling NaN quiet).
pub fn reset_pong(pong: &crate::effects::pong::Pong, effects: &crate::effects::owner::Effects) {
    let rules = crate::effects::pong::rules(pong, effects);
    crate::effects::pong::with_court(pong, |c| c.reset(&rules));
}

/// `effects::gather_spline_points`: record `s` emptied, then the points of object
/// `object`'s faces of kind `kind` (+7, low five bits) added (`add_face_points`; the faces
/// counted); the ids made pointers into the object points and the points copied to +0x654
/// (both the last first). Its last step, finding the farthest points, keeps nothing (a
/// slip kept as is): not made. Refuses an object past 0xFF, a negative face count or faces
/// past the course's faces.
pub fn gather_spline_points(
    course: &Course,
    faces_table: &crate::scene::faces_store::Faces,
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
        let first: u32 = scene.objects.slots.get().first_face[object].get();
        let faces: u32 = scene.objects.slots.get().face_counts[object].get();
        if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
            return Err(BadInput("an object's faces past g_faces").into());
        }
        s.put(paths, COUNT, 0)?;
        paths.spline_head.found.set(0);
        for f in first..first + faces {
            let face: u32 = crate::scene::objects::face_cell(faces_table, f as usize).get();
            if (course.stage.object_faces.byte(face + 7)? & 0x1F) as u32 == kind {
                paths
                    .spline_head
                    .found
                    .set(paths.spline_head.found.get().wrapping_add(1));
                add_face_points(&course.stage.object_faces, paths, s, face)?;
            }
        }
        let n = s.get(paths, COUNT)?;
        let points: u32 = course.lists.get().object_points.get();
        for k in (0..n).rev() {
            let id = s.get(paths, IDS + 4 * k)?;
            s.put(paths, IDS + 4 * k, id.wrapping_mul(12).wrapping_add(points))?;
        }
        for k in (0..n).rev() {
            let from = s.get(paths, IDS + 4 * k)?;
            for c in 0..3 {
                let v = course.stage.object_points.word(from + 4 * c)?;
                s.put(paths, COORDS + k * 12 + 4 * c, v)?;
            }
        }
    }
    Ok(())
}

/// An angle as `fld dword [x]; fchs` leaves it.
fn negated(f: u32) -> [u8; 10] {
    let mut a = crate::math::matrix3::extended(f);
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
        crate::math::matrix3::multiply_matrices3(a, b, out);
    }
}

/// `effects::turn_spline_points`: the turns about X, Y, Z by minus record `s`'s +0x20, +0x24,
/// +0x28 into the work turns, X x Y into the XY turn, that x Z into the turn, and the
/// record's points turned by it in place. Refuses more than 0x80 points or an angle with
/// no sine and cosine, before writing anything.
pub fn turn_spline_points(paths: &Paths, s: Spline) -> Result<(), Refused> {
    use crate::math::matrix3::{Axis, turn_matrix3, turnable};
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
    crate::math::matrix3::transform_int_points(
        &points[..],
        &points[..],
        n,
        &paths.spline_work.turn,
    )?;
    let turned: Vec<u32> = points
        .iter()
        .map(crate::memory::session::PlainDword::get)
        .collect();
    s.put_words(paths, COORDS, &turned)?;
    Ok(())
}

/// `effects::place_shape_points`: the turns about X, Y, Z by record `s`'s +0x20, +0x24,
/// +0x28 (not negated, as `turn_spline_points` does) into the work turns, X x Y into the XY
/// turn, that x Z into the turn; the record's points turned by it into the ordered
/// coordinates; then, from the last, each written plus the record's +0xC54 .. through the
/// record's pointer at +0x454. Refuses more than 0x80 points, an angle with no sine and
/// cosine, and a null pointer among the points', before writing anything.
pub fn place_shape_points(
    object_points: &crate::scene::object_points::ObjectPoints,
    paths: &Paths,
    s: Spline,
) -> Result<(), Refused> {
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
            write_point(object_points, paths, s, k, k)?;
        }
    }
    Ok(())
}

/// Whether the record's three angles (at `offsets`, not negated) can be turned by.
pub fn turnable_record(paths: &Paths, s: Spline, offsets: [u32; 3]) -> Result<(), Refused> {
    use crate::math::matrix3::{extended, turnable};
    for o in offsets {
        turnable(&extended(s.get(paths, o)?))?;
    }
    Ok(())
}

/// The turns about X, Y, Z by the record's floats at `offsets` (not negated) into the work
/// turns, X x Y into the XY turn, that x Z into the turn, as
/// `effects::place_shape_points` and `effects::place_graded_points` do.
pub fn turn_record(paths: &Paths, s: Spline, offsets: [u32; 3]) -> Result<(), Refused> {
    use crate::math::matrix3::{Axis, extended, turn_matrix3};
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

/// Record `s`'s `count` points from point `first` turned by the turn into the ordered
/// coordinates (`scene::transform_int_points`). Refuses points past the
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
    crate::math::matrix3::transform_int_points(
        &points[..],
        &paths.spline_work.ordered_coords,
        count,
        &paths.spline_work.turn,
    )
}

/// Ordered point `k` plus record `s`'s +0xC54 .. written through the record's pointer
/// `pointer` (at +0x454).
pub fn write_point(
    object_points: &crate::scene::object_points::ObjectPoints,
    paths: &Paths,
    s: Spline,
    k: u32,
    pointer: u32,
) -> Result<(), Refused> {
    {
        let to = s.get(paths, IDS + 4 * pointer)?;
        let ordered = &paths.spline_work.ordered_coords;
        for c in 0..3 {
            let from = ordered
                .get((3 * k + c) as usize)
                .ok_or(BadInput("an ordered point past 0x80"))?;
            // `to` is a place of `scene::object_points`.
            let v = from.get().wrapping_add(s.get(paths, CENTRE + 4 * c)?);
            object_points.write(to + 4 * c, &v.to_le_bytes())?;
        }
    }
    Ok(())
}

/// `effects::order_spline_points`: record `s`'s ids and points put in grade order (each
/// grade's the last first) through the ordered ids and coordinates; +0 .. +8 each grade's
/// start, +0xC .. +0x14 its count; then the count's ids and points moved back (as a
/// byte move would, as many as the count, whatever their grade). Refuses more
/// than 0x80 points or a negative count. Converted:
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
            .map(crate::memory::session::PlainDword::get)
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

/// `effects::bound_spline_points`: record `s`'s points' smallest and largest in each part;
/// the y and z sizes as floats into the spline size + 4, + 8 (its x is left as it was: a
/// slip kept as is); the box's middle (half the size, shifted right, plus the smallest) into the
/// centre (+0xC54); the angle between the spline's up vector and the size
/// (`scene::angle_between_vectors`, both normalised) into +0x20. Refuses more than 0x80
/// points. Converted: `rc2k_core::effects::splines::bounds` (the floats, the sizes made
/// floats and the angle, stay the x87's: the first round as its control word says).
pub fn bound_spline_points(
    paths: &Paths,
    work: &crate::effects::work_store::MarksWork,
    s: Spline,
) -> Result<(), Refused> {
    let n = s.count(paths)?;
    let b = rc2k_core::effects::splines::bounds(&s.triples(paths, COORDS, n)?);
    let size = b.size;
    let (up, size_at) = (&paths.spline_work.up, &paths.spline_work.size);
    size_at[1].set(x87::from_int(size[1]));
    size_at[2].set(x87::from_int(size[2]));
    for (c, &m) in b.middle.iter().enumerate() {
        s.put(paths, CENTRE + 4 * c as u32, m as u32)?;
    }
    let angle = crate::scene::vector3::angle_between_vectors(up, size_at, work);
    s.put(paths, 0x20, crate::math::matrix3::rounded(angle))?;
    Ok(())
}

/// What `make_spline` refuses for an object: past 0xFF, a negative face count or faces past
/// the course's faces, an angle of its tables with no sine and cosine.
fn spline_checks(
    paths: &Paths,
    object: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    use crate::scene::objects::FACES;
    if object >= 0x100 {
        return Err(BadInput("an object past 0xFF").into());
    }
    let first: u32 = scene.objects.slots.get().first_face[object as usize].get();
    let faces: u32 = scene.objects.slots.get().face_counts[object as usize].get();
    if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
        return Err(BadInput("an object's faces past g_faces").into());
    }
    for item in paths.spline_constants.angles.iter().take(12) {
        crate::math::matrix3::turnable(&negated(item.get().to_bits()))?;
    }
    Ok(())
}

/// `effects::make_spline`: `object` kept as the spline object; unless 8 are made, the next
/// record of each table for it: of the faces of that table's kind gathered, bounded,
/// centred, given the table's angles and turned; the second and third also given the first
/// table's record 0's centre and +0x20 first (record 0's whatever the place: a slip kept as
/// is) and then graded (+0x18, +0x1C 0x58F0, 0x6F78), ordered and their ends found; the
/// third's +0x44 .. the highest point plus the spline shift. Then the place counted.
/// Refuses a negative place, an object past 0xFF or faces past the course's faces, an angle
/// with no sine and cosine, before writing anything else.
pub fn make_spline(
    course: &Course,
    faces_table: &crate::scene::faces_store::Faces,
    paths: &Paths,
    work: &crate::effects::work_store::MarksWork,
    object: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    {
        paths.spline_work.object.set(object);
        let place = paths.spline_work.place.get() as i32;
        if place >= 8 {
            return Ok(());
        }
        if place < 0 {
            return Err(BadInput("a negative spline place").into());
        }
        spline_checks(paths, object, scene)?;
        let offset = place as u32 * SPLINE_LEN;
        paths.spline_work.offset.set(offset);
        for t in 0..4 {
            let record = Spline::record(t, place as u32);
            gather_spline_points(
                course,
                faces_table,
                paths,
                record,
                object,
                paths.spline_kinds.chosen[1 + t].get(),
                scene,
            )?;
            bound_spline_points(paths, work, record)?;
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
                    x87::loaded(&paths.spline_constants.angles[3 * t + k]).to_bits(),
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
        paths.spline_work.place.set(place as u32 + 1);
    }
    Ok(())
}
const KINDS: u32 = 0x22;
const MOST_OBJECTS: u32 = (0x2708 - 0x26BC) / 4;

/// `effects::load_splines`, in step 1: once (then the loaded flag is set) `files\m\spline.dat` (0xC00
/// bytes at most, `files::read_whole_file`) into the spline file; the five values of the
/// marks' kind kept; the place count 0; `make_spline` for each of the settings' objects
/// (+0x26BC, the last first). Refuses a kind of 0x22 or
/// more, more than 19 objects, and what `make_spline` would refuse for any of them, before
/// writing anything.
pub fn load_splines(
    course: &Course,
    faces_table: &crate::scene::faces_store::Faces,
    paths: &Paths,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let kind: u32 = effects.kinds.marks_kind.kind.get();
    if kind >= KINDS {
        return Err(BadInput("a marks' kind of 0x22 or more").into());
    }
    let marks = effects.settings()?;
    let count = marks.object_count.get();
    if count > MOST_OBJECTS as i32 {
        return Err(BadInput("more than 19 spline objects").into());
    }
    let objects: Vec<u32> = (0..count.max(0) as u32)
        .rev()
        .map(|k| marks.object(k as usize))
        .collect::<Result<_, _>>()?;
    // The place starts at 0: the first eight are made, the rest only noted.
    for &o in objects.iter().take(8) {
        spline_checks(paths, o, scene)?;
    }
    if !paths.spline_kinds.loaded.is_set() {
        crate::files::read_whole_file::read_whole_file_plain_bytes(
            &crate::files::cstring(&paths.spline_path.path, "a spline file's path")?,
            &paths.spline_head.file,
        )?;
        paths.spline_kinds.loaded.set(true);
    }
    for n in 0..5 {
        let v = paths.spline_kinds.by_kind[(n * KINDS + kind) as usize].get();
        paths.spline_kinds.chosen[n as usize].set(v);
    }
    paths.spline_work.place.set(0);
    for o in objects {
        make_spline(course, faces_table, paths, &effects.setup.work, o, scene)?;
    }
    paths.spline_head.made.set(0);
    Ok(())
}

/// The car models' tables and the splines' head: the points and looks by car model
/// (`effects::emit_exhaust_smoke`, `effects::draw_mark_slots`, `effects::cars_particles`), then the
/// spline variables the looks table runs into (two dwords past it are read) and the spline file.
#[derive(rc2k_derive::Cells)]
pub struct SplineHead {
    /// Three floats by car model (34 models), a mark slot's point is an index into it (12 bytes
    /// each).
    pub points: [PlainFloat; 102],
    /// The looks by the car's model (two dwords 8 bytes apart, 33 pairs; models 32 and 33 read past
    /// the end: `found`, `wave`).
    pub looks: [PlainDword; 66],
    /// How many faces of the kind `gather_spline_points` found.
    pub found: PlainDword,
    /// The wave's angle (`effects::run_splines`).
    pub angle: PlainFloat,
    /// The wave (`effects::run_splines`).
    pub wave: PlainFloat,
    /// Added to the third table's highest point for its +0x44.
    pub shift: [PlainDword; 3],
    /// 0 or 1, which of the two graded records is being placed.
    pub switch: PlainDword,
    /// The splines' fade-in counter, 0 .. 0xFF.
    pub made: PlainDword,
    /// `files\m\spline.dat` (0xC00 bytes at most): 12 bytes an entry, the blend weights.
    pub file: PlainBytes<3072>,
}

impl SplineHead {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineHead {
            points: [
                PlainFloat::new(2070.1),
                PlainFloat::new(265.6),
                PlainFloat::new(-572.5),
                PlainFloat::new(2055.2),
                PlainFloat::new(271.4),
                PlainFloat::new(-453.4),
                PlainFloat::new(2167.5),
                PlainFloat::new(229.9),
                PlainFloat::new(-463.5),
                PlainFloat::new(2232.2),
                PlainFloat::new(265.9),
                PlainFloat::new(472.5),
                PlainFloat::new(2204.0),
                PlainFloat::new(275.5),
                PlainFloat::new(427.5),
                PlainFloat::new(2012.3),
                PlainFloat::new(182.2),
                PlainFloat::new(-486.1),
                PlainFloat::new(2132.9),
                PlainFloat::new(296.0),
                PlainFloat::new(476.5),
                PlainFloat::new(2384.4),
                PlainFloat::new(236.3),
                PlainFloat::new(-547.2),
                PlainFloat::new(2184.4),
                PlainFloat::new(173.5),
                PlainFloat::new(-411.8),
                PlainFloat::new(2200.1),
                PlainFloat::new(252.0),
                PlainFloat::new(396.1),
                PlainFloat::new(2444.9),
                PlainFloat::new(259.8),
                PlainFloat::new(-458.9),
                PlainFloat::new(2019.6),
                PlainFloat::new(240.2),
                PlainFloat::new(-513.8),
                PlainFloat::new(2128.0),
                PlainFloat::new(232.8),
                PlainFloat::new(476.5),
                PlainFloat::new(1874.5),
                PlainFloat::new(244.1),
                PlainFloat::new(392.2),
                PlainFloat::new(2037.8),
                PlainFloat::new(203.6),
                PlainFloat::new(-538.7),
                PlainFloat::new(2243.2),
                PlainFloat::new(263.7),
                PlainFloat::new(-538.6),
                PlainFloat::new(2343.1),
                PlainFloat::new(319.1),
                PlainFloat::new(-501.5),
                PlainFloat::new(2135.3),
                PlainFloat::new(220.0),
                PlainFloat::new(-414.0),
                PlainFloat::new(1947.3),
                PlainFloat::new(340.3),
                PlainFloat::new(-548.3),
                PlainFloat::new(2029.5),
                PlainFloat::new(220.0),
                PlainFloat::new(-454.1),
                PlainFloat::new(2238.6),
                PlainFloat::new(299.3),
                PlainFloat::new(-264.7),
                PlainFloat::new(2024.5),
                PlainFloat::new(224.5),
                PlainFloat::new(405.7),
                PlainFloat::new(2365.3),
                PlainFloat::new(236.1),
                PlainFloat::new(-542.2),
                PlainFloat::new(1850.0),
                PlainFloat::new(350.0),
                PlainFloat::new(-400.0),
                PlainFloat::new(2060.0),
                PlainFloat::new(280.0),
                PlainFloat::new(480.0),
                PlainFloat::new(2120.0),
                PlainFloat::new(260.0),
                PlainFloat::new(-520.0),
                PlainFloat::new(1120.0),
                PlainFloat::new(180.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1120.0),
                PlainFloat::new(1120.0),
                PlainFloat::new(0.0),
                PlainFloat::new(950.0),
                PlainFloat::new(1600.0),
                PlainFloat::new(-300.0),
                PlainFloat::new(800.0),
                PlainFloat::new(1000.0),
                PlainFloat::new(-300.0),
                PlainFloat::new(2500.0),
                PlainFloat::new(250.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1450.0),
                PlainFloat::new(2800.0),
                PlainFloat::new(-0.0),
                PlainFloat::new(1450.0),
                PlainFloat::new(2800.0),
                PlainFloat::new(-0.0),
                PlainFloat::new(1929.0),
                PlainFloat::new(307.0),
                PlainFloat::new(-174.0),
            ],
            looks: [
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
                PlainDword::new(0xD800),
                PlainDword::new(0x14000),
            ],
            found: PlainDword::new(0x0),
            angle: PlainFloat::new(0.0),
            wave: PlainFloat::new(0.0),
            shift: [
                PlainDword::new(0x4000),
                PlainDword::new(0xFFFF8000),
                PlainDword::new(0x0),
            ],
            switch: PlainDword::new(0x0),
            made: PlainDword::new(0x0),
            file: PlainBytes::new([0; 3072]),
        }
    }
}

/// The splines' work places (`effects::bend_points`, `effects::splines`, `effects::place_spline`).
#[derive(rc2k_derive::Cells)]
pub struct SplineWork {
    /// Four points of three whole numbers: A, B, C and D (`effects::bend_points`).
    pub bend: [PlainDword; 12],
    /// The object being made.
    pub object: PlainDword,
    /// The next of 8 places.
    pub place: PlainDword,
    /// The place times 0xE90.
    pub offset: PlainDword,
    /// The vector the angle is measured from.
    pub up: [PlainFloat; 3],
    /// The points' size, as floats.
    pub size: [PlainFloat; 3],
    /// The X turn times the Y turn (a matrix of 12 floats).
    pub turn_xy: [PlainFloat; 12],
    /// All three turns.
    pub turn: [PlainFloat; 12],
    /// The turn about X.
    pub turn_x: [PlainFloat; 12],
    /// The turn about Y.
    pub turn_y: [PlainFloat; 12],
    /// The turn about Z.
    pub turn_z: [PlainFloat; 12],
    /// `order_spline_points`' work, one id a point (at most 0x80 points).
    pub ordered_ids: [PlainDword; 128],
    /// The ordered points, three whole numbers each.
    pub ordered_coords: [PlainDword; 384],
}

impl SplineWork {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineWork {
            bend: [const { PlainDword::new(0x0) }; 12],
            object: PlainDword::new(0x0),
            place: PlainDword::new(0x0),
            offset: PlainDword::new(0x0),
            up: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
            size: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            turn_xy: [const { PlainFloat::new(0.0) }; 12],
            turn: [const { PlainFloat::new(0.0) }; 12],
            turn_x: [const { PlainFloat::new(0.0) }; 12],
            turn_y: [const { PlainFloat::new(0.0) }; 12],
            turn_z: [const { PlainFloat::new(0.0) }; 12],
            ordered_ids: [const { PlainDword::new(0x0) }; 128],
            ordered_coords: [const { PlainDword::new(0x0) }; 384],
        }
    }
}

/// The four tables of eight spline records of 0xE90 bytes (`effects::splines`).
#[derive(rc2k_derive::Cells)]
pub struct SplineTables {
    /// Eight spline records.
    pub a: PlainBytes<29824>,
    /// Eight spline records, table B.
    pub b: PlainBytes<29824>,
    /// Eight spline records, table C.
    pub c: PlainBytes<29824>,
    /// Eight spline records, table D.
    pub d: PlainBytes<29824>,
}

impl SplineTables {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineTables {
            a: PlainBytes::new(crate::memory::session::with_runs::<29824>(&[
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
            b: PlainBytes::new(crate::memory::session::with_runs::<29824>(&[
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
            c: PlainBytes::new(crate::memory::session::with_runs::<29824>(&[
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
            d: PlainBytes::new(crate::memory::session::with_runs::<29824>(&[
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
}

/// The splines' values by kind (`effects::splines`).
#[derive(rc2k_derive::Cells)]
pub struct SplineKinds {
    /// The chosen kind, then the four kinds: the five values of the marks' kind.
    pub chosen: [PlainDword; 5],
    /// Five tables of 0x22 values by kind.
    pub by_kind: [PlainDword; 170],
    /// Set once the spline file is read.
    pub loaded: PlainFlag,
}

impl SplineKinds {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineKinds {
            chosen: [
                PlainDword::new(0x0),
                PlainDword::new(0x13),
                PlainDword::new(0x19),
                PlainDword::new(0x18),
                PlainDword::new(0x1C),
            ],
            by_kind: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x1),
                PlainDword::new(0x0),
                PlainDword::new(0x1),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x1),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x13),
                PlainDword::new(0x13),
                PlainDword::new(0xC),
                PlainDword::new(0x12),
                PlainDword::new(0x13),
                PlainDword::new(0x13),
                PlainDword::new(0x15),
                PlainDword::new(0x10),
                PlainDword::new(0x4),
                PlainDword::new(0x15),
                PlainDword::new(0x10),
                PlainDword::new(0xE),
                PlainDword::new(0x13),
                PlainDword::new(0x1C),
                PlainDword::new(0x13),
                PlainDword::new(0x15),
                PlainDword::new(0x11),
                PlainDword::new(0xE),
                PlainDword::new(0xA),
                PlainDword::new(0xB),
                PlainDword::new(0x11),
                PlainDword::new(0x16),
                PlainDword::new(0x10),
                PlainDword::new(0xE),
                PlainDword::new(0x15),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x13),
                PlainDword::new(0x18),
                PlainDword::new(0x18),
                PlainDword::new(0x1A),
                PlainDword::new(0x19),
                PlainDword::new(0x1C),
                PlainDword::new(0x18),
                PlainDword::new(0x1B),
                PlainDword::new(0x1C),
                PlainDword::new(0x18),
                PlainDword::new(0x1A),
                PlainDword::new(0x19),
                PlainDword::new(0x19),
                PlainDword::new(0x1A),
                PlainDword::new(0x12),
                PlainDword::new(0x18),
                PlainDword::new(0x18),
                PlainDword::new(0x1A),
                PlainDword::new(0x12),
                PlainDword::new(0xD),
                PlainDword::new(0xE),
                PlainDword::new(0x14),
                PlainDword::new(0x19),
                PlainDword::new(0x1C),
                PlainDword::new(0x11),
                PlainDword::new(0x1B),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x18),
                PlainDword::new(0x19),
                PlainDword::new(0x19),
                PlainDword::new(0x1B),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0x19),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0x19),
                PlainDword::new(0x19),
                PlainDword::new(0x18),
                PlainDword::new(0x1A),
                PlainDword::new(0x19),
                PlainDword::new(0x11),
                PlainDword::new(0x19),
                PlainDword::new(0x19),
                PlainDword::new(0x19),
                PlainDword::new(0x11),
                PlainDword::new(0xE),
                PlainDword::new(0xF),
                PlainDword::new(0x15),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0x12),
                PlainDword::new(0x1A),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x19),
                PlainDword::new(0x1C),
                PlainDword::new(0x1C),
                PlainDword::new(0x1C),
                PlainDword::new(0x1B),
                PlainDword::new(0x1F),
                PlainDword::new(0x1C),
                PlainDword::new(0x1C),
                PlainDword::new(0x1F),
                PlainDword::new(0x1B),
                PlainDword::new(0x1B),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0x1B),
                PlainDword::new(0x8),
                PlainDword::new(0x1A),
                PlainDword::new(0x1B),
                PlainDword::new(0x1C),
                PlainDword::new(0x1C),
                PlainDword::new(0x1B),
                PlainDword::new(0x1A),
                PlainDword::new(0x17),
                PlainDword::new(0x1C),
                PlainDword::new(0x1F),
                PlainDword::new(0x14),
                PlainDword::new(0x1C),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0xFFFFFFFF),
                PlainDword::new(0x1C),
            ],
            loaded: PlainFlag::new(false),
        }
    }
}

/// The splines' file name (`effects::splines`).
#[derive(rc2k_derive::Cells)]
pub struct SplinePath {
    /// `files\m\spline.dat`, the file's name.
    pub path: PlainBytes<19>,
}

impl SplinePath {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplinePath {
            path: PlainBytes::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x6D, 0x5C, 0x73, 0x70, 0x6C, 0x69, 0x6E, 0x65,
                0x2E, 0x64, 0x61, 0x74, 0x00,
            ]),
        }
    }
}

/// The splines' constants (`effects::run_splines`, `effects::place_spline`, `effects::splines`).
#[derive(rc2k_derive::Cells)]
pub struct SplineConstants {
    /// The wave's constants.
    pub wave_k: [PlainFloat; 3],
    /// `place_spline`'s factors and limits.
    pub place_k: [PlainFloat; 8],
    /// Each table's +0x24, +0x2C, +0x28 (four times three).
    pub angles: [PlainFloat; 12],
}

impl SplineConstants {
    /// The values the game starts with.
    pub const fn new() -> Self {
        SplineConstants {
            wave_k: [
                PlainFloat::new(128.0),
                PlainFloat::new(0.2),
                PlainFloat::new(1.0),
            ],
            place_k: [
                PlainFloat::new(0.8),
                PlainFloat::new(0.4),
                PlainFloat::new(0.0),
                PlainFloat::new(0.6),
                PlainFloat::new(0.1),
                PlainFloat::new(0.0),
                PlainFloat::new(0.6),
                PlainFloat::new(0.1),
            ],
            angles: [const { PlainFloat::new(0.0) }; 12],
        }
    }
}

impl SplineHead {
    /// Dword `i` of the looks table: model 32 and 33 of the cars (34 models, pairs 8 bytes
    /// apart) run two and three dwords past the table's end into `found`, `angle` and `wave`
    /// (a slip kept as is).
    pub fn look(&self, i: usize) -> u32 {
        match i {
            0..66 => self.looks[i].get(),
            66 => self.found.get(),
            // Floats read as dwords, past the table.
            67 => self.angle.get().to_bits(),
            68 => self.wave.get().to_bits(),
            _ => panic!("the looks table is read at most up to `wave`"),
        }
    }
}
