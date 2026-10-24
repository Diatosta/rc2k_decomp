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
use crate::framework::{game_addr, game_ptr, game_read, game_write, rva};
use crate::memory::allocate::allocate;
use crate::scene::allocate_object_memory::TAKEN_RVA;
use crate::scene::project_point::RECORDS_RVA;

// ---- game data ----

const LIST_RVA: usize = rva(0x5C6E50); // stru_5C6E50
const BYTES: u32 = 0x4_0000;

// ---- Rust version ----

/// `take`: what stands for `memory::allocate`, given the bytes and the list.
pub unsafe fn allocate_point_records(
    take: &mut dyn FnMut(u32, u32) -> Result<u32, Refused>,
) -> Result<(), Refused> {
    unsafe {
        let list = game_addr(LIST_RVA) as u32;
        game_write(TAKEN_RVA, game_read::<u32>(TAKEN_RVA).wrapping_add(BYTES));
        let block = take(BYTES, list)?;
        game_write(RECORDS_RVA, block);
        Ok(())
    }
}

pub unsafe fn allocate_point_records_in_game() -> Result<(), Refused> {
    unsafe {
        allocate_point_records(&mut |bytes, _| Ok(allocate(game_ptr(LIST_RVA), bytes)? as u32))
    }
}
