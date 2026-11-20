//! `effects::morph_object_points`: for each car (the settings' `cars_copy`, the last first), when
//! its eight `factors` differ from the `kept_factors`: `morph_kept` -1 (the start `morph_start`
//! is copied into it first), the factors kept, and for each of
//! its first object's points the matching point of the second object blended toward from the
//! base list (`point_block`) by `effects::blend_corner`, where the base and the second
//! object's point differ. Then `effects::measure_lines`.
//!
//! Refuses (before anything) null settings, more than 8 cars, and for a car whose factors changed:
//! an object past 0xFF, a null base list or one (`Paths::block_points`) shorter than the points,
//! points past the object points' room, a point count below 0; and `measure_lines`' refusals (all
//! it may write is put back first).

use crate::effects::blend_corner::blend_corner;
use crate::effects::lines::measure_lines;
use crate::effects::paths_store::Paths;
use crate::errors::{BadInput, Refused};
use crate::race::effects_settings_store::CarEffects;
use crate::race::owner_course::Course;
use crate::scene::object_slots_store::SlotTable;
use crate::scene::objects::SLOTS;
use crate::scene::owner::Scene;

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

/// Whether a block's factors differ from its kept ones (compared by bits, as an integer
/// compare: a NaN or a signed zero counts as it is stored).
fn changed(b: &CarEffects) -> bool {
    b.factors
        .iter()
        .zip(&b.kept_factors)
        .any(|(f, kept)| f.bits() != kept.bits())
}

/// A block's objects' first points (in the object points) and their count.
fn points(course: &Course, scene: &Scene, b: &CarEffects) -> Result<(u32, u32, i32), Refused> {
    Ok({
        let base: u32 = course.lists.get().object_points.get();
        let first = |id: u32| {
            base.wrapping_add(
                12 * scene
                    .objects
                    .slots
                    .get()
                    .slot_value(SlotTable::FirstPoint, id as usize),
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
                .slot_value(SlotTable::PointCounts, a as usize) as i32,
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
            }
            let (a, c, n) = points(course, scene, b)?;
            let points = b.point_block.get();
            for k in 0..n.max(0) as u32 {
                let (to, from) = (a + 12 * k, c + 12 * k);
                let base = crate::effects::lines::block_point(paths, points, k)?;
                if crate::effects::try_any(0..3, |i| {
                    Ok(base[i as usize] != course.stage.object_points.word(from + 4 * i)?)
                })? {
                    blend_corner(b, to, from, base, effects)?;
                }
            }
        }
        measure_lines(course, faces_table, paths, scene, effects)?;
    }
    Ok(())
}
