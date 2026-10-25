//! `hud::set_batch_texture` (0x45A855): the device's texture set to the batch's
//! (`g_kindFirstSlots`[`dword_6D14FE`], `scene::set_texture`).
//!
//! In:  nothing
//! Out: all kept (`pusha` .. `popa`)
//!
//! Caller: `hud::add_polygon`. Refuses a texture past `g_kindFirstSlots` (0x400 dwords; the HUD
//! asks for 0xC0 .. 0xDC), before anything.

use crate::errors::Refused;
use crate::hud::batch::SHOWN_TEXTURE_RVA;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::store::game_read;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::batch::texture_slot`
pub fn set_batch_texture() -> Result<(), Refused> {
    let n = rc2k_core::hud::batch::texture_slot(unsafe { game_read(SHOWN_TEXTURE_RVA) })?;
    crate::scene::set_texture::set_texture(KIND_TEXTURES.kind_first_slots[n].bits());
    Ok(())
}
