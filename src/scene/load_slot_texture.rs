//! `scene::load_slot_texture` (0x425ADE): gives a texture slot the texture it wants,
//! if it holds another.
//!
//! In:  ecx = the slot
//! Out: every register preserved
//!
//! Nothing if the slot wants none (`dword_5DA968[slot]` is -1) or holds what it wants
//! (`dword_5DB968[slot]`). Else `scene::load_texture` (0x44E778, which is only `sub_41A8F9` of
//! the Direct3D layer: not ported) with eax = the texture and ebx = the slot, `dword_6415F8` +
//! 1, and the slot holds it. One caller, `scene::load_slot_textures`. Refuses a slot past the
//! table.

use crate::errors::BadInput;
use crate::framework::game_write;
use crate::race::frame_store::FRAME_WORK;
use crate::scene::textures::{LOADED_RVA, WANTED_RVA, entry as slot_entry};

// ---- Rust version ----

/// `load`: what stands for `scene::load_texture`, given the texture and the slot.
// Converted: `rc2k_core::scene::textures::slot_load`
pub unsafe fn load_slot_texture(slot: u32, load: &mut dyn FnMut(u32, u32)) -> Result<(), BadInput> {
    unsafe {
        let Some(wanted) = rc2k_core::scene::textures::slot_load(
            slot,
            || slot_entry(WANTED_RVA, slot),
            || slot_entry(LOADED_RVA, slot),
        )?
        else {
            return Ok(());
        };
        load(wanted, slot);
        FRAME_WORK
            .kind_textures_done
            .set_bits(FRAME_WORK.kind_textures_done.bits().wrapping_add(1));
        game_write(LOADED_RVA + slot as usize * 4, wanted);
        Ok(())
    }
}

/// With the game's loading (`scene::load_texture`, the platform's).
pub unsafe fn load_slot_texture_in_game(slot: u32) -> Result<(), BadInput> {
    unsafe { load_slot_texture(slot, &mut crate::platform_3d::load_texture) }
}
