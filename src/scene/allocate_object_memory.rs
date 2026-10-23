//! `scene::allocate_object_memory` (0x41F814): takes the memory the objects' points
//! and faces are loaded into.
//!
//! In:  nothing
//! Out: eax = the memory's end, ebx = the list it is taken on; everything else
//!      preserved
//!
//! One block on the list `stru_5C6DD0`, counted in `dword_5C6F90`: in a race 0x300000
//! bytes, 0xC0000 of them for the points and 0x240000 for the faces; else 0x580000,
//! 0x180000 and 0x400000. `g_objectPoints` = the block, `dword_5C6FA8` (the faces'
//! memory) = what follows the points' part, `dword_5C6FAC` and `dword_5C6FB0` the two
//! rooms.
//! One caller, `scene::set_up_object_memory`.
//! Shadow mode runs the original alone and checks what it left.
//! Self-test: a redirected copy whose allocation is the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, Regs, game_addr, game_ptr, game_read, game_write, rva};
use crate::memory::allocate::allocate;
use crate::scene::objects::{FACE_MEMORY_RVA, FACE_ROOM_RVA, POINTS_ROOM_RVA, POINTS_RVA};
use crate::screens::harness::Left;

// ---- game data ----

const IN_RACE_RVA: usize = rva(0x5C5CEC); // g_inRace
const LIST_RVA: usize = rva(0x5C6DD0); // stru_5C6DD0
pub const TAKEN_RVA: usize = rva(0x5C6F90); // dword_5C6F90

use rc2k_core::scene::objects::object_rooms as rooms;

// ---- Rust version ----

/// `take`: what stands for `memory::allocate`, given the bytes and the list.
// Converted: `rc2k_core::scene::objects::object_rooms`
pub unsafe fn allocate_object_memory(
    take: &mut dyn FnMut(u32, u32) -> Result<u32, Refused>,
) -> Result<Left, Refused> {
    unsafe {
        let (points, faces) = rooms(game_read::<u32>(IN_RACE_RVA) != 0);
        let list = game_addr(LIST_RVA) as u32;
        game_write(
            TAKEN_RVA,
            game_read::<u32>(TAKEN_RVA).wrapping_add(points + faces),
        );
        let block = take(points + faces, list)?;
        game_write(POINTS_RVA, block);
        game_write(FACE_MEMORY_RVA, block.wrapping_add(points));
        game_write(POINTS_ROOM_RVA, points);
        game_write(FACE_ROOM_RVA, faces);
        Ok(Left::nothing()
            .with(EAX, block.wrapping_add(points + faces))
            .with(EBX, list))
    }
}

pub unsafe fn allocate_object_memory_in_game() -> Result<Left, Refused> {
    unsafe {
        allocate_object_memory(&mut |bytes, _| Ok(allocate(game_ptr(LIST_RVA), bytes)? as u32))
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { allocate_object_memory_in_game() }?.apply(r);
    Ok(())
}
