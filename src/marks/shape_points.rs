//! `make_masks`' shape set (`g_marksShapeSet`, a `MarkShapes` of 0xAAC bytes; IDA,
//! 2026-10-17): two lists of points (the masks' points lists A and B), turned by two
//! matrices into `g_marksTurnedA` / `B` and, offset, into the points their targets name;
//! and the two farthest apart of each list on one face.

use crate::errors::{BadInput, Refused};
use crate::marks::kind_values::MARKS_KIND;
use crate::marks::mask_picks::PICKS;
use crate::marks::mask_vars::MASKS;
use crate::marks::masks::{POINTS_A, POINTS_B, PointList};
use crate::marks::shape_work::{MarkShapes, SHAPE_WORK};
use crate::marks::tilt_shape::SHAPE_TILT;
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
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
pub fn farthest_pair(n: usize) -> &'static [Dword; 2] {
    SHAPE_WORK.work.farthest(n)
}

/// The working block's turned points of list `n` (`g_marksTurnedA`, `_B`).
pub fn turned(n: usize) -> &'static Bytes<0x300> {
    SHAPE_WORK.work.turned(n)
}

/// The shape set the turns use (`g_marksShapeSet`: the first of the sets).
fn set() -> &'static MarkShapes {
    &SHAPE_WORK.sets[0]
}

/// The turn about X, and the turns about Z of lists A and B (3 x 3 floats).
fn turn_x() -> u32 {
    crate::pointers::game_address(SHAPE_WORK.turn_x.as_ptr())
}
fn turn_z(n: usize) -> u32 {
    crate::pointers::game_address([&SHAPE_WORK.turn_a, &SHAPE_WORK.turn_b][n].as_ptr())
}
pub const MOST_POINTS: u32 = 0x40;

fn get(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read("a game field")
}

fn put(at: u32, v: u32) -> Result<(), BadInput> {
    GamePtr::new(at).write(v, "a game field")
}

/// `marks::find_farthest_pairs` (0x4BDD1D): for points lists A and B, of the pairs of
/// their points on one face (`g_marksPointsAFaces`), the farthest apart by
/// `g_marksTurnedA` / `B` (the length rounded; a later pair as far wins) into
/// `g_marksFarthestA` / `B` (the inner then the outer number). Refuses a list of more
/// than 0x40.
pub fn find_farthest_pairs() -> Result<(), Refused> {
    {
        let lists: [&PointList; 2] = [&POINTS_A, &POINTS_B];
        for list in lists {
            if list.count.get() as i32 > MOST_POINTS as i32 {
                return Err(BadInput("a points list of more than 0x40").into());
            }
        }
        for (list, n) in lists.into_iter().zip(0..) {
            let faces = crate::pointers::game_address(list.faces.as_ptr());
            let points = turned(n).addr();
            let count = (list.count.get() as i32).max(0) as u32;
            let mut farthest = 0i32;
            for j in (0..count).rev() {
                for i in (0..count).rev() {
                    if i == j || get(faces + 4 * i)? != get(faces + 4 * j)? {
                        continue;
                    }
                    let d = |o: u32| -> Result<_, Refused> {
                        Ok(get(points + i * 12 + o)?.wrapping_sub(get(points + j * 12 + o)?))
                    };
                    let length = x87::int_length3(d(0)? as i32, d(4)? as i32, d(8)? as i32);
                    if farthest > length {
                        continue;
                    }
                    farthest = length;
                    write_bits(farthest_pair(n), [i, j]);
                }
            }
        }
    }
    Ok(())
}

/// What `turn_shape_points` refuses for turns `a`, `b`.
pub fn turn_shape_checks(a: u32, b: u32) -> Result<(), Refused> {
    for n in 0..2 {
        let n = set().count(n).get();
        if n as i32 > MOST_POINTS as i32 || n == 0x8000_0000 {
            return Err(BadInput("a shape list of more than 0x40").into());
        }
    }
    for t in [extended(SHAPE_TILT.tilt.bits()), extended(a), extended(b)] {
        turnable(&t)?;
    }
    Ok(())
}

/// `marks::turn_shape_points` (0x4BD5ED): the turn about X by `g_marksShapeTilt` and about
/// Z by `a` and by `b` (floats; the original's ebx and ecx; `dword_764F40` left holding
/// `b`); X times each into the shape set's two matrices; its two lists of points turned by
/// them into `g_marksTurnedA` / `B`, and each turned point plus the list's offset
/// (`fiadd`) into the point its target names (the last first). Refuses a list of more than
/// 0x40 or of 0x80000000, or an angle `fsincos` cannot take, before writing.
pub fn turn_shape_points(a: u32, b: u32) -> Result<(), Refused> {
    use crate::marks::splines::{multiply, transform};
    turn_shape_checks(a, b)?;
    let set = set();
    let angles = [extended(SHAPE_TILT.tilt.bits()), extended(a), extended(b)];
    turn_matrix3(Axis::X, turn_x(), angles[0])?;
    SHAPE_INPUTS.scratch.set(a);
    turn_matrix3(Axis::Z, turn_z(0), angles[1])?;
    SHAPE_INPUTS.scratch.set(b);
    turn_matrix3(Axis::Z, turn_z(1), angles[2])?;
    for n in 0..2 {
        multiply(
            turn_x(),
            turn_z(n),
            crate::pointers::game_address(set.matrix(n).as_ptr()),
        );
    }
    for n in 0..2 {
        transform(
            set.turned(n).addr(),
            turned(n).addr(),
            set.count(n).get(),
            crate::pointers::game_address(set.matrix(n).as_ptr()),
        )?;
    }
    for n in 0..2 {
        let turned = turned(n).addr();
        for k in (0..(set.count(n).get() as i32).max(0) as u32).rev() {
            let target = set.ids(n)[k as usize].get();
            let point = turned + 12 * k;
            for c in 0..3 {
                let v = x87::int_sum(
                    get(point + 4 * c)? as i32,
                    set.low(n)[c as usize].get() as i32,
                );
                put(target + 4 * c, v as u32)?;
            }
        }
    }
    Ok(())
}

/// The two shapes' steps and ends for `turn_all_shapes`: the step (each shape's +0), a, b,
/// and the switch that picks the second result.
static ALL_STEPS: [(&Dword, usize, &Dword, &Dword); 2] = [
    (
        &MASKS.shapes[0].step,
        0,                         // `dword_89C974`, `ShapeSetTuning::turn_angles`
        &SHAPE_WORK.work.steps[0], // dword_884FC4
        &PICKS.flags[0],           // dword_844604
    ),
    (
        &MASKS.shapes[1].step,
        2,                         // `dword_89C97C`
        &SHAPE_WORK.work.steps[1], // dword_884FC8
        &PICKS.flags[1],           // dword_844608
    ),
];

/// `marks::turn_all_shapes` (0x4BD122): for each of the settings' objects (the last
/// first) the two shapes' eased steps (`smooth_steps` from `dword_89C974` / `_7C` to
/// `dword_884FC4` / `_C8` by each shape's +0), the second result of each unless its switch
/// (`dword_844604` / `_08`) is 0, and `turn_shape_points` with them. The object's offset
/// (times 0xAB8) is worked out and not used: the one set is turned every time (section
/// 16). Refuses what `turn_shape_points` would, before writing.
pub fn turn_all_shapes() -> Result<(), Refused> {
    use crate::marks::turns::{smooth_pair, smooth_steps};
    {
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        let count = get(settings + 0x26B8)? as i32;
        let pick = |n: usize, (first, second): (u32, u32)| -> u32 {
            if ALL_STEPS[n].3.get() == 0 {
                first
            } else {
                second
            }
        };
        if count > 0 {
            let mut turns = [0u32; 2];
            for (n, &(step, a, b, _)) in ALL_STEPS.iter().enumerate() {
                turns[n] = pick(
                    n,
                    smooth_pair(step.get(), SHAPE_SET_TUNING.turn_angles[a].bits(), b.get()),
                );
            }
            turn_shape_checks(turns[0], turns[1])?;
        }
        for _ in 0..count.max(0) {
            let mut turns = [0u32; 2];
            for (n, &(step, a, b, _)) in ALL_STEPS.iter().enumerate() {
                turns[n] = pick(
                    n,
                    smooth_steps(step.get(), SHAPE_SET_TUNING.turn_angles[a].bits(), b.get()),
                );
            }
            turn_shape_points(turns[0], turns[1])?;
        }
    }
    Ok(())
}
/// The working block a shape set is made in (`g_marksFarthestA` .. 0x885A6C, a
/// `MarkShapes` image) and the sets it is copied to.
pub const SET_LEN: u32 = 0xAB8;
pub const SETS: u32 = 0x22;
/// The sets held (`SHAPE_WORK.sets`): the game makes at most 19 (it refuses more objects);
/// the nominal 0x22nd would run past the range this port types (0x89C000).
const SETS_HELD: u32 = 33;

/// What `make_shape_set` refuses for object `object`: a set count past 0x22 (or below 0),
/// an object past 0xFF or its faces past `g_faces`, a tilt `fsincos` cannot take.
fn shape_set_checks(object: u32) -> Result<(), Refused> {
    use crate::scene::object_slots_store::OBJECT_SLOTS;
    use crate::scene::objects::FACES;
    let made = SHAPE_WORK.sets_made.get();
    if made > SETS {
        return Err(BadInput("more than 0x22 shape sets").into());
    }
    if made == SETS {
        return Ok(());
    }
    if object >= 0x100 {
        return Err(BadInput("an object past 0xFF").into());
    }
    let first: u32 = OBJECT_SLOTS.first_face[object as usize].get();
    let faces: u32 = OBJECT_SLOTS.face_counts[object as usize].get();
    if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
        return Err(BadInput("an object's faces past g_faces").into());
    }
    let steps: [u32; 2] = read_bits(&SHAPE_INPUTS.tilts);
    for t in steps {
        let mut a = extended(t);
        a[9] ^= 0x80;
        turnable(&a)?;
    }
    Ok(())
}

/// `marks::make_shape_set` (0x4BD1A1): unless 0x22 are made, in the working block: the
/// inputs (0x764F60 ..) copied, four scaled; points lists A and B emptied and filled from
/// object ebp's faces of the two kinds (+7: `dword_89C724`, `_28`; `collect_points_a` /
/// `_b`); their ids made pointers into `g_objectPoints`, the points copied to
/// `g_marksTurnedA` / `B`, each list's lowest point (y, the first found lowest) taken from
/// all of it (kept at +0xA40 / +0xA4C), all 0x40 turned about X by minus the tilts
/// (`turn_matrix3_x`, `transform_int_points`); `find_farthest_pairs`; the block copied to
/// set n (`memory::move_bytes`), n counted. Refuses a set count past 0x22, an object past
/// 0xFF or faces past `g_faces`, a tilt `fsincos` cannot take, before writing.
pub fn make_shape_set(object: u32) -> Result<(), Refused> {
    use crate::marks::masks::collect_points;
    use crate::scene::object_slots_store::OBJECT_SLOTS;
    shape_set_checks(object)?;
    let made: u32 = SHAPE_WORK.sets_made.get();
    if made == SETS {
        return Ok(());
    }
    if made >= SETS_HELD {
        return Err(BadInput("a shape set past the 33 held (the game makes at most 19)").into());
    }
    let work = &SHAPE_WORK.work;
    let inputs: [u32; 6] = [
        SHAPE_INPUTS.tilts[0].get(),
        SHAPE_INPUTS.tilts[1].get(),
        SHAPE_INPUTS.ends[0].bits(),
        SHAPE_INPUTS.ends[1].bits(),
        SHAPE_INPUTS.ends[2].bits(),
        SHAPE_INPUTS.ends[3].bits(),
    ];
    let scales: [u32; 4] = read_bits(&SHAPE_SET_TUNING.step_scales);
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
    for list in [&POINTS_A, &POINTS_B] {
        list.count.set(0);
    }
    let object = object as usize;
    let first: u32 = OBJECT_SLOTS.first_face[object].get();
    SHAPE_WORK
        .first_point
        .set(OBJECT_SLOTS.first_point[object].get());
    let faces: u32 = OBJECT_SLOTS.face_counts[object].get();
    let kinds: [u32; 2] = read_bits(&SHAPE_SET_TUNING.face_kinds);
    for f in first..first + faces {
        let face: u32 = crate::scene::objects::face_cell(f as usize).get();
        let kind = GamePtr::new(face + 7).read::<u8>("a face kind")?;
        if kind == kinds[0] as u8 {
            collect_points(&POINTS_A, face)?;
        } else if kind == kinds[1] as u8 {
            collect_points(&POINTS_B, face)?;
        }
    }
    let lists = [&POINTS_A, &POINTS_B];
    let points: u32 = COURSE_LISTS.object_points.bits();
    for list in lists {
        let ids = crate::pointers::game_address(list.ids.as_ptr());
        for k in (0..list.count.get()).rev() {
            put(
                ids + 4 * k,
                get(ids + 4 * k)?.wrapping_mul(12).wrapping_add(points),
            )?;
        }
    }
    for (list, n) in lists.into_iter().zip(0..) {
        let ids = crate::pointers::game_address(list.ids.as_ptr());
        let turned = turned(n).addr();
        for k in (0..list.count.get()).rev() {
            let from = get(ids + 4 * k)?;
            for c in 0..3 {
                put(turned + k * 12 + 4 * c, get(from + 4 * c)?)?;
            }
        }
    }
    for (list, n) in lists.into_iter().zip(0..) {
        let turned = turned(n).addr();
        let mut lowest = 0x7FFF_FFFF;
        for k in (0..list.count.get()).rev() {
            let p = turned + k * 12;
            if (get(p + 4)? as i32) < lowest {
                lowest = get(p + 4)? as i32;
                for c in 0..3 {
                    work.low(n)[c as usize].set(get(p + 4 * c)?);
                }
            }
        }
    }
    for (list, n) in lists.into_iter().zip(0..) {
        let turned = turned(n).addr();
        for k in (0..list.count.get()).rev() {
            let p = turned + k * 12;
            let d = |c: u32| -> Result<_, Refused> {
                Ok(get(p + 4 * c)?.wrapping_sub(work.low(n)[c as usize].get()))
            };
            let v = [d(0)?, d(1)?, d(2)?];
            for (c, x) in v.into_iter().enumerate() {
                put(p + 4 * c as u32, x)?;
            }
        }
    }
    let turn_x = turn_x();
    for n in 0..2 {
        let mut angle = extended(steps[6 + n]);
        angle[9] ^= 0x80;
        turn_matrix3(Axis::X, turn_x, angle)?;
        let turned = turned(n).addr();
        crate::marks::splines::transform(turned, turned, MOST_POINTS, turn_x)?;
    }
    find_farthest_pairs()?;
    let set = crate::pointers::game_address(core::ptr::addr_of!(SHAPE_WORK.sets)) + made * SET_LEN;
    GamePtr::new(crate::pointers::game_address(core::ptr::addr_of!(
        SHAPE_WORK.work
    )))
    .copy_to(GamePtr::new(set), SET_LEN as usize, "a shape set")?;
    SHAPE_WORK.sets_made.set(made + 1);
    Ok(())
}

/// `marks::make_shape_sets` (0x4BD019): no sets made; the two face kinds of `g_marksKind`
/// into `dword_89C724`, `_28`; for each of the settings' objects (the last first) the set's
/// inputs (the tilt twice into 0x764F60, _64; the kind's two values and two floats into
/// 0x764F6C .. _78) and `make_shape_set`. Refuses a kind of 0x22 or more, more than 19
/// objects, what `make_shape_set` refuses for any of them, before writing.
pub fn make_shape_sets() -> Result<(), Refused> {
    {
        let kind: u32 = MARKS_KIND.kind.bits();
        if kind >= SETS {
            return Err(BadInput("a marks' kind of 0x22 or more").into());
        }
        let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
        let count = get(settings + 0x26B8)? as i32;
        if count > 19 {
            return Err(BadInput("more than 19 shape objects").into());
        }
        let objects: Vec<u32> = (0..count.max(0) as u32)
            .rev()
            .map(|k| get(settings + 0x26BC + 4 * k))
            .collect::<Result<_, _>>()?;
        let t = &SHAPE_SET_TUNING;
        let k = kind as usize; // under 0x22, checked above
        // The kind's two face kinds and the set's two ends (`dword_89C72C`, `_7B4`, `_8C4`, `_83C`).
        let by_kind = [
            t.first_face_kind[k].get(),
            t.second_face_kind[k].get(),
            t.end_a[k].bits(),
            t.end_b[k].bits(),
        ];
        let tilt: u32 = SHAPE_TILT.tilt.bits();
        let tilts = [tilt, tilt];
        let ends = [
            by_kind[2],
            x87::loaded(t.set_ends[0].bits()),
            by_kind[3],
            x87::loaded(t.set_ends[1].bits()),
        ];
        // Checked as the sets will see them: none made, these tilts.
        let saved: ([u32; 2], u32) = (read_bits(&SHAPE_INPUTS.tilts), SHAPE_WORK.sets_made.get());
        write_bits(&SHAPE_INPUTS.tilts, tilts);
        SHAPE_WORK.sets_made.set(0);
        let checked = objects.iter().try_for_each(|&o| shape_set_checks(o));
        write_bits(&SHAPE_INPUTS.tilts, saved.0);
        SHAPE_WORK.sets_made.set(saved.1);
        checked?;
        SHAPE_WORK.sets_made.set(0);
        t.face_kinds[0].set(by_kind[0]);
        t.face_kinds[1].set(by_kind[1]);
        for o in objects {
            write_bits(&SHAPE_INPUTS.tilts, tilts);
            write_bits(&SHAPE_INPUTS.ends, ends);
            make_shape_set(o)?;
        }
    }
    Ok(())
}

/// The shape sets' inputs (`marks::shape_points`, `marks::turns`) (IDA 0x764F40 .. 0x764F7C, the image's layout; phase 7, typed state).
#[repr(C)]
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

pub static SHAPE_INPUTS: ShapeInputs = ShapeInputs {
    scratch: Dword::new(0x0),
    _gap_764f44: Bytes::new([0; 28]),
    tilts: [Dword::new(0x0), Dword::new(0x0)],
    _gap_764f68: Bytes::new([0; 4]),
    ends: [
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
        Float::from_bits(0x0),
    ],
};

impl ShapeInputs {
    pub const START: usize = 0x764F40;
    pub const LEN: usize = 0x3C;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&SHAPE_INPUTS as *const ShapeInputs) as usize
    }
}

#[cfg(test)]
mod shape_inputs_tests {
    use super::*;

    #[test]
    fn shape_inputs_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<ShapeInputs>(), ShapeInputs::LEN);
        let p = &SHAPE_INPUTS as *const ShapeInputs as *const u8;
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

/// The shape sets' tables and constants (`marks::make_shape_sets`, `make_shape_set`, `turn_all_shapes`) (IDA 0x89C724 .. 0x89C994, the image's layout; phase 7, typed state).
#[repr(C)]
pub struct ShapeSetTuning {
    /// 0x89C724: `g_marksShapeKinds`, `dword_89C728`: the two face kinds the shape sets are made for (set by `marks::make_shape_sets` from the kind's entries of the two tables that follow).
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
    /// 0x89C974: `dword_89C974`, `dword_89C978`, `dword_89C97C`, `dword_89C980`: each shape's eased turn from its first angle (0) and a second (-pi / 2, which the port does not read), as bits (`marks::turn_all_shapes`).
    pub turn_angles: [Float; 4],
    /// 0x89C984: `flt_89C984` .. `flt_89C990`: 0.0175 (degrees to radians) four times, the set's step scales (`marks::make_shape_set`).
    pub step_scales: [Float; 4],
}

pub static SHAPE_SET_TUNING: ShapeSetTuning = ShapeSetTuning {
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
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0x42B40000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0x42B40000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0x42960000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2A00000),
    ],
    end_a: [
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0x42A00000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0x42A00000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2960000),
        Float::from_bits(0x42C60000),
        Float::from_bits(0x42B40000),
        Float::from_bits(0x42B40000),
        Float::from_bits(0x42B40000),
        Float::from_bits(0x42B40000),
        Float::from_bits(0x42B40000),
        Float::from_bits(0x42B40000),
        Float::from_bits(0x42B40000),
        Float::from_bits(0x42B40000),
        Float::from_bits(0x42B40000),
        Float::from_bits(0xC2B40000),
        Float::from_bits(0xC2B40000),
    ],
    end_pairs: [
        Float::from_bits(0xC2B40000),
        Float::from_bits(0x0),
        Float::from_bits(0xC2A00000),
        Float::from_bits(0x0),
        Float::from_bits(0x42A00000),
        Float::from_bits(0x0),
        Float::from_bits(0x42B40000),
        Float::from_bits(0x0),
    ],
    set_ends: [Float::from_bits(0x0), Float::from_bits(0x0)],
    turn_angles: [
        Float::from_bits(0x0),
        Float::from_bits(0xBFC90FDB),
        Float::from_bits(0x0),
        Float::from_bits(0xBFC90FDB),
    ],
    step_scales: [
        Float::from_bits(0x3C8EFA35),
        Float::from_bits(0x3C8EFA35),
        Float::from_bits(0x3C8EFA35),
        Float::from_bits(0x3C8EFA35),
    ],
};

impl ShapeSetTuning {
    pub const START: usize = 0x89C724;
    pub const LEN: usize = 0x270;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&SHAPE_SET_TUNING as *const ShapeSetTuning) as usize
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
                crate::pointers::host_at(ShapeSetTuning::home()) as *const u8,
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
