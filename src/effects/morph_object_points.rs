//! `effects::morph_object_points` (0x4D08DD, reached through the jumps at 0x4D074D and
//! 0x4D0815): for each car (the settings' +0x5C, the last first; its block at +0x29C of
//! its 0x4C8 bytes) the block's +0xC its +8; when its eight factors (+0x90 ..) differ from
//! the kept ones (+0xB0 ..): +0xC -1, the factors kept (also in `MorphWork.kept[0]` ..), and
//! for each of its first object's (+0) points the matching point of the second object
//! (+4) blended toward from the base list (+0x8C) by `effects::blend_corner`, where the
//! base and the second object's point differ. Then `effects::measure_lines`.
//!
//! Works on `g_marksSettings`.
//!
//! Caller: `effects::frame_marks`. Refuses (before anything) null settings, more than 8 cars, and
//! for a car whose factors changed: an object past 0xFF, a null base list or one (`lines::
//! BLOCK_POINTS`, Rust's since phase 12) shorter than the points, points past the
//! object points' room, a point count below 0 (the loop runs on); and `measure_lines`'
//! refusals (all it may write is put back first).

use crate::effects::blend_corner::blend_corner;
use crate::effects::lines::measure_lines;
use crate::effects::paths::Paths;
use crate::errors::{BadInput, Refused};
use crate::race::effects_settings_store::CarEffects;
use crate::race::owner_course::Course;
use crate::scene::objects::SLOTS;
use crate::scene::owner::Scene;
use crate::session::{PlainDword, PlainFloat};

const MOST_CARS: i32 = 8;

/// The cars' blocks, the last first (none for a count of 0 or less).
fn blocks(effects: &crate::effects::owner::Effects) -> Result<Vec<&'static CarEffects>, Refused> {
    let marks = effects.settings()?;
    Ok({
        let count = marks.cars_copy.get() as i32;
        (0..count.clamp(0, MOST_CARS) as u32)
            .rev()
            .map(|n| &marks.cars[n as usize])
            .collect()
    })
}

/// Whether a block's factors differ from its kept ones (compared by bits, as the original's
/// integer compare: a NaN or a signed zero counts as it is stored).
fn changed(b: &CarEffects) -> bool {
    b.factors
        .iter()
        .zip(&b.kept_factors)
        .any(|(f, kept)| f.bits() != kept.bits())
}

/// A block's objects' first points (in `g_objectPoints`) and their count.
fn points(course: &Course, scene: &Scene, b: &CarEffects) -> Result<(u32, u32, i32), Refused> {
    Ok({
        let base: u32 = course.lists.get().object_points.get();
        let first = |id: u32| {
            base.wrapping_add(
                12 * scene
                    .objects
                    .slots
                    .get()
                    .slot_value(&scene.objects.slots.get().first_point, id as usize),
            )
        };
        let (a, c) = (b.object.get(), b.second_object.get());
        (
            first(a),
            first(c),
            scene
                .objects
                .slots
                .get()
                .slot_value(&scene.objects.slots.get().point_counts, a as usize) as i32,
        )
    })
}

/// Why the function refuses, if it does, worked out without writing.
fn refusal(
    effects: &crate::effects::owner::Effects,
    course: &Course,
    paths: &Paths,
    scene: &Scene,
) -> Result<Option<&'static str>, Refused> {
    let marks = effects.settings()?;
    Ok({
        if marks.cars_copy.get() as i32 > MOST_CARS {
            return Ok(Some("more than 8 cars"));
        }
        let base: u32 = course.lists.get().object_points.get();
        let room: u32 = course.lists.get().object_points_room.get();
        for b in blocks(effects)? {
            if !changed(b) {
                continue;
            }
            if b.object.get() >= SLOTS || b.second_object.get() >= SLOTS {
                return Ok(Some("an object past 0xFF"));
            }
            if b.point_block.get() == 0 {
                return Ok(Some("a null base point list"));
            }
            let (a, c, n) = points(course, scene, b)?;
            if n < 0 {
                return Ok(Some("a negative point count: the loop runs on"));
            }
            if n > 0
                && crate::effects::lines::block_point(paths, b.point_block.get(), n as u32 - 1)
                    .is_err()
            {
                return Ok(Some("a base point list shorter than the object's points"));
            }
            let end = base.wrapping_add(room);
            for p in [a, c] {
                if p < base || p.wrapping_add(12 * n as u32) > end {
                    return Ok(Some("points past the object points' room"));
                }
            }
        }
        None
    })
}

pub fn morph_object_points(
    course: &Course,
    faces_table: &crate::scene::faces_store::Faces,
    paths: &Paths,
    scene: &Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    if let Some(why) = refusal(effects, course, paths, scene)? {
        return Err(BadInput(why).into());
    }
    {
        for b in blocks(effects)? {
            b.morph_kept.set(b.morph_start.get());
            if !changed(b) {
                continue;
            }
            b.morph_kept.set(0xFFFF_FFFF);
            for k in 0..8 {
                let v = b.factors[k].get();
                b.kept_factors[k].set(v);
                paths.morph_work.kept[k].set(v);
            }
            let (a, c, n) = points(course, scene, b)?;
            let points = b.point_block.get();
            for k in 0..n.max(0) as u32 {
                let (to, from) = (a + 12 * k, c + 12 * k);
                let base = crate::effects::lines::block_point(paths, points, k)?;
                if crate::effects::try_any(0..3, |i| {
                    Ok(base[i as usize] != crate::scene::object_points::word(from + 4 * i)?)
                })? {
                    blend_corner(b, to, from, base, effects)?;
                }
            }
        }
        measure_lines(course, faces_table, paths, scene, effects)?;
    }
    Ok(())
}

/// The object morph's work (IDA 0x93C6CC .. 0x93C6F4).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MorphWork {
    /// 0x93C6CC: `g_marksTaken`: the block last taken for a settings block's points (`lines::set_up_blocks`)
    pub taken: PlainDword,
    /// 0x93C6D4: `dword_93C6D4` .. `dword_93C6F0`: the factors kept
    pub kept: [PlainFloat; 8],
}

impl MorphWork {
    /// The values the game starts with (the image's bytes).
    pub const fn new() -> Self {
        MorphWork {
            taken: PlainDword::new(0x0),
            kept: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
        }
    }
}
