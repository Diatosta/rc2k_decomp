//! `scene::measure_object` (0x4222B8): finds the middle of the object being loaded
//! and moves its points so that the middle is at 0.
//!
//! In:  nothing (`dword_634FEC` the object's slot)
//!
//! The middle (`dword_634B70` ..) = 0; the lowest of each of x, y, z (`dword_634B88` ..) from
//! 0x640000 down and the highest (`dword_634B94` ..) from -0x640000 up over the slot's points;
//! half the extent of each (`dword_634B7C` ..) = (highest - lowest) halved, the middle = that
//! plus the lowest; then the middle is taken off every point. Eleven call sites in
//! `scene::load_object` (the parts of a car). Refuses a slot past the tables and points without
//! their memory.

use crate::errors::BadInput;
use crate::scene::objects::SLOTS;

// ---- game data ----

/// More points than this are not an object's (the test and the regions' guard).
const MOST: u32 = 0x10_0000;

// ---- Rust version ----

fn points_of(
    course: &crate::race::owner_course::Course,
    slot: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(usize, u32), BadInput> {
    let objects = &scene.objects;
    if slot >= SLOTS {
        return Err(BadInput("an object's slot past the tables"));
    }
    let count = objects.slots.point_counts[slot as usize].get();
    let memory: u32 = course.lists.object_points.bits();
    if count > MOST || (count != 0 && memory == 0) {
        return Err(BadInput("points without their memory, or too many"));
    }
    let first = objects.slots.first_point[slot as usize].get();
    Ok((memory.wrapping_add(first.wrapping_mul(12)) as usize, count))
}

// Converted: `rc2k_core::scene::objects::measure`
pub fn measure_object(
    course: &crate::race::owner_course::Course,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    use rc2k_core::scene::objects::{HIGHEST_START, LOWEST_START, measure};
    {
        let slot: u32 = scene.drawing.object_work.loading_slot.bits();
        let (at, count) = points_of(course, slot, scene)?;
        let mut points: Vec<[u32; 3]> = if count == 0 {
            Vec::new()
        } else {
            (0..count)
                .map(|n| crate::scene::object_points::point(at as u32 + 12 * n))
                .collect::<Result<_, _>>()?
        };
        let Some(m) = measure(&mut points) else {
            // The half extents are not touched for an object of no points.
            crate::session::write_first(&scene.objects.slots.measures[..3], &[0u32; 3]);
            crate::session::write_first(
                &scene.objects.slots.measures[6..],
                &[[LOWEST_START; 3], [HIGHEST_START; 3]].concat(),
            );
            return Ok(());
        };
        for (n, p) in points.iter().enumerate() {
            crate::scene::object_points::set_point(at as u32 + 12 * n as u32, *p)?;
        }
        crate::session::write_first(
            &scene.objects.slots.measures,
            &[m.middle, m.half, m.lowest, m.highest].concat(),
        );
        Ok(())
    }
}
