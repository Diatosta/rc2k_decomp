//! `scene::allocate_point_records` (0x41F976): takes the memory of the points'
//! records.
//!
//! In:  nothing
//! Out: eax = the memory, ebx = the list it is taken on; everything else preserved
//!
//! 0x40000 bytes (0x2000 records of 0x20 bytes: 0x1000 for a scene's points and 0x1000 for an
//! object's) on the list `stru_5C6E50`, counted in `dword_5C6F90`, to `g_pointRecords`. One
//! caller, the front end's 3D set-up.

use crate::errors::Refused;
use crate::memory::allocate::allocate;
use crate::race::course_lists_store::COURSE_LISTS;

// ---- game data ----

const BYTES: u32 = 0x4_0000;

// ---- Rust version ----

/// `take`: what stands for `memory::allocate`, given the bytes and the list.
pub fn allocate_point_records(
    take: &mut dyn FnMut(u32, u32) -> Result<u32, Refused>,
) -> Result<(), Refused> {
    {
        let list = COURSE_LISTS.point_records_list[0].addr() as u32;
        COURSE_LISTS
            .course_bytes_total
            .set_bits(COURSE_LISTS.course_bytes_total.bits().wrapping_add(BYTES));
        let block = take(BYTES, list)?;
        COURSE_LISTS.point_records.set_bits(block);
        Ok(())
    }
}

pub fn allocate_point_records_in_game() -> Result<(), Refused> {
    allocate_point_records(&mut |bytes, _| {
        Ok(crate::pointers::game_address(allocate(
            crate::pointers::host_at(COURSE_LISTS.point_records_list[0].addr()) as *mut _,
            bytes,
        )?))
    })
}
