//! `scene::load_slot_textures` (0x425778): every texture slot in use gets the texture
//! it wants.
//!
//! In:  nothing
//! Out: ecx = the number of slots; everything else preserved
//!
//! `scene::load_slot_texture` for the slots 0 .. `dword_5DC970`. Six callers. Refuses no slots
//! (the original goes through 2^32) and more than the table has. The registers above are the
//! original's; a call by address goes through `routes.rs`, which maps them.

use crate::errors::BadInput;
use crate::framework::game_read;
use crate::scene::load_slot_texture::load_slot_texture_in_game;
use crate::scene::textures::SLOT_COUNT_RVA;

// ---- Rust version ----

/// The number of slots in use (ecx after the original, which `scene::draw_carousel`
/// reads in the main menu).
// Converted: `rc2k_core::scene::textures::slots_in_use`
pub unsafe fn load_slot_textures(
    load: &mut dyn FnMut(u32) -> Result<(), BadInput>,
) -> Result<u32, BadInput> {
    unsafe {
        let count = rc2k_core::scene::textures::slots_in_use(game_read(SLOT_COUNT_RVA))?;
        for slot in 0..count {
            load(slot)?;
        }
        Ok(count)
    }
}

/// With the game's loading.
pub unsafe fn load_slot_textures_in_game() -> Result<u32, BadInput> {
    unsafe { load_slot_textures(&mut |slot| load_slot_texture_in_game(slot)) }
}
