//! `scene::measure_object` (0x4222B8): finds the middle of the object being loaded
//! and moves its points so that the middle is at 0.
//!
//! In:  nothing (`dword_634FEC` the object's slot)
//! Out: ebx = the slot, ecx = 0, esi = past the object's points, eax = the middle's z;
//!      for an object of no points: ebx = the slot, ecx = 0, esi = its points' place
//!
//! The middle (`dword_634B70` ..) = 0; the lowest of each of x, y, z (`dword_634B88` ..) from
//! 0x640000 down and the highest (`dword_634B94` ..) from -0x640000 up over the slot's points;
//! half the extent of each (`dword_634B7C` ..) = (highest - lowest) halved, the middle = that
//! plus the lowest; then the middle is taken off every point. Eleven call sites in
//! `scene::load_object` (the parts of a car). Refuses a slot past the tables and points without
//! their memory.

use crate::errors::BadInput;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::scene::object_slots_store::OBJECT_SLOTS;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::objects::SLOTS;

// ---- game data ----

/// More points than this are not an object's (the test and the regions' guard).
const MOST: u32 = 0x10_0000;

// ---- Rust version ----

unsafe fn points_of(slot: u32) -> Result<(usize, u32), BadInput> {
    if slot >= SLOTS {
        return Err(BadInput("an object's slot past the tables"));
    }
    let count = OBJECT_SLOTS.point_counts[slot as usize].get();
    let memory: u32 = COURSE_LISTS.object_points.bits();
    if count > MOST || (count != 0 && memory == 0) {
        return Err(BadInput("points without their memory, or too many"));
    }
    let first = OBJECT_SLOTS.first_point[slot as usize].get();
    Ok((memory.wrapping_add(first.wrapping_mul(12)) as usize, count))
}

// Converted: `rc2k_core::scene::objects::measure`
pub unsafe fn measure_object() -> Result<(), BadInput> {
    use rc2k_core::scene::objects::{HIGHEST_START, LOWEST_START, measure};
    unsafe {
        let slot: u32 = OBJECT_WORK.loading_slot.bits();
        let (at, count) = points_of(slot)?;
        let points: &mut [[u32; 3]] = if count == 0 {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(at as *mut [u32; 3], count as usize)
        };
        let Some(m) = measure(points) else {
            // The half extents are not touched for an object of no points.
            crate::session::write_first(&OBJECT_SLOTS.measures[..3], &[0u32; 3]);
            crate::session::write_first(
                &OBJECT_SLOTS.measures[6..],
                &[[LOWEST_START; 3], [HIGHEST_START; 3]].concat(),
            );
            return Ok(());
        };
        crate::session::write_first(
            &OBJECT_SLOTS.measures,
            &[m.middle, m.half, m.lowest, m.highest].concat(),
        );
        Ok(())
    }
}
