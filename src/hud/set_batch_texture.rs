//! `hud::set_batch_texture` (0x45A855): the device's texture set to the batch's
//! (`g_kindFirstSlots`[`dword_6D14FE`], `scene::set_texture`).
//!
//! In:  nothing
//! Out: all kept (`pusha` .. `popa`)
//!
//! Caller: `hud::add_polygon`. Refuses a texture past `g_kindFirstSlots` (0x400 dwords; the HUD
//! asks for 0xC0 .. 0xDC), before anything.

use crate::errors::Refused;
use crate::race::owner_frame::Frame;
use crate::scene::owner::scene;

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::batch::texture_slot`
pub fn set_batch_texture(frame: &Frame) -> Result<(), Refused> {
    let n = rc2k_core::hud::batch::texture_slot(frame.panel_and_overlay.panel.shown_texture.get())?;
    crate::scene::set_texture::set_texture(scene().textures.kinds.kind_first_slots[n].bits());
    Ok(())
}
