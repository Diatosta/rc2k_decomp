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
use crate::framework::{game_read, game_write, rva};
use crate::scene::objects::{
    FIRST_POINT_RVA, POINT_COUNTS_RVA, POINTS_RVA, SLOT_RVA, SLOTS, of_slot,
};

// ---- game data ----

/// `dword_634B70` .. `dword_634B9C`: the middle, half the extent, the lowest, the
/// highest, three dwords each.
pub const MEASURES_RVA: usize = rva(0x634B70);
/// More points than this are not an object's (the test and the regions' guard).
const MOST: u32 = 0x10_0000;

// ---- Rust version ----

unsafe fn points_of(slot: u32) -> Result<(usize, u32), BadInput> {
    unsafe {
        if slot >= SLOTS {
            return Err(BadInput("an object's slot past the tables"));
        }
        let count = of_slot(POINT_COUNTS_RVA, slot);
        let memory: u32 = game_read(POINTS_RVA);
        if count > MOST || (count != 0 && memory == 0) {
            return Err(BadInput("points without their memory, or too many"));
        }
        let first = of_slot(FIRST_POINT_RVA, slot);
        Ok((memory.wrapping_add(first.wrapping_mul(12)) as usize, count))
    }
}

// Converted: `rc2k_core::scene::objects::measure`
pub unsafe fn measure_object() -> Result<(), BadInput> {
    use rc2k_core::scene::objects::{HIGHEST_START, LOWEST_START, measure};
    unsafe {
        let slot: u32 = game_read(SLOT_RVA);
        let (at, count) = points_of(slot)?;
        let points: &mut [[u32; 3]] = if count == 0 {
            &mut []
        } else {
            std::slice::from_raw_parts_mut(at as *mut [u32; 3], count as usize)
        };
        let Some(m) = measure(points) else {
            // The half extents are not touched for an object of no points.
            game_write(MEASURES_RVA, [0u32; 3]);
            game_write(MEASURES_RVA + 0x18, [[LOWEST_START; 3], [HIGHEST_START; 3]]);
            return Ok(());
        };
        game_write(MEASURES_RVA, [m.middle, m.half, m.lowest, m.highest]);
        Ok(())
    }
}
