//! `scene::assign_menu_textures` (0x490DF3): the eight slots of kind 0x258 get the
//! menu's own textures.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! For each of the eight numbers of `dword_75A622` that is not -1: the slot (kind 0x258's first
//! slot, `dword_5D2448`, plus its place in the eight) wants that number plus kind 0x258's first
//! texture (`dword_5D1448`). Two callers. Refuses a slot past the table.

use crate::errors::BadInput;
use crate::framework::{game_read, game_write, rva};
use crate::scene::textures::{KIND_SLOT_RVA, KIND_TEXTURE_RVA, WANTED_RVA};

// ---- game data ----

const OWN_RVA: usize = rva(0x75A622); // dword_75A622, eight numbers
const KIND: usize = 0x258;
const FIRST_TEXTURE_RVA: usize = KIND_TEXTURE_RVA + KIND * 4; // dword_5D1448
const FIRST_SLOT_RVA: usize = KIND_SLOT_RVA + KIND * 4; // dword_5D2448

// ---- Rust version ----

pub unsafe fn assign_menu_textures() -> Result<(), BadInput> {
    // Converted: `rc2k_core::scene::textures::menu_textures`.
    unsafe {
        let wanted = rc2k_core::scene::textures::menu_textures(
            game_read(OWN_RVA),
            game_read(FIRST_SLOT_RVA),
            game_read(FIRST_TEXTURE_RVA),
        )?;
        for (slot, texture) in wanted {
            game_write(WANTED_RVA + slot as usize * 4, texture);
        }
        Ok(())
    }
}
