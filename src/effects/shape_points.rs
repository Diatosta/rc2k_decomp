//! `make_masks`' shape set (a `MarkShapes` of 0xAAC bytes): two lists of points (the masks' points
//! lists A and B), turned by two matrices into the turned lists A and B and, offset, into the
//! points their targets name; and the two farthest apart of each list on one face.

use crate::effects::mask_store::Masks;
use crate::effects::masks::PointList;
use crate::effects::shape_work_store::MarkShapes;
use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{Axis, extended, turn_matrix3, turnable};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::memory::session::{PlainBytes, PlainDword, PlainFloat, read_bits, write_bits};
use crate::race::owner_course::Course;

impl MarkShapes {
    /// List `n`'s (0 A, 1 B) count of points.
    pub fn count(&self, n: usize) -> &PlainDword {
        [&self.count_a, &self.count_b][n]
    }
    /// List `n`'s point ids (the targets: pointers into the object's points).
    pub fn ids(&self, n: usize) -> &[PlainDword; 64] {
        [&self.ids_a, &self.ids_b][n]
    }
    /// List `n`'s turned points (12 bytes each).
    pub fn turned(&self, n: usize) -> &PlainBytes<0x300> {
        [&self.turned_a, &self.turned_b][n]
    }
    /// List `n`'s farthest pair.
    pub fn farthest(&self, n: usize) -> &[PlainDword; 2] {
        [&self.farthest_a, &self.farthest_b][n]
    }
    /// List `n`'s lowest point.
    pub fn low(&self, n: usize) -> &[PlainDword; 3] {
        [&self.low_a, &self.low_b][n]
    }
    /// List `n`'s turn.
    pub fn matrix(&self, n: usize) -> &[PlainFloat; 9] {
        [&self.matrix_a, &self.matrix_b][n]
    }
}

/// The working block's farthest pair of list `n`.
pub(super) fn farthest_pair(drops: &WindscreenDrops, n: usize) -> &[PlainDword; 2] {
    drops.shape_work.work.farthest(n)
}

/// The working block's turned points of list `n`.
pub fn turned(drops: &WindscreenDrops, n: usize) -> &PlainBytes<0x300> {
    drops.shape_work.work.turned(n)
}

/// The shape set the turns use (the first of the sets).
fn set(drops: &WindscreenDrops) -> &MarkShapes {
    &drops.shape_work.sets[0]
}

/// The turn about X, and the turns about Z of lists A and B (3 x 3 floats).
fn turn_x(drops: &WindscreenDrops) -> &[PlainFloat; 9] {
    &drops.shape_work.turn_x
}
fn turn_z(drops: &WindscreenDrops, n: usize) -> &[PlainFloat; 9] {
    [&drops.shape_work.turn_a, &drops.shape_work.turn_b][n]
}
pub const MOST_POINTS: u32 = 0x40;

/// Point `k`'s part `c` of turned points `t` (12 bytes a point).
fn part(t: &PlainBytes<0x300>, k: u32, c: u32) -> u32 {
    t.get_u32((12 * k + 4 * c) as usize)
}

/// A points list's count, refused past 0x40 (its tables' end).
fn listed(count: &PlainDword) -> Result<u32, BadInput> {
    let n = count.get();
    if n > MOST_POINTS {
        return Err(BadInput("a points list of more than 0x40"));
    }
    Ok(n)
}

/// `effects::find_farthest_pairs`: for points lists A and B, of the pairs of their points on
/// one face, the farthest apart by the turned lists A / B (the length rounded; a later
/// pair as far wins) into the farthest pairs A / B (the inner then the outer number).
/// Refuses a list of more than 0x40.
fn find_farthest_pairs(drops: &WindscreenDrops) -> Result<(), Refused> {
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
pub(super) fn turn_shape_checks(drops: &WindscreenDrops, a: u32, b: u32) -> Result<(), Refused> {
    for n in 0..2 {
        let n = set(drops).count(n).get();
        if n as i32 > MOST_POINTS as i32 || n == 0x8000_0000 {
            return Err(BadInput("a shape list of more than 0x40").into());
        }
    }
    for t in [extended(drops.tilt.tilt.get()), extended(a), extended(b)] {
        turnable(&t)?;
    }
    Ok(())
}

/// `effects::turn_shape_points`: the turn about X by the shape tilt and about Z by `a` and by `b`
/// (floats); X times each into the shape set's two matrices; its two lists of points turned by them
/// into the turned lists A and B, and each turned point plus the list's whole-number offset into
/// the point its target names (the last first). Refuses a list of more than 0x40 or of
/// 0x80000000, or an angle with no sine and cosine, before writing.
pub(super) fn turn_shape_points(
    object_points: &crate::scene::object_points_store::ObjectPoints,
    drops: &WindscreenDrops,
    a: u32,
    b: u32,
) -> Result<(), Refused> {
    use crate::math::matrix3::{multiply_matrices3, transform_int_points};
    turn_shape_checks(drops, a, b)?;
    let set = set(drops);
    let angles = [extended(drops.tilt.tilt.get()), extended(a), extended(b)];
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
                // `target` is a place of `scene::object_points_store`.
                object_points.write(target + 4 * c, &(v as u32).to_le_bytes())?;
            }
        }
    }
    Ok(())
}

/// The two shapes' steps and ends for `turn_all_shapes`: the step (each shape's `ease_step`), a, b,
/// and the switch that picks the second result.
fn all_steps<'a>(
    masks: &'a crate::effects::mask_store::Masks,
    drops: &'a WindscreenDrops,
) -> [(&'a PlainDword, usize, &'a PlainDword, &'a PlainDword); 2] {
    [
        (
            &masks.vars.shapes[0].ease_step,
            0, // `ShapeSetTuning.turn_angles[0]`, `ShapeSetTuning::turn_angles`
            &drops.shape_work.work.steps[0], // ShapeWork.work.steps[0]
            &drops.picks.flags[0],
        ),
        (
            &masks.vars.shapes[1].ease_step,
            2,                               // `ShapeSetTuning.turn_angles[2]`
            &drops.shape_work.work.steps[1], // ShapeWork.work.steps[1]
            &drops.picks.flags[1],
        ),
    ]
}

/// `effects::turn_all_shapes`: for each of the settings' objects (the last first) the two shapes'
/// eased steps (`smooth_steps` from `ShapeSetTuning.turn_angles[0]` / `[2]` to
/// `ShapeWork.work.steps[0]` / `[1]` by each shape's `ease_step`), the second result of each unless
/// its switch (`MaskPicks.flags[0]` / `[1]`) is 0, and `turn_shape_points` with them. The object's
/// offset (times 0xAB8) is worked out and not used: the one set is turned every time (a slip kept
/// as is). Refuses what `turn_shape_points` would, before writing.
pub(super) fn turn_all_shapes(
    masks: &Masks,
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    use crate::effects::turns::{smooth_pair, smooth_steps};
    {
        let marks = effects.settings()?;
        let count = marks.object_count.get();
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
                        drops.tuning.turn_angles[a].get(),
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
                        drops.tuning.turn_angles[a].get(),
                        b.get(),
                        effects,
                    ),
                );
            }
            turn_shape_points(&effects.stage()?.object_points, drops, turns[0], turns[1])?;
        }
    }
    Ok(())
}
/// The working block a shape set is made in and the sets it is copied to.
pub const SETS: u32 = 0x22;
/// The sets held (`SHAPE_WORK.sets`): the game makes at most 19 (it refuses more objects);
/// the nominal 0x22nd would run past the range this port types.
const SETS_HELD: u32 = 33;

/// What `make_shape_set` refuses for object `object`: a set count past 0x22 (or below 0),
/// an object past 0xFF or its faces past the course's faces, a tilt with no sine and cosine.
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
    let first: u32 = scene.objects.slots.get().first_face[object as usize].get();
    let faces: u32 = scene.objects.slots.get().face_counts[object as usize].get();
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

/// `effects::make_shape_set`: unless 0x22 are made, in the working block: the inputs copied,
/// four scaled; points lists A and B emptied and filled from the object's faces of the two
/// kinds (`ShapeSetTuning.face_kinds`; `collect_points`); their ids made pointers into
/// the object points, the points copied to the turned lists A and B, each list's lowest
/// point (y, the first found lowest) taken from all of it (kept in the work block), all
/// 0x40 turned about X by minus the tilts (`turn_matrix3`, `transform_int_points`);
/// `find_farthest_pairs`; the block copied to set n, n counted.
/// Refuses a set count past 0x22, an object past 0xFF or faces past the course's faces, a
/// tilt with no sine and cosine, before writing.
fn make_shape_set(
    course: &Course,
    faces_table: &crate::scene::faces_store::Faces,
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
    let ends: [u32; 4] = read_bits(&drops.inputs.ends);
    let inputs: [u32; 6] = [
        drops.inputs.tilts[0].get(),
        drops.inputs.tilts[1].get(),
        ends[0],
        ends[1],
        ends[2],
        ends[3],
    ];
    let scales: [u32; 4] = read_bits(&drops.tuning.step_scales);
    // The steps: a, b, c, d scaled, the tilts, two zeros.
    let steps = [
        x87::mul(inputs[2], scales[0]).to_bits(),
        x87::mul(inputs[4], scales[1]).to_bits(),
        x87::mul(inputs[3], scales[2]).to_bits(),
        x87::mul(inputs[5], scales[3]).to_bits(),
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
    let first: u32 = scene.objects.slots.get().first_face[object].get();
    drops
        .shape_work
        .first_point
        .set(scene.objects.slots.get().first_point[object].get());
    let faces: u32 = scene.objects.slots.get().face_counts[object].get();
    let kinds: [u32; 2] = read_bits(&drops.tuning.face_kinds);
    for f in first..first + faces {
        let face: u32 = crate::scene::objects::face_cell(faces_table, f as usize).get();
        let kind = course.stage.object_faces.byte(face + 7)?;
        if kind == kinds[0] as u8 {
            collect_points(&course.stage.object_faces, &PointList::a(drops), face)?;
        } else if kind == kinds[1] as u8 {
            collect_points(&course.stage.object_faces, &PointList::b(drops), face)?;
        }
    }
    let lists = [PointList::a(drops), PointList::b(drops)];
    let counts = [listed(lists[0].count)?, listed(lists[1].count)?];
    let points: u32 = course.lists.get().object_points.get();
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
                let v = course.stage.object_points.word(from + 4 * c)?;
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
        crate::math::matrix3::transform_int_points(
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

/// `effects::make_shape_sets`: no sets made; the two face kinds of the marks' kind into
/// `ShapeSetTuning.face_kinds[0]`, `_28`; for each of the settings' objects (the last
/// first) the set's inputs (the tilt twice; the kind's two values and two floats) and
/// `make_shape_set`. Refuses a kind of 0x22 or more, more than 19 objects, what
/// `make_shape_set` refuses for any of them, before writing.
pub(super) fn make_shape_sets(
    course: &Course,
    faces_table: &crate::scene::faces_store::Faces,
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
        let count = marks.object_count.get();
        if count > 19 {
            return Err(BadInput("more than 19 shape objects").into());
        }
        let objects: Vec<u32> = (0..count.max(0) as u32)
            .rev()
            .map(|k| marks.object(k as usize))
            .collect::<Result<_, _>>()?;
        let t = &drops.tuning;
        let k = kind as usize; // under 0x22, checked above
        // The kind's two face kinds and its two ends (`ShapeSetTuning`'s `first_face_kind`,
        // `second_face_kind`, `end_a`, `end_b`).
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
            x87::loaded(&t.set_ends[0]).to_bits(),
            by_kind[3],
            x87::loaded(&t.set_ends[1]).to_bits(),
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
            make_shape_set(course, faces_table, drops, o, scene)?;
        }
    }
    Ok(())
}
