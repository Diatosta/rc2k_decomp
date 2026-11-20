//! `scene::measure_object`: finds the middle of the object being loaded and moves its points
//! so that the middle is at 0 (`loading_slot` is the object's slot).
//!
//! The middle = 0; the lowest of each of x, y, z from 0x640000 down and the highest from
//! -0x640000 up over the slot's points; half the extent of each = (highest - lowest) halved,
//! the middle = that plus the lowest; then the middle is taken off every point. Used by
//! `scene::load_object` for a measured car part whose shape has no points of its own to measure
//! (the shape's own are measured before they are stored). Refuses a slot past the tables and
//! points without their memory.

use crate::errors::BadInput;
use crate::scene::objects::SLOTS;

// ---- game data ----

/// More points than this are not an object's (the test and the regions' guard).
const MOST: u32 = 0x10_0000;

fn points_of(
    course: &crate::race::owner_course::Course,
    slot: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(usize, u32), BadInput> {
    let objects = &scene.objects;
    if slot >= SLOTS {
        return Err(BadInput("an object's slot past the tables"));
    }
    let count = objects.slots.get().point_counts[slot as usize].get();
    let memory: u32 = course.lists.get().object_points.get();
    if count > MOST || (count != 0 && memory == 0) {
        return Err(BadInput("points without their memory, or too many"));
    }
    let first = objects.slots.get().first_point[slot as usize].get();
    Ok((memory.wrapping_add(first.wrapping_mul(12)) as usize, count))
}

// Logic: `rc2k_core::scene::objects::measure`
pub fn measure_object(
    course: &crate::race::owner_course::Course,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    use rc2k_core::scene::objects::{HIGHEST_START, LOWEST_START, measure};
    {
        let slot: u32 = scene.object_work().loading_slot;
        let (at, count) = points_of(course, slot, scene)?;
        let mut points: Vec<[u32; 3]> = if count == 0 {
            Vec::new()
        } else {
            (0..count)
                .map(|n| course.stage.object_points.point(at as u32 + 12 * n))
                .collect::<Result<_, _>>()?
        };
        let Some(m) = measure(&mut points) else {
            // The half extents are not touched for an object of no points.
            let mut slots = scene.loaded_slots_mut();
            slots.measures[..3].copy_from_slice(&[0u32; 3]);
            slots.measures[6..].copy_from_slice(&[[LOWEST_START; 3], [HIGHEST_START; 3]].concat());
            return Ok(());
        };
        for (n, p) in points.iter().enumerate() {
            course
                .stage
                .object_points
                .set_point(at as u32 + 12 * n as u32, *p)?;
        }
        scene
            .loaded_slots_mut()
            .measures
            .copy_from_slice(&[m.middle, m.half, m.lowest, m.highest].concat());
        Ok(())
    }
}
