//! `make_masks`' shape set (a `MarkShapes` of 0xAAC bytes): two lists of points (the masks' points
//! lists A and B), turned by two matrices into the turned lists A and B and, offset, into the
//! points their targets name; and the two farthest apart of each list on one face.

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::errors::{BadInput, Refused};
use crate::math::matrix3::{
    Axis, extended, multiply_matrices3_plain, transform_int_points_bytes, turn_matrix3_plain,
    turnable,
};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::owner_course::Course;

/// The working block's farthest pair of list `n`.
pub(super) fn farthest_pair(drops: &WindscreenDrops, n: usize) -> [u32; 2] {
    drops.shape_work.borrow().work.farthest[n]
}

pub const MOST_POINTS: u32 = 0x40;

/// Point `k`'s part `c` of turned points `t` (12 bytes a point).
fn part(t: &[u8; 0x300], k: u32, c: u32) -> u32 {
    let at = (12 * k + 4 * c) as usize;
    u32::from_le_bytes([t[at], t[at + 1], t[at + 2], t[at + 3]])
}

/// Point `k`'s part `c` of turned points `t` set to `v`.
fn set_part(t: &mut [u8; 0x300], k: u32, c: u32, v: u32) {
    let at = (12 * k + 4 * c) as usize;
    t[at..at + 4].copy_from_slice(&v.to_le_bytes());
}

/// A points list's count, refused past 0x40 (its tables' end).
fn listed(count: u32) -> Result<u32, BadInput> {
    if count > MOST_POINTS {
        return Err(BadInput("a points list of more than 0x40"));
    }
    Ok(count)
}

/// `effects::find_farthest_pairs`: for points lists A and B, of the pairs of their points on
/// one face, the farthest apart by the turned lists A / B (the length rounded; a later
/// pair as far wins) into the farthest pairs A / B (the inner then the outer number).
/// Refuses a list of more than 0x40.
fn find_farthest_pairs(drops: &WindscreenDrops) -> Result<(), Refused> {
    {
        let counts = drops.shape_work.borrow().work.count;
        for count in counts {
            if count as i32 > MOST_POINTS as i32 {
                return Err(BadInput("a points list of more than 0x40").into());
            }
        }
        for n in 0..2 {
            let (faces, points) = {
                let work = drops.shape_work.borrow();
                (work.work.faces[n], work.work.turned[n])
            };
            let count = (counts[n] as i32).max(0) as u32;
            let mut farthest = 0i32;
            let mut pair = None;
            for j in (0..count).rev() {
                for i in (0..count).rev() {
                    if i == j || faces[i as usize] != faces[j as usize] {
                        continue;
                    }
                    let d = |c: u32| part(&points, i, c).wrapping_sub(part(&points, j, c)) as i32;
                    let length = x87::int_length3(d(0), d(1), d(2));
                    if farthest > length {
                        continue;
                    }
                    farthest = length;
                    pair = Some([i, j]);
                }
            }
            if let Some(pair) = pair {
                drops.shape_work.borrow_mut().work.farthest[n] = pair;
            }
        }
    }
    Ok(())
}

/// What `turn_shape_points` refuses for turns `a`, `b`.
pub(super) fn turn_shape_checks(drops: &WindscreenDrops, a: u32, b: u32) -> Result<(), Refused> {
    for n in 0..2 {
        let n = drops.shape_work.borrow().sets[0].count[n];
        if n as i32 > MOST_POINTS as i32 || n == 0x8000_0000 {
            return Err(BadInput("a shape list of more than 0x40").into());
        }
    }
    for t in [extended(drops.tilt.borrow().tilt), extended(a), extended(b)] {
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
    turn_shape_checks(drops, a, b)?;
    let angles = [extended(drops.tilt.borrow().tilt), extended(a), extended(b)];
    let mut guard = drops.shape_work.borrow_mut();
    let work = &mut *guard;
    turn_matrix3_plain(Axis::X, &mut work.turn_x, angles[0])?;
    turn_matrix3_plain(Axis::Z, &mut work.turn_z[0], angles[1])?;
    turn_matrix3_plain(Axis::Z, &mut work.turn_z[1], angles[2])?;
    for n in 0..2 {
        work.sets[0].matrix[n] = multiply_matrices3_plain(&work.turn_x, &work.turn_z[n]);
    }
    for n in 0..2 {
        transform_int_points_bytes(
            &work.sets[0].turned[n],
            &mut work.work.turned[n],
            work.sets[0].count[n],
            &work.sets[0].matrix[n],
        )?;
    }
    for n in 0..2 {
        let turned = &work.work.turned[n];
        let set = &work.sets[0];
        for k in (0..(set.count[n] as i32).max(0) as u32).rev() {
            let target = set.ids[n][k as usize];
            for c in 0..3 {
                let v = bh::to_int(
                    bh::int(part(turned, k, c) as i32) + bh::int(set.low[n][c as usize] as i32),
                );
                // `target` is a place of `scene::object_points_store`.
                object_points.write(target + 4 * c, &(v as u32).to_le_bytes())?;
            }
        }
    }
    Ok(())
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
        // Each shape's step (its `ease_step`), the index of its first angle in the tuning's
        // turn angles, its end (`work.steps`) and the switch that picks the second result.
        let all_steps = {
            let ease_steps = {
                let vars = masks.vars.borrow();
                [vars.shapes[0].ease_step, vars.shapes[1].ease_step]
            };
            let ends = {
                let work = drops.shape_work.borrow();
                [work.work.steps[0], work.work.steps[1]]
            };
            let flags = drops.picks.borrow().flags;
            [
                (ease_steps[0], 0, ends[0], flags[0]),
                (ease_steps[1], 2, ends[1], flags[1]),
            ]
        };
        let turn_angles = drops.tuning.borrow().turn_angles;
        let pick = |n: usize, (first, second): (u32, u32)| -> u32 {
            if all_steps[n].3 == 0 { first } else { second }
        };
        if count > 0 {
            let mut turns = [0u32; 2];
            for (n, &(step, a, b, _)) in all_steps.iter().enumerate() {
                turns[n] = pick(n, smooth_pair(step, turn_angles[a], b, effects));
            }
            turn_shape_checks(drops, turns[0], turns[1])?;
        }
        for _ in 0..count.max(0) {
            let mut turns = [0u32; 2];
            for (n, &(step, a, b, _)) in all_steps.iter().enumerate() {
                turns[n] = pick(n, smooth_steps(step, turn_angles[a], b, effects));
            }
            turn_shape_points(&effects.stage()?.object_points, drops, turns[0], turns[1])?;
        }
    }
    Ok(())
}
/// The working block a shape set is made in and the sets it is copied to.
pub const SETS: u32 = 0x22;
/// The sets held (`ShapeWork.sets`): the game makes at most 19 (it refuses more objects);
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
    let made = drops.shape_work.borrow().sets_made;
    if made > SETS {
        return Err(BadInput("more than 0x22 shape sets").into());
    }
    if made == SETS {
        return Ok(());
    }
    if object >= 0x100 {
        return Err(BadInput("an object past 0xFF").into());
    }
    let first: u32 = scene.objects.slots.borrow().first_face[object as usize];
    let faces: u32 = scene.objects.slots.borrow().face_counts[object as usize];
    if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
        return Err(BadInput("an object's faces past g_faces").into());
    }
    let steps: [u32; 2] = drops.inputs.borrow().tilts;
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
    faces_table: crate::scene::faces_store::FacesRef,
    drops: &WindscreenDrops,
    object: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    use crate::effects::masks::collect_points;
    shape_set_checks(drops, object, scene)?;
    let made: u32 = drops.shape_work.borrow().sets_made;
    if made == SETS {
        return Ok(());
    }
    if made >= SETS_HELD {
        return Err(BadInput("a shape set past the 33 held (the game makes at most 19)").into());
    }
    let (tilts, ends) = {
        let inputs = drops.inputs.borrow();
        (inputs.tilts, inputs.ends.map(f32::to_bits))
    };
    let inputs: [u32; 6] = [tilts[0], tilts[1], ends[0], ends[1], ends[2], ends[3]];
    let scales: [u32; 4] = drops.tuning.borrow().step_scales.map(f32::to_bits);
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
    {
        let mut work = drops.shape_work.borrow_mut();
        work.work.steps = steps;
        work.work.count = [0; 2];
    }
    let object = object as usize;
    let first: u32 = scene.objects.slots.borrow().first_face[object];
    let faces: u32 = scene.objects.slots.borrow().face_counts[object];
    let kinds: [u32; 2] = drops.tuning.borrow().face_kinds;
    for f in first..first + faces {
        let face: u32 = crate::scene::objects::face(faces_table, f as usize);
        let kind = course.stage.object_faces.byte(face + 7)?;
        if kind == kinds[0] as u8 {
            collect_points(&course.stage.object_faces, drops, 0, face)?;
        } else if kind == kinds[1] as u8 {
            collect_points(&course.stage.object_faces, drops, 1, face)?;
        }
    }
    let counts = {
        let work = drops.shape_work.borrow();
        [listed(work.work.count[0])?, listed(work.work.count[1])?]
    };
    let points: u32 = course.lists.borrow().object_points;
    {
        let mut guard = drops.shape_work.borrow_mut();
        let sw = &mut *guard;
        for (n, &count) in counts.iter().enumerate() {
            for id in sw.work.ids[n][..count as usize].iter_mut().rev() {
                *id = id.wrapping_mul(12).wrapping_add(points);
            }
        }
        for (n, &count) in counts.iter().enumerate() {
            for k in (0..count).rev() {
                let from = sw.work.ids[n][k as usize];
                for c in 0..3 {
                    let v = course.stage.object_points.word(from + 4 * c)?;
                    set_part(&mut sw.work.turned[n], k, c, v);
                }
            }
        }
        for (n, &count) in counts.iter().enumerate() {
            let mut lowest = 0x7FFF_FFFF;
            for k in (0..count).rev() {
                if (part(&sw.work.turned[n], k, 1) as i32) < lowest {
                    lowest = part(&sw.work.turned[n], k, 1) as i32;
                    for c in 0..3 {
                        sw.work.low[n][c as usize] = part(&sw.work.turned[n], k, c);
                    }
                }
            }
        }
        for (n, &count) in counts.iter().enumerate() {
            for k in (0..count).rev() {
                for c in 0..3 {
                    let v = part(&sw.work.turned[n], k, c).wrapping_sub(sw.work.low[n][c as usize]);
                    set_part(&mut sw.work.turned[n], k, c, v);
                }
            }
        }
        for n in 0..2 {
            let mut angle = extended(steps[6 + n]);
            angle[9] ^= 0x80;
            turn_matrix3_plain(Axis::X, &mut sw.turn_x, angle)?;
            let source = sw.work.turned[n];
            transform_int_points_bytes(&source, &mut sw.work.turned[n], MOST_POINTS, &sw.turn_x)?;
        }
    }
    find_farthest_pairs(drops)?;
    {
        let mut work = drops.shape_work.borrow_mut();
        let finished = work.work.clone();
        work.sets[made as usize] = finished;
        work.sets_made = made + 1;
    }
    Ok(())
}

/// `effects::make_shape_sets`: no sets made; the two face kinds of the marks' kind into
/// `ShapeSetTuning.face_kinds[0]`, `_28`; for each of the settings' objects (the last
/// first) the set's inputs (the tilt twice; the kind's two values and two floats) and
/// `make_shape_set`. Refuses a kind of 0x22 or more, more than 19 objects, what
/// `make_shape_set` refuses for any of them, before writing.
pub(super) fn make_shape_sets(
    course: &Course,
    faces_table: crate::scene::faces_store::FacesRef,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let kind: u32 = effects.kinds.marks_kind.get();
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
        let k = kind as usize; // under 0x22, checked above
        // The kind's two face kinds and its two ends (`ShapeSetTuning`'s `first_face_kind`,
        // `second_face_kind`, `end_a`, `end_b`), and the sets' second ends.
        let (by_kind, set_ends) = {
            let t = drops.tuning.borrow();
            (
                [
                    t.first_face_kind[k],
                    t.second_face_kind[k],
                    t.end_a[k].to_bits(),
                    t.end_b[k].to_bits(),
                ],
                t.set_ends,
            )
        };
        let tilt: u32 = drops.tilt.borrow().tilt.to_bits();
        let tilts = [tilt, tilt];
        let ends = [
            by_kind[2],
            x87::loaded(set_ends[0]).to_bits(),
            by_kind[3],
            x87::loaded(set_ends[1]).to_bits(),
        ]
        .map(f32::from_bits);
        // Checked as the sets will see them: none made, these tilts.
        let saved: ([u32; 2], u32) = (
            drops.inputs.borrow().tilts,
            drops.shape_work.borrow().sets_made,
        );
        drops.inputs.borrow_mut().tilts = tilts;
        drops.shape_work.borrow_mut().sets_made = 0;
        let checked = objects
            .iter()
            .try_for_each(|&o| shape_set_checks(drops, o, scene));
        drops.inputs.borrow_mut().tilts = saved.0;
        drops.shape_work.borrow_mut().sets_made = saved.1;
        checked?;
        drops.shape_work.borrow_mut().sets_made = 0;
        drops.tuning.borrow_mut().face_kinds = [by_kind[0], by_kind[1]];
        for o in objects {
            {
                let mut inputs = drops.inputs.borrow_mut();
                inputs.tilts = tilts;
                inputs.ends = ends;
            }
            make_shape_set(course, faces_table, drops, o, scene)?;
        }
    }
    Ok(())
}
