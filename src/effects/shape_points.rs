//! `make_masks`' shape set (`g_marksShapeSet`, a `MarkShapes` of 0xAAC bytes): two lists of points (the masks' points lists A and B), turned by two
//! matrices into `g_marksTurnedA` / `B` and, offset, into the points their targets name;
//! and the two farthest apart of each list on one face.

use crate::effects::mask_store::Masks;
use crate::effects::masks::PointList;
use crate::effects::shape_work::MarkShapes;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::owner_course::Course;
use crate::scene::matrix3::{Axis, extended, turn_matrix3, turnable};
use crate::session::{Bytes, Dword, Float, read_bits, write_bits};

impl MarkShapes {
    /// List `n`'s (0 A, 1 B) count of points.
    pub fn count(&self, n: usize) -> &Dword {
        [&self.count_a, &self.count_b][n]
    }
    /// List `n`'s point ids (the targets: pointers into the object's points).
    pub fn ids(&self, n: usize) -> &[Dword; 64] {
        [&self.ids_a, &self.ids_b][n]
    }
    /// List `n`'s turned points (12 bytes each).
    pub fn turned(&self, n: usize) -> &Bytes<0x300> {
        [&self.turned_a, &self.turned_b][n]
    }
    /// List `n`'s farthest pair.
    pub fn farthest(&self, n: usize) -> &[Dword; 2] {
        [&self.farthest_a, &self.farthest_b][n]
    }
    /// List `n`'s lowest point.
    pub fn low(&self, n: usize) -> &[Dword; 3] {
        [&self.low_a, &self.low_b][n]
    }
    /// List `n`'s turn.
    pub fn matrix(&self, n: usize) -> &[Float; 9] {
        [&self.matrix_a, &self.matrix_b][n]
    }
}

/// The working block's farthest pair of list `n` (`g_marksFarthestA`, `_B`).
pub fn farthest_pair(drops: &WindscreenDrops, n: usize) -> &[Dword; 2] {
    drops.shape_work.work.farthest(n)
}

/// The working block's turned points of list `n` (`g_marksTurnedA`, `_B`).
pub fn turned(drops: &WindscreenDrops, n: usize) -> &Bytes<0x300> {
    drops.shape_work.work.turned(n)
}

/// The shape set the turns use (`g_marksShapeSet`: the first of the sets).
fn set(drops: &WindscreenDrops) -> &MarkShapes {
    &drops.shape_work.sets[0]
}

/// The turn about X, and the turns about Z of lists A and B (3 x 3 floats).
fn turn_x(drops: &WindscreenDrops) -> &[Float; 9] {
    &drops.shape_work.turn_x
}
fn turn_z(drops: &WindscreenDrops, n: usize) -> &[Float; 9] {
    [&drops.shape_work.turn_a, &drops.shape_work.turn_b][n]
}
pub const MOST_POINTS: u32 = 0x40;

/// Point `k`'s part `c` of turned points `t` (12 bytes a point).
fn part(t: &Bytes<0x300>, k: u32, c: u32) -> u32 {
    t.get_u32((12 * k + 4 * c) as usize)
}

/// A points list's count, refused past 0x40 (its tables' end).
fn listed(count: &Dword) -> Result<u32, BadInput> {
    let n = count.get();
    if n > MOST_POINTS {
        return Err(BadInput("a points list of more than 0x40"));
    }
    Ok(n)
}

/// `effects::find_farthest_pairs` (0x4BDD1D): for points lists A and B, of the pairs of
/// their points on one face (`g_marksPointsAFaces`), the farthest apart by
/// `g_marksTurnedA` / `B` (the length rounded; a later pair as far wins) into
/// `g_marksFarthestA` / `B` (the inner then the outer number). Refuses a list of more
/// than 0x40.
pub fn find_farthest_pairs(drops: &WindscreenDrops) -> Result<(), Refused> {
    {
        let lists: [PointList; 2] = [PointList::a(drops), PointList::b(drops)];
        for list in lists {
            if list.count.get() as i32 > MOST_POINTS as i32 {
                return Err(BadInput("a points list of more than 0x40").into());
            }
        }
        for (list, n) in lists.into_iter().zip(0..) {
            let (faces, points) = (list.faces, turned(drops, n));
            let count = (list.count.get() as i32).max(0) as u32;
            let mut farthest = 0i32;
            for j in (0..count).rev() {
                for i in (0..count).rev() {
                    if i == j || faces[i as usize].get() != faces[j as usize].get() {
                        continue;
                    }
                    let d = |c: u32| part(points, i, c).wrapping_sub(part(points, j, c)) as i32;
                    let length = x87::int_length3(d(0), d(1), d(2));
                    if farthest > length {
                        continue;
                    }
                    farthest = length;
                    write_bits(farthest_pair(drops, n), [i, j]);
                }
            }
        }
    }
    Ok(())
}

/// What `turn_shape_points` refuses for turns `a`, `b`.
pub fn turn_shape_checks(drops: &WindscreenDrops, a: u32, b: u32) -> Result<(), Refused> {
    for n in 0..2 {
        let n = set(drops).count(n).get();
        if n as i32 > MOST_POINTS as i32 || n == 0x8000_0000 {
            return Err(BadInput("a shape list of more than 0x40").into());
        }
    }
    for t in [extended(drops.tilt.tilt.bits()), extended(a), extended(b)] {
        turnable(&t)?;
    }
    Ok(())
}

/// `effects::turn_shape_points` (0x4BD5ED): the turn about X by the shape tilt and about
/// Z by `a` and by `b` (floats); X times each into the shape set's two matrices; its two lists of points turned by
/// them into the turned lists A and B, and each turned point plus the list's offset
/// (`fiadd`) into the point its target names (the last first). Refuses a list of more than
/// 0x40 or of 0x80000000, or an angle `fsincos` cannot take, before writing.
pub fn turn_shape_points(drops: &WindscreenDrops, a: u32, b: u32) -> Result<(), Refused> {
    use crate::scene::matrix3::{multiply_matrices3, transform_int_points};
    turn_shape_checks(drops, a, b)?;
    let set = set(drops);
    let angles = [extended(drops.tilt.tilt.bits()), extended(a), extended(b)];
    turn_matrix3(Axis::X, turn_x(drops), angles[0])?;
    drops.inputs.scratch.set(a);
    turn_matrix3(Axis::Z, turn_z(drops, 0), angles[1])?;
    drops.inputs.scratch.set(b);
    turn_matrix3(Axis::Z, turn_z(drops, 1), angles[2])?;
    for n in 0..2 {
        multiply_matrices3(turn_x(drops), turn_z(drops, n), set.matrix(n));
    }
    for n in 0..2 {
        transform_int_points(
            set.turned(n),
            turned(drops, n),
            set.count(n).get(),
            set.matrix(n),
        )?;
    }
    for n in 0..2 {
        let turned = turned(drops, n);
        for k in (0..(set.count(n).get() as i32).max(0) as u32).rev() {
            let target = set.ids(n)[k as usize].get();
            for c in 0..3 {
                let v = bh::to_int(
                    bh::int(part(turned, k, c) as i32)
                        + bh::int(set.low(n)[c as usize].get() as i32),
                );
                // `target` is a place of `scene::object_points` (phase 13).
                crate::scene::object_points::write(target + 4 * c, &(v as u32).to_le_bytes())?;
            }
        }
    }
    Ok(())
}

/// The two shapes' steps and ends for `turn_all_shapes`: the step (each shape's +0), a, b,
/// and the switch that picks the second result.
fn all_steps<'a>(
    masks: &'a crate::effects::mask_store::Masks,
    drops: &'a WindscreenDrops,
) -> [(&'a Dword, usize, &'a Dword, &'a Dword); 2] {
    [
        (
            &masks.vars.shapes[0].ease_step,
            0,                               // `dword_89C974`, `ShapeSetTuning::turn_angles`
            &drops.shape_work.work.steps[0], // dword_884FC4
            &drops.picks.flags[0],           // dword_844604
        ),
        (
            &masks.vars.shapes[1].ease_step,
            2,                               // `dword_89C97C`
            &drops.shape_work.work.steps[1], // dword_884FC8
            &drops.picks.flags[1],           // dword_844608
        ),
    ]
}

/// `effects::turn_all_shapes` (0x4BD122): for each of the settings' objects (the last
/// first) the two shapes' eased steps (`smooth_steps` from `dword_89C974` / `_7C` to
/// `dword_884FC4` / `_C8` by each shape's +0), the second result of each unless its switch
/// (`dword_844604` / `_08`) is 0, and `turn_shape_points` with them. The object's offset
/// (times 0xAB8) is worked out and not used: the one set is turned every time (section
/// 16). Refuses what `turn_shape_points` would, before writing.
pub fn turn_all_shapes(
    masks: &Masks,
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use crate::effects::turns::{smooth_pair, smooth_steps};
    {
        let marks = effects.settings()?;
        let count = marks.object_count.get() as i32;
        let all_steps = all_steps(masks, drops);
        let pick = |n: usize, (first, second): (u32, u32)| -> u32 {
            if all_steps[n].3.get() == 0 {
                first
            } else {
                second
            }
        };
        if count > 0 {
            let mut turns = [0u32; 2];
            for (n, &(step, a, b, _)) in all_steps.iter().enumerate() {
                turns[n] = pick(
                    n,
                    smooth_pair(
                        step.get(),
                        drops.tuning.turn_angles[a].bits(),
                        b.get(),
                        effects,
                    ),
                );
            }
            turn_shape_checks(drops, turns[0], turns[1])?;
        }
        for _ in 0..count.max(0) {
            let mut turns = [0u32; 2];
            for (n, &(step, a, b, _)) in all_steps.iter().enumerate() {
                turns[n] = pick(
                    n,
                    smooth_steps(
                        drops,
                        step.get(),
                        drops.tuning.turn_angles[a].bits(),
                        b.get(),
                        effects,
                    ),
                );
            }
            turn_shape_points(drops, turns[0], turns[1])?;
        }
    }
    Ok(())
}
/// The working block a shape set is made in (`g_marksFarthestA` .. 0x885A6C, a
/// `MarkShapes` image) and the sets it is copied to.
pub const SETS: u32 = 0x22;
/// The sets held (`SHAPE_WORK.sets`): the game makes at most 19 (it refuses more objects);
/// the nominal 0x22nd would run past the range this port types (0x89C000).
const SETS_HELD: u32 = 33;

/// What `make_shape_set` refuses for object `object`: a set count past 0x22 (or below 0),
/// an object past 0xFF or its faces past `g_faces`, a tilt `fsincos` cannot take.
fn shape_set_checks(
    drops: &WindscreenDrops,
    object: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    use crate::scene::objects::FACES;
    let made = drops.shape_work.sets_made.get();
    if made > SETS {
        return Err(BadInput("more than 0x22 shape sets").into());
    }
    if made == SETS {
        return Ok(());
    }
    if object >= 0x100 {
        return Err(BadInput("an object past 0xFF").into());
    }
    let first: u32 = scene.objects.slots.first_face[object as usize].get();
    let faces: u32 = scene.objects.slots.face_counts[object as usize].get();
    if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
        return Err(BadInput("an object's faces past g_faces").into());
    }
    let steps: [u32; 2] = read_bits(&drops.inputs.tilts);
    for t in steps {
        let mut a = extended(t);
        a[9] ^= 0x80;
        turnable(&a)?;
    }
    Ok(())
}

/// `effects::make_shape_set` (0x4BD1A1): unless 0x22 are made, in the working block: the
/// inputs (0x764F60 ..) copied, four scaled; points lists A and B emptied and filled from
/// the object's faces of the two kinds (+7: two kind numbers; `collect_points_a` /
/// `_b`); their ids made pointers into the object points, the points copied to
/// the turned lists A and B, each list's lowest point (y, the first found lowest) taken from
/// all of it (kept at +0xA40 / +0xA4C), all 0x40 turned about X by minus the tilts
/// (`turn_matrix3_x`, `transform_int_points`); `find_farthest_pairs`; the block copied to
/// set n (`memory::move_bytes`), n counted. Refuses a set count past 0x22, an object past
/// 0xFF or faces past `g_faces`, a tilt `fsincos` cannot take, before writing.
pub fn make_shape_set(
    course: &Course,
    drops: &WindscreenDrops,
    object: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    use crate::effects::masks::collect_points;
    shape_set_checks(drops, object, scene)?;
    let made: u32 = drops.shape_work.sets_made.get();
    if made == SETS {
        return Ok(());
    }
    if made >= SETS_HELD {
        return Err(BadInput("a shape set past the 33 held (the game makes at most 19)").into());
    }
    let work = &drops.shape_work.work;
    let inputs: [u32; 6] = [
        drops.inputs.tilts[0].get(),
        drops.inputs.tilts[1].get(),
        drops.inputs.ends[0].bits(),
        drops.inputs.ends[1].bits(),
        drops.inputs.ends[2].bits(),
        drops.inputs.ends[3].bits(),
    ];
    let scales: [u32; 4] = read_bits(&drops.tuning.step_scales);
    // 0x884FC4 .. : a, b (0x764F6C, _74), c, d (_70, _78) scaled, the tilts, two zeros.
    let steps = [
        x87::mul(inputs[2], scales[0]),
        x87::mul(inputs[4], scales[1]),
        x87::mul(inputs[3], scales[2]),
        x87::mul(inputs[5], scales[3]),
        0,
        0,
        inputs[0],
        inputs[1],
    ];
    write_bits(&work.steps, steps);
    for list in [PointList::a(drops), PointList::b(drops)] {
        list.count.set(0);
    }
    let object = object as usize;
    let first: u32 = scene.objects.slots.first_face[object].get();
    drops
        .shape_work
        .first_point
        .set(scene.objects.slots.first_point[object].get());
    let faces: u32 = scene.objects.slots.face_counts[object].get();
    let kinds: [u32; 2] = read_bits(&drops.tuning.face_kinds);
    for f in first..first + faces {
        let face: u32 = crate::scene::objects::face_cell(f as usize).get();
        let kind = crate::scene::object_faces::byte(face + 7)?;
        if kind == kinds[0] as u8 {
            collect_points(&PointList::a(drops), face)?;
        } else if kind == kinds[1] as u8 {
            collect_points(&PointList::b(drops), face)?;
        }
    }
    let lists = [PointList::a(drops), PointList::b(drops)];
    let counts = [listed(lists[0].count)?, listed(lists[1].count)?];
    let points: u32 = course.lists.object_points.get();
    for (list, n) in lists.into_iter().zip(counts) {
        for id in list.ids[..n as usize].iter().rev() {
            id.set(id.get().wrapping_mul(12).wrapping_add(points));
        }
    }
    for ((list, count), n) in lists.into_iter().zip(counts).zip(0..) {
        let turned = turned(drops, n);
        for k in (0..count).rev() {
            let from = list.ids[k as usize].get();
            for c in 0..3 {
                let v = crate::scene::object_points::word(from + 4 * c)?;
                turned.set_u32((12 * k + 4 * c) as usize, v);
            }
        }
    }
    for (count, n) in counts.into_iter().zip(0..) {
        let turned = turned(drops, n);
        let mut lowest = 0x7FFF_FFFF;
        for k in (0..count).rev() {
            if (part(turned, k, 1) as i32) < lowest {
                lowest = part(turned, k, 1) as i32;
                for c in 0..3 {
                    work.low(n)[c as usize].set(part(turned, k, c));
                }
            }
        }
    }
    for (count, n) in counts.into_iter().zip(0..) {
        let turned = turned(drops, n);
        for k in (0..count).rev() {
            for c in 0..3 {
                let v = part(turned, k, c).wrapping_sub(work.low(n)[c as usize].get());
                turned.set_u32((12 * k + 4 * c) as usize, v);
            }
        }
    }
    let turn_x = turn_x(drops);
    for n in 0..2 {
        let mut angle = extended(steps[6 + n]);
        angle[9] ^= 0x80;
        turn_matrix3(Axis::X, turn_x, angle)?;
        crate::scene::matrix3::transform_int_points(
            turned(drops, n),
            turned(drops, n),
            MOST_POINTS,
            turn_x,
        )?;
    }
    find_farthest_pairs(drops)?;
    drops.shape_work.sets[made as usize].copy_from(&drops.shape_work.work);
    drops.shape_work.sets_made.set(made + 1);
    Ok(())
}

/// `effects::make_shape_sets` (0x4BD019): no sets made; the two face kinds of `g_marksKind`
/// into `dword_89C724`, `_28`; for each of the settings' objects (the last first) the set's
/// inputs (the tilt twice into 0x764F60, _64; the kind's two values and two floats into
/// 0x764F6C .. _78) and `make_shape_set`. Refuses a kind of 0x22 or more, more than 19
/// objects, what `make_shape_set` refuses for any of them, before writing.
pub fn make_shape_sets(
    course: &Course,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let kind: u32 = effects.kinds.marks_kind.kind.get();
        if kind >= SETS {
            return Err(BadInput("a marks' kind of 0x22 or more").into());
        }
        let marks = effects.settings()?;
        let count = marks.object_count.get() as i32;
        if count > 19 {
            return Err(BadInput("more than 19 shape objects").into());
        }
        let objects: Vec<u32> = (0..count.max(0) as u32)
            .rev()
            .map(|k| marks.at(0x26BC + 4 * k).read::<u32>("a shape object"))
            .collect::<Result<_, _>>()?;
        let t = &drops.tuning;
        let k = kind as usize; // under 0x22, checked above
        // The kind's two face kinds and the set's two ends (`dword_89C72C`, `_7B4`, `_8C4`, `_83C`).
        let by_kind = [
            t.first_face_kind[k].get(),
            t.second_face_kind[k].get(),
            t.end_a[k].bits(),
            t.end_b[k].bits(),
        ];
        let tilt: u32 = drops.tilt.tilt.bits();
        let tilts = [tilt, tilt];
        let ends = [
            by_kind[2],
            x87::loaded(&t.set_ends[0]),
            by_kind[3],
            x87::loaded(&t.set_ends[1]),
        ];
        // Checked as the sets will see them: none made, these tilts.
        let saved: ([u32; 2], u32) = (
            read_bits(&drops.inputs.tilts),
            drops.shape_work.sets_made.get(),
        );
        write_bits(&drops.inputs.tilts, tilts);
        drops.shape_work.sets_made.set(0);
        let checked = objects
            .iter()
            .try_for_each(|&o| shape_set_checks(drops, o, scene));
        write_bits(&drops.inputs.tilts, saved.0);
        drops.shape_work.sets_made.set(saved.1);
        checked?;
        drops.shape_work.sets_made.set(0);
        t.face_kinds[0].set(by_kind[0]);
        t.face_kinds[1].set(by_kind[1]);
        for o in objects {
            write_bits(&drops.inputs.tilts, tilts);
            write_bits(&drops.inputs.ends, ends);
            make_shape_set(course, drops, o, scene)?;
        }
    }
    Ok(())
}

/// The shape sets' inputs (`effects::shape_points`, `effects::turns`) (IDA 0x764F40 .. 0x764F7C).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ShapeInputs {
    /// 0x764F40: `dword_764F40`: the angle in hand while a turn matrix is made (`turn_shape_points`, `smooth_steps`; a scratch, nothing reads it back).
    pub scratch: Dword,
    /// 0x764F44: Not named: 28 bytes (a gap, or unreferenced data).
    pub _gap_764f44: Bytes<28>,
    /// 0x764F60: `g_marksShapeInputs`, `dword_764F64`: the tilts a shape set is made with (`make_shape_set`: copied into the working block; `make_shape_sets` writes both).
    pub tilts: [Dword; 2],
    /// 0x764F68: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_764f68: Bytes<4>,
    /// 0x764F6C: `flt_764F6C` .. `flt_764F78`: the set's four ends (`make_shape_sets` writes them, `make_shape_set` scales them into the working block).
    pub ends: [Float; 4],
}

impl ShapeInputs {
    pub const fn new() -> Self {
        ShapeInputs {
            scratch: Dword::new(0x0),
            _gap_764f44: Bytes::new([0; 28]),
            tilts: [Dword::new(0x0), Dword::new(0x0)],
            _gap_764f68: Bytes::new([0; 4]),
            ends: [
                Float::new(0.0),
                Float::new(0.0),
                Float::new(0.0),
                Float::new(0.0),
            ],
        }
    }
}

impl ShapeInputs {
    pub const START: usize = 0x764F40;
    pub const LEN: usize = 0x3C;
}

#[cfg(test)]
mod shape_inputs_tests {
    use super::*;

    #[test]
    fn shape_inputs_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<ShapeInputs>(), ShapeInputs::LEN);
        let p = (&raw const crate::effects::owner::EFFECTS.windscreen_drops.inputs).cast::<u8>();
        let ours: Vec<u8> = (0..size_of::<ShapeInputs>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(ShapeInputs::START, ShapeInputs::LEN),
            "SHAPE_INPUTS starts differently from the image"
        );
        assert_eq!(
            offset_of!(ShapeInputs, scratch),
            0x764F40 - ShapeInputs::START
        );
        assert_eq!(
            offset_of!(ShapeInputs, tilts),
            0x764F60 - ShapeInputs::START
        );
        assert_eq!(offset_of!(ShapeInputs, ends), 0x764F6C - ShapeInputs::START);
    }
}

/// The shape sets' tables and constants (`effects::make_shape_sets`, `make_shape_set`, `turn_all_shapes`) (IDA 0x89C724 .. 0x89C994).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ShapeSetTuning {
    /// 0x89C724: `g_marksShapeKinds`, `dword_89C728`: the two face kinds the shape sets are made for (set by `effects::make_shape_sets` from the kind's entries of the two tables that follow).
    pub face_kinds: [Dword; 2],
    /// 0x89C72C: `dword_89C72C` ..: the kind's first face kind, by `g_marksKind` (0x22 entries).
    pub first_face_kind: [Dword; 34],
    /// 0x89C7B4: `dword_89C7B4` ..: the kind's second face kind, by `g_marksKind` (0x22 entries).
    pub second_face_kind: [Dword; 34],
    /// 0x89C83C: `dword_89C83C` ..: the second shape's end by `g_marksKind` (floats: -80, -90 .. as bits; 0x22 entries), put into the set's inputs (`flt_764F74`).
    pub end_b: [Float; 34],
    /// 0x89C8C4: `dword_89C8C4` ..: the first shape's end by `g_marksKind` (floats as bits; 0x22 entries), put into the set's inputs (`flt_764F6C`).
    pub end_a: [Float; 34],
    /// 0x89C94C: `flt_89C94C` .. `flt_89C968`: four (end, 0) pairs, -90, -80, 80, 90 (read by the original `make_shape_sets`, which overwrites what it takes; the port reads none of them).
    pub end_pairs: [Float; 8],
    /// 0x89C96C: `flt_89C96C`, `flt_89C970`: the sets' second ends of the first and second shape (0 at first).
    pub set_ends: [Float; 2],
    /// 0x89C974: `dword_89C974`, `dword_89C978`, `dword_89C97C`, `dword_89C980`: each shape's eased turn from its first angle (0) and a second (-pi / 2, which the port does not read), as bits (`effects::turn_all_shapes`).
    pub turn_angles: [Float; 4],
    /// 0x89C984: `flt_89C984` .. `flt_89C990`: 0.0175 (degrees to radians) four times, the set's step scales (`effects::make_shape_set`).
    pub step_scales: [Float; 4],
}

impl ShapeSetTuning {
    pub const fn new() -> Self {
        ShapeSetTuning {
            face_kinds: [Dword::new(0x1E), Dword::new(0x1D)],
            first_face_kind: [
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0x1E),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0x1E),
            ],
            second_face_kind: [
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x1D),
                Dword::new(0x10),
                Dword::new(0x1D),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0xFFFFFFFF),
                Dword::new(0x1D),
            ],
            end_b: [
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(90.0),
                Float::new(-80.0),
                Float::new(90.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(-80.0),
                Float::new(75.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-80.0),
            ],
            end_a: [
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(80.0),
                Float::new(-90.0),
                Float::new(80.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-90.0),
                Float::new(-75.0),
                Float::new(99.0),
                Float::new(90.0),
                Float::new(90.0),
                Float::new(90.0),
                Float::new(90.0),
                Float::new(90.0),
                Float::new(90.0),
                Float::new(90.0),
                Float::new(90.0),
                Float::new(90.0),
                Float::new(-90.0),
                Float::new(-90.0),
            ],
            end_pairs: [
                Float::new(-90.0),
                Float::new(0.0),
                Float::new(-80.0),
                Float::new(0.0),
                Float::new(80.0),
                Float::new(0.0),
                Float::new(90.0),
                Float::new(0.0),
            ],
            set_ends: [Float::new(0.0), Float::new(0.0)],
            turn_angles: [
                Float::new(0.0),
                Float::new(-1.5707964),
                Float::new(0.0),
                Float::new(-1.5707964),
            ],
            step_scales: [
                Float::new(0.017453292),
                Float::new(0.017453292),
                Float::new(0.017453292),
                Float::new(0.017453292),
            ],
        }
    }
}

impl ShapeSetTuning {
    pub const START: usize = 0x89C724;
    pub const LEN: usize = 0x270;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(
            &raw const crate::effects::owner::EFFECTS.windscreen_drops.tuning,
        ) as usize
    }
}

#[cfg(test)]
mod typed_tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn shape_set_tuning_starts_as_the_games() {
        assert_eq!(size_of::<ShapeSetTuning>(), ShapeSetTuning::LEN);
        let image = crate::moved::image_bytes(ShapeSetTuning::START, ShapeSetTuning::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(ShapeSetTuning::home()).cast_const(),
                ShapeSetTuning::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "SHAPE_SET_TUNING starts differently from the image"
        );
        assert_eq!(
            offset_of!(ShapeSetTuning, first_face_kind),
            0x89C72C - ShapeSetTuning::START
        );
        assert_eq!(
            offset_of!(ShapeSetTuning, end_a),
            0x89C8C4 - ShapeSetTuning::START
        );
        assert_eq!(
            offset_of!(ShapeSetTuning, set_ends),
            0x89C96C - ShapeSetTuning::START
        );
        assert_eq!(
            offset_of!(ShapeSetTuning, turn_angles),
            0x89C974 - ShapeSetTuning::START
        );
        assert_eq!(
            offset_of!(ShapeSetTuning, step_scales),
            0x89C984 - ShapeSetTuning::START
        );
    }
}
